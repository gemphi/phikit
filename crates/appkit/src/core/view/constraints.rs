use super::NSView;
use crate::graphics::{NSLayoutAttribute, NSLayoutRelation};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct WeakViewRef {
    pub view: Option<Arc<Mutex<NSView>>>,
}

impl WeakViewRef {
    pub fn new(view: &NSView) -> Self {
        // This is a stub for now as NSView doesn't easily expose an Arc to itself
        // unless we use a specific pattern.
        Self { view: None }
    }
}

#[derive(Clone, Debug)]
pub struct NSLayoutConstraint {
    pub first_item: WeakViewRef,
    pub first_attribute: NSLayoutAttribute,
    pub relation: NSLayoutRelation,
    pub second_item: Option<WeakViewRef>,
    pub second_attribute: NSLayoutAttribute,
    pub multiplier: f32,
    pub constant: f32,
    pub priority: f32,
}

impl NSLayoutConstraint {
    pub fn new(
        first: &NSView,
        first_attr: NSLayoutAttribute,
        relation: NSLayoutRelation,
        second: Option<&NSView>,
        second_attr: NSLayoutAttribute,
        multiplier: f32,
        constant: f32,
    ) -> Self {
        Self {
            first_item: WeakViewRef::new(first),
            first_attribute: first_attr,
            relation,
            second_item: second.map(|v| WeakViewRef::new(v)),
            second_attribute: second_attr,
            multiplier,
            constant,
            priority: 1000.0,
        }
    }

    pub fn apply(&self, first_view: &mut NSView) {
        let mut frame = first_view.frame();

        // Simplified solving logic (does not handle second_item yet)
        let value = self.constant;

        match self.first_attribute {
            NSLayoutAttribute::Leading => frame.origin.x = value,
            NSLayoutAttribute::Top => frame.origin.y = value,
            NSLayoutAttribute::Width => frame.size.width = value,
            NSLayoutAttribute::Height => frame.size.height = value,
            _ => {}
        }

        first_view.set_frame(frame);
    }

    pub fn leading(first: &NSView, second: &NSView, constant: f32) -> Self {
        Self::new(
            first,
            NSLayoutAttribute::Leading,
            NSLayoutRelation::Equal,
            Some(second),
            NSLayoutAttribute::Leading,
            1.0,
            constant,
        )
    }

    pub fn top(first: &NSView, second: &NSView, constant: f32) -> Self {
        Self::new(
            first,
            NSLayoutAttribute::Top,
            NSLayoutRelation::Equal,
            Some(second),
            NSLayoutAttribute::Top,
            1.0,
            constant,
        )
    }

    pub fn width(view: &NSView, constant: f32) -> Self {
        Self::new(
            view,
            NSLayoutAttribute::Width,
            NSLayoutRelation::Equal,
            None,
            NSLayoutAttribute::NotAnAttribute,
            1.0,
            constant,
        )
    }

    pub fn height(view: &NSView, constant: f32) -> Self {
        Self::new(
            view,
            NSLayoutAttribute::Height,
            NSLayoutRelation::Equal,
            None,
            NSLayoutAttribute::NotAnAttribute,
            1.0,
            constant,
        )
    }
}
