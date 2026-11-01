//! # NSLevelIndicator — Level Display Control
//!
//! Displays a level indicator for volume, capacity, or other continuous values.
//! Supports different visual styles including continuous, discrete, and rating indicators.
//!
//! ## Apple Documentation
//! - [NSLevelIndicator](https://developer.apple.com/documentation/appkit/nslevelindicator)
//! - [NSControl](https://developer.apple.com/documentation/appkit/nscontrol)
//!
//! ## Usage
//!
//! ```rust,ignore
//! // Create a continuous level indicator (like a volume meter)
//! let mut level = NSLevelIndicator::new(frame);
//! level.set_min_value(0.0);
//! level.set_max_value(100.0);
//! level.set_value(75.0);
//! level.set_level_indicator_style(NSLevelIndicatorStyle::Continuous);
//!
//! // Create a discrete level indicator (like a signal strength meter)
//! let mut signal = NSLevelIndicator::new(frame);
//! signal.set_min_value(0.0);
//! signal.set_max_value(5.0);
//! signal.set_value(3.0);
//! signal.set_level_indicator_style(NSLevelIndicatorStyle::Discrete);
//!
//! // Create a rating indicator (like a star rating)
//! let mut rating = NSLevelIndicator::new(frame);
//! rating.set_min_value(0.0);
//! rating.set_max_value(5.0);
//! rating.set_value(4.0);
//! rating.set_level_indicator_style(NSLevelIndicatorStyle::Rating);
//! ```
//!
//! ## See Also
//! - [NSProgressIndicator](https://developer.apple.com/documentation/appkit/nsprogressindicator)
//! - [NSSlider](https://developer.apple.com/documentation/appkit/nsslider)

use crate::view::NSView;
use crate::graphics::NSRect;
use std::sync::{Arc, Mutex};

/// Level indicator visual style.
///
/// Apple: [NSLevelIndicatorStyle](https://developer.apple.com/documentation/appkit/nslevelindicatorstyle)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NSLevelIndicatorStyle {
    /// Continuous bar indicator (like a volume meter).
    ///
    /// Apple: `NSLevelIndicatorStyleContinuousCapacity`
    Continuous,
    
    /// Discrete segmented indicator (like signal strength).
    ///
    /// Apple: `NSLevelIndicatorStyleDiscreteCapacity`
    Discrete,
    
    /// Rating indicator (like star ratings).
    ///
    /// Apple: `NSLevelIndicatorStyleRating`
    Rating,
    
    /// Relevancy indicator (like search relevance).
    ///
    /// Apple: `NSLevelIndicatorStyleRelevancy`
    Relevancy,
}

/// Internal state for the level indicator.
#[derive(Debug, Clone)]
pub(crate) struct LevelIndicatorState {
    pub(crate) value: f64,
    pub(crate) min_value: f64,
    pub(crate) max_value: f64,
    pub(crate) warning_value: f64,
    pub(crate) critical_value: f64,
    pub(crate) style: NSLevelIndicatorStyle,
    pub(crate) is_editable: bool,
}

impl Default for LevelIndicatorState {
    fn default() -> Self {
        Self {
            value: 0.0,
            min_value: 0.0,
            max_value: 100.0,
            warning_value: 75.0,
            critical_value: 90.0,
            style: NSLevelIndicatorStyle::Continuous,
            is_editable: false,
        }
    }
}

/// A level indicator control for displaying volume, capacity, or rating values.
///
/// Apple: [NSLevelIndicator](https://developer.apple.com/documentation/appkit/nslevelindicator)
#[derive(Clone)]
pub struct NSLevelIndicator {
    view: NSView,
    state: Arc<Mutex<LevelIndicatorState>>,
}

impl NSLevelIndicator {
    /// Create a new level indicator.
    ///
    /// Apple: `[[NSLevelIndicator alloc] initWithFrame:]`
    pub fn new(frame: NSRect) -> Self {
        let state = Arc::new(Mutex::new(LevelIndicatorState::default()));
        let view = NSView::new(frame).with_name("NSLevelIndicator");
        
        Self { view, state }
    }

