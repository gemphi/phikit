//! Each joint's step (`UpdateAngle`, `Lock`, `UpdateAngleApply`): the
//! Jacobian's change applied, clamped to the joint's limits.

use super::jacobian::BJacobian;
use super::segment::{BIkMath, BJoint, BSegment};
use glam::DVec3;

impl BSegment {
    /// `UpdateAngle`: `true` when a limit clamped the change (`delta` holds
    /// the clamped change, `clamp` which degrees of freedom).
    pub fn update_angle(&mut self, jac: &BJacobian, delta: &mut DVec3, clamp: &mut [bool; 3]) -> bool {
        let (id, locked, basis) = (self.dof_id, self.locked, self.basis);
        let translation = self.translation;
        match &mut self.joint {
            BJoint::Null => false,
            BJoint::Spherical { new_basis, limit, min, max, min_y, max_y, locked_a } => {
                if locked[0] && locked[1] && locked[2] {
                    return false;
                }
                let dq = DVec3::new(jac.angle_update(id), jac.angle_update(id + 1), jac.angle_update(id + 2));
                *new_basis = BIkMath::from_rotation_vector(dq).map_or(basis, |m| basis * m);
                if !limit[0] && !limit[1] && !limit[2] {
                    return false;
                }
                let mut a = BIkMath::spherical_range(new_basis);
                for k in 0..3 {
                    if locked[k] {
                        a[k] = locked_a[k];
                    }
                }
                let (mut ax, mut ay, mut az) = (a.x, a.y, a.z);
                *clamp = [false; 3];
                if limit[1] {
                    if a.y > *max_y {
                        ay = *max_y;
                        clamp[1] = true;
                    } else if a.y < *min_y {
                        ay = *min_y;
                        clamp[1] = true;
                    }
                }
                if limit[0] && limit[2] {
                    if BIkMath::ellipse_clamp(&mut ax, &mut az, min, max) {
                        clamp[0] = true;
                        clamp[2] = true;
                    }
                } else if limit[0] {
                    if ax < min[0] {
                        ax = min[0];
                        clamp[0] = true;
                    } else if ax > max[0] {
                        ax = max[0];
                        clamp[0] = true;
                    }
                } else if limit[2] {
                    if az < min[1] {
                        az = min[1];
                        clamp[2] = true;
                    } else if az > max[1] {
                        az = max[1];
                        clamp[2] = true;
                    }
                }
                let swung = || BIkMath::swing(ax, az) * BIkMath::rotation_angle(ay, 1);
                if !clamp[0] && !clamp[1] && !clamp[2] {
                    if locked[0] || locked[1] || locked[2] {
                        *new_basis = swung();
                    }
                    return false;
                }
                *new_basis = swung();
                *delta = BIkMath::axis_angle(&(basis.transpose() * *new_basis));
                if !(locked[0] || locked[2]) && (clamp[0] || clamp[2]) {
                    locked_a.x = ax;
                    locked_a.z = az;
                }
                if !locked[1] && clamp[1] {
                    locked_a.y = ay;
                }
                true
            }
            BJoint::Revolute { angle, new_angle, limit, min, max, .. } => {
                if locked[0] {
                    return false;
                }
                *new_angle = *angle + jac.angle_update(id);
                clamp[0] = false;
                if !*limit {
                    return false;
                }
                if *new_angle > *max {
                    delta[0] = *max - *angle;
                } else if *new_angle < *min {
                    delta[0] = *min - *angle;
                } else {
                    return false;
                }
                clamp[0] = true;
                *new_angle = *angle + delta[0];
                true
            }
            BJoint::Swing { new_basis, limit_x, limit_z, min, max } => {
                if locked[0] && locked[1] {
                    return false;
                }
                let dq = DVec3::new(jac.angle_update(id), 0.0, jac.angle_update(id + 1));
                *new_basis = match BIkMath::from_rotation_vector(dq) {
                    Some(m) => BIkMath::remove_twist(&(basis * m)),
                    None => basis,
                };
                if !*limit_x && !*limit_z {
                    return false;
                }
                let a = BIkMath::spherical_range(new_basis);
                let (mut ax, mut az) = (0.0, 0.0);
                clamp[0] = false;
                clamp[1] = false;
                if *limit_x && *limit_z {
                    ax = a.x;
                    az = a.z;
                    if BIkMath::ellipse_clamp(&mut ax, &mut az, min, max) {
                        clamp[0] = true;
                        clamp[1] = true;
                    }
                } else if *limit_x {
                    if ax < min[0] {
                        ax = min[0];
                        clamp[0] = true;
                    } else if ax > max[0] {
                        ax = max[0];
                        clamp[0] = true;
                    }
                } else if *limit_z {
                    if az < min[1] {
                        az = min[1];
                        clamp[1] = true;
                    } else if az > max[1] {
                        az = max[1];
                        clamp[1] = true;
                    }
                }
                if !clamp[0] && !clamp[1] {
                    return false;
                }
                *new_basis = BIkMath::swing(ax, az);
                *delta = BIkMath::axis_angle(&(basis.transpose() * *new_basis));
                delta[1] = delta[2];
                delta[2] = 0.0;
                true
            }
            BJoint::Elbow { twist, angle, new_twist, new_angle, limit, limit_twist, min, max, min_twist, max_twist, .. } => {
                if locked[0] && locked[1] {
                    return false;
                }
                clamp[0] = false;
                clamp[1] = false;
                if !locked[0] {
                    *new_angle = *angle + jac.angle_update(id);
                    if *limit {
                        if *new_angle > *max {
                            delta[0] = *max - *angle;
                            *new_angle = *max;
                            clamp[0] = true;
                        } else if *new_angle < *min {
                            delta[0] = *min - *angle;
                            *new_angle = *min;
                            clamp[0] = true;
                        }
                    }
                }
                if !locked[1] {
                    *new_twist = *twist + jac.angle_update(id + 1);
                    if *limit_twist {
                        if *new_twist > *max_twist {
                            delta[1] = *max_twist - *twist;
                            *new_twist = *max_twist;
                            clamp[1] = true;
                        } else if *new_twist < *min_twist {
                            delta[1] = *min_twist - *twist;
                            *new_twist = *min_twist;
                            clamp[1] = true;
                        }
                    }
                }
                clamp[0] || clamp[1]
            }
            BJoint::Translate { enabled, limit, min, max, new_translation, .. } => {
                let (mut dof_id, mut dof, mut clamped) = (id, 0, false);
                for i in 0..3 {
                    if !enabled[i] {
                        new_translation[i] = translation[i];
                        continue;
                    }
                    clamp[dof] = false;
                    if !locked[dof] {
                        new_translation[i] = translation[i] + jac.angle_update(dof_id);
                        if limit[i] {
                            if new_translation[i] > max[i] {
                                delta[dof] = max[i] - translation[i];
                                new_translation[i] = max[i];
                                clamped = true;
                                clamp[dof] = true;
                            } else if new_translation[i] < min[i] {
                                delta[dof] = min[i] - translation[i];
                                new_translation[i] = min[i];
                                clamped = true;
                                clamp[dof] = true;
                            }
                        }
                    }
                    dof_id += 1;
                    dof += 1;
                }
                clamped
            }
        }
    }

