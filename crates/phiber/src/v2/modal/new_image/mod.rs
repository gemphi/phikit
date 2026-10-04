//! New Image — `image.new`'s dialog (`image_new_draw`, `IMAGE_OT_new`):
//! Name, Width and Height (1024 × 1024), Generated Type (Blank, UV Grid,
//! Color Grid), the Color for a blank one (black), Alpha on; Float and
//! Tiled are not offered (every image here is 8-bit, untiled). OK makes
//! the image (`scene::image_list::BImage::generated`) and shows it.
//! Opening is `ctx.modal.open = new_image`, the fields `image.new.*`;
//! `BNewImageRule` takes the OK.

use crate::v2::scene::image_list::{BGenerated, BImage, BImageList};
use crate::v2::state::{BKey as _, BPath, BState, ImageKey, MenuKey, ModalKey, log, rule::BRule, val::{BValue, BValueKind, V_EMPTY, V_MODAL_NEW_IMAGE, V_TRUE}};
use crate::v2::view::prim::{BAction, BControl, BCtrl, BCtrlKind, BOption};
use crate::v2::view::{BView, BViewItem};

pub const TITLE: &str = "New Image";
/// `IMAGE_OT_new`'s defaults: `DATA_("Untitled")`, 1024, black, alpha on, Blank.
pub const DEFAULT_NAME: &str = "Untitled";
pub const DEFAULT_SIZE: i64 = 1024;

#[derive(Clone, Copy, Debug, Default)]
pub struct BNewImageModal;

impl BNewImageModal {
    fn kind(s: &BState) -> BGenerated {
        BGenerated::of_name(s.get(ImageKey::New("type")).unwrap_or(BGenerated::NAME_LIST[0]))
    }
}

impl BViewItem for BNewImageModal {
    fn build(&self, s: &BState) -> BView {
        let name = s.get(ImageKey::New("name")).unwrap_or(DEFAULT_NAME).to_string();
        let field = BView::text_field(DEFAULT_NAME, name, crate::v2::state::edit::is_editing(s, ImageKey::New("name")), BAction::Set(ImageKey::New("name").path(), String::new()));
        let size = |field: &'static str, label: &str| BView::prop_row("", BCtrl::read(BCtrlKind::Int { min: 1, max: 16_384 }, ImageKey::New(field).path(), BValue::Int(DEFAULT_SIZE), s).with_label(label));
        let kind = BNewImageModal::kind(s);
        let types = BCtrl::read(BCtrlKind::Select, ImageKey::New("type").path(), BValue::Enum(BGenerated::NAME_LIST[0].to_string()), s)
            .with_options(BGenerated::NAME_LIST.iter().map(|n| BOption { key: n.to_string(), label: n.to_string(), icon: String::new() }))
            .with_menu(MenuKey::Top("new_image", "type").path(), s);
        let mut rows = vec![BView::split(vec![BView::label("Name"), field]), size("width", "Width"), size("height", "Height"), BView::prop_row("", types.with_label("Generated Type"))];
        if kind == BGenerated::Blank {
            rows.push(BView::prop_row("", BCtrl::read(BCtrlKind::Color, ImageKey::New("color").path(), BValue::Color([0.0, 0.0, 0.0, 1.0]), s).with_label("Color")));
        }
        rows.push(BView::check_row(s, "", ImageKey::New("alpha").path(), true, "Alpha"));
        rows.push(BView::row(vec![BView::spacer(), BView::Button(BControl::new().with_label("OK").with_action(BAction::Set(ImageKey::New("commit").path(), V_TRUE.to_string())))]));
        BView::column(rows)
    }
}

/// `image.new.commit`: the image made as the fields say, shown, and the
/// dialog closed; the fields cleared for the next.
pub struct BNewImageRule;

impl BRule for BNewImageRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == ImageKey::New("commit").path()
    }
    fn after(&self, path: &BPath, _: &str, s: &mut BState) {
        s.remove(path);
        if s.get(ModalKey::Open) != Some(V_MODAL_NEW_IMAGE) {
            return;
        }
        let name = s.get(ImageKey::New("name")).filter(|n| !n.trim().is_empty()).unwrap_or(DEFAULT_NAME).trim().to_string();
        let size = |field: &'static str| match s.value(&ImageKey::New(field).path(), BValueKind::Int) {
            Some(BValue::Int(v)) => v.clamp(1, 16_384) as u32,
            _ => DEFAULT_SIZE as u32,
        };
        let color = match s.value(&ImageKey::New("color").path(), BValueKind::Color) {
            Some(BValue::Color(c)) => c,
            _ => [0.0, 0.0, 0.0, 1.0],
        };
        let alpha = s.get(ImageKey::New("alpha")).is_none_or(|v| v == V_TRUE);
        let image = BImage::generated(&name, size("width"), size("height"), color, alpha, BNewImageModal::kind(s));
        let made = BImageList::add(image);
        log::push(format!("New image '{made}'"));
        s.set(ImageKey::Shown, made);
        for field in ["name", "width", "height", "type", "color", "alpha"] {
            s.remove(ImageKey::New(field));
        }
        crate::v2::state::edit::stop(s);
        s.set(ModalKey::Open, V_EMPTY);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ok_makes_the_image_as_the_fields_say_and_shows_it() {
        let mut s = BState::new();
        s.set(ModalKey::Open, V_MODAL_NEW_IMAGE);
        s.set(ImageKey::New("name"), "Grid");
        s.set(ImageKey::New("width"), BValue::Int(64).encode());
        s.set(ImageKey::New("height"), BValue::Int(32).encode());
        s.set(ImageKey::New("type"), "UV Grid");
        BNewImageRule.after(&ImageKey::New("commit").path(), V_TRUE, &mut s);
        let shown = s.get(ImageKey::Shown).unwrap_or("").to_string();
        assert!(shown.starts_with("Grid"), "{shown}");
        let image = BImageList::get(&shown).expect("made");
        assert_eq!((image.w, image.h, image.bgra.len()), (64, 32, 64 * 32 * 4));
        assert_eq!(s.get(ModalKey::Open), Some(V_EMPTY), "the dialog closes");
        assert!(s.get(ImageKey::New("name")).is_none(), "the fields are cleared");
        // Defaults: Untitled, 1024², blank black, alpha on — and a second
        // Untitled is `.001`.
        s.set(ModalKey::Open, V_MODAL_NEW_IMAGE);
        BNewImageRule.after(&ImageKey::New("commit").path(), V_TRUE, &mut s);
        let first = s.get(ImageKey::Shown).unwrap_or("").to_string();
        let image = BImageList::get(&first).expect("made");
        assert_eq!((image.w, image.h), (1024, 1024));
        assert_eq!(&image.bgra[..4], &[0, 0, 0, 255]);
        s.set(ModalKey::Open, V_MODAL_NEW_IMAGE);
        BNewImageRule.after(&ImageKey::New("commit").path(), V_TRUE, &mut s);
        let second = s.get(ImageKey::Shown).unwrap_or("").to_string();
        assert_ne!(first, second);
        assert!(second.starts_with("Untitled"), "{second}");
    }
}
