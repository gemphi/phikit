//! What each modifier does with a driven setting.
//!
//! `driver.rs` decides what a driver's value *is* — typed, clamped, integers
//! truncated toward zero — and `mod_driver_truth.rs` measures that against
//! Blender. This is the other half: that the value then lands on the right
//! field, with the reader's own guards still applied, and that a setting with
//! no field to land in says so instead of being dropped.

use super::driver::BModValue;
use super::{BModifier, array, displace, lattice, mirror, solidify, subsurf, triangulate, weld};

fn f(v: f32) -> BModValue {
    BModValue::Float(v)
}
fn i(v: i32) -> BModValue {
    BModValue::Int(v)
}

#[test]
fn a_subdivision_takes_levels_and_quality_under_the_readers_own_cap() {
    let mut m = subsurf::BSubsurf::new(1, false, true, false);
    assert!(m.set_driven("levels", 0, i(3)));
    assert_eq!(m.levels, 3);
    // `read` clamps a file's levels to 6 to keep a runaway mesh out of the
    // viewport; a driver must not be a way around that.
    assert!(m.set_driven("levels", 0, i(11)));
    assert_eq!(m.levels, 6, "a driven level takes the same cap the file's does");
    assert!(m.set_driven("quality", 0, i(7)));
    assert_eq!(m.quality, 7);
    assert!(m.set_driven("quality", 0, i(0)));
    assert_eq!(m.quality, 1, "quality starts at 1, as its RNA range says");
}

#[test]
fn render_levels_is_reported_rather_than_setting_the_viewport_level() {
    // `render_levels` is Blender's render-time count and this reader is the
    // viewport's. Landing it on `levels` would subdivide the viewport to the
    // render setting, which is a worse bug than not supporting it.
    let mut m = subsurf::BSubsurf::new(2, false, true, false);
    assert!(!m.set_driven("render_levels", 0, i(6)));
    assert_eq!(m.levels, 2, "and it left the viewport level alone");
}

#[test]
fn a_vector_setting_takes_one_component_at_a_time() {
    let mut m = array::BArray::for_test();
    assert!(m.set_driven("constant_offset_displace", 2, f(-0.7)));
    assert_eq!(m.constant[2], -0.7);
    assert_eq!(m.constant[0], 0.0, "and leaves the others alone");
    assert!(m.set_driven("relative_offset_displace", 1, f(2.5)));
    assert_eq!(m.relative[1], 2.5);
    assert!(!m.set_driven("constant_offset_displace", 3, f(1.0)), "a fourth component is not one");
}

#[test]
fn an_arrays_count_cannot_be_driven_below_one() {
    let mut m = array::BArray::for_test();
    assert!(m.set_driven("count", 0, i(5)));
    assert_eq!(m.count, 5);
    assert!(m.set_driven("count", 0, i(-3)));
    assert_eq!(m.count, 1, "an Array of zero copies is one copy, as Blender's minimum says");
}

#[test]
fn the_thresholds_that_clamp_at_zero_do() {
    let mut w = weld::BWeld::for_test();
    assert!(w.set_driven("merge_threshold", 0, f(-1.0)));
    assert_eq!(w.merge_dist, 0.0, "a merge distance cannot go negative");
    let mut m = mirror::BMirror::for_test();
    assert!(m.set_driven("merge_threshold", 0, f(-1.0)));
    assert_eq!(m.tolerance_for_test(), 0.0);
}

#[test]
fn the_free_floats_really_are_free() {
    // `Displace.strength` and `Solidify.thickness` are unbounded both ways —
    // a negative thickness is a real, usable value, so nothing clamps here.
    let mut d = displace::BDisplace::for_test();
    assert!(d.set_driven("strength", 0, f(-0.7)));
    assert_eq!(d.strength, -0.7);
    assert!(d.set_driven("mid_level", 0, f(-0.7)), "the path spells it mid_level, the field is midlevel");
    assert_eq!(d.midlevel, -0.7);
    let mut s = solidify::BSolidify::for_test();
    assert!(s.set_driven("thickness", 0, f(-0.7)));
    assert_eq!(s.thickness, -0.7);
    let mut l = lattice::BLattice::for_test();
    assert!(l.set_driven("strength", 0, f(-2.0)));
    assert_eq!(l.strength, -2.0);
}

#[test]
fn a_setting_with_no_field_says_so() {
    // These two are modelled by the driver table and measured against
    // Blender, and this reader has nowhere to put them. Reporting `false`
    // is what keeps the gap countable instead of silent — the same choice
    // `specular_intensity` gets on a material.
    let mut s = solidify::BSolidify::for_test();
    assert!(!s.set_driven("material_offset", 0, i(2)));
    assert!(!s.set_driven("material_offset_rim", 0, i(2)));
    let mut m = mirror::BMirror::for_test();
    assert!(!m.set_driven("bisect_threshold", 0, f(0.5)));
}

