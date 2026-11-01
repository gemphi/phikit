//! # NSTextField — Text Input Control
//!
//! Text input control with high-fidelity state management.
//!
//! ## Apple Documentation
//! - [NSTextField](https://developer.apple.com/documentation/appkit/nstextfield)
//! - [NSTextAlignment](https://developer.apple.com/documentation/appkit/nstextalignment)
//!
//! ## Usage
//!
//! ```rust,ignore
//! let mut field = NSTextField::new(frame);
//! field.set_string_value("Enter text");
//! field.set_placeholder_string("Placeholder");
//! ```
//!
//! ## See Also
//! - [NSTextField](https://developer.apple.com/documentation/appkit/nstextfield)
//! - [NSControl](https://developer.apple.com/documentation/appkit/nscontrol)

mod editing;
mod field_types;
pub(crate) mod rendering;

pub use field_types::{NSTextAlignment, NSTextField};
