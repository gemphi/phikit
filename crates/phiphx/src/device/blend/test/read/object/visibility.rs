use super::*;

#[test]
fn the_bits_are_blenders() {
    let object = BlendVisibility::of_object(OB_HIDE_SELECT | OB_HIDE_RENDER);
    assert_eq!(object, BlendVisibility { hide_viewport: false, hide_select: true, hide_render: true });
    let collection = BlendVisibility::of_collection(COLLECTION_HIDE_RENDER | 1 << 2);
    assert_eq!(collection, BlendVisibility { hide_viewport: false, hide_select: false, hide_render: true }, "1<<2 is the retired bit");
    assert_eq!(BlendShadowCast::of_object(0, 0), BlendShadowCast::default(), "casts both ways by default");
    assert_eq!(BlendShadowCast::of_object(1 << 9, 1 << 8), BlendShadowCast { display: false, ray: false });
    assert!(!BlendVisibility::default().any() && object.any());
}
