//! Integration tests for NSStepper delegate pattern

use rust_appkit::controls::stepper::{NSStepper, NSStepperDelegate};
use rust_appkit::graphics::NSRect;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct TestDelegate {
    value_changes: Arc<Mutex<Vec<f64>>>,
    increments: Arc<Mutex<Vec<f64>>>,
    decrements: Arc<Mutex<Vec<f64>>>,
}

impl TestDelegate {
    fn new() -> Self {
        Self {
            value_changes: Arc::new(Mutex::new(Vec::new())),
            increments: Arc::new(Mutex::new(Vec::new())),
            decrements: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl NSStepperDelegate for TestDelegate {
    fn value_did_change(&mut self, value: f64) {
        self.value_changes.lock().unwrap().push(value);
    }

    fn did_increment(&mut self, new_value: f64) {
        self.increments.lock().unwrap().push(new_value);
    }

    fn did_decrement(&mut self, new_value: f64) {
        self.decrements.lock().unwrap().push(new_value);
    }
}

#[test]
fn test_stepper_set_delegate() {
    let mut stepper = NSStepper::new(NSRect::sized(50.0, 30.0));
    let delegate = TestDelegate::new();
    
    // Test that set_delegate() method exists and can be called
    stepper.set_delegate(Box::new(delegate.clone()));
    
    // Verify initial value
    assert_eq!(stepper.value(), 0.0);
}

#[test]
fn test_stepper_delegate_increment() {
    let mut stepper = NSStepper::new(NSRect::sized(50.0, 30.0));
    stepper.set_min_value(0.0);
    stepper.set_max_value(10.0);
    stepper.set_increment(1.0);
    stepper.set_value(5.0);
    
    let delegate = TestDelegate::new();
    let value_changes = delegate.value_changes.clone();
    let increments = delegate.increments.clone();
    
    stepper.set_delegate(Box::new(delegate));
    
    // Increment the value
    stepper.increment_value();
    
    // Verify the value changed
    assert_eq!(stepper.value(), 6.0);
    
    // Verify delegate was called
    assert_eq!(value_changes.lock().unwrap().len(), 1);
    assert_eq!(value_changes.lock().unwrap()[0], 6.0);
    assert_eq!(increments.lock().unwrap().len(), 1);
    assert_eq!(increments.lock().unwrap()[0], 6.0);
}

#[test]
fn test_stepper_delegate_decrement() {
    let mut stepper = NSStepper::new(NSRect::sized(50.0, 30.0));
    stepper.set_min_value(0.0);
    stepper.set_max_value(10.0);
    stepper.set_increment(1.0);
    stepper.set_value(5.0);
    
    let delegate = TestDelegate::new();
    let value_changes = delegate.value_changes.clone();
    let decrements = delegate.decrements.clone();
    
    stepper.set_delegate(Box::new(delegate));
    
    // Decrement the value
    stepper.decrement_value();
    
    // Verify the value changed
    assert_eq!(stepper.value(), 4.0);
    
    // Verify delegate was called
    assert_eq!(value_changes.lock().unwrap().len(), 1);
    assert_eq!(value_changes.lock().unwrap()[0], 4.0);
    assert_eq!(decrements.lock().unwrap().len(), 1);
    assert_eq!(decrements.lock().unwrap()[0], 4.0);
}

#[test]
fn test_stepper_delegate_wrapping() {
    let mut stepper = NSStepper::new(NSRect::sized(50.0, 30.0));
    stepper.set_min_value(0.0);
    stepper.set_max_value(10.0);
    stepper.set_increment(1.0);
    stepper.set_value(10.0);
    stepper.set_wraps(true);
    
    let delegate = TestDelegate::new();
    let value_changes = delegate.value_changes.clone();
    
    stepper.set_delegate(Box::new(delegate));
    
    // Increment past max should wrap to min
    stepper.increment_value();
    
    // Verify wrapping occurred
    assert_eq!(stepper.value(), 0.0);
    assert_eq!(value_changes.lock().unwrap()[0], 0.0);
}
