//! # NSMenu — Menu System
//!
//! Menu bar, items, and dropdown menus.
//!
//! ## Apple Documentation
//! - [NSMenu](https://developer.apple.com/documentation/appkit/nsmenu)
//! - [NSMenuItem](https://developer.apple.com/documentation/appkit/nsmenuitem)
//! - [NSMenuDelegate](https://developer.apple.com/documentation/appkit/nsmenudelegate)
//!
//! ## Usage
//!
//! ```rust,ignore
//! let mut menu = Menu::new("File");
//! menu.add_item(MenuItem::new("Open", || open_file()));
//! menu.add_item(MenuItem::new("Save", || save_file()));
//! menu.add_separator();
//! menu.add_item(MenuItem::new("Quit", || quit()));
//! ```
//!
//! ## See Also
//! - [NSPopUpButton](https://developer.apple.com/documentation/appkit/nspopupbutton)
//! - [NSToolbar](https://developer.apple.com/documentation/appkit/nstoolbar)

pub mod delegate;
pub mod dropdown;
pub mod item;
pub mod menu;
pub mod view;

use std::sync::Arc;

pub type MenuAction = Arc<dyn Fn() + Send + Sync>;

pub use delegate::MenuBarDelegate;
pub use dropdown::{MenuDropdownDelegate, MenuOverlayDelegate};
pub use item::MenuItem;
pub use menu::{Menu, MenuBar};
pub use view::MenuBarView;
