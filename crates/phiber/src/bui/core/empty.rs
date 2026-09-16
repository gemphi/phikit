//! EmptyView — zero-size empty element.

use gpui::prelude::*;
use gpui::{AnyElement, Div, div};

pub struct EmptyView;

impl EmptyView {
    pub fn new() -> Self {
        Self
    }

    pub fn with_div() -> Div {
        div()
    }

    pub fn into_elm() -> AnyElement {
        div().into_any_element()
    }
}
