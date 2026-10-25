//! How a `BSport` finds its spec. The index, not the content: each sport's
//! numbers are in its own `set.rs`.

use super::{BSport, BSportSpec, basketball, beach, netball, soccer, volleyball};

impl BSport {
    pub fn spec(self) -> &'static BSportSpec {
        match self {
            BSport::Basketball => &basketball::SPEC,
            BSport::Netball => &netball::SPEC,
            BSport::Volleyball => &volleyball::SPEC,
            BSport::Beach => &beach::SPEC,
            BSport::Soccer => &soccer::SPEC,
        }
    }
}
