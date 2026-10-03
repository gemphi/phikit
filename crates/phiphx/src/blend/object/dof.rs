//! Camera depth of field — `CameraDOFSettings` (`DNA_camera_types.h`),
//! embedded in `Camera.dof`.
//!
//! Blender describes the aperture as an f-stop, which is a *ratio*, so the
//! radius a renderer needs depends on the lens:
//!
//! ```text
//! aperture radius = (lens in metres) / (2 × f-stop)       perspective
//! aperture radius = 1 / (2 × f-stop)                      orthographic
//! ```
//!
//! That is `blender_camera_sync` in `intern/cycles/blender/camera.cpp`. The
//! orthographic case has no lens to scale by, so Cycles drops the factor
//! rather than treating the lens as 1 mm.
//!
//! The focus plane is a *plane* perpendicular to the camera's axis, not a
//! sphere around the lens: Cycles takes `focal_distance / D.z`, so a ray
//! towards a corner of the frame focuses further away than one down the
//! middle. Getting that wrong curves the plane of focus and shows up as a
//! blurred centre or blurred corners depending on which way it is wrong.

use crate::blend::file::types::*;

/// `Camera.dof` — Properties → Object Data → Depth of Field.
#[derive(Debug, Clone, PartialEq)]
pub struct BlendDof {
    /// `CAM_DOF_ENABLED`. Everything below is ignored while this is false,
    /// exactly as Cycles ignores it.
    pub on: bool,
    /// `focus_distance` — used when no focus object is named.
    pub focus_distance: f32,
    /// `focus_object`'s name, empty when the camera focuses by distance.
    /// The object's *distance along the camera axis* is what counts, so this
    /// can only be resolved where the scene's transforms are known.
    pub focus_object: String,
    pub fstop: f32,
    /// 0 means a round aperture; 3 or more gives that many straight edges.
    pub blades: u32,
    /// Aperture rotation, radians.
    pub rotation: f32,
    /// Anamorphic squeeze: the bokeh is divided by this across.
    pub ratio: f32,
}

impl Default for BlendDof {
    fn default() -> Self {
        // Blender's own defaults, from the DNA initialisers.
        Self { on: false, focus_distance: 10.0, focus_object: String::new(), fstop: 2.8, blades: 0, rotation: 0.0, ratio: 1.0 }
    }
}

impl BlendDof {
    /// The aperture radius a renderer jitters the ray origin over, in world
    /// units. Zero when depth of field is off — which is what makes a
    /// pinhole camera the natural default rather than a special case.
    pub fn aperture(&self, lens_mm: f32, orthographic: bool) -> f32 {
        if !self.on {
            return 0.0;
        }
        let fstop = self.fstop.max(1e-5);
        if orthographic { 1.0 / (2.0 * fstop) } else { (lens_mm * 1e-3) / (2.0 * fstop) }
    }
}

impl BlendFile {
    /// Read `Camera.dof` off a `CA` block.
    pub(crate) fn extract_dof(&self, ca: &BlendBlock, dna: &DnaType) -> BlendDof {
        let ps = self.header.pointer_size as usize;
        let dflt = BlendDof::default();
        let at = |field: &str| {
            Some(Self::struct_field_offset_exact("Camera", "dof", dna, ps)?
                + Self::struct_field_offset_exact("CameraDOFSettings", field, dna, ps)?)
        };
        let float = |field: &str, or: f32| match at(field) {
            Some(o) if o + 4 <= ca.data.len() => self.read_f32(&ca.data[o..o + 4]),
            _ => or,
        };
        let int = |field: &str, or: u32| match at(field) {
            Some(o) if o + 4 <= ca.data.len() => self.read_i32(&ca.data[o..o + 4]) as u32,
            _ => or,
        };
        // `flag` is a short (`eCamera_DOF_Flag`), not an int.
        let flag = match at("flag") {
            Some(o) if o + 2 <= ca.data.len() => self.read_u16(&ca.data[o..o + 2]),
            _ => 0,
        };
        let focus_object = at("*focus_object")
            .map(|o| Self::read_ptr(&ca.data, o, ps, self.header.little_endian))
            .filter(|addr| *addr != 0)
            .and_then(|addr| self.block_at_code(addr, "OB"))
            .and_then(|ob| self.extract_name_from_block(ob, "name").ok())
            .unwrap_or_default();
        BlendDof {
            on: flag & 1 != 0,
            focus_distance: float("focus_distance", dflt.focus_distance),
            focus_object,
            fstop: float("aperture_fstop", dflt.fstop),
            blades: int("aperture_blades", dflt.blades),
            rotation: float("aperture_rotation", dflt.rotation),
            ratio: float("aperture_ratio", dflt.ratio),
        }
    }
}
