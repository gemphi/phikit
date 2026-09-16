//! Element structs — each BView variant delegates to its own element.
//! Each struct provides `with_elm()` (leaf) or `with_div()` (container)
//! plus `into_elm()` and `into_popup_item()` for interactive rendering with action handlers.
//!
//! Fully flat module layout with zero subdirectories.

pub mod column;
pub mod dropdown;
pub mod empty;
pub mod htab_bar;
pub mod icon_list;
pub mod label;
pub mod panel;
pub mod prop_row;
pub mod row;
pub mod scroll;
pub mod section;
pub mod separator;
pub mod spatial;
pub mod tab;

pub use column::ColumnView;
pub use dropdown::DropdownView;
pub use empty::EmptyView;
pub use htab_bar::{HTabBarView, VTabBarView};
pub use icon_list::{IconButtonView, IconListView};
pub use label::LabelView;
pub use panel::PanelView;
pub use prop_row::PropRowView;
pub use row::RowView;
pub use scroll::ScrollView;
pub use section::SectionView;
pub use separator::SeparatorView;
pub use spatial::SpatialView;
pub use tab::TabView;
