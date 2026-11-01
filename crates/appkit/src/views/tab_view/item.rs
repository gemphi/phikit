use crate::NSView;

#[derive(Clone)]
pub struct NSTabViewItem {
    pub(crate) label: String,
    pub(crate) view: NSView,
}

impl NSTabViewItem {
    pub fn new(label: &str, view: NSView) -> Self {
        Self {
            label: label.to_string(),
            view,
        }
    }
    pub fn label(&self) -> &str {
        &self.label
    }
    pub fn view(&self) -> &NSView {
        &self.view
    }
}
