//! # NSTokenField Delegate
//!
//! Delegate protocol for token field events and completion support.
//!
//! ## Apple Documentation
//! - [NSTokenFieldDelegate](https://developer.apple.com/documentation/appkit/nstokenfielddelegate)
//! - [NSTokenField](https://developer.apple.com/documentation/appkit/nstokenfield)

use crate::prelude::*;
use std::sync::{Arc, Mutex};

/// Delegate protocol for NSTokenField events.
///
/// Apple: [NSTokenFieldDelegate](https://developer.apple.com/documentation/appkit/nstokenfielddelegate)
pub trait NSTokenFieldDelegate: Send + Sync {
    /// Called when the token array changes.
    ///
    /// Apple: Equivalent to observing `objectValue` changes
    fn tokens_did_change(&mut self, tokens: Vec<String>) {
        let _ = tokens;
    }

    /// Provide completion suggestions for a substring.
    ///
    /// Apple: `tokenField:completionsForSubstring:indexOfToken:indexOfSelectedItem:`
    fn completions_for_substring(&self, substring: &str) -> Vec<String> {
        let _ = substring;
        Vec::new()
    }

    /// Called when a token is about to be added.
    /// Return `true` to allow, `false` to reject.
    ///
    /// Apple: `tokenField:shouldAddObjects:atIndex:`
    fn should_add_token(&mut self, token: &str) -> bool {
        let _ = token;
        true
    }

    /// Called when a token is about to be removed.
    /// Return `true` to allow, `false` to reject.
    fn should_remove_token(&mut self, token: &str, index: usize) -> bool {
        let _ = (token, index);
        true
    }

    /// Provide a display string for a token.
    ///
    /// Apple: `tokenField:displayStringForRepresentedObject:`
    fn display_string_for_token(&self, token: &str) -> String {
        token.to_string()
    }

    /// Called when editing begins.
    ///
    /// Apple: `controlTextDidBeginEditing:`
    fn did_begin_editing(&mut self) {}

    /// Called when editing ends.
    ///
    /// Apple: `controlTextDidEndEditing:`
    fn did_end_editing(&mut self) {}
}

/// Internal state for token field.
pub(crate) struct TokenFieldState {
    pub(crate) tokens: Vec<String>,
    pub(crate) current_input: String,
    pub(crate) placeholder: String,
    pub(crate) tokenizing_chars: Vec<char>,
    pub(crate) is_editing: bool,
    pub(crate) selected_token_index: Option<usize>,
    pub(crate) completions: Vec<String>,
}

impl Default for TokenFieldState {
    fn default() -> Self {
        Self {
            tokens: Vec::new(),
            current_input: String::new(),
            placeholder: String::new(),
            tokenizing_chars: vec![',', ';', '\n'],
            is_editing: false,
            selected_token_index: None,
            completions: Vec::new(),
        }
    }
}

/// Internal view delegate for NSTokenField rendering and events.
pub(crate) struct NSTokenFieldViewDelegate {
    pub(crate) state: Arc<Mutex<TokenFieldState>>,
    pub(crate) delegate: Arc<Mutex<Option<Box<dyn NSTokenFieldDelegate>>>>,
}

impl NSTokenFieldViewDelegate {
    /// Calculate the width needed for a token.
    fn token_width(&self, token: &str, context: &mut crate::render::RenderContext) -> f32 {
        // Measure text width (simplified - in real implementation would use proper text metrics)
        let char_width = 8.0; // Approximate character width
        let text_width = token.len() as f32 * char_width;
        let padding = 16.0; // Left + right padding
        let close_button = 16.0; // Close button width
        text_width + padding + close_button
    }

    /// Draw a single token.
    fn draw_token(
        &self,
        token: &str,
        x: f32,
        y: f32,
        is_selected: bool,
        context: &mut crate::render::RenderContext,
    ) -> f32 {
        let token_height = 20.0;
        let padding = 8.0;
        let close_button_size = 12.0;

        // Calculate token width
        let char_width = 8.0;
        let text_width = token.len() as f32 * char_width;
        let token_width = text_width + padding * 2.0 + close_button_size + 4.0;

        // Draw token background
        let bg_color = if is_selected {
            crate::theme::color(crate::theme::keys::CLR_ACCENT)
        } else {
            crate::theme::color(crate::theme::keys::CLR_BG_SELECTED)
        };
        context.set_color(bg_color);
        context.fill_rounded_rect(
            NSRect::new(x, y, token_width, token_height),
            token_height / 2.0,
        );

        // Draw token text
        let text_color = if is_selected {
            crate::theme::color(crate::theme::keys::CLR_TEXT_ON_ACCENT)
        } else {
            crate::theme::color(crate::theme::keys::CLR_TEXT_LABEL)
        };
        context.set_color(text_color);
        context.draw_text(token, NSPoint::new(x + padding, y + 4.0), 14.0);

        // Draw close button (×)
        let close_x = x + token_width - close_button_size - 4.0;
        let close_y = y + 4.0;
        context.draw_text("×", NSPoint::new(close_x, close_y), 14.0);

        token_width
    }

