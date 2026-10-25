//! Where the rows sit: each row's top and centre in the window, the row
//! under a point, and which NLA track a row is.

use super::{BRowKind, BTimeRow, NLA_ROW_HEIGHT, ROW_HEIGHT, ROW_SKIP};
use crate::v2::host::BHostType;

impl BTimeRow {
    /// `NLATRACK_HEIGHT` for the NLA editor, `ANIM_UI_get_channel_height`
    /// for the rest.
    pub fn height(host: BHostType) -> f32 {
        if host == BHostType::Nla { NLA_ROW_HEIGHT } else { ROW_HEIGHT }
    }

    /// `ANIM_UI_get_first_channel_top` and `ANIM_UI_get_channel_step`
    /// (`NLATRACK_FIRST_TOP` / `NLATRACK_STEP`): the top of row `index`,
    /// measured from the top of the window, for rows `height` tall.
    /// `scrub` is the band above the rows (`BTimeView::scrub_height`), which
    /// these are all measured past. It is a parameter and not a constant so the
    /// theme decides it; `SCRUB_HEIGHT` remains as Blender's own number for the
    /// callers that mean that and nothing else.
    pub fn top(index: usize, height: f32, scrub: f32) -> f32 {
        scrub + ROW_SKIP + index as f32 * (height + ROW_SKIP)
    }

    pub fn center(index: usize, height: f32, scrub: f32) -> f32 {
        Self::top(index, height, scrub) + height / 2.0
    }

    /// `UI_view2d_listview_view_to_cell`: the row at `y` **from the top of the
    /// window**, each row's cell its height and the skip below it.
    ///
    /// From the top of the WINDOW, not of the rows: it subtracts `top(0)`
    /// itself, and `top` already counts past the scrub band. A caller that
    /// subtracts the band before calling this subtracts it twice — which
    /// `expander_write` did, putting every expander click a band's worth of
    /// pixels down the list.
    pub fn index_at(y: f32, height: f32, scrub: f32) -> Option<usize> {
        let from = y - Self::top(0, height, scrub);
        (from >= 0.0).then(|| (from / (height + ROW_SKIP)) as usize)
    }

    /// The NLA track row `index` of `rows` is: its object, and its place in
    /// that object's stack counted from the bottom (the rows list it last
    /// first).
    pub fn track_at(rows: &[Self], index: usize) -> Option<(String, usize)> {
        let BRowKind::Track(_) = rows.get(index)?.kind else { return None };
        let head = rows[..index].iter().rposition(|r| matches!(r.kind, BRowKind::Object { .. }))?;
        let BRowKind::Object { name: object, .. } = &rows[head].kind else { return None };
        let tracks = rows[head + 1..].iter().take_while(|r| !matches!(r.kind, BRowKind::Object { .. })).filter(|r| matches!(r.kind, BRowKind::Track(_))).count();
        let above = rows[head + 1..index].iter().filter(|r| matches!(r.kind, BRowKind::Track(_))).count();
        Some((object.clone(), tracks - 1 - above))
    }
}
