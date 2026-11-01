//! Icon rendering — loads SVG files from assets/icons/ and rasterises via resvg.
//!
//! Each IconId maps to a .svg file embedded at compile time. The SVG is rendered
//! through NSImage::new_with_svg() which handles theme-aware tinting (currentColor
//! is replaced with the active theme's CLR_TEXT_LABEL), anti-aliased output via
//! resvg/tiny-skia, and proper scaling to any requested size.
//!
//! Fallback: if an icon has no SVG file yet, the old procedural renderer is used.

mod cache;
mod procedural;
mod rasterize;
pub mod svg_map;

pub use cache::{get, get_icon_pixels, get_sized, NSIconCache};
pub use procedural::{render_icon_procedural, NSIconProcedural};
pub use rasterize::{render_icon_to_buffer, NSIconRasterizer};
pub use svg_map::{icon_name, svg_data, NSIconSvgMap};

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::IconId;

    /// Verify all toolbar action icons (select, move, rotate, scale, camera)
    /// have SVG data embedded from assets/icons/ and that the resvg rasterizer
    /// produces non-empty pixel buffers.
    ///
    /// Validates: Requirements 9.1
    #[test]
    fn toolbar_icons_load_and_rasterize_non_empty() {
        let toolbar_icons = vec![
            IconId::Select,
            IconId::Move,
            IconId::Rotate,
            IconId::Scale,
            IconId::Camera,
        ];

        for icon_id in &toolbar_icons {
            // 1. Verify SVG data exists for this icon
            let svg_bytes = svg_data(icon_id)
                .unwrap_or_else(|| panic!("SVG data missing for toolbar icon {:?}", icon_id));
            assert!(!svg_bytes.is_empty(), "SVG data is empty for {:?}", icon_id);

            // 2. Verify the SVG content is valid XML with an <svg element
            let svg_str = std::str::from_utf8(svg_bytes)
                .unwrap_or_else(|_| panic!("SVG data for {:?} is not valid UTF-8", icon_id));
            assert!(
                svg_str.contains("<svg"),
                "SVG data for {:?} missing <svg element",
                icon_id
            );

            // 3. Verify resvg rasterization produces non-empty pixel buffer
            //    get_icon_pixels returns 2x size for retina display (Req 9.2)
            let logical_size: u32 = 24;
            let retina_size = logical_size * 2;
            let (pixels, w, h) = get_icon_pixels(icon_id, logical_size);
            assert_eq!(
                w, retina_size,
                "Icon {:?} width should be 2x logical size",
                icon_id
            );
            assert_eq!(
                h, retina_size,
                "Icon {:?} height should be 2x logical size",
                icon_id
            );
            assert_eq!(
                pixels.len(),
                (retina_size * retina_size * 4) as usize,
                "Icon {:?} pixel buffer should be (2N)*(2N)*4 bytes",
                icon_id
            );

            // 4. Verify at least some pixels are non-zero (icon rendered something)
            let has_content = pixels.iter().any(|&p| p != 0);
            assert!(
                has_content,
                "Icon {:?} rendered an entirely blank buffer",
                icon_id
            );
        }
    }

    /// Verify SVG data is present for each toolbar icon individually.
    #[test]
    fn svg_data_exists_for_select() {
        assert!(svg_data(&IconId::Select).is_some());
    }

    #[test]
    fn svg_data_exists_for_move() {
        assert!(svg_data(&IconId::Move).is_some());
    }

    #[test]
    fn svg_data_exists_for_rotate() {
        assert!(svg_data(&IconId::Rotate).is_some());
    }

    #[test]
    fn svg_data_exists_for_scale() {
        assert!(svg_data(&IconId::Scale).is_some());
    }

    #[test]
    fn svg_data_exists_for_camera() {
        assert!(svg_data(&IconId::Camera).is_some());
    }

    /// Verify that rasterizing at different sizes produces correctly sized buffers.
    /// get_icon_pixels returns 2x the requested size for retina display.
    #[test]
    fn icon_rasterization_respects_requested_size() {
        let sizes: &[u32] = &[16, 20, 24, 32, 48];
        for &size in sizes {
            let retina_size = size * 2;
            let (pixels, w, h) = get_icon_pixels(&IconId::Select, size);
            assert_eq!(w, retina_size, "Width mismatch for logical size {}", size);
            assert_eq!(h, retina_size, "Height mismatch for logical size {}", size);
            assert_eq!(
                pixels.len(),
                (retina_size * retina_size * 4) as usize,
                "Buffer size mismatch for icon logical size {}",
                size
            );
            // Non-empty check
            let has_content = pixels.iter().any(|&p| p != 0);
            assert!(has_content, "Icon rendered blank at logical size {}", size);
        }
    }
}
