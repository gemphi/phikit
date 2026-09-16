//! Accordion primitive — the Properties section heading. The entire row owns
//! one toggle action; its disclosure chip is visual only, avoiding nested
//! duplicate dispatch.

use crate::v2::view::BView;
use crate::v2::view::prim::BAction;

impl BView {
    pub fn accordion_header(title: impl Into<String>, expanded: bool, action: BAction) -> Self {
        Self::AccordionHeader {
            title: title.into(),
            expanded,
            action,
        }
    }
}
