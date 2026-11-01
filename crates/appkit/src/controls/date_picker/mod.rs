//! # NSDatePicker — Date and Time Selection Control
//!
//! Date picker control for selecting dates and times with various display modes.
//!
//! ## Apple Documentation
//! - [NSDatePicker](https://developer.apple.com/documentation/appkit/nsdatepicker)
//! - [NSDatePickerCell](https://developer.apple.com/documentation/appkit/nsdatepickercell)
//!
//! ## Usage
//!
//! ```rust,ignore
//! use chrono::Local;
//!
//! let mut date_picker = NSDatePicker::new(frame);
//! date_picker.set_date_picker_mode(NSDatePickerMode::Single);
//! date_picker.set_date_picker_style(NSDatePickerStyle::TextFieldAndStepper);
//! date_picker.set_date(Local::now());
//! date_picker.set_delegate(Box::new(MyDatePickerDelegate));
//! ```
//!
//! ## Delegate Pattern
//!
//! ```rust,ignore
//! struct MyDelegate;
//!
//! impl NSDatePickerDelegate for MyDelegate {
//!     fn date_did_change(&mut self, date: DateTime<Local>) {
//!         println!("Date changed to: {}", date);
//!     }
//! }
//! ```
//!
//! ## See Also
//! - [NSDatePicker](https://developer.apple.com/documentation/appkit/nsdatepicker)
//! - [NSControl](https://developer.apple.com/documentation/appkit/nscontrol)

pub mod delegate;

pub use delegate::NSDatePickerDelegate;

use crate::control::NSControl;
use crate::graphics::NSRect;
use crate::view::NSView;
use chrono::{DateTime, Local};
use delegate::{DatePickerState, NSDatePickerViewDelegate};
use std::sync::{Arc, Mutex};

/// Date picker mode determining what can be selected.
///
/// Apple: [NSDatePickerMode](https://developer.apple.com/documentation/appkit/nsdatepicker/mode)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NSDatePickerMode {
    /// Single date and time selection.
    ///
    /// Apple: `single`
    Single,
    
    /// Date range selection.
    ///
    /// Apple: `range`
    Range,
}

/// Date picker visual style.
///
/// Apple: [NSDatePickerStyle](https://developer.apple.com/documentation/appkit/nsdatepicker/style)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NSDatePickerStyle {
    /// Text field with stepper arrows.
    ///
    /// Apple: `textFieldAndStepper`
    TextFieldAndStepper,
    
    /// Graphical clock and calendar.
    ///
    /// Apple: `clockAndCalendar`
    ClockAndCalendar,
    
    /// Text field only.
    ///
    /// Apple: `textField`
    TextField,
}

/// Date picker element visibility flags.
///
/// Apple: [NSDatePickerElementFlags](https://developer.apple.com/documentation/appkit/nsdatepicker/elementflags)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NSDatePickerElementFlags {
    /// Show hour and minute.
    pub hour_minute: bool,
    
    /// Show second.
    pub second: bool,
    
    /// Show year, month, and day.
    pub year_month_day: bool,
    
    /// Show era (e.g., BC/AD).
    pub era: bool,
    
    /// Show time zone.
    pub time_zone: bool,
}

impl NSDatePickerElementFlags {
    /// All date and time elements.
    pub fn all() -> Self {
        Self {
            hour_minute: true,
            second: true,
            year_month_day: true,
            era: false,
            time_zone: false,
        }
    }
    
    /// Date only (year, month, day).
    pub fn date_only() -> Self {
        Self {
            hour_minute: false,
            second: false,
            year_month_day: true,
            era: false,
            time_zone: false,
        }
    }
    
    /// Time only (hour, minute, second).
    pub fn time_only() -> Self {
        Self {
            hour_minute: true,
            second: true,
            year_month_day: false,
            era: false,
            time_zone: false,
        }
    }
}

impl Default for NSDatePickerElementFlags {
    fn default() -> Self {
        Self::all()
    }
}

/// Date picker control.
///
/// Apple: [NSDatePicker](https://developer.apple.com/documentation/appkit/nsdatepicker)
#[derive(Clone)]
pub struct NSDatePicker {
    control: NSControl,
    state: Arc<Mutex<DatePickerState>>,
    delegate: Arc<Mutex<Option<Box<dyn NSDatePickerDelegate>>>>,
}

