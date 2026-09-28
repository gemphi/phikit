//! Collaboration modal (Activity ▸ Chat / Stream). Shows the session state
//! the collaboration service writes into `BState` — status, server and
//! peers — and nothing invented: with no service connected it says so.
//! The transport (sync, presence, chat, stream) is the service's job; this
//! view only reads `collab.*`.

use crate::v2::state::ActivityKey;
use crate::v2::state::AppKey;
use crate::v2::state::BState;
use crate::v2::view::prim::BColumn;
use crate::v2::view::prim::BCard;
use crate::v2::view::prim::BRowShade;
use crate::v2::view::prim::BTableCell;
use crate::v2::view::BView;
use crate::v2::view::BViewItem;

/// Peer slots (`collab.peer.0` … `collab.peer.15`).
const PEER_SLOTS: [&str; 16] = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15"];

#[derive(Clone, Copy, Debug, Default)]
pub struct BCollabModal;

fn peers(s: &BState) -> Vec<BView> {
    PEER_SLOTS
        .iter()
        .map_while(|n| s.get(AppKey::CollabPeer(n)).map(str::to_string))
        .map(|line| {
            let (name, state) = line.split_once('\t').unwrap_or((line.as_str(), ""));
            let cell_list = vec![BTableCell::flex(BView::label(name)), BTableCell::flex(BView::label(state))];
            BView::table_row(cell_list, BRowShade::Flat, None)
        })
        .collect()
}

impl BViewItem for BCollabModal {
    fn build(&self, s: &BState) -> BView {
        let tab = s.get(ActivityKey::Tab).unwrap_or("Chat");
        let status = s.get(AppKey::CollabStatus).unwrap_or("offline");
        let server = s.get(AppKey::CollabServer).unwrap_or("no collaboration server configured");
        let mut body = vec![
            BView::label(format!("{tab} · {status}")),
            BView::label(server),
        ];
        let list = peers(s);
        if list.is_empty() {
            body.push(BView::label("No one else is in this session."));
        } else {
            body.push(BCard::headless().body(list).into());
        }
        BView::from(BCard::column(BColumn::Body, body))
    }
}
