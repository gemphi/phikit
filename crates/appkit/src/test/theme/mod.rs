//! Tests of `theme`: the trait, through a theme made for the purpose.

mod look;
mod shade;

use crate::pui::core::PColor;
use crate::theme::*;

/// A theme for the tests: two tokens.
struct Day;

impl PTheme for Day {
    fn color(&self, token: &str) -> Option<PColor> {
        match token {
            "control.fill" => Some(PColor::new(0.9, 0.9, 0.9, 1.0)),
            _ => None,
        }
    }

    fn size(&self, token: &str) -> Option<f32> {
        match token {
            "control.radius" => Some(4.0),
            _ => None,
        }
    }
}

#[test]
fn a_token_the_theme_has_is_a_colour_or_a_size_and_another_is_nothing() {
    let theme: &dyn PTheme = &Day;
    assert_eq!(theme.color("control.fill"), Some(PColor::new(0.9, 0.9, 0.9, 1.0)));
    assert_eq!(theme.size("control.radius"), Some(4.0));
    assert_eq!((theme.color("no.such.token"), theme.size("control.fill")), (None, None));
}
