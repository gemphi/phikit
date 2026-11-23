//! Collection View Builder Module
//!
//! this module provides specialized assembly functions for grid-based UI components.
//! it coordinates the construction of the Project Browser and Asset Library.
//! mirrors Apple's tiered approach to complex UI construction and data binding.

pub mod asset_library;
pub mod constants;
pub mod data_source;
pub mod project_browser;

pub use asset_library::build_rcp_asset_library;
pub use data_source::{GenericAssetDataSource, GenericGridItemDelegate};
pub use project_browser::build_rcp_project_browser;
