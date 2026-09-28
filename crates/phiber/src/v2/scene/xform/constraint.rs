//! What a transform is held to — Blender's constraint keys during G / R / S
//! (`transform_constraints.cc`): X / Y / Z hold it to that world axis,
//! Shift X / Y / Z to the plane of the other two ("locking" the one named);
//! the same key again frees it.

use glam::Vec3;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum BConstraint {
    #[default]
    Free,
    /// Along world axis 0 X, 1 Y, 2 Z.
    Axis(usize),
    /// In the plane of the other two: the axis named is locked.
    Plane(usize),
}

pub static AXES: [&str; 3] = ["x", "y", "z"];

impl BConstraint {
    /// World axis `i`.
    pub fn unit(i: usize) -> Vec3 {
        [Vec3::X, Vec3::Y, Vec3::Z][i.min(2)]
    }

    /// X / Y / Z pressed (`plane` with Shift): held to it, or — pressed
    /// again — free.
    pub fn pressed(self, axis: &str, plane: bool) -> Self {
        let Some(i) = AXES.iter().position(|a| *a == axis) else { return self };
        let next = if plane { Self::Plane(i) } else { Self::Axis(i) };
        if next == self { Self::Free } else { next }
    }

    /// How a scale spreads a factor over the three axes.
    pub fn spread(self, factor: f32) -> Vec3 {
        match self {
            Self::Free => Vec3::splat(factor),
            Self::Axis(i) => Vec3::ONE + Self::unit(i) * (factor - 1.0),
            Self::Plane(i) => Vec3::splat(factor) + Self::unit(i) * (1.0 - factor),
        }
    }

    /// The axis a typed distance runs along: the one held, the first free
    /// one of a plane, X when free — Blender's first numeric field.
    pub fn typed_axis(self) -> Vec3 {
        match self {
            Self::Axis(i) => Self::unit(i),
            Self::Plane(0) => Vec3::Y,
            Self::Plane(_) | Self::Free => Vec3::X,
        }
    }

    /// The header's words for it, as Blender's read-out has them.
    pub fn label(self) -> String {
        match self {
            Self::Free => String::new(),
            Self::Axis(i) => format!(" along {}", AXES[i].to_uppercase()),
            Self::Plane(i) => {
                let free: Vec<String> = (0..3).filter(|k| *k != i).map(|k| AXES[k].to_uppercase()).collect();
                format!(" along {} and {}", free[0], free[1])
            }
        }
    }

    pub fn encode(self) -> String {
        match self {
            Self::Free => String::new(),
            Self::Axis(i) => i.to_string(),
            Self::Plane(i) => format!("p{i}"),
        }
    }

    pub fn parse(raw: &str) -> Self {
        match (raw.strip_prefix('p'), raw.parse::<usize>()) {
            (Some(i), _) => i.parse().map_or(Self::Free, |i: usize| Self::Plane(i.min(2))),
            (None, Ok(i)) => Self::Axis(i.min(2)),
            _ => Self::Free,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_key_holds_it_the_same_key_frees_it_and_shift_locks_the_axis_named() {
        let z = BConstraint::Free.pressed("z", false);
        assert_eq!(z, BConstraint::Axis(2));
        assert_eq!(z.pressed("z", false), BConstraint::Free);
        assert_eq!(z.pressed("z", true), BConstraint::Plane(2), "Shift Z: the X-Y plane");
        assert_eq!(BConstraint::Plane(2).spread(2.0), Vec3::new(2.0, 2.0, 1.0));
        assert_eq!(BConstraint::Axis(0).spread(2.0), Vec3::new(2.0, 1.0, 1.0));
        assert_eq!(BConstraint::Plane(2).label(), " along X and Y");
        for c in [BConstraint::Free, BConstraint::Axis(1), BConstraint::Plane(0)] {
            assert_eq!(BConstraint::parse(&c.encode()), c);
        }
    }
}
