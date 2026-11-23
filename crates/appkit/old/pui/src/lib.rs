//! # Phi UI Kit (PUI)
//!
//! SwiftUI-style declarative widgets built on `rust-appkit` `NSView`.
//!
//! ## Two flavours, one source of truth
//!
//! Every widget has **one canonical full-name struct** (e.g. `PTable`).
//! Short 3-letter aliases (e.g. `PTbl`) are `pub use ... as` re-exports — so
//! they always track the canonical definition automatically.
//!
//! ```ignore
//! use rust_pui_kit::*;          // canonical forms (PTable, PField, ...)
//! use rust_pui_kit::aliases::*; // 3-letter forms  (PTbl,  PFld,    ...)
//! ```
//!
//! ## Widget Roster
//!
//! | Canonical      | Alias    | Purpose                       | SwiftUI            |
//! |----------------|----------|-------------------------------|--------------------|
//! | `PLabel`       | `PLbl`   | Text label                    | `Text`             |
//! | `PButton`      | `PBtn`   | Pressable button              | `Button`           |
//! | `PTextField`   | `PTxt`   | Text input                    | `TextField`        |
//! | `PToggle`      | `PTgl`   | Boolean switch                | `Toggle`           |
//! | `PSlider`      | `PSld`   | Continuous value slider       | `Slider`           |
//! | `PSegmented`   | `PSeg`   | Segmented picker              | `Picker(.segmented)` |
//! | `PColorSwatch` | `PCsw`   | Circular color swatch         | `Circle().fill`    |
//! | `PField`       | `PFld`   | Labelled row                  | `LabeledContent`   |
//! | `PForm`        | `PFrm`   | Vertical form                 | `Form`             |
//! | `PRow`         | `PRw`    | Horizontal row                | `HStack`           |
//! | `PColumn`      | `PCol`   | Vertical column               | `VStack`           |
//! | `PTable`       | `PTbl`   | Table with header + body      | `Table`            |
//! | `PKeyVal`    | `PKVP`   | Key/value pair                | `LabeledContent`   |
//! | `PFieldSet`    | `PFSet`  | Titled group of fields        | `GroupBox`         |
//! | `PSection`     | `PSec`   | Section with header           | `Section`          |
//!
//! ## Trait Roster
//!
//! | Canonical      | Alias    | Purpose                                  |
//! |----------------|----------|------------------------------------------|
//! | `PWidget`      | `Pwgt`   | Anything that builds an `NSView`         |
//! | `PStyle`       | `Psty`   | Chainable styling modifiers              |
//! | `PInteractive` | `Pint`   | Chainable interaction callbacks          |
//! | `PContainer`   | `Pcnt`   | Holds typed children                     |
//! | `PTerse`       | `Ptrs`   | Compact 3-letter ops (var/let/has/...)   |
//!
//! ## Terse 3-letter Operations (`PTerse` / `Ptrs`)
//!
//! All generics share these chainable ops:
//! - `var(name)` — variable / value identifier
//! - `r#let(value)` — literal / fixed value
//! - `has(child)` — add child content
//! - `sub(text)` — secondary / subtitle text
//! - `r#pub(callback)` — publish a callback
//! - `ops(|s| ...)` — inline mutator block

pub mod traits;
pub mod widgets;
pub mod aliases;

// Canonical full-name re-exports
pub use traits::{PWidget, PStyle, PInteractive, PContainer, PTerse};
pub use widgets::{
    PLabel, PButton, PTextField, PToggle, PSlider, PSegmented, PColorSwatch,
    PField, PForm, PRow, PColumn, PTable, PKeyVal, PFieldSet, PSection,
};

// Re-export NSView etc. for convenience
pub use rust_appkit::prelude::*;
