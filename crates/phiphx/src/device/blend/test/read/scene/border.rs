use super::*;

#[test]
fn the_view_s_own_border_reads_its_flag() {
    let v = BlendViewBorder::of(BlendViewBorder::V3D_RENDER_BORDER | 4, [0.5, 1.0, 0.0, 0.5]);
    assert_eq!(v, BlendViewBorder { on: true, rect: [0.5, 1.0, 0.0, 0.5] });
    // The lower right quarter of a 200 × 100 region: rows 50 ..< 100.
    assert_eq!(v.pixels(200, 100), Some((100, 50, 100, 50)));
    assert_eq!(BlendViewBorder::of(4, v.rect).pixels(200, 100), None);
    assert_eq!(BlendViewBorder::of(0, [0.0; 4]).rect, [0.0, 1.0, 0.0, 1.0], "a view that never had a box");
    assert_eq!(BlendViewBorder::default(), BlendViewBorder { on: false, rect: [0.0, 1.0, 0.0, 1.0] });
}

#[test]
fn the_region_becomes_a_pixel_rectangle_from_the_top() {
    let b = BlendRenderBorder { on: true, crop: false, rect: [0.25, 0.75, 0.5, 1.0] };
    // The top half's middle of a 200 × 100 frame: x 50 ..< 150, rows 0 ..< 50.
    assert_eq!(b.pixels(200, 100), Some((50, 0, 100, 50)));
    assert_eq!(BlendRenderBorder { on: false, ..b }.pixels(200, 100), None);
    assert_eq!(BlendRenderBorder { rect: [0.5, 0.5, 0.0, 1.0], ..b }.pixels(200, 100), None, "empty");
    assert_eq!(BlendRenderBorder::of(BlendRenderBorder::R_BORDER | BlendRenderBorder::R_CROP, [0.0, 1.0, 0.0, 1.0]), BlendRenderBorder { on: true, crop: true, rect: [0.0, 1.0, 0.0, 1.0] });
    assert_eq!(b.mode_over(BlendRenderBorder::R_CROP | 1), BlendRenderBorder::R_BORDER | 1, "the bits follow, the rest stays");
}
