//! A sheet: a painter that keeps what was drawn on it, as a list of marks
//! in the order they were made. It is the seam between `pui` and whatever
//! puts a picture on a screen: a view draws on a sheet, and the screen
//! plays the marks (`src/screen`). gpui's `Scene` is the model.

use super::{PColor, PFit, PGlass, PPaint, PPoint, PRect, PSize};
use crate::theme::PTheme;

/// One thing drawn.
#[derive(Debug, Clone, PartialEq)]
pub enum PMark {
    /// A rectangle filled with a colour, its corners rounded by `radius`.
    Fill { rect: PRect, radius: f32, color: PColor },
    /// A rounded rectangle that is `from` at one side and `to` at the
    /// other: `turn` says where `to` lies, in degrees, 0 over `from` and
    /// 90 to its right.
    Blend { rect: PRect, radius: f32, from: PColor, to: PColor, turn: f32 },
    /// A pane of glass: what was drawn before it shows through, blurred.
    Glass { rect: PRect, radius: f32, glass: PGlass },
    /// A line of text with its top left at `at`.
    Text { at: PPoint, text: String, color: PColor },
    /// A heading with its top left at `at`: text in the heading's size.
    Head { at: PPoint, text: String, color: PColor },
    /// The icon of that name, fitted to `rect` and tinted.
    Icon { name: String, rect: PRect, color: PColor },
    /// The picture of that name in `rect`, its corners rounded, lying
    /// as `fit` says.
    Picture { name: String, rect: PRect, radius: f32, fit: PFit },
    /// What follows, up to the `Unclip` that ends it, is kept inside the
    /// rectangle.
    Clip(PRect),
    Unclip,
}

/// What tells a sheet how large the picture of a name is, when it has it.
pub type PKnown<'a> = &'a dyn Fn(&str) -> Option<PSize>;

/// A painter that keeps what was drawn on it.
pub struct PSheet<'a> {
    theme: &'a dyn PTheme,
    /// How wide a line of text is set, when the screen can say.
    gauge: Option<&'a dyn Fn(&str) -> f32>,
    /// How large a picture is, when the screen has it.
    known: Option<PKnown<'a>>,
    /// What was drawn, the first mark the lowest.
    pub mark: Vec<PMark>,
}

impl<'a> PSheet<'a> {
    /// An empty sheet in that theme. Text is measured by counting.
    pub fn new(theme: &'a dyn PTheme) -> PSheet<'a> {
        PSheet { theme, gauge: None, known: None, mark: Vec::new() }
    }

    /// An empty sheet whose text is measured by the screen it is for.
    pub fn gauged(theme: &'a dyn PTheme, gauge: &'a dyn Fn(&str) -> f32) -> PSheet<'a> {
        PSheet { theme, gauge: Some(gauge), known: None, mark: Vec::new() }
    }

    /// The same sheet, told how large each picture is by what has them.
    pub fn sized(mut self, known: PKnown<'a>) -> PSheet<'a> {
        self.known = Some(known);
        self
    }

    /// Every line of text and every heading drawn, in order.
    pub fn text_list(&self) -> Vec<&str> {
        let text = self.mark.iter().filter_map(|mark| match mark {
            PMark::Text { text, .. } | PMark::Head { text, .. } => Some(text.as_str()),
            _ => None,
        });
        text.collect()
    }
}

impl PPaint for PSheet<'_> {
    fn theme(&self) -> &dyn PTheme {
        self.theme
    }

    fn fill(&mut self, rect: PRect, color: PColor) {
        self.round(rect, 0.0, color);
    }

    fn round(&mut self, rect: PRect, radius: f32, color: PColor) {
        self.mark.push(PMark::Fill { rect, radius, color });
    }

    fn blend(&mut self, rect: PRect, radius: f32, from: PColor, to: PColor, turn: f32) {
        self.mark.push(PMark::Blend { rect, radius, from, to, turn });
    }

    fn glass(&mut self, rect: PRect, radius: f32) {
        let glass = PGlass::of(self.theme);
        self.mark.extend(glass.map(|glass| PMark::Glass { rect, radius, glass }));
    }

    fn text(&mut self, at: PPoint, text: &str, color: PColor) {
        self.mark.push(PMark::Text { at, text: text.to_string(), color });
    }

    fn head(&mut self, at: PPoint, text: &str, color: PColor) {
        self.mark.push(PMark::Head { at, text: text.to_string(), color });
    }

    fn icon(&mut self, name: &str, rect: PRect, color: PColor) {
        self.mark.push(PMark::Icon { name: name.to_string(), rect, color });
    }

    fn picture(&mut self, name: &str, rect: PRect, radius: f32, fit: PFit) {
        self.mark.push(PMark::Picture { name: name.to_string(), rect, radius, fit });
    }

    fn natural(&self, name: &str) -> Option<PSize> {
        self.known.and_then(|known| known(name))
    }

    fn clip(&mut self, rect: PRect) {
        self.mark.push(PMark::Clip(rect));
    }

    fn unclip(&mut self) {
        self.mark.push(PMark::Unclip);
    }

    fn measure(&self, text: &str) -> f32 {
        match self.gauge {
            Some(gauge) => gauge(text),
            None => text.chars().count() as f32 * self.size("text.advance"),
        }
    }
}
