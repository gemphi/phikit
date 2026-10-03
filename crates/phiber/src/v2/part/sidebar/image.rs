//! The Image and UV editors' sidebar panels — `bl_ui/space_image.py`.
//!
//! Blender groups the N panel by `bl_category`, and both editors are the
//! same space (`SpaceImageEditor`) in two modes, so they share one table:
//!
//! | tab | panels |
//! |---|---|
//! | Image | `IMAGE_PT_image_properties` |
//! | Tool | `IMAGE_PT_tools_active` |
//! | View | `IMAGE_PT_view_display`, `IMAGE_PT_uv_cursor`, `IMAGE_PT_udim_grid` |
//!
//! The tabs were on the strip already and opened nothing. What is new here
//! is only the content — the tab set in `set.rs` is unchanged apart from
//! naming these.
//!
//! RNA names are `Image` (`rna_image.cc`), `SpaceImageEditor` and
//! `SpaceUVEditor` (`rna_space.cc`). `IMAGE_PT_image_properties` draws
//! `template_image`, which is the datablock's source, its file, its colour
//! space and its alpha handling; the rows here are those, spelled out,
//! because a template is a shorthand for a fixed set of properties and this
//! tree declares its rows.

use crate::v2::part::props::prop::{BEnumItem, BPropSpec, BUnit, b, e, en, f, i, s};
use crate::v2::part::props::set::{BSectionSpec, sec};

/// `Image.source` — where the pixels come from.
static SOURCE: &[BEnumItem] = &[
    en("FILE", "Single Image"),
    en("SEQUENCE", "Image Sequence"),
    en("MOVIE", "Movie"),
    en("GENERATED", "Generated"),
    en("VIEWER", "Viewer Node"),
    en("TILED", "UDIM Tiles"),
];

/// `Image.alpha_mode` — how the alpha channel is read.
static ALPHA: &[BEnumItem] = &[
    en("STRAIGHT", "Straight"),
    en("PREMUL", "Premultiplied"),
    en("CHANNEL_PACKED", "Channel Packed"),
    en("NONE", "None"),
];

/// `IMAGE_PT_image_properties` — `template_image`, spelled out.
static IMAGE_PROPS: &[BPropSpec] = &[
    e("image.source", "Source", SOURCE),
    s("image.filepath", "File Path"),
    s("image.colorspace_settings.name", "Color Space"),
    e("image.alpha_mode", "Alpha", ALPHA),
    b("image.use_view_as_render", "View as Render"),
];

/// `IMAGE_PT_view_display`: the image's aspect, whether it repeats outside
/// its bounds, and — in UV mode — whether the cursor reads in pixels.
static DISPLAY: &[BPropSpec] = &[
    f("image.display_aspect", "Aspect Ratio", BUnit::None),
    b("show_repeat", "Repeat Image"),
    b("uv_editor.show_pixel_coords", "Pixel Coordinates"),
];

/// `IMAGE_PT_uv_cursor`: `SpaceImageEditor.cursor_location`, the 2D cursor
/// the UV editor pivots and snaps around.
static CURSOR: &[BPropSpec] = &[
    f("cursor_location.x", "Location X", BUnit::None),
    f("cursor_location.y", "Location Y", BUnit::None),
];

/// `IMAGE_PT_udim_grid`: `SpaceUVEditor.tile_grid_shape`, how many UDIM
/// tiles the grid draws. Closed by default, as Blender has it.
static UDIM: &[BPropSpec] = &[
    i("uv_editor.tile_grid_shape.x", "Grid Shape X", 1, 10),
    i("uv_editor.tile_grid_shape.y", "Grid Shape Y", 1, 10),
];

/// The Image tab.
pub static IMAGE: &[BSectionSpec] = &[sec("Image", IMAGE_PROPS)];

/// The View tab, in Blender's order.
pub static VIEW: &[BSectionSpec] = &[
    sec("Display", DISPLAY),
    sec("2D Cursor", CURSOR),
    UDIM_GRID,
];

const UDIM_GRID: BSectionSpec = sec("UDIM Grid", UDIM).closed();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_row_names_an_rna_property_and_a_label() {
        for section in IMAGE.iter().chain(VIEW) {
            assert!(!section.title.is_empty());
            assert!(!section.props.is_empty(), "{} draws nothing", section.title);
            for prop in section.props {
                assert!(!prop.key.is_empty(), "{} has a row with no key", section.title);
                assert!(!prop.label.is_empty(), "{} has a row with no label", section.title);
            }
        }
    }

    #[test]
    fn the_udim_grid_is_closed_as_blender_has_it() {
        assert!(UDIM_GRID.closed);
        assert!(!IMAGE[0].closed, "the image itself opens");
    }
}
