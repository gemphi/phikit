//! `BMaterialDrivers`'s own units: the clamp each write path takes, and
//! where a socket's value lands in the BSDF.

use super::*;
use crate::blend::scene::types::BlendMaterial;
use crate::blend::shade::material::VIEWPORT_PBR;

/// A material as the reader leaves a node-less one: the defaults a driver
/// then writes over.
fn material() -> BlendMaterial {
    BlendMaterial {
        name: "M".to_string(),
        base_color: [0.8, 0.8, 0.8, 1.0],
        viewport_color: [0.8, 0.8, 0.8, 1.0],
        viewport_pbr: VIEWPORT_PBR,
        roughness: 0.5,
        metallic: 0.0,
        base_color_texture: None,
        bsdf: Default::default(),
        mix: None,
        graph: Default::default(),
        normal_map: None,
        maps: Default::default(),
        procedural: Default::default(),
        bake: None,
    }
}

fn sockets() -> Vec<(String, Vec<BMatSocket>)> {
    let s = |name: &str, rgba: bool, linked: bool| BMatSocket { name: name.to_string(), rgba, linked };
    vec![(
        "Principled BSDF".to_string(),
        vec![s("Base Color", true, false), s("Metallic", false, false), s("Roughness", false, true)],
    )]
}

#[test]
fn a_material_property_clamps_to_its_rna_hard_range() {
    assert_eq!(BMatProp::Roughness.range(), (0.0, 1.0));
    assert_eq!(BMatProp::Metallic.range(), (0.0, 1.0));
    assert_eq!(BMatProp::SpecularIntensity.range(), (0.0, 1.0));
    // A colour is free above 1 and clamps at 0 below.
    assert_eq!(BMatProp::DiffuseColor(0).range(), (0.0, f32::MAX));
}

#[test]
fn a_path_names_the_property_it_writes() {
    assert_eq!(BMatProp::of("roughness", 0), Some(BMatProp::Roughness));
    assert_eq!(BMatProp::of("diffuse_color", 2), Some(BMatProp::DiffuseColor(2)));
    assert_eq!(BMatProp::of("diffuse_color", 4), None, "a colour has four components");
    assert_eq!(BMatProp::of("blend_method", 0), None, "not a float this models");
}

#[test]
fn a_float_socket_keeps_what_a_driver_gives_it_and_an_rgba_socket_clamps_at_zero() {
    let set = BMaterialDrivers { inputs: sockets(), ..Default::default() };
    let sock = |input: usize, component: usize| BMatWrite::Socket {
        node: "Principled BSDF".to_string(),
        input,
        component,
    };
    // Metallic declares 0..1 in RNA and still takes -3 and 5 from a driver.
    assert_eq!(set.clamp_for(&sock(1, 0), -3.0), -3.0);
    assert_eq!(set.clamp_for(&sock(1, 0), 5.0), 5.0);
    // Base Color is RGBA: 0 below, free above.
    assert_eq!(set.clamp_for(&sock(0, 0), -2.0), 0.0);
    assert_eq!(set.clamp_for(&sock(0, 0), 4.0), 4.0);
    // A material property clamps both ends.
    assert_eq!(set.clamp_for(&BMatWrite::Own(BMatProp::Roughness), -1.0), 0.0);
    assert_eq!(set.clamp_for(&BMatWrite::Own(BMatProp::Roughness), 3.0), 1.0);
    assert_eq!(set.clamp_for(&BMatWrite::Own(BMatProp::DiffuseColor(1)), 3.0), 3.0);
}

#[test]
fn an_unknown_node_leaves_the_value_alone() {
    let set = BMaterialDrivers { inputs: sockets(), ..Default::default() };
    let write = BMatWrite::Socket { node: "Nope".to_string(), input: 0, component: 0 };
    assert_eq!(set.clamp_for(&write, -5.0), -5.0, "no socket list, no clamp to apply");
}

#[test]
fn a_linked_socket_is_written_but_never_folded_into_the_surface() {
    let mut m = material();
    let before = m.bsdf.roughness;
    let set = BMaterialDrivers { inputs: sockets(), surface: Some("Principled BSDF".to_string()), ..Default::default() };
    // Roughness is linked in `sockets()`, so the fold skips it.
    set.fold(&mut m, &BMatWrite::Socket { node: "Principled BSDF".into(), input: 2, component: 0 }, 0.123);
    assert_eq!(m.bsdf.roughness, before, "a link feeds the shader, not `default_value`");
    // Metallic is not linked, so it lands.
    set.fold(&mut m, &BMatWrite::Socket { node: "Principled BSDF".into(), input: 1, component: 0 }, 0.75);
    assert_eq!(m.bsdf.metallic, 0.75);
}

#[test]
fn only_the_surfaces_own_node_reaches_the_bsdf() {
    let mut m = material();
    let set = BMaterialDrivers { inputs: sockets(), surface: Some("Other".to_string()), ..Default::default() };
    set.fold(&mut m, &BMatWrite::Socket { node: "Principled BSDF".into(), input: 1, component: 0 }, 0.9);
    assert_eq!(m.bsdf.metallic, 0.0, "this Principled is not the one the Output reaches");
}

#[test]
fn a_colour_component_lands_on_its_own_channel() {
    let mut m = material();
    let set = BMaterialDrivers { inputs: sockets(), surface: Some("Principled BSDF".to_string()), ..Default::default() };
    for c in 0..4 {
        set.fold(&mut m, &BMatWrite::Socket { node: "Principled BSDF".into(), input: 0, component: c }, 0.1 * c as f32);
    }
    assert_eq!(m.bsdf.base, [0.0, 0.1, 0.2, 0.30000001]);
}
