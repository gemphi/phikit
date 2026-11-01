//! Text editing operations: insertion, deletion, cursor movement, and responder management.

use super::field_types::NSTextField;

impl NSTextField {
    /// Insert text at current caret position
    pub fn insert_text(&mut self, text: &str) {
        let mut inner = self.inner.lock().unwrap();
        if inner.editable {
            let pos = inner.caret_position;
            inner.text.insert_str(pos, text);
            inner.caret_position += text.len();
            // Clear selection
            inner.selection_start = inner.caret_position;
            inner.selection_end = inner.caret_position;
            // Trigger action callback
            let action_clone = inner.action.clone();
            let text_clone = inner.text.clone();
            drop(inner); // Release lock before calling action
            if let Some(action) = action_clone {
                action(text_clone);
            }
        }
    }

    /// Delete backward (backspace)
    pub fn delete_backward(&mut self) {
        let mut inner = self.inner.lock().unwrap();
        if inner.editable && inner.caret_position > 0 {
            let pos = inner.caret_position;
            inner.text.remove(pos - 1);
            inner.caret_position -= 1;
            inner.selection_start = inner.caret_position;
            inner.selection_end = inner.caret_position;
            let action_clone = inner.action.clone();
            let text_clone = inner.text.clone();
            drop(inner);
            if let Some(action) = action_clone {
                action(text_clone);
            }
        }
    }

    /// Delete forward (delete key)
    pub fn delete_forward(&mut self) {
        let mut inner = self.inner.lock().unwrap();
        if inner.editable && inner.caret_position < inner.text.len() {
            let pos = inner.caret_position;
            inner.text.remove(pos);
            let action_clone = inner.action.clone();
            let text_clone = inner.text.clone();
            drop(inner);
            if let Some(action) = action_clone {
                action(text_clone);
            }
        }
    }

    /// Move cursor left
    pub fn move_left(&mut self) {
        let mut inner = self.inner.lock().unwrap();
        if inner.caret_position > 0 {
            inner.caret_position -= 1;
            let pos = inner.caret_position;
            inner.selection_start = pos;
            inner.selection_end = pos;
        }
    }

    /// Move cursor right
    pub fn move_right(&mut self) {
        let mut inner = self.inner.lock().unwrap();
        let text_len = inner.text.len();
        if inner.caret_position < text_len {
            inner.caret_position += 1;
            let pos = inner.caret_position;
            inner.selection_start = pos;
            inner.selection_end = pos;
        }
    }

    /// Move to beginning of line
    pub fn move_to_beginning(&mut self) {
        let mut inner = self.inner.lock().unwrap();
        inner.caret_position = 0;
        inner.selection_start = 0;
        inner.selection_end = 0;
    }

    /// Move to end of line
    pub fn move_to_end(&mut self) {
        let mut inner = self.inner.lock().unwrap();
        let len = inner.text.len();
        inner.caret_position = len;
        inner.selection_start = len;
        inner.selection_end = len;
    }

    /// Handle enter/return key
    pub fn insert_newline(&mut self) {
        let inner = self.inner.lock().unwrap();
        if inner.editable {
            // Trigger action callback on enter (commit)
            let action_clone = inner.action.clone();
            let text_clone = inner.text.clone();
            drop(inner);
            if let Some(action) = action_clone {
                action(text_clone);
            }
        }
    }

    /// Accept first responder status
    pub fn become_first_responder(&mut self) -> bool {
        let mut inner = self.inner.lock().unwrap();
        inner.is_first_responder = true;
        true
    }

    /// Resign first responder status
    pub fn resign_first_responder(&mut self) -> bool {
        let mut inner = self.inner.lock().unwrap();
        inner.is_first_responder = false;
        true
    }

    /// Check if currently first responder
    pub fn is_first_responder(&self) -> bool {
        self.inner.lock().unwrap().is_first_responder
    }

    /// Check if can become first responder (editable text fields can)
    pub fn accepts_first_responder(&self) -> bool {
        self.inner.lock().unwrap().editable
    }

    /// Get current caret position
    pub fn caret_position(&self) -> usize {
        self.inner.lock().unwrap().caret_position
    }

    /// Set caret position
    pub fn set_caret_position(&mut self, pos: usize) {
        let mut inner = self.inner.lock().unwrap();
        let len = inner.text.len();
        inner.caret_position = pos.min(len);
    }
}
