//! A caller: whoever opened a line. One person may have several pages
//! open, so a caller holds a line for each. A guest has no handle.

use phiecs::prelude::Aspect;

/// Who a line belongs to.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Caller {
    /// Who they are to whoever knows them: only they may speak for
    /// their pages.
    pub user: String,
    pub name: String,
    pub handle: Option<String>,
}

impl Aspect for Caller {}

impl Caller {
    pub fn new(user: &str, name: &str, handle: Option<&str>) -> Caller {
        Caller { user: user.to_string(), name: name.to_string(), handle: handle.map(str::to_string) }
    }
}
