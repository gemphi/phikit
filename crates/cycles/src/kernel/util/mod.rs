// SPDX-License-Identifier: Apache-2.0
//! What the kernels share: `intern/cycles/kernel/util`. So far the compact
//! ray differentials and the reading of a lookup table.

mod differential;
mod table;

pub use differential::{Differential3, differential_make_compact, differential_zero_compact};
pub use table::lookup_table_read;