    /// Handle token creation from current input.
    fn create_token_from_input(&mut self) {
        let mut state = self.state.lock().unwrap();
        let input = state.current_input.trim().to_string();

        if input.is_empty() {
            return;
        }

        // Check with delegate if token should be added
        let should_add = if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
            delegate.should_add_token(&input)
        } else {
            true
        };

        if should_add {
            state.tokens.push(input);
            state.current_input.clear();

            let tokens = state.tokens.clone();
            drop(state);

            // Notify delegate
            if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
                delegate.tokens_did_change(tokens);
            }
        }
    }

    /// Handle token removal.
    fn remove_token_at_index(&mut self, index: usize) {
        let mut state = self.state.lock().unwrap();

        if index >= state.tokens.len() {
            return;
        }

        let token = state.tokens[index].clone();

        // Check with delegate if token should be removed
        let should_remove = if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
            delegate.should_remove_token(&token, index)
        } else {
            true
        };

        if should_remove {
            state.tokens.remove(index);

            let tokens = state.tokens.clone();
            drop(state);

            // Notify delegate
            if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
                delegate.tokens_did_change(tokens);
            }
        }
    }
}

impl NSViewDelegate for NSTokenFieldViewDelegate {
    fn draw(&self, view: &NSView, context: &mut crate::render::RenderContext) {
        let state = self.state.lock().unwrap();
        let frame = view.bounds();

        // Draw background
        let bg_color = if state.is_editing {
            crate::theme::color(crate::theme::keys::CLR_BG_CONTROL)
        } else {
            crate::theme::color(crate::theme::keys::CLR_BG_CONTROL)
        };
        context.set_color(bg_color);
        context.fill_rounded_rect(frame, 4.0);

        // Draw border
        let border_color = if state.is_editing {
            crate::theme::color(crate::theme::keys::CLR_ACCENT)
        } else {
            crate::theme::color(crate::theme::keys::CLR_BORDER)
        };
        context.set_color(border_color);
        context.stroke_rounded_rect(frame, 4.0, 1.0);

        // Draw tokens
        let mut x = frame.min_x() + 8.0;
        let y = frame.min_y() + 4.0;
        let token_spacing = 4.0;

        for (index, token) in state.tokens.iter().enumerate() {
            let is_selected = state.selected_token_index == Some(index);
            let token_width = self.draw_token(token, x, y, is_selected, context);
            x += token_width + token_spacing;
        }

        // Draw current input text
        if state.is_editing || !state.current_input.is_empty() {
            context.set_color(crate::theme::color(crate::theme::keys::CLR_TEXT_LABEL));
            context.draw_text(&state.current_input, NSPoint::new(x, y + 4.0), 14.0);

            // Draw cursor
            if state.is_editing {
                let cursor_x = x + (state.current_input.len() as f32 * 8.0);
                context.set_color(crate::theme::color(crate::theme::keys::CLR_TEXT_LABEL));
                context.fill_rect(NSRect::new(cursor_x, y + 2.0, 1.0, 16.0));
            }
        } else if state.tokens.is_empty() && !state.placeholder.is_empty() {
            // Draw placeholder if no tokens and no input
            context.set_color(crate::theme::color(crate::theme::keys::CLR_TEXT_SECONDARY));
            context.draw_text(&state.placeholder, NSPoint::new(x, y + 4.0), 14.0);
        }

        // Draw completions dropdown if available
        if !state.completions.is_empty() && state.is_editing {
            let dropdown_y = frame.max_y() + 2.0;
            let dropdown_height = (state.completions.len() as f32 * 24.0).min(120.0);
            let dropdown_rect = NSRect::new(
                frame.min_x(),
                dropdown_y,
                frame.width(),
                dropdown_height,
            );

            // Draw dropdown background
            context.set_color(crate::theme::color(crate::theme::keys::CLR_BG_CONTROL));
            context.fill_rounded_rect(dropdown_rect, 4.0);
            context.set_color(crate::theme::color(crate::theme::keys::CLR_BORDER));
            context.stroke_rounded_rect(dropdown_rect, 4.0, 1.0);

            // Draw completion items
            let mut item_y = dropdown_y + 4.0;
            for completion in state.completions.iter().take(5) {
                context.set_color(crate::theme::color(crate::theme::keys::CLR_TEXT_LABEL));
                context.draw_text(completion, NSPoint::new(frame.min_x() + 8.0, item_y), 14.0);
                item_y += 24.0;
            }
        }
    }

