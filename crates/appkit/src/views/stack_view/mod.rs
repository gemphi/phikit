//! # NSStackView — Auto-Layout Stack Container
//!
//! A view that arranges subviews in a horizontal or vertical stack with automatic layout.
//!
//! ## Apple Documentation
//! - [NSStackView](https://developer.apple.com/documentation/appkit/nsstackview)
//! - [NSView](https://developer.apple.com/documentation/appkit/nsview)
//!
//! ## Usage
//!
//! ```rust,ignore
//! let mut stack = NSStackView::new(frame);
//! stack.set_orientation(NSStackViewOrientation::Vertical);
//! stack.set_distribution(NSStackViewDistribution::FillEqually);
//! stack.add_arranged_subview(view1);
//! stack.add_arranged_subview(view2);
//! ```
//!
//! ## See Also
//! - [NSSplitView](https://developer.apple.com/documentation/appkit/nssplitview)
//! - [NSBox](https://developer.apple.com/documentation/appkit/nsbox)

pub mod stack_layout;
pub mod stack_rendering;

pub use stack_layout::NSStackViewDelegate;
pub use stack_rendering::{NSStackView, NSStackViewDistribution};
