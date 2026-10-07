//! Drivers on a light — `Light.adt->drivers`, writing the lamp's own
//! properties. A light has no node tree in Blender 5.x, so unlike a
//! material or a world there is no second ID to read.
//!
//! The hard ranges are the whole content of this module, and they are not
//! uniform — which is why each one is named here rather than a single
//! clamp applied to all:
//!
//! | property | hard range | what that means for a driver |
//! | --- | --- | --- |
//! | `energy` | unbounded | a driver may make a lamp's power **negative** |
//! | `color` | 0 .. ∞ | clamps at 0, free above 1 |
//! | `shadow_soft_size` | 0 .. ∞ | a radius cannot go negative |
//! | `spot_size` | **0.01745** .. π | a cone cannot close past 1° |
//! | `spot_blend` | 0 .. 1 | clamps both ends |
//! | `size`, `size_y` | 0 .. ∞ | an area light's edges |
//! | `angle` | 0 .. π | a sun's angular diameter |
//!
//! `spot_size`'s non-zero minimum is the one that no uniform rule would
//! give: driving it to 0 leaves a 1° cone, not a closed one.

use crate::blend::anim::driver::{BlendDriver, BlendDriverScene};
use crate::blend::object::data::BlendLightData;

/// A `Light` property a driver may write, with the component for `color`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BLightProp {
    Energy,
    Color(usize),
    /// `shadow_soft_size` — `Light.radius`, the soft-shadow radius.
    Radius,
    SpotSize,
    SpotBlend,
    /// `size` — `Light.area_size`.
    AreaSize,
    /// `size_y` — `Light.area_sizey`.
    AreaSizeY,
    /// `angle` — `Light.sun_angle`.
    SunAngle,
}

/// `spot_size`'s hard minimum: one degree, in radians (`rna_light.cc`).
pub const SPOT_SIZE_MIN: f32 = 0.017453292;

/// `Lamp.type` of an area light (`eLightType::LA_AREA`).
pub const LA_AREA: u8 = 4;

/// An area light's stored energy is **4/π of the watts the UI shows** —
/// Blender keeps it pre-multiplied so its own radiance works out to
/// `energy / (4 A)` rather than `watts / (π A)`. `BlendLightData::energy`
/// holds the stored number, which is what the renderer consumes
/// (`tests/light_truth.rs`), so a driver's value — which is the **RNA**
/// watts, since that is the property a driver writes — is converted on the
/// way in, exactly as Blender's own RNA setter converts it.
///
/// Point, spot and sun lights store the watts directly and need no
/// conversion; this applies to [`LA_AREA`] alone.
pub const AREA_ENERGY_SCALE: f32 = 4.0 / std::f32::consts::PI;

impl BLightProp {
    /// The RNA path and component a driver's curve names.
    pub fn of(path: &str, index: usize) -> Option<Self> {
        match path {
            "energy" => Some(Self::Energy),
            "color" if index < 3 => Some(Self::Color(index)),
            "shadow_soft_size" => Some(Self::Radius),
            "spot_size" => Some(Self::SpotSize),
            "spot_blend" => Some(Self::SpotBlend),
            "size" => Some(Self::AreaSize),
            "size_y" => Some(Self::AreaSizeY),
            "angle" => Some(Self::SunAngle),
            _ => None,
        }
    }

    /// `(hard_min, hard_max)` — what `RNA_property_float_set` clamps to.
    pub fn range(self) -> (f32, f32) {
        let pi = std::f32::consts::PI;
        match self {
            Self::Energy => (f32::MIN, f32::MAX),
            Self::SpotSize => (SPOT_SIZE_MIN, pi),
            Self::SpotBlend => (0.0, 1.0),
            Self::SunAngle => (0.0, pi),
            _ => (0.0, f32::MAX),
        }
    }
}

/// One driver on a light.
#[derive(Clone, Debug, PartialEq)]
pub struct BLightDriver {
    pub prop: BLightProp,
    pub driver: BlendDriver,
}

