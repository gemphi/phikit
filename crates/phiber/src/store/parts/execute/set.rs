//! Execute set - per-host execute bar specs (static data).

use super::{DEFAULT_EXEC, ExecSpec};
use crate::store::hosts::BHostType;

// Blender file open dialog footer: filename field + Open / Cancel.
pub static FILE_EXEC: ExecSpec = ExecSpec {
    field: "Female-02--Charry-Whims-Rigged--Lace.blend",
    ok: "Open",
    cancel: "Cancel",
};

/// Execute bar spec for a given host.
pub fn with_idx(host: BHostType) -> &'static ExecSpec {
    match host {
        BHostType::File => &FILE_EXEC,
        _ => &DEFAULT_EXEC,
    }
}
