//! The caret-bearing text of the field being typed into — a gpui `Element`
//! that shapes the field's line, draws the selection, caret and IME-marked
//! underline, registers the window's text-input handler for this field's
//! bounds, and captures the shaped line and bounds back onto the window entity
//! so the platform can place an IME popup and map a click to an offset.
//!
//! Ported from gpui's `examples/input.rs` `TextElement`, adapted to phiber: the
//! text is the field's `BState` value (passed in), and the caret lives on
//! `BPhiberApp` (`app::input`), not on a per-field entity.

use crate::v2::app::BPhiberApp;
use super::color::to_hsla;
use super::ctx::BRenderCtx;
use gpui::{
    App, Bounds, Element, ElementId, ElementInputHandler, Entity, FocusHandle, GlobalElementId,
    Hsla, LayoutId, PaintQuad, Pixels, ShapedLine, SharedString, Style, TextRun, UnderlineStyle,
    Window, fill, point, prelude::*, px, relative, size,
};

/// The text content of the active field, with its caret. Built through
/// [`BFieldText::of`] by whichever control holds the caret.
pub struct BFieldText {
    pub app: Entity<BPhiberApp>,
    pub focus: FocusHandle,
    /// The field's current value (the same string the caret offsets index).
    pub content: SharedString,
    pub placeholder: SharedString,
    pub text: Hsla,
    pub placeholder_color: Hsla,
    pub caret: Hsla,
    pub selection: Hsla,
    pub font_size: Pixels,
    /// The caret's own width, and the IME underline's thickness: the theme's
    /// hairline, as every other one-pixel rule in the app takes it. They were
    /// written here as `px(1.5)` and `px(1.0)` — the two places the field drew
    /// a line whose weight nothing else could reach.
    pub hairline: Pixels,
}

pub struct BFieldPrepaint {
    line: Option<ShapedLine>,
    cursor: Option<PaintQuad>,
    selection: Option<PaintQuad>,
}

impl BFieldText {
    /// The caret-bearing element for whatever field holds the caret, dressed
    /// from the theme.
    ///
    /// `None` when the pass has no window behind it — a headless render or a
    /// test — which is the caller's cue to draw the plain `value|` fallback
    /// instead. Both the search field and the number button build their typed
    /// state through this: the search field had the only copy, which is why
    /// the number button, having no copy, could not be typed into at all.
    pub fn of(value: &str, placeholder: &str, ctx: &BRenderCtx) -> Option<Self> {
        let (app, focus) = ctx.input.clone()?;
        let mut selection = ctx.style.colors.accent;
        selection.3 = 0.30;
        let mut placeholder_color = ctx.style.colors.text;
        placeholder_color.3 = (placeholder_color.3 * 0.45).clamp(0.0, 1.0);
        Some(Self {
            app,
            focus,
            content: SharedString::from(value.to_string()),
            placeholder: SharedString::from(placeholder.to_string()),
            text: to_hsla(ctx.style.colors.text),
            placeholder_color: to_hsla(placeholder_color),
            caret: to_hsla(ctx.style.colors.accent),
            selection: to_hsla(selection),
            font_size: px(ctx.style.typography.body),
            hairline: px(ctx.style.strokes.hairline),
        })
    }
}

impl IntoElement for BFieldText {
    type Element = Self;
    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for BFieldText {
    type RequestLayoutState = ();
    type PrepaintState = BFieldPrepaint;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = window.line_height().into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let app = self.app.read(cx);
        let caret = app.input.caret.clone();
        let selected = caret.selected.clone();
        let cursor = caret.cursor();
        let marked = caret.marked.clone();

        let (display, color) = if self.content.is_empty() {
            (self.placeholder.clone(), self.placeholder_color)
        } else {
            (self.content.clone(), self.text)
        };

        let font = window.text_style().font();
        let run = TextRun {
            len: display.len(),
            font,
            color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        // An IME composition underlines its marked span.
        let runs = match marked.as_ref().filter(|_| !self.content.is_empty()) {
            Some(m) if m.end <= display.len() => vec![
                TextRun { len: m.start, ..run.clone() },
                TextRun {
                    len: m.end - m.start,
                    underline: Some(UnderlineStyle { color: Some(run.color), thickness: self.hairline, wavy: false }),
                    ..run.clone()
                },
                TextRun { len: display.len() - m.end, ..run.clone() },
            ]
            .into_iter()
            .filter(|r| r.len > 0)
            .collect(),
            _ => vec![run],
        };

        let line = window.text_system().shape_line(display, self.font_size, &runs, None);

        // A caret when the selection is empty, else a highlight over the span.
        let (selection, cursor_quad) = if selected.is_empty() {
            let x = line.x_for_index(cursor.min(self.content.len()));
            (
                None,
                Some(fill(
                    Bounds::new(
                        point(bounds.left() + x, bounds.top()),
                        size(self.hairline, bounds.bottom() - bounds.top()),
                    ),
                    self.caret,
                )),
            )
        } else {
            let s = line.x_for_index(selected.start.min(self.content.len()));
            let e = line.x_for_index(selected.end.min(self.content.len()));
            (
                Some(fill(
                    Bounds::from_corners(
                        point(bounds.left() + s, bounds.top()),
                        point(bounds.left() + e, bounds.bottom()),
                    ),
                    self.selection,
                )),
                None,
            )
        };

        BFieldPrepaint { line: Some(line), cursor: cursor_quad, selection }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        window.handle_input(&self.focus, ElementInputHandler::new(bounds, self.app.clone()), cx);
        if let Some(selection) = prepaint.selection.take() {
            window.paint_quad(selection);
        }
        let Some(line) = prepaint.line.take() else { return };
        let _ = line.paint(bounds.origin, window.line_height(), gpui::TextAlign::Left, None, window, cx);
        if self.focus.is_focused(window)
            && let Some(cursor) = prepaint.cursor.take()
        {
            window.paint_quad(cursor);
        }
        // Hand the line and bounds to the window entity for IME placement and
        // click-to-offset (`app::input`'s `bounds_for_range` / mouse mapping).
        self.app.update(cx, |app, _| {
            app.input.last_line = Some(line);
            app.input.last_bounds = Some(bounds);
        });
    }
}