    fn mouse_down(&mut self, view: &mut NSView, event: &crate::event::NSEvent) {
        let mut state = self.state.lock().unwrap();
        state.is_editing = true;

        let frame = view.frame();
        let click_pos = event.location_in_window();

        // Check if clicked on a token's close button
        let mut x = frame.min_x() + 8.0;
        let y = frame.min_y() + 4.0;
        let token_spacing = 4.0;

        for (index, token) in state.tokens.clone().iter().enumerate() {
            let char_width = 8.0;
            let text_width = token.len() as f32 * char_width;
            let padding = 8.0;
            let close_button_size = 12.0;
            let token_width = text_width + padding * 2.0 + close_button_size + 4.0;

            // Check if clicked on close button
            let close_x = x + token_width - close_button_size - 4.0;
            let close_rect = NSRect::new(close_x, y, close_button_size, 20.0);

            if close_rect.contains_point(&click_pos) {
                drop(state);
                self.remove_token_at_index(index);
                return;
            }

            // Check if clicked on token body (for selection)
            let token_rect = NSRect::new(x, y, token_width, 20.0);
            if token_rect.contains_point(&click_pos) {
                state.selected_token_index = Some(index);
                return;
            }

            x += token_width + token_spacing;
        }

        // Clicked in empty space - deselect tokens
        state.selected_token_index = None;

        // Notify delegate
        drop(state);
        if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
            delegate.did_begin_editing();
        }
    }

    fn mouse_up(&mut self, _view: &mut NSView, _event: &crate::event::NSEvent) {
        // Keep editing state active
    }

    fn key_down(&mut self, _view: &mut NSView, event: &crate::event::NSEvent) {
        if let Some(chars) = event.characters() {
            let mut state = self.state.lock().unwrap();

            // Check for tokenizing characters
            if chars.len() == 1 {
                let ch = chars.chars().next().unwrap();

                if state.tokenizing_chars.contains(&ch) {
                    drop(state);
                    self.create_token_from_input();
                    return;
                }
            }

            // Handle special keys
            match chars.as_str() {
                "Backspace" => {
                    if state.current_input.is_empty() {
                        // Remove last token if input is empty
                        if let Some(index) = state.selected_token_index {
                            drop(state);
                            self.remove_token_at_index(index);
                        } else if !state.tokens.is_empty() {
                            let last_index = state.tokens.len() - 1;
                            drop(state);
                            self.remove_token_at_index(last_index);
                        }
                    } else {
                        state.current_input.pop();
                    }
                }
                "Enter" | "Return" => {
                    drop(state);
                    self.create_token_from_input();
                }
                "Escape" => {
                    state.current_input.clear();
                    state.completions.clear();
                }
                "ArrowLeft" => {
                    if state.current_input.is_empty() && !state.tokens.is_empty() {
                        // Navigate tokens
                        if let Some(index) = state.selected_token_index {
                            if index > 0 {
                                state.selected_token_index = Some(index - 1);
                            }
                        } else {
                            state.selected_token_index = Some(state.tokens.len() - 1);
                        }
                    }
                }
                "ArrowRight" => {
                    if let Some(index) = state.selected_token_index {
                        if index < state.tokens.len() - 1 {
                            state.selected_token_index = Some(index + 1);
                        } else {
                            state.selected_token_index = None;
                        }
                    }
                }
                _ => {
                    // Add character to input
                    state.current_input.push_str(&chars);

                    // Request completions
                    let input = state.current_input.clone();
                    drop(state);

                    if !input.is_empty() {
                        if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
                            let completions = delegate.completions_for_substring(&input);
                            self.state.lock().unwrap().completions = completions;
                        }
                    }
                }
            }
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
