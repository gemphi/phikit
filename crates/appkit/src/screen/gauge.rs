//! How wide a line of text is set, remembered. Setting a line to learn
//! its width is the dearest thing a frame does, and a frame asks for the
//! same lines as the one before it, many times over: every width is set
//! once and kept, for the face and size it was set in. A look that
//! changes the face or the size changes the key, so nothing is kept
//! wrong. The theme says how many widths are kept at most
//! (`screen.memo`); past that they are all let go and set again.

use std::cell::RefCell;
use std::collections::HashMap;

use gpui::Window;

use super::play::Type;
use crate::theme::PTheme;

/// The theme's token for how many widths are kept at most.
pub const MEMO: &str = "screen.memo";

/// The widths set so far: by the type they were set in and the text.
#[derive(Default)]
pub struct Gauge {
    kept: RefCell<HashMap<(String, String), f32>>,
}

impl Gauge {
    /// How wide a line is set in that face: kept, or set now and kept.
    pub fn width(&self, text: &str, face: &Type, theme: &dyn PTheme, window: &Window) -> f32 {
        let key = (face.key(), text.to_string());
        let kept = self.kept.borrow().get(&key).copied();
        kept.unwrap_or_else(|| {
            let wide = face.width(text, window);
            let most = theme.size(MEMO).unwrap_or(f32::MAX) as usize;
            let mut all = self.kept.borrow_mut();
            (all.len() >= most).then(|| all.clear());
            all.insert(key, wide);
            wide
        })
    }
}
