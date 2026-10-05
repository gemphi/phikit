//! Catalog tests — that every type is well-formed, found by idname, searched by
//! label, and declares the sockets its source does.

use super::*;

#[test]
fn a_type_is_found_by_its_idname() {
    assert_eq!(find("ShaderNodeEmission").unwrap().label(), "Emission");
    assert!(find("ShaderNodeNonsense").is_none(), "a type not in the seed is None, not a guess");
}

#[test]
fn search_filters_by_label_and_keeps_the_menu_order() {
    // Empty query is the whole menu.
    assert_eq!(search("").len(), CATALOG.len());
    // "mix" finds Mix Shader and Mix (Legacy), case-insensitively.
    let mixes: Vec<&str> = search("MIX").iter().map(|t| t.label()).collect();
    assert!(mixes.contains(&"Mix Shader") && mixes.contains(&"Mix (Legacy)"), "{mixes:?}");
    // A substring that is not a prefix still matches.
    assert_eq!(search("ump").iter().map(|t| t.idname()).collect::<Vec<_>>(), ["ShaderNodeBump"]);
    assert!(search("zzzz").is_empty());
}

#[test]
fn material_output_declares_four_shading_inputs_and_no_output() {
    let out = find("ShaderNodeOutputMaterial").unwrap().sockets();
    assert_eq!(out.outputs.len(), 0, "an output node outputs nothing");
    let names: Vec<&str> = out.inputs.iter().map(|s| s.name).collect();
    assert_eq!(names, ["Surface", "Volume", "Displacement", "Thickness"]);
    assert_eq!(out.inputs[0].kind, SOCK_SHADER);
    assert_eq!(out.inputs[2].kind, SOCK_VECTOR);
}

#[test]
fn principled_declares_its_full_input_list_from_source() {
    // The node I would have got wrong from memory. Checked against
    // `node_shader_bsdf_principled.cc`: one shader output, the gpu-internal
    // Weight omitted, everything else present and in order.
    let p = find("ShaderNodeBsdfPrincipled").unwrap().sockets();
    assert_eq!(p.outputs.len(), 1);
    assert_eq!(p.outputs[0].name, "BSDF");
    assert_eq!(p.inputs.len(), 31, "31 shown inputs (32 declared, Weight omitted)");
    assert_eq!(p.inputs[0].name, "Base Color");
    assert_eq!(p.inputs.last().unwrap().name, "Thin Film IOR");
    assert!(p.inputs.iter().all(|s| s.name != "Weight"), "the gpu-internal Weight is not drawn");
    // Thin Wall is the one boolean.
    assert_eq!(p.inputs.iter().find(|s| s.name == "Thin Wall").unwrap().kind, SOCK_BOOLEAN);
}

#[test]
fn a_socket_whose_identifier_differs_from_its_name_keeps_both() {
    // Brightness/Contrast: the row reads "Brightness", a link resolves "Bright".
    let bc = find("ShaderNodeBrightContrast").unwrap().sockets();
    let bright = bc.inputs.iter().find(|s| s.name == "Brightness").unwrap();
    assert_eq!(bright.identifier, "Bright", "the link identifier is not the label");
    let socket = bright.to_socket();
    assert_eq!(socket.name, "Brightness");
    assert_eq!(socket.identifier, "Bright");
}

#[test]
fn the_link_drag_search_offers_only_types_that_can_take_the_dragged_end() {
    use crate::v2::view::draw::canvas::node::socket::BSocketSide;
    // Dropping a link from an OUTPUT needs a node with an INPUT. Every type has
    // one except the pure sources — RGB, Value, Texture Coordinate — so those
    // are the ones the search must NOT offer.
    let takes_input = search_offering("", BSocketSide::Input);
    let idn: Vec<&str> = takes_input.iter().map(|t| t.idname()).collect();
    assert!(!idn.contains(&"ShaderNodeRGB"), "a node with no input cannot receive a dropped output");
    assert!(!idn.contains(&"ShaderNodeValue"));
    assert!(!idn.contains(&"ShaderNodeTexCoord"));
    assert!(idn.contains(&"ShaderNodeBsdfDiffuse"), "a node with inputs is offered");

    // Dropping a link from an INPUT needs a node with an OUTPUT. Everything has
    // one except the sink, Material Output.
    let gives_output = search_offering("", BSocketSide::Output);
    let idn2: Vec<&str> = gives_output.iter().map(|t| t.idname()).collect();
    assert!(!idn2.contains(&"ShaderNodeOutputMaterial"), "the sink has no output to offer");
    assert!(idn2.contains(&"ShaderNodeRGB"), "a pure source is offered to an input");

    // The query still narrows: "emission" and takes-input keeps just Emission.
    assert_eq!(
        search_offering("emission", BSocketSide::Input).iter().map(|t| t.idname()).collect::<Vec<_>>(),
        ["ShaderNodeEmission"],
    );
}

#[test]
fn the_catalog_groups_into_add_menu_submenus() {
    let groups = by_category();
    // Every type lands in exactly one group, and the groups cover the catalog.
    let total: usize = groups.iter().map(|(_, v)| v.len()).sum();
    assert_eq!(total, CATALOG.len(), "every type is grouped once");
    // The categories are distinct (no category split across two groups).
    for (i, (c, _)) in groups.iter().enumerate() {
        assert!(!groups[i + 1..].iter().any(|(d, _)| d == c), "category {c:?} appears twice");
    }
    // A known type is under the heading it declares.
    let shader = groups.iter().find(|(c, _)| *c == BAddCategory::Shader).unwrap();
    assert_eq!(shader.0.label(), "Shader");
    assert!(shader.1.iter().any(|t| t.idname() == "ShaderNodeEmission"));
    assert!(groups.iter().find(|(c, _)| *c == BAddCategory::Output).unwrap().1.iter().any(|t| t.idname() == "ShaderNodeOutputMaterial"));
}

#[test]
fn every_seed_type_is_well_formed() {
    assert!(!CATALOG.is_empty());
    for ty in CATALOG {
        assert!(!ty.idname().is_empty() && !ty.label().is_empty(), "{}", ty.idname());
        let decl = ty.sockets();
        for s in decl.inputs.iter().chain(&decl.outputs) {
            assert!(!s.name.is_empty() && !s.identifier.is_empty(), "{} has a nameless socket", ty.idname());
            assert!(
                [SOCK_FLOAT, SOCK_VECTOR, SOCK_RGBA, SOCK_SHADER, SOCK_BOOLEAN].contains(&s.kind),
                "{} socket {} has an unexpected kind {}",
                ty.idname(), s.name, s.kind
            );
        }
    }
    // idnames are unique — a duplicate would make `find` ambiguous.
    for (i, a) in CATALOG.iter().enumerate() {
        assert!(!CATALOG[i + 1..].iter().any(|b| b.idname() == a.idname()), "duplicate {}", a.idname());
    }
}
