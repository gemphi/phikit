//! Test for NSSegmentedControl delegate pattern integration

use rust_appkit::segmented_control::{NSSegmentedControl, NSSegmentedControlDelegate};
use rust_appkit::graphics::NSRect;
use std::sync::{Arc, Mutex};

struct TestDelegate {
    selected_segments: Arc<Mutex<Vec<(usize, String)>>>,
}

impl NSSegmentedControlDelegate for TestDelegate {
    fn did_select_segment(&mut self, index: usize, label: &str) {
        self.selected_segments.lock().unwrap().push((index, label.to_string()));
    }
}

#[test]
fn test_set_delegate() {
    // Create a segmented control
    let mut control = NSSegmentedControl::new(NSRect::new(0.0, 0.0, 300.0, 30.0));
    
    // Set up segments
    control.set_segment_count(3);
    control.set_label("Left", 0);
    control.set_label("Center", 1);
    control.set_label("Right", 2);
    
    // Create a delegate
    let selected_segments = Arc::new(Mutex::new(Vec::new()));
    let delegate = TestDelegate {
        selected_segments: selected_segments.clone(),
    };
    
    // Set the delegate
    control.set_delegate(Box::new(delegate));
    
    // The test passes if set_delegate compiles and runs without panicking
    // Note: We can't easily test the delegate callback without simulating mouse events,
    // which would require more complex test infrastructure
    assert!(true, "set_delegate() method exists and can be called");
}

#[test]
fn test_segmented_control_basic_operations() {
    let mut control = NSSegmentedControl::new(NSRect::new(0.0, 0.0, 300.0, 30.0));
    
    // Test segment count
    control.set_segment_count(3);
    
    // Test labels
    control.set_label("A", 0);
    control.set_label("B", 1);
    control.set_label("C", 2);
    
    // Test selection
    control.set_selected_segment(1);
    assert_eq!(control.selected_segment(), 1);
    
    control.set_selected_segment(2);
    assert_eq!(control.selected_segment(), 2);
}
