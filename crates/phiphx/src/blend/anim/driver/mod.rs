//! An **object's** drivers, evaluated — not the rig's. `blend/rig/driver`
//! solves the drivers an armature object carries over its own bones; this
//! is the other half of `AnimData.drivers`: a driver on an object's own
//! `location`, `rotation_euler`, `scale` or custom property, reading
//! another object's channels, final transform, custom property, or the
//! distance or angle between two of them.
//!
//! Blender's order is animation, then drivers: the depsgraph evaluates the
//! animation system for a property and the driver afterwards, so a driver
//! wins over a keyed value on the same channel. The pass here runs on the
//! `raw` the actions and NLA have already written
//! (`BlendMotionList::solve`), which is that order exactly. Custom-property
//! drivers run before channel drivers, so the "property on the rig object,
//! driver reading it" idiom resolves in one frame the way the depsgraph
//! orders it.
//!
//! **One limit, stated rather than hidden:** a variable reads its target's
//! **action-animated** channel, not its driven one. Blender's depsgraph
//! orders driver against driver; this pass is two-step and otherwise
//! unordered, so a channel driver chained off another channel driver's
//! output reads the keyed value underneath it. A driver reading a keyed or
//! still object — which is almost all of them — is exact.

pub mod chan;
mod pass;
pub(crate) mod read;
pub mod socket;
pub mod space;
pub mod state;
mod target;
mod var;
mod wire;

pub use pass::{BDriverProps, BDriverRun, BlendDriverSet, BlendPropDriver};
pub use socket::{BDriverSocket, socket_clamp, socket_path};
pub use state::BDriverState;
pub use var::{BDriverValue, BlendDriverEnd, BlendDriverRead, BlendDriverScene};

use super::fcurve::BFCurve;
use crate::blend::rig::driver::expr::BExpr;

/// `DRIVER_TYPE_*`, as `rig/driver` names them.
use crate::blend::rig::driver::{DRIVER_AVERAGE, DRIVER_MAX, DRIVER_MIN, DRIVER_PYTHON, DRIVER_SUM};

#[derive(Clone, Debug, PartialEq)]
pub struct BlendDriverVar {
    pub name: String,
    pub read: BlendDriverRead,
}

/// A driver's own evaluation — `ChannelDriver` and the F-curve over it,
/// without what it writes. Both an object's channel drivers and its
/// custom-property drivers are one of these plus a target.
#[derive(Clone, Debug, PartialEq)]
pub struct BlendDriver {
    /// `ChannelDriver.type` and its expression.
    pub kind: u32,
    pub expression: String,
    pub vars: Vec<BlendDriverVar>,
    /// The expression parsed against `frame` and the variable names
    /// (`None`: not a simple expression — it reads 0).
    pub parsed: Option<BExpr>,
    /// The driver's own F-curve, mapping the driver's value — **not** the
    /// frame — to the property's.
    pub curve: BFCurve,
}

/// One driver on an object's own transform channel.
#[derive(Clone, Debug, PartialEq)]
pub struct BlendObjectDriver {
    /// The object property it writes (`location`, `rotation_euler`,
    /// `rotation_quaternion`, `rotation_axis_angle`, `scale`) and the index
    /// of the curve.
    pub property: String,
    pub index: usize,
    pub driver: BlendDriver,
}

impl BlendDriver {
    /// `evaluate_driver`, then the driver's F-curve over its value — the
    /// same two steps the rig's drivers take, so the two agree on what a
    /// Sum of one variable or a Python expression comes to.
    pub fn value(&self, scene: &BlendDriverScene, frame: f32) -> f32 {
        let vals: Vec<f64> = self.vars.iter().map(|v| v.read.value(scene)).collect();
        let dv = match self.kind {
            DRIVER_PYTHON => {
                let params: Vec<f64> = std::iter::once(frame as f64).chain(vals.iter().copied()).collect();
                self.parsed.as_ref().and_then(|e| e.eval(&params)).map_or(0.0, |v| v as f32)
            }
            DRIVER_MIN | DRIVER_MAX if !vals.is_empty() => {
                let pick = if self.kind == DRIVER_MIN { f64::min } else { f64::max };
                vals.iter().copied().reduce(pick).unwrap_or(0.0) as f32
            }
            DRIVER_SUM | DRIVER_AVERAGE if vals.len() == 1 => vals[0] as f32,
            DRIVER_SUM => vals.iter().sum::<f64>() as f32,
            DRIVER_AVERAGE if !vals.is_empty() => (vals.iter().sum::<f64>() / vals.len() as f64) as f32,
            _ => 0.0,
        };
        if self.curve.keys.is_empty() { self.curve.generated(dv).unwrap_or(dv) } else { self.curve.evaluate(dv) }
    }

    /// A muted driver curve is switched off, as a muted animation curve is.
    pub fn muted(&self) -> bool {
        self.curve.muted()
    }
}
