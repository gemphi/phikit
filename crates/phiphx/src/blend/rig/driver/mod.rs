//! The rig object's drivers: what each drives on the rig (a constraint's
//! influence or mute, an Armature constraint's bone weight, a channel, a
//! custom property), what its variables read, and its value at a frame —
//! `evaluate_driver` (sum, average, min, max, a simple expression), then its
//! F-curve (keys, else the Generator, else the value itself).

pub mod expr;
mod path;
mod read;

pub use path::{BRnaPath, BRnaStep};

use super::solve::BChannelValue;
use super::{BRig, BRigState};
use crate::blend::anim::fcurve::BFCurve;
use expr::BExpr;

/// `DRIVER_TYPE_*`.
pub const DRIVER_AVERAGE: u32 = 0;
pub const DRIVER_PYTHON: u32 = 1;
pub const DRIVER_SUM: u32 = 2;
pub const DRIVER_MIN: u32 = 3;
pub const DRIVER_MAX: u32 = 4;

/// What a driver writes.
#[derive(Clone, Debug, PartialEq)]
pub enum BDriven {
    Influence { bone: usize, con: usize },
    Mute { bone: usize, con: usize },
    TargetWeight { bone: usize, con: usize, target: usize },
    /// A channel property (`location`, `rotation_euler`, …) at the curve's
    /// index.
    Channel { bone: usize, property: String },
    Prop { bone: usize, name: String },
    /// Something the pose does not read (a bone's `hide`, …).
    Other(String),
}

/// Where a variable's value comes from.
#[derive(Clone, Debug, PartialEq)]
pub enum BDriverSource {
    /// A custom property of a bone of this rig.
    Prop { bone: usize, name: String },
    /// A channel value of a bone of this rig.
    Channel { bone: usize, property: String, index: usize },
    /// A variable kind or path this solver does not read (reads 0).
    Unsupported,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BDriverVar {
    pub name: String,
    pub source: BDriverSource,
}

/// One driver F-curve of the rig object.
#[derive(Clone, Debug)]
pub struct BRigDriver {
    pub driven: BDriven,
    pub curve: BFCurve,
    /// `ChannelDriver.type`.
    pub kind: u32,
    pub expression: String,
    pub vars: Vec<BDriverVar>,
    /// The expression parsed against `frame` and the variables (`None`: not
    /// a simple expression — it reads 0, as a file whose scripts do not run).
    pub parsed: Option<BExpr>,
}

impl BRigDriver {
    /// A variable's value on the rig as animated.
    fn var(rig: &BRig, state: &BRigState, v: &BDriverVar) -> f64 {
        match &v.source {
            BDriverSource::Prop { bone, name } => state.prop(rig, *bone, name) as f64,
            BDriverSource::Channel { bone, property, index } => state.raw[*bone].value(property, *index).unwrap_or(0.0) as f64,
            BDriverSource::Unsupported => 0.0,
        }
    }

    /// `evaluate_driver`, then the curve over its value.
    pub fn value(&self, rig: &BRig, state: &BRigState, frame: f32) -> f32 {
        let vals: Vec<f64> = self.vars.iter().map(|v| Self::var(rig, state, v)).collect();
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

    /// Write this driver's value into `state`.
    pub fn apply(&self, rig: &BRig, state: &mut BRigState, frame: f32) {
        let v = self.value(rig, state, frame);
        match &self.driven {
            BDriven::Influence { bone, con } => state.influence[*bone][*con] = v,
            // `ANIMSYS_FLOAT_AS_BOOL`.
            BDriven::Mute { bone, con } => state.muted[*bone][*con] = v > 1.0 - f32::EPSILON,
            BDriven::TargetWeight { bone, con, target } => {
                if let Some(w) = state.weights[*bone][*con].get_mut(*target) {
                    *w = v;
                }
            }
            BDriven::Channel { bone, property } => state.raw[*bone].set(property, self.curve.index as usize, v),
            BDriven::Prop { bone, name } => _ = state.props.insert((*bone, name.clone()), v),
            BDriven::Other(_) => {}
        }
    }
}
