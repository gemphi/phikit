//! NSUndoManager — Standard Apple Undo/Redo Engine
//!
//! Mirrors Apple's NSUndoManager architectural pattern exactly.

use std::sync::{Arc, Mutex};

/// Trait for actions that can be undone.
/// Generic over the state type `T` that the action modifies.
pub trait NSReversibleAction<T>: Send + Sync {
    fn apply(&self, state: &mut T);
    fn undo(&self, state: &mut T);
    fn name(&self) -> String;
}

/// Manages the history of actions for Undo/Redo (Apple: NSUndoManager)
pub struct NSUndoManager<T> {
    undo_stack: Vec<Box<dyn NSReversibleAction<T>>>,
    redo_stack: Vec<Box<dyn NSReversibleAction<T>>>,
}

impl<T> NSUndoManager<T> {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    /// Register an undoable action and execute it immediately.
    pub fn register_undo(&mut self, action: Box<dyn NSReversibleAction<T>>, state: &mut T) {
        action.apply(state);
        self.undo_stack.push(action);
        self.redo_stack.clear(); // Standard behavior: new action clears redo
    }

    pub fn undo(&mut self, state: &mut T) {
        if let Some(action) = self.undo_stack.pop() {
            action.undo(state);
            self.redo_stack.push(action);
        }
    }

    pub fn redo(&mut self, state: &mut T) {
        if let Some(action) = self.redo_stack.pop() {
            action.apply(state);
            self.undo_stack.push(action);
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn undo_action_name(&self) -> Option<String> {
        self.undo_stack.last().map(|a| format!("Undo {}", a.name()))
    }

    pub fn redo_action_name(&self) -> Option<String> {
        self.redo_stack.last().map(|a| format!("Redo {}", a.name()))
    }
}

impl<T> Default for NSUndoManager<T> {
    fn default() -> Self {
        Self::new()
    }
}