    /// Create a continuous level indicator with specified range.
    ///
    /// # Example
    /// ```rust,ignore
    /// let volume = NSLevelIndicator::continuous(0.0, 100.0, 75.0);
    /// ```
    pub fn continuous(min: f64, max: f64, value: f64) -> Self {
        let mut indicator = Self::new(NSRect::sized(200.0, 16.0));
        indicator.set_min_value(min);
        indicator.set_max_value(max);
        indicator.set_value(value);
        indicator.set_level_indicator_style(NSLevelIndicatorStyle::Continuous);
        indicator
    }

    /// Create a discrete level indicator with specified range.
    ///
    /// # Example
    /// ```rust,ignore
    /// let signal = NSLevelIndicator::discrete(0.0, 5.0, 3.0);
    /// ```
    pub fn discrete(min: f64, max: f64, value: f64) -> Self {
        let mut indicator = Self::new(NSRect::sized(100.0, 16.0));
        indicator.set_min_value(min);
        indicator.set_max_value(max);
        indicator.set_value(value);
        indicator.set_level_indicator_style(NSLevelIndicatorStyle::Discrete);
        indicator
    }

    /// Create a rating indicator with specified maximum rating.
    ///
    /// # Example
    /// ```rust,ignore
    /// let rating = NSLevelIndicator::rating(5.0, 4.0);
    /// ```
    pub fn rating(max: f64, value: f64) -> Self {
        let mut indicator = Self::new(NSRect::sized(100.0, 20.0));
        indicator.set_min_value(0.0);
        indicator.set_max_value(max);
        indicator.set_value(value);
        indicator.set_level_indicator_style(NSLevelIndicatorStyle::Rating);
        indicator
    }

    // ── Value Management ─────────────────────────────────────────────────────

    /// Set the current value.
    ///
    /// The value is clamped to the range [min_value, max_value].
    ///
    /// Apple: `setDoubleValue:`
    pub fn set_value(&mut self, value: f64) {
        let mut state = self.state.lock().unwrap();
        state.value = value.clamp(state.min_value, state.max_value);
    }

    /// Get the current value.
    ///
    /// Apple: `doubleValue`
    pub fn value(&self) -> f64 {
        self.state.lock().unwrap().value
    }

    /// Set the minimum value.
    ///
    /// Apple: `setMinValue:`
    pub fn set_min_value(&mut self, min: f64) {
        let mut state = self.state.lock().unwrap();
        state.min_value = min;
        // Clamp current value to new range
        state.value = state.value.clamp(state.min_value, state.max_value);
    }

    /// Get the minimum value.
    ///
    /// Apple: `minValue`
    pub fn min_value(&self) -> f64 {
        self.state.lock().unwrap().min_value
    }

    /// Set the maximum value.
    ///
    /// Apple: `setMaxValue:`
    pub fn set_max_value(&mut self, max: f64) {
        let mut state = self.state.lock().unwrap();
        state.max_value = max;
        // Clamp current value to new range
        state.value = state.value.clamp(state.min_value, state.max_value);
    }

    /// Get the maximum value.
    ///
    /// Apple: `maxValue`
    pub fn max_value(&self) -> f64 {
        self.state.lock().unwrap().max_value
    }

    // ── Threshold Values ─────────────────────────────────────────────────────

    /// Set the warning threshold value.
    ///
    /// When the value exceeds this threshold, the indicator may display in a warning color.
    ///
    /// Apple: `setWarningValue:`
    pub fn set_warning_value(&mut self, warning: f64) {
        self.state.lock().unwrap().warning_value = warning;
    }

    /// Get the warning threshold value.
    ///
    /// Apple: `warningValue`
    pub fn warning_value(&self) -> f64 {
        self.state.lock().unwrap().warning_value
    }

    /// Set the critical threshold value.
    ///
    /// When the value exceeds this threshold, the indicator may display in a critical color.
    ///
    /// Apple: `setCriticalValue:`
    pub fn set_critical_value(&mut self, critical: f64) {
        self.state.lock().unwrap().critical_value = critical;
    }

