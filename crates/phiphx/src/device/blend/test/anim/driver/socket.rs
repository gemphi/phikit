use crate::device::blend::anim::driver::socket::socket_path;

#[test]
fn a_socket_path_is_a_node_name_and_an_input_index() {
    assert_eq!(
        socket_path("nodes[\"Principled BSDF\"].inputs[2].default_value"),
        Some(("Principled BSDF".to_string(), 2))
    );
    assert_eq!(socket_path("nodes[\"Background\"].inputs[1].default_value"), Some(("Background".to_string(), 1)));
    assert_eq!(socket_path("nodes[\"Noise Texture\"].inputs[11].default_value"), Some(("Noise Texture".to_string(), 11)));
}

#[test]
fn a_path_that_is_not_a_socket_is_not_read_as_one() {
    assert_eq!(socket_path("nodes[\"Mapping\"].rotation"), None, "a node's own property");
    assert_eq!(socket_path("roughness"), None, "the datablock's own property");
    assert_eq!(socket_path("nodes[\"RGB\"].outputs[0].default_value"), None, "an output cannot be driven");
    assert_eq!(socket_path("nodes[\"N\"].inputs[].default_value"), None);
    assert_eq!(socket_path("nodes[\"N\"].inputs[x].default_value"), None);
    assert_eq!(socket_path("nodes[\"\"].inputs[0].default_value"), None);
}

#[test]
fn the_node_name_ends_at_its_own_quote() {
    assert_eq!(
        socket_path("nodes[\"a].inputs[0\"].inputs[3].default_value"),
        Some(("a].inputs[0".to_string(), 3))
    );
}
