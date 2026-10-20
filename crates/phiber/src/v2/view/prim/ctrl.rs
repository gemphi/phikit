//! Controls — every widget that edits one state value.
//!
//! Blender's `uiBut` types, as one vocabulary: a checkbox, a toggle, a
//! radio row, a dropdown, a number, a slider, a text field, a search field,
//! a colour swatch, a path field. Each control names the key it edits, the
//! category of value there (`BValueKind`), and the value it holds now; the
//! one renderer (`view/draw/atom/ctrl.rs`) draws it by kind, and
//! every edit is `BAction::Set(key, value.encode())`. Nothing else knows how
//! a value is stored, so the render engine, a sample count and a background
//! colour are set the same way from a property panel, a menu or a script.
//!
//! A `Button`, a `Tab` and a `MenuItem` are *not* controls: they run an
//! action but hold no value (`BControl`).

use crate::v2::state::{BKey as _};
use super::BAction;
use crate::v2::state::{BPath, BState, val::{BValue, BValueKind}};
use crate::v2::view::BView;

/// One option of an enum control.
#[derive(Clone, Debug, PartialEq)]
pub struct BOption {
    pub key: String,
    pub label: String,
    pub icon: String,
}

/// What kind of widget edits the value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BCtrlKind {
    /// A checkbox: a box and the label — `Bool`.
    Check,
    /// A switch, as Apple draws one: the label fills the row and a pill track
    /// sits at its end, the knob slid to the side it is on — `Bool`. The
    /// properties' and popovers' rows (`BView::check_row`), so a column of
    /// them lines up whatever each label says.
    Switch,
    /// A dropdown: the chosen option, a menu of the rest — `Enum`.
    Select,
    /// A whole number with a step either side — `Int`.
    Int { min: i64, max: i64 },
    /// A number with a step either side — `Float`.
    Number { min: f32, max: f32, step: f32 },
    /// A number drawn as a bar of its range — `Float`.
    Slider { min: f32, max: f32 },
    /// A line of text — `Text`.
    Text,
    /// A line of text that filters a list — `Text`.
    Search,
    /// A swatch and its hex — `Color`.
    Color,
    /// A file or directory — `Path`.
    Path,
}

impl BCtrlKind {
    /// The category of value each widget edits.
    pub const fn value_kind(self) -> BValueKind {
        match self {
            Self::Check | Self::Switch => BValueKind::Bool,
            Self::Select => BValueKind::Enum,
            Self::Int { .. } => BValueKind::Int,
            Self::Number { .. } | Self::Slider { .. } => BValueKind::Float,
            Self::Text | Self::Search => BValueKind::Text,
            Self::Color => BValueKind::Color,
            Self::Path => BValueKind::Path,
        }
    }
}

/// A control: which widget, which key, what it holds.
#[derive(Clone, Debug, PartialEq)]
pub struct BCtrl {
    pub kind: BCtrlKind,
    pub key: BPath,
    pub value: BValue,
    pub label: Option<String>,
    pub tooltip: Option<String>,
    /// The options of a `Select`.
    pub options: Vec<BOption>,
    /// Typed controls: whether the caret is in this one.
    pub editing: bool,
    /// A `Select`: whether its menu is open, and the key that says so.
    pub open: bool,
    pub open_key: Option<BPath>,
}

impl BCtrl {
    /// A control for `key`, reading its value from the state or falling back
    /// to `default` when the state has none (or holds something that is not
    /// of this kind).
    pub fn read(kind: BCtrlKind, key: BPath, default: BValue, s: &BState) -> Self {
        let value = s.value(&key, kind.value_kind()).unwrap_or(default);
        let editing = crate::v2::state::edit::is_editing(s, &key);
        let ctrl = Self {
            kind,
            key,
            value,
            label: None,
            tooltip: None,
            options: Vec::new(),
            editing,
            open: false,
            open_key: None,
        };
        // A colour opens its picker from its swatch, and the picker has to
        // record that it is open somewhere. Without a key nothing could, so
        // no colour control anywhere — Preferences, Properties, the Shading
        // popover, New Image — ever opened one: the swatch was a picture of
        // a button. The key sits under the colour's own, so two colours never
        // share one and opening one is not opening another.
        match kind {
            BCtrlKind::Color => {
                let open_key = Self::picker_key(&ctrl.key);
                ctrl.with_menu(open_key, s)
            }
            _ => ctrl,
        }
    }

    /// Where a colour control records that its picker is open.
    fn picker_key(key: &BPath) -> BPath {
        let mut segs = key.segs.to_vec();
        segs.push("picker");
        BPath::owned(segs)
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn with_tooltip(mut self, tip: impl Into<String>) -> Self {
        self.tooltip = Some(tip.into());
        self
    }

    pub fn with_options(mut self, options: impl IntoIterator<Item = BOption>) -> Self {
        self.options = options.into_iter().collect();
        self
    }

    /// A `Select` reads whether it is open from `open_key`.
    pub fn with_menu(mut self, open_key: BPath, s: &BState) -> Self {
        self.open = s.get(&open_key) == Some(crate::v2::state::val::V_TRUE);
        self.open_key = Some(open_key);
        self
    }

    /// The write for a new value of this control's key.
    pub fn set(&self, value: BValue) -> BAction {
        BAction::Set(self.key.clone(), value.encode())
    }

    /// The write that flips a `Bool` control.
    pub fn flip(&self) -> BAction {
        let on = matches!(self.value, BValue::Bool(true));
        self.set(BValue::Bool(!on))
    }

    /// The write that moves a number control by `steps` steps, clamped.
    pub fn nudge(&self, steps: f32) -> Option<BAction> {
        let (min, max, step) = match self.kind {
            // Blender's number button steps by the property's own step,
            // which is wider on a wider property. Ours stepped by ONE on
            // every integer row, so Max Samples — range 0..100000 — moved
            // a single sample a click: getting from 1024 to 64 by the
            // arrows is 960 clicks. Four on a wide row, one where the
            // range is small enough that single units are the point (a
            // frame number, a count of a handful).
            BCtrlKind::Int { min, max } => {
                let step = if (max - min) > 100 { 4.0 } else { 1.0 };
                (min as f32, max as f32, step)
            }
            BCtrlKind::Number { min, max, step } => (min, max, step),
            BCtrlKind::Slider { min, max } => (min, max, (max - min) / 100.0),
            _ => return None,
        };
        let now = match self.value {
            BValue::Float(v) => v,
            BValue::Int(v) => v as f32,
            _ => min,
        };
        let next = (now + steps * step).clamp(min, max);
        Some(self.set(match self.value {
            BValue::Int(_) => BValue::Int(next.round() as i64),
            _ => BValue::Float(next),
        }))
    }

    /// The write that puts the caret in a typed control.
    pub fn focus(&self) -> BAction {
        BAction::Set(crate::v2::state::AppKey::Edit.path(), self.key.encode())
    }

    /// The label of the chosen option, for `Radio` and `Select`.
    pub fn chosen(&self) -> Option<&BOption> {
        let BValue::Enum(key) = &self.value else { return None };
        self.options.iter().find(|o| &o.key == key)
    }
}

impl BView {
    pub fn ctrl(ctrl: BCtrl) -> Self {
        Self::Ctrl(Box::new(ctrl))
    }
}
