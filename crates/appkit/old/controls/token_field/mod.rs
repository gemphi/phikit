//! # NSTokenField — Token and Tag Input Control
//!
//! A text field that displays tokens (tags) as discrete, removable items.
//! Commonly used for email recipients, tags, and other multi-value inputs.
//!
//! ## Apple Documentation
//! - [NSTokenField](https://developer.apple.com/documentation/appkit/nstokenfield)
//! - [NSTokenFieldDelegate](https://developer.apple.com/documentation/appkit/nstokenfielddelegate)
//!
//! ## Usage
//!
//! ```rust,ignore
//! let mut token_field = NSTokenField::new(frame);
//! token_field.add_token("Design");
//! token_field.add_token("Development");
//! token_field.add_token("Testing");
//! token_field.set_delegate(Box::new(MyTokenFieldDelegate));
//! ```
//!
//! ## Delegate Pattern
//!
//! ```rust,ignore
//! struct MyDelegate;
//!
//! impl NSTokenFieldDelegate for MyDelegate {
//!     fn tokens_did_change(&mut self, tokens: Vec<String>) {
//!         println!("Tokens: {:?}", tokens);
//!     }
//!     
//!     fn completions_for_substring(&self, substring: &str) -> Vec<String> {
//!         // Return completion suggestions
//!         vec!["Design".to_string(), "Development".to_string()]
//!     }
//! }
//! ```
//!
//! ## See Also
//! - [NSTokenField](https://developer.apple.com/documentation/appkit/nstokenfield)
//! - [NSTextField](https://developer.apple.com/documentation/appkit/nstextfield)
//! - [NSControl](https://developer.apple.com/documentation/appkit/nscontrol)

pub mod delegate;

pub use delegate::NSTokenFieldDelegate;

use crate::control::NSControl;
use crate::graphics::NSRect;
use crate::view::NSView;
use delegate::{NSTokenFieldViewDelegate, TokenFieldState};
use std::sync::{Arc, Mutex};

/// Token field control for tag and token input.
///
/// Apple: [NSTokenField](https://developer.apple.com/documentation/appkit/nstokenfield)
#[derive(Clone)]
pub struct NSTokenField {
    control: NSControl,
    state: Arc<Mutex<TokenFieldState>>,
    delegate: Arc<Mutex<Option<Box<dyn NSTokenFieldDelegate>>>>,
}

impl NSTokenField {
    /// Create a new token field.
    ///
    /// Apple: `[[NSTokenField alloc] initWithFrame:]`
    pub fn new(frame: NSRect) -> Self {
        let state = Arc::new(Mutex::new(TokenFieldState::default()));
        let delegate = Arc::new(Mutex::new(None));

        let mut control = NSControl::new(frame);
        control.view_mut().set_delegate(NSTokenFieldViewDelegate {
            state: state.clone(),
            delegate: delegate.clone(),
        });

        Self {
            control,
            state,
            delegate,
        }
    }

    /// Create a token field with default styling.
    pub fn with_tokens(tokens: Vec<String>) -> Self {
        let mut field = Self::new(NSRect::sized(300.0, 28.0));
        for token in tokens {
            field.add_token(&token);
        }
        field
    }

    // ── Token Management ─────────────────────────────────────────────

