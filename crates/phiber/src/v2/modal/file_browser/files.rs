//! Shared File Browser table used by the modal client.

use crate::v2::part::custom::file::table;
use crate::v2::state::BState;
use crate::v2::view::BView;

pub fn view(state: &BState) -> BView {
    table::view(state)
}
