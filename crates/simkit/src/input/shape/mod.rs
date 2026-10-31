//! The shaping a raw device axis goes through before it is a demand.
//!
//! A stick that drives a drone needs all of this: a dead zone, because a
//! worn gimbal does not rest at zero and a drone that drifts is a drone
//! that crashes; an expo curve, because full travel has to reach the
//! airframe's limit while the middle of the stick stays fine enough to
//! hold a hover; and a trim, because every airframe pulls somewhere.
//!
//! The knobs and their order are a radio's. AirSim reads its stick through
//! `RCData` (`common/CommonStructs.hpp`) and then through
//! `simple_flight`'s `RemoteControl::getGoalValue`, which applies exactly
//! this kind of per-channel deflection and dead-zone treatment; this is
//! that step, pulled out so it can be tested and remapped on its own.

/// Whether an axis swings both ways from a centre (a stick, a wheel) or
/// only one way from a stop (a collective, a trigger, a brake pedal).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Span {
    /// `-1.0 ..= 1.0`, resting at `0.0`.
    #[default]
    Centred,
    /// `0.0 ..= 1.0`, resting at `0.0`.
    Unipolar,
}

impl Span {
    /// The low and high end of the span.
    pub fn range(self) -> (f32, f32) {
        match self {
            Span::Centred => (-1.0, 1.0),
            Span::Unipolar => (0.0, 1.0),
        }
    }

    /// Where the axis rests, which is where the dead zone sits.
    pub fn rest(self) -> f32 {
        0.0
    }

    /// `value` reflected about the middle of the span's travel, so an
    /// inverted axis still covers the same range.
    pub fn flip(self, value: f32) -> f32 {
        match self {
            Span::Centred => -value,
            Span::Unipolar => 1.0 - value,
        }
    }

    /// `value`, read on this span, expressed on `target`.
    pub fn transpose(self, value: f32, target: Span) -> f32 {
        match (self, target) {
            (Span::Centred, Span::Unipolar) => (value + 1.0) * 0.5,
            (Span::Unipolar, Span::Centred) => value * 2.0 - 1.0,
            _ => value,
        }
    }
}

/// How one raw axis becomes one demand.
///
/// [`apply`](Self::apply) runs the steps in a radio's order, and the order
/// is the contract the tests hold it to:
///
/// 1. clamp the raw value into `source`;
/// 2. invert it, if the device's sense is backwards;
/// 3. take out the dead zone around the resting point, rescaling what is
///    left so full travel still reaches the end;
/// 4. bend it through the expo curve;
/// 5. multiply by `scale` and add `trim` — both in `source` units, because
///    a trim lever moves the stick, not the airframe;
/// 6. transpose onto `target` and clamp there.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxisShape {
    /// The span the device reports on.
    pub source: Span,
    /// The span the demand is wanted on.
    pub target: Span,
    /// Whether the device's sense is backwards (a pulled stick reading
    /// positive when a climb is wanted negative, say).
    pub invert: bool,
    /// The fraction of travel either side of rest that reads as rest,
    /// `0.0 ..= 0.95`.
    pub dead_zone: f32,
    /// How much of the curve is cubic, `0.0` straight to `1.0` fully
    /// cubic. A radio's "expo".
    pub expo: f32,
    /// Output gain, applied after the curve.
    pub scale: f32,
    /// A standing bias added after the curve, to trim out a drift.
    pub trim: f32,
}

impl Default for AxisShape {
    fn default() -> Self {
        Self::stick()
    }
}

impl AxisShape {
    /// A centred stick: a small dead zone, a little expo, no trim.
    pub fn stick() -> Self {
        Self {
            source: Span::Centred,
            target: Span::Centred,
            invert: false,
            dead_zone: 0.05,
            expo: 0.3,
            scale: 1.0,
            trim: 0.0,
        }
    }

    /// A lever or pedal that rests at its stop: a dead zone only at the
    /// bottom, and no expo, because a throttle wants to be linear.
    pub fn lever() -> Self {
        Self {
            source: Span::Unipolar,
            target: Span::Unipolar,
            invert: false,
            dead_zone: 0.02,
            expo: 0.0,
            scale: 1.0,
            trim: 0.0,
        }
    }

    /// A centred stick read as a lever, which is how a gamepad's left
    /// stick becomes a collective: centre is half throttle.
    pub fn stick_as_lever() -> Self {
        Self {
            source: Span::Centred,
            target: Span::Unipolar,
            invert: false,
            dead_zone: 0.05,
            expo: 0.0,
            scale: 1.0,
            trim: 0.0,
        }
    }

    /// The same shape with the device's sense reversed.
    pub fn inverted(mut self) -> Self {
        self.invert = true;
        self
    }

    /// The same shape with a different dead zone.
    pub fn with_dead_zone(mut self, dead_zone: f32) -> Self {
        self.dead_zone = dead_zone;
        self
    }

    /// The same shape with a different expo.
    pub fn with_expo(mut self, expo: f32) -> Self {
        self.expo = expo;
        self
    }

    /// The same shape with a different output gain.
    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    /// The same shape with a different trim.
    pub fn with_trim(mut self, trim: f32) -> Self {
        self.trim = trim;
        self
    }

    /// The same shape reporting onto `target`.
    pub fn onto(mut self, target: Span) -> Self {
        self.target = target;
        self
    }

    /// `raw`, read off the device, as a demand.
    pub fn apply(&self, raw: f32) -> f32 {
        let (lo, hi) = self.source.range();
        let clamped = raw.clamp(lo, hi);
        let sensed = match self.invert {
            true => self.source.flip(clamped),
            false => clamped,
        };
        let live = self.past_dead_zone(sensed);
        let curved = self.through_expo(live);
        let biased = curved * self.scale + self.trim;
        let (tlo, thi) = self.target.range();
        self.source
            .transpose(biased.clamp(lo, hi), self.target)
            .clamp(tlo, thi)
    }

    /// `value` with the dead zone taken out, what is left stretched back
    /// over the full span so the end of the travel still reaches the end.
    fn past_dead_zone(&self, value: f32) -> f32 {
        let dead = self.dead_zone.clamp(0.0, 0.95);
        let offset = value - self.source.rest();
        let reach = offset.abs();
        match reach <= dead {
            true => self.source.rest(),
            false => self.source.rest() + offset.signum() * (reach - dead) / (1.0 - dead),
        }
    }

    /// `value` bent through the cubic expo blend a radio uses: straight at
    /// `expo == 0`, pure cube at `expo == 1`. Odd in `value`, so it keeps
    /// the ends of the travel and only softens the middle.
    fn through_expo(&self, value: f32) -> f32 {
        let e = self.expo.clamp(0.0, 1.0);
        (1.0 - e) * value + e * value * value * value
    }
}

#[cfg(test)]
mod tests;