#[test]
fn a_property_this_modifier_has_never_heard_of_is_refused() {
    let mut t = triangulate::BTriangulate { min_vertices: 4 };
    assert!(t.set_driven("min_vertices", 0, i(6)));
    assert_eq!(t.min_vertices, 6);
    assert!(t.set_driven("min_vertices", 0, i(2)));
    assert_eq!(t.min_vertices, 4, "four corners is the floor, as its RNA range says");
    assert!(!t.set_driven("strength", 0, f(1.0)), "`strength` belongs to other modifiers");
}

#[test]
fn a_level_zero_subdivision_is_disabled_as_blender_disables_it() {
    // `MOD_subsurf.cc`'s `is_disabled` → `get_render_subsurf_level(…) == 0`.
    // The level-0 case used to be handled by refusing to build the modifier
    // at read time, which is the same answer from the file's saved value and
    // the WRONG one once a driver can change it.
    let m = subsurf::BSubsurf::new(0, false, true, false);
    assert!(m.disabled(), "a Subdivision at level 0 does nothing, so Blender drops it");
    let m = subsurf::BSubsurf::new(1, false, true, false);
    assert!(!m.disabled());
}

#[test]
fn a_driver_can_switch_a_level_zero_subdivision_back_on() {
    // Blender asks `is_disabled` of the EVALUATED modifier, so a driver that
    // raises the level decides the answer. This is the case the old read-time
    // guard could not express: the modifier was never built, so there was
    // nothing for the driver to raise.
    let mut m = subsurf::BSubsurf::new(0, false, true, false);
    assert!(m.disabled());
    assert!(m.set_driven("levels", 0, i(2)));
    assert!(!m.disabled(), "driven to 2, it is no longer a no-op");
    assert_eq!(m.levels, 2);
}

// --- `is_disabled`, swept from the Blender sources (batch 190) -------------
// Of the eight types this reader builds, four have an `is_disabled` in
// `MOD_*.cc` and four do not. The four that do not are asserted here too, so
// "no rule" is a recorded finding rather than an omission someone re-checks.

#[test]
fn a_displace_with_nothing_to_displace_along_is_disabled() {
    // `(!texture && direction == MOD_DISP_DIR_RGB_XYZ) || strength == 0`.
    let mut d = displace::BDisplace::for_test();
    assert!(!d.disabled());
    d.strength = 0.0;
    assert!(d.disabled(), "zero strength moves nothing");
    let mut d = displace::BDisplace::for_test();
    d.direction = 4; // MOD_DISP_DIR_RGB_XYZ
    assert!(d.disabled(), "RGB->XYZ with no texture has no colours to read");
}

#[test]
fn a_deform_modifier_without_its_object_is_disabled() {
    let mut l = lattice::BLattice::for_test();
    assert!(l.disabled(), "for_test leaves the lattice unset");
    l.lattice = 42;
    assert!(!l.disabled());
}

#[test]
fn the_four_types_blender_never_disables_are_never_disabled() {
    // Mirror, Triangulate, Solidify and Weld have no `is_disabled` at all in
    // `MOD_*.cc`. Array has one, but it only rejects a placeholder object of
    // the wrong type (curve_ob / start_cap / end_cap), none of which this
    // reader reads — so it is never disabled here either.
    assert!(!mirror::BMirror::for_test().disabled());
    assert!(!triangulate::BTriangulate { min_vertices: 4 }.disabled());
    assert!(!solidify::BSolidify::for_test().disabled());
    assert!(!weld::BWeld::for_test().disabled());
    assert!(!array::BArray::for_test().disabled());
}

// --- the per-frame stack (batch 192) ---------------------------------------

#[test]
fn a_clone_is_an_independent_stack_not_a_shared_one() {
    // This is the whole reason it is a clone and not interior mutability:
    // two frames evaluated at once must not fight over one field.
    // `BKE_modifier_copydata_generic` is what Blender does for the same
    // reason — the depsgraph drives a copy, never the file's own stack.
    let a: Box<dyn BModifier> = Box::new(subsurf::BSubsurf::new(2, false, true, false));
    let mut b = a.clone_box();
    assert!(b.set_driven("levels", 0, i(5)));
    // The original is untouched, which is what makes the source shareable.
    let mut back = a.clone_box();
    assert!(back.set_driven("quality", 0, i(4)));
    assert!(!b.disabled());
    // And a clone of a level-0 Subdivision is still disabled until driven.
    let c: Box<dyn BModifier> = Box::new(subsurf::BSubsurf::new(0, false, true, false));
    let mut c = c.clone_box();
    assert!(c.disabled());
    assert!(c.set_driven("levels", 0, i(3)));
    assert!(!c.disabled(), "the clone takes the driven level, as the depsgraph's copy does");
}

#[test]
fn a_clone_starts_with_an_empty_runtime_cache() {
    // Blender's generic copy ends with "Runtime fields are never to be
    // preserved". `BSubsurf`'s topology tables are exactly that, and a clone
    // handed a different mesh must not read another mesh's adjacency.
    let m = subsurf::BSubsurf::new(1, false, true, false);
    let c = m.clone_box();
    assert!(!c.disabled(), "and the settings did come across");
}
