//! Execute part - execute bar spec (Blender RGN_TYPE_EXECUTE footer).
//!
//! Pure static data: filename field text and confirm/cancel button labels.

pub mod set;

/// Execute bar spec for a host.
#[derive(Clone, Copy, Debug)]
pub struct ExecSpec {
    /// Optional filename/path field text (empty = no field).
    pub field: &'static str,
    pub ok: &'static str,
    pub cancel: &'static str,
}

pub static DEFAULT_EXEC: ExecSpec = ExecSpec {
    field: "",
    ok: "OK",
    cancel: "Cancel",
};