    /// Get the critical threshold value.
    ///
    /// Apple: `criticalValue`
    pub fn critical_value(&self) -> f64 {
        self.state.lock().unwrap().critical_value
    }

    // ── Style and Appearance ─────────────────────────────────────────────────

    /// Set the level indicator style.
    ///
    /// Apple: `setLevelIndicatorStyle:`
    pub fn set_level_indicator_style(&mut self, style: NSLevelIndicatorStyle) {
        self.state.lock().unwrap().style = style;
    }

    /// Get the level indicator style.
    ///
    /// Apple: `levelIndicatorStyle`
    pub fn level_indicator_style(&self) -> NSLevelIndicatorStyle {
        self.state.lock().unwrap().style
    }

    /// Set whether the level indicator is editable.
    ///
    /// When editable, the user can click to change the value (useful for rating indicators).
    ///
    /// Apple: `setEditable:`
    pub fn set_editable(&mut self, editable: bool) {
        self.state.lock().unwrap().is_editable = editable;
    }

    /// Get whether the level indicator is editable.
    ///
    /// Apple: `isEditable`
    pub fn is_editable(&self) -> bool {
        self.state.lock().unwrap().is_editable
    }

    // ── View Access ──────────────────────────────────────────────────────────

    /// Get a reference to the underlying view.
    pub fn view(&self) -> &NSView {
        &self.view
    }

    /// Get a mutable reference to the underlying view.
    pub fn view_mut(&mut self) -> &mut NSView {
        &mut self.view
    }

    /// Clone the underlying view.
    pub fn view_clone(&self) -> NSView {
        self.view.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_indicator_creation() {
        let indicator = NSLevelIndicator::new(NSRect::sized(200.0, 16.0));
        assert_eq!(indicator.value(), 0.0);
        assert_eq!(indicator.min_value(), 0.0);
        assert_eq!(indicator.max_value(), 100.0);
    }

    #[test]
    fn test_value_clamping() {
        let mut indicator = NSLevelIndicator::new(NSRect::sized(200.0, 16.0));
        indicator.set_min_value(0.0);
        indicator.set_max_value(100.0);
        
        indicator.set_value(150.0);
        assert_eq!(indicator.value(), 100.0);
        
        indicator.set_value(-10.0);
        assert_eq!(indicator.value(), 0.0);
        
        indicator.set_value(50.0);
        assert_eq!(indicator.value(), 50.0);
    }

    #[test]
    fn test_continuous_factory() {
        let indicator = NSLevelIndicator::continuous(0.0, 100.0, 75.0);
        assert_eq!(indicator.value(), 75.0);
        assert_eq!(indicator.level_indicator_style(), NSLevelIndicatorStyle::Continuous);
    }

    #[test]
    fn test_discrete_factory() {
        let indicator = NSLevelIndicator::discrete(0.0, 5.0, 3.0);
        assert_eq!(indicator.value(), 3.0);
        assert_eq!(indicator.level_indicator_style(), NSLevelIndicatorStyle::Discrete);
    }

    #[test]
    fn test_rating_factory() {
        let indicator = NSLevelIndicator::rating(5.0, 4.0);
        assert_eq!(indicator.value(), 4.0);
        assert_eq!(indicator.min_value(), 0.0);
        assert_eq!(indicator.max_value(), 5.0);
        assert_eq!(indicator.level_indicator_style(), NSLevelIndicatorStyle::Rating);
    }

    #[test]
    fn test_threshold_values() {
        let mut indicator = NSLevelIndicator::new(NSRect::sized(200.0, 16.0));
        
        indicator.set_warning_value(75.0);
        assert_eq!(indicator.warning_value(), 75.0);
        
        indicator.set_critical_value(90.0);
        assert_eq!(indicator.critical_value(), 90.0);
    }

    #[test]
    fn test_editable() {
        let mut indicator = NSLevelIndicator::new(NSRect::sized(200.0, 16.0));
        assert_eq!(indicator.is_editable(), false);
        
        indicator.set_editable(true);
        assert_eq!(indicator.is_editable(), true);
    }
}