/// Every driver one light carries.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BLightDrivers {
    /// The `Light` datablock's name, which is how a caller matches it to
    /// the light it read.
    pub light: String,
    pub drivers: Vec<BLightDriver>,
}

impl BLightDrivers {
    pub fn is_empty(&self) -> bool {
        self.drivers.is_empty()
    }

    /// Every driver's value at `frame`, clamped as Blender clamps it on the
    /// way in. A muted driver is left out, as it writes nothing.
    pub fn values(&self, scene: &BlendDriverScene, frame: f32) -> Vec<(BLightProp, f32)> {
        self.drivers
            .iter()
            .filter(|d| !d.driver.muted())
            .map(|d| {
                let (lo, hi) = d.prop.range();
                (d.prop, d.driver.value(scene, frame).clamp(lo, hi))
            })
            .collect()
    }

    /// Write this frame's driven values over `light`.
    ///
    /// `energy` on an **area** light is scaled by [`AREA_ENERGY_SCALE`] on
    /// the way in, because a driver writes the RNA watts and the field
    /// holds Blender's stored value. Every other property is stored in the
    /// same units RNA reports.
    pub fn apply(&self, light: &mut BlendLightData, scene: &BlendDriverScene, frame: f32) {
        for (prop, value) in self.values(scene, frame) {
            match prop {
                // The RNA watts a driver writes, in the units the field
                // holds — see `AREA_ENERGY_SCALE`.
                BLightProp::Energy if light.kind == LA_AREA => light.energy = value * AREA_ENERGY_SCALE,
                BLightProp::Energy => light.energy = value,
                BLightProp::Color(c) => light.color[c] = value,
                BLightProp::Radius => light.radius = value,
                BLightProp::SpotSize => light.spot_size = value,
                BLightProp::SpotBlend => light.spot_blend = value,
                BLightProp::AreaSize => light.area_size = value,
                BLightProp::AreaSizeY => light.area_sizey = value,
                BLightProp::SunAngle => light.sun_angle = value,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_property_carries_its_own_hard_range() {
        // The one unbounded property: a driver may make a lamp negative.
        let (lo, hi) = BLightProp::Energy.range();
        assert!(lo < -1e30 && hi > 1e30, "energy is unbounded, not 0..1");
        // The one with a non-zero minimum.
        assert_eq!(BLightProp::SpotSize.range().0, SPOT_SIZE_MIN);
        assert_eq!(BLightProp::SpotSize.range().1, std::f32::consts::PI);
        assert_eq!(BLightProp::SpotBlend.range(), (0.0, 1.0));
        assert_eq!(BLightProp::SunAngle.range(), (0.0, std::f32::consts::PI));
        for prop in [BLightProp::Color(0), BLightProp::Radius, BLightProp::AreaSize, BLightProp::AreaSizeY] {
            assert_eq!(prop.range(), (0.0, f32::MAX), "{prop:?} clamps at 0 and is free above");
        }
    }

    #[test]
    fn a_path_names_the_property_it_writes() {
        assert_eq!(BLightProp::of("energy", 0), Some(BLightProp::Energy));
        assert_eq!(BLightProp::of("color", 2), Some(BLightProp::Color(2)));
        assert_eq!(BLightProp::of("color", 3), None, "a light's colour is three components");
        assert_eq!(BLightProp::of("shadow_soft_size", 0), Some(BLightProp::Radius));
        assert_eq!(BLightProp::of("angle", 0), Some(BLightProp::SunAngle));
        assert_eq!(BLightProp::of("use_shadow", 0), None, "not a float this models");
    }

    #[test]
    fn driving_a_spot_closed_leaves_it_one_degree_open() {
        let (lo, _) = BLightProp::SpotSize.range();
        assert_eq!((-2.0f32).clamp(lo, std::f32::consts::PI), SPOT_SIZE_MIN);
        assert_eq!(0.0f32.clamp(lo, std::f32::consts::PI), SPOT_SIZE_MIN);
    }
}