    /// `Lock`.
    pub fn lock(&mut self, dof: usize, jac: &mut BJacobian, delta: &DVec3) {
        let id = self.dof_id;
        match &self.joint {
            BJoint::Null => {}
            BJoint::Spherical { .. } => {
                if dof == 1 {
                    self.locked[1] = true;
                    jac.lock(id + 1, delta[1]);
                } else {
                    self.locked[0] = true;
                    self.locked[2] = true;
                    jac.lock(id, delta[0]);
                    jac.lock(id + 2, delta[2]);
                }
            }
            BJoint::Revolute { .. } => {
                self.locked[0] = true;
                jac.lock(id, delta[0]);
            }
            BJoint::Swing { .. } => {
                self.locked[0] = true;
                self.locked[1] = true;
                jac.lock(id, delta[0]);
                jac.lock(id + 1, delta[1]);
            }
            BJoint::Elbow { .. } | BJoint::Translate { .. } => {
                self.locked[dof] = true;
                jac.lock(id + dof, delta[dof]);
            }
        }
    }

    /// `UpdateAngleApply`.
    pub fn apply_angle(&mut self) {
        match &mut self.joint {
            BJoint::Null => {}
            BJoint::Spherical { new_basis, .. } | BJoint::Swing { new_basis, .. } => self.basis = *new_basis,
            BJoint::Revolute { axis, angle, new_angle, .. } => {
                *angle = *new_angle;
                self.basis = BIkMath::rotation_angle(*angle, *axis);
            }
            BJoint::Elbow { axis, twist, angle, new_twist, new_angle, cos_twist, sin_twist, .. } => {
                *angle = *new_angle;
                *twist = *new_twist;
                (*sin_twist, *cos_twist) = twist.sin_cos();
                self.basis = BIkMath::rotation_angle(*angle, *axis) * BIkMath::rotation(*sin_twist, *cos_twist, 1);
            }
            BJoint::Translate { new_translation, .. } => self.translation = *new_translation,
        }
    }

    pub fn unlock(&mut self) {
        self.locked = [false; 3];
    }
}