    /// Add a token to the field.
    ///
    /// Apple: Part of `objectValue` array manipulation
    pub fn add_token(&mut self, token: &str) {
        let mut state = self.state.lock().unwrap();
        if !token.is_empty() {
            state.tokens.push(token.to_string());
            state.current_input.clear();
            
            let tokens = state.tokens.clone();
            drop(state);
            
            // Notify delegate
            if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
                delegate.tokens_did_change(tokens);
            }
        }
    }

    /// Remove a token at the specified index.
    ///
    /// Apple: Part of `objectValue` array manipulation
    pub fn remove_token(&mut self, index: usize) {
        let mut state = self.state.lock().unwrap();
        if index < state.tokens.len() {
            state.tokens.remove(index);
            
            let tokens = state.tokens.clone();
            drop(state);
            
            // Notify delegate
            if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
                delegate.tokens_did_change(tokens);
            }
        }
    }

    /// Remove all tokens from the field.
    ///
    /// Apple: `setObjectValue:` with empty array
    pub fn remove_all_tokens(&mut self) {
        let mut state = self.state.lock().unwrap();
        state.tokens.clear();
        
        let tokens = state.tokens.clone();
        drop(state);
        
        // Notify delegate
        if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
            delegate.tokens_did_change(tokens);
        }
    }

    /// Get all tokens.
    ///
    /// Apple: `objectValue`
    pub fn tokens(&self) -> Vec<String> {
        self.state.lock().unwrap().tokens.clone()
    }

    /// Set all tokens at once.
    ///
    /// Apple: `setObjectValue:`
    pub fn set_tokens(&mut self, tokens: Vec<String>) {
        let mut state = self.state.lock().unwrap();
        state.tokens = tokens.clone();
        
        drop(state);
        
        // Notify delegate
        if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
            delegate.tokens_did_change(tokens);
        }
    }

    /// Get the number of tokens.
    pub fn token_count(&self) -> usize {
        self.state.lock().unwrap().tokens.len()
    }

    /// Get a token at the specified index.
    pub fn token_at_index(&self, index: usize) -> Option<String> {
        self.state
            .lock()
            .unwrap()
            .tokens
            .get(index)
            .map(|s| s.clone())
    }

    // ── Input Management ─────────────────────────────────────────────

    /// Get the current input text (not yet converted to a token).
    ///
    /// Apple: Part of `stringValue`
    pub fn current_input(&self) -> String {
        self.state.lock().unwrap().current_input.clone()
    }

    /// Set the current input text.
    pub fn set_current_input(&mut self, text: &str) {
        self.state.lock().unwrap().current_input = text.to_string();
    }

    /// Clear the current input text.
    pub fn clear_current_input(&mut self) {
        self.state.lock().unwrap().current_input.clear();
    }

    // ── Configuration ────────────────────────────────────────────────

    /// Set the placeholder text shown when the field is empty.
    ///
    /// Apple: `setPlaceholderString:`
    pub fn set_placeholder_string(&mut self, placeholder: &str) {
        self.state.lock().unwrap().placeholder = placeholder.to_string();
    }

    /// Get the placeholder text.
    ///
    /// Apple: `placeholderString`
    pub fn placeholder_string(&self) -> String {
        self.state.lock().unwrap().placeholder.clone()
    }

    /// Set the tokenizing character set (characters that trigger token creation).
    ///
    /// Apple: `setTokenizingCharacterSet:`
    pub fn set_tokenizing_characters(&mut self, chars: Vec<char>) {
        self.state.lock().unwrap().tokenizing_chars = chars;
    }

    /// Get the tokenizing character set.
    ///
    /// Apple: `tokenizingCharacterSet`
    pub fn tokenizing_characters(&self) -> Vec<char> {
        self.state.lock().unwrap().tokenizing_chars.clone()
    }

    /// Set whether the field is currently editing.
    pub fn set_is_editing(&mut self, is_editing: bool) {
        self.state.lock().unwrap().is_editing = is_editing;
    }

    /// Check if the field is currently being edited.
    pub fn is_editing(&self) -> bool {
        self.state.lock().unwrap().is_editing
    }

    /// Set the selected token index (for keyboard navigation).
    pub fn set_selected_token_index(&mut self, index: Option<usize>) {
        self.state.lock().unwrap().selected_token_index = index;
    }

    /// Get the selected token index.
    pub fn selected_token_index(&self) -> Option<usize> {
        self.state.lock().unwrap().selected_token_index
    }

    // ── Completion Support ───────────────────────────────────────────

    /// Show completion suggestions for the current input.
    pub fn show_completions(&mut self) {
        let state = self.state.lock().unwrap();
        let input = state.current_input.clone();
        drop(state);
        
        if !input.is_empty() {
            if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
                let completions = delegate.completions_for_substring(&input);
                self.state.lock().unwrap().completions = completions;
            }
        }
    }

    /// Get the current completion suggestions.
    pub fn completions(&self) -> Vec<String> {
        self.state.lock().unwrap().completions.clone()
    }

    /// Clear completion suggestions.
    pub fn clear_completions(&mut self) {
        self.state.lock().unwrap().completions.clear();
    }

    // ── Delegate ─────────────────────────────────────────────────────

    /// Set the delegate for token field events.
    ///
    /// Apple: `setDelegate:`
    pub fn set_delegate(&mut self, delegate: Box<dyn NSTokenFieldDelegate>) {
        *self.delegate.lock().unwrap() = Some(delegate);
    }

    // ── View Access ──────────────────────────────────────────────────

    pub fn view(&self) -> NSView {
        self.control.view().clone()
    }

    pub fn view_mut(&mut self) -> &mut NSView {
        self.control.view_mut()
    }

    pub fn view_clone(&self) -> NSView {
        self.control.view().clone()
    }
}
