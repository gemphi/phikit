//! PUI Widgets — canonical full-name structs.
//!
//! These are the source of truth. Short aliases (`PLbl`, `PBtn`, ...)
//! live in the `aliases` module and are `pub use ... as` re-exports.

mod label;
mod button;
mod text_field;
mod toggle;
mod slider;
mod segmented;
mod color_swatch;
mod field;
mod form;
mod row;
mod column;
mod table;
mod key_value;
mod field_set;
mod section;

pub use label::PLabel;
pub use button::PButton;
pub use text_field::PTextField;
pub use toggle::PToggle;
pub use slider::PSlider;
pub use segmented::PSegmented;
pub use color_swatch::PColorSwatch;
pub use field::PField;
pub use form::PForm;
pub use row::PRow;
pub use column::PColumn;
pub use table::PTable;
pub use key_value::PKeyVal;
pub use field_set::PFieldSet;
pub use section::PSection;