impl NSDatePicker {
    /// Create a new date picker.
    ///
    /// Apple: `[[NSDatePicker alloc] initWithFrame:]`
    pub fn new(frame: NSRect) -> Self {
        let state = Arc::new(Mutex::new(DatePickerState::default()));
        let delegate = Arc::new(Mutex::new(None));
        
        let mut control = NSControl::new(frame);
        control.view_mut().set_delegate(NSDatePickerViewDelegate {
            state: state.clone(),
            delegate: delegate.clone(),
        });
        
        Self {
            control,
            state,
            delegate,
        }
    }

    /// Create a date picker with default styling.
    pub fn with_mode(mode: NSDatePickerMode) -> Self {
        let mut picker = Self::new(NSRect::sized(200.0, 28.0));
        picker.set_date_picker_mode(mode);
        picker
    }

    // ── Date Management ──────────────────────────────────────────────

    /// Set the current date.
    ///
    /// Apple: `setDateValue:`
    pub fn set_date(&mut self, date: DateTime<Local>) {
        let mut state = self.state.lock().unwrap();
        state.date = date;
        state.end_date = None; // Clear range when setting single date
    }

    /// Get the current date.
    ///
    /// Apple: `dateValue`
    pub fn date(&self) -> DateTime<Local> {
        self.state.lock().unwrap().date
    }

    /// Set the date range (for range mode).
    ///
    /// Apple: `setDateValue:` and `setTimeInterval:`
    pub fn set_date_range(&mut self, start: DateTime<Local>, end: DateTime<Local>) {
        let mut state = self.state.lock().unwrap();
        state.date = start;
        state.end_date = Some(end);
    }

    /// Get the end date (for range mode).
    ///
    /// Apple: `timeInterval`
    pub fn end_date(&self) -> Option<DateTime<Local>> {
        self.state.lock().unwrap().end_date
    }

    // ── Configuration ────────────────────────────────────────────────

    /// Set the date picker mode.
    ///
    /// Apple: `setDatePickerMode:`
    pub fn set_date_picker_mode(&mut self, mode: NSDatePickerMode) {
        self.state.lock().unwrap().mode = mode;
    }

    /// Get the date picker mode.
    ///
    /// Apple: `datePickerMode`
    pub fn date_picker_mode(&self) -> NSDatePickerMode {
        self.state.lock().unwrap().mode
    }

    /// Set the date picker style.
    ///
    /// Apple: `setDatePickerStyle:`
    pub fn set_date_picker_style(&mut self, style: NSDatePickerStyle) {
        self.state.lock().unwrap().style = style;
    }

    /// Get the date picker style.
    ///
    /// Apple: `datePickerStyle`
    pub fn date_picker_style(&self) -> NSDatePickerStyle {
        self.state.lock().unwrap().style
    }

    /// Set which date/time elements are displayed.
    ///
    /// Apple: `setDatePickerElements:`
    pub fn set_date_picker_elements(&mut self, elements: NSDatePickerElementFlags) {
        self.state.lock().unwrap().elements = elements;
    }

    /// Get which date/time elements are displayed.
    ///
    /// Apple: `datePickerElements`
    pub fn date_picker_elements(&self) -> NSDatePickerElementFlags {
        self.state.lock().unwrap().elements
    }

    /// Set the minimum selectable date.
    ///
    /// Apple: `setMinDate:`
    pub fn set_min_date(&mut self, date: Option<DateTime<Local>>) {
        self.state.lock().unwrap().min_date = date;
    }

    /// Get the minimum selectable date.
    ///
    /// Apple: `minDate`
    pub fn min_date(&self) -> Option<DateTime<Local>> {
        self.state.lock().unwrap().min_date
    }

    /// Set the maximum selectable date.
    ///
    /// Apple: `setMaxDate:`
    pub fn set_max_date(&mut self, date: Option<DateTime<Local>>) {
        self.state.lock().unwrap().max_date = date;
    }

    /// Get the maximum selectable date.
    ///
    /// Apple: `maxDate`
    pub fn max_date(&self) -> Option<DateTime<Local>> {
        self.state.lock().unwrap().max_date
    }

    // ── Delegate ─────────────────────────────────────────────────────

    /// Set the delegate for date picker events.
    ///
    /// Apple: `setTarget:action:` (we use delegate pattern)
    pub fn set_delegate(&mut self, delegate: Box<dyn NSDatePickerDelegate>) {
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
