//! What a 3D view draws over the scene: its Overlays and Gizmos popovers
//! (`VIEW3D_PT_overlay`, `VIEW3D_PT_gizmo_display`), as the file saved them.
//!
//! Blender keeps these as bits spread over six fields of `View3D`
//! (`DNA_view3d_types.h`): `overlay.flag` (mostly "hide" bits), `flag`
//! (outlines, relationship lines, every origin), `flag2` (every overlay),
//! `gridflag` (floor and axes), `gizmo_flag` (more "hide" bits) and
//! `gizmo_show_object`. Each switch is read by its RNA name, which is the
//! name Phiber keys it by, so a reader of this list never needs the bits.

use crate::blend::file::types::*;

/// Where one switch lives, and whether its bit hides or shows.
#[derive(Clone, Copy)]
enum BOverlayBit {
    /// `View3D.overlay.flag`.
    Overlay(u32, bool),
    /// `View3D.flag`.
    Flag(u32, bool),
    /// `View3D.flag2`.
    Flag2(u32, bool),
    /// `View3D.gridflag`.
    Grid(u32, bool),
    /// `View3D.gizmo_flag`.
    Gizmo(u32, bool),
    /// `View3D.gizmo_show_object`.
    GizmoObject(u32, bool),
}

const HIDES: bool = false;
const SHOWS: bool = true;

/// Every switch by RNA name (`rna_space.cc`), with its bit.
const SWITCH: [(&str, BOverlayBit); 21] = [
    ("show_overlays", BOverlayBit::Flag2(1 << 2, HIDES)),
    ("show_floor", BOverlayBit::Grid(1 << 0, SHOWS)),
    ("show_axis_x", BOverlayBit::Grid(1 << 1, SHOWS)),
    ("show_axis_y", BOverlayBit::Grid(1 << 2, SHOWS)),
    ("show_axis_z", BOverlayBit::Grid(1 << 3, SHOWS)),
    ("show_text", BOverlayBit::Overlay(1 << 5, HIDES)),
    ("show_stats", BOverlayBit::Overlay(1 << 11, SHOWS)),
    ("show_cursor", BOverlayBit::Overlay(1 << 1, HIDES)),
    ("show_extras", BOverlayBit::Overlay(1 << 9, HIDES)),
    ("show_bones", BOverlayBit::Overlay(1 << 8, HIDES)),
    ("show_relationship_lines", BOverlayBit::Flag(1 << 2, HIDES)),
    ("show_outline_selected", BOverlayBit::Flag(1 << 11, SHOWS)),
    ("show_object_origins", BOverlayBit::Overlay(1 << 10, HIDES)),
    ("show_object_origins_all", BOverlayBit::Flag(1 << 15, SHOWS)),
    ("show_gizmo", BOverlayBit::Gizmo(1 << 0, HIDES)),
    ("show_gizmo_navigate", BOverlayBit::Gizmo(1 << 1, HIDES)),
    ("show_gizmo_context", BOverlayBit::Gizmo(1 << 2, HIDES)),
    ("show_gizmo_tool", BOverlayBit::Gizmo(1 << 3, HIDES)),
    ("show_gizmo_object_translate", BOverlayBit::GizmoObject(1 << 0, SHOWS)),
    ("show_gizmo_object_rotate", BOverlayBit::GizmoObject(1 << 1, SHOWS)),
    ("show_gizmo_object_scale", BOverlayBit::GizmoObject(1 << 2, SHOWS)),
];

/// The six fields, as read.
#[derive(Clone, Copy)]
struct BOverlayWord {
    overlay: u32,
    flag: u32,
    flag2: u32,
    grid: u32,
    gizmo: u32,
    gizmo_object: u32,
}

impl BOverlayWord {
    /// A new 3D view's (`DNA_view3d_defaults.h`): outlines on, the floor and
    /// the X and Y axes on, everything else as its bit's absence says.
    const DEFAULT: Self = Self { overlay: 0, flag: 1 << 11, flag2: 0, grid: 0b1_0111, gizmo: 0, gizmo_object: 0 };

    fn on(&self, bit: BOverlayBit) -> bool {
        let (word, mask, shows) = match bit {
            BOverlayBit::Overlay(m, s) => (self.overlay, m, s),
            BOverlayBit::Flag(m, s) => (self.flag, m, s),
            BOverlayBit::Flag2(m, s) => (self.flag2, m, s),
            BOverlayBit::Grid(m, s) => (self.grid, m, s),
            BOverlayBit::Gizmo(m, s) => (self.gizmo, m, s),
            BOverlayBit::GizmoObject(m, s) => (self.gizmo_object, m, s),
        };
        (word & mask != 0) == shows
    }
}

/// A 3D view's overlay and gizmo switches, by RNA name.
#[derive(Clone, Debug, PartialEq)]
pub struct BlendOverlay {
    pub list: Vec<(&'static str, bool)>,
}

impl Default for BlendOverlay {
    /// What a new 3D view shows.
    fn default() -> Self {
        Self::of(BOverlayWord::DEFAULT)
    }
}

impl BlendOverlay {
    fn of(word: BOverlayWord) -> Self {
        Self { list: SWITCH.iter().map(|&(name, bit)| (name, word.on(bit))).collect() }
    }

    /// The switch `name` (an RNA name), if it is one.
    pub fn get(&self, name: &str) -> Option<bool> {
        self.list.iter().find(|(n, _)| *n == name).map(|&(_, on)| on)
    }
}

impl BlendFile {
    /// The switches of the `View3D` space block `v3d`.
    pub(crate) fn overlay_of(&self, v3d: &BlendBlock, dna: &DnaType) -> BlendOverlay {
        let field = |f: &str| self.int_field(v3d, "View3D", f, dna);
        let overlay = self
            .off("View3D", "overlay", dna)
            .zip(self.off("View3DOverlay", "flag", dna))
            .and_then(|(a, b)| v3d.data.get(a + b..a + b + 4))
            .map_or(0, |b| self.read_i32(b) as u32);
        BlendOverlay::of(BOverlayWord {
            overlay,
            flag: field("flag"),
            flag2: field("flag2"),
            grid: field("gridflag"),
            gizmo: field("gizmo_flag"),
            gizmo_object: field("gizmo_show_object"),
        })
    }
}
