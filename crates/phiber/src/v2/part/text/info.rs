//! Info editor report log — Blender's `SpaceInfo` report list
//! (`info_report.cc`). Newest last, capped at `REPORT_MAX`, stored as
//! `ctx.info.report.<n>` = "<LEVEL>\t<message>".

use crate::v2::state::{BPath, BState};
use crate::v2::view::prim::{BCard, BColumn};
use crate::v2::view::prim::{BRowShade};
use crate::v2::view::BView;

pub const REPORT_MAX: usize = 32;
const SLOTS: [&str; REPORT_MAX] = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15", "16", "17", "18", "19", "20", "21", "22", "23", "24", "25", "26", "27", "28", "29", "30", "31"];

/// Report severity (`eReportType` subset, with Blender's icons).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BReportLevel {
    Info,
    Warning,
    Error,
}

impl BReportLevel {
    fn slug(self) -> &'static str {
        match self {
            Self::Info => "INFO",
            Self::Warning => "WARNING",
            Self::Error => "ERROR",
        }
    }
    fn icon(slug: &str) -> &'static str {
        match slug {
            "ERROR" => "alert_triangle",
            "WARNING" => "alert_triangle",
            _ => "info",
        }
    }
}

fn slot(n: usize) -> BPath {
    BPath::owned(vec!["info", "report", SLOTS[n.min(REPORT_MAX - 1)]])
}

/// All stored reports, oldest first.
pub fn reports(s: &BState) -> Vec<(String, String)> {
    (0..REPORT_MAX)
        .map_while(|n| s.get(&slot(n)).filter(|v| !v.is_empty()).map(str::to_string))
        .map(|v| match v.split_once('\t') {
            Some((lvl, msg)) => (lvl.to_string(), msg.to_string()),
            None => ("INFO".to_string(), v),
        })
        .collect()
}

/// Append a report, dropping the oldest when full.
pub fn push(state: &mut BState, level: BReportLevel, message: impl Into<String>) {
    let mut all = reports(state);
    all.push((level.slug().to_string(), message.into()));
    let start = all.len().saturating_sub(REPORT_MAX);
    for (n, (lvl, msg)) in all[start..].iter().enumerate() {
        state.set(&slot(n), format!("{lvl}\t{msg}"));
    }
}

pub fn view(s: &BState) -> BView {
    let rows: Vec<BView> = reports(s)
        .into_iter()
        .enumerate()
        .map(|(i, (lvl, msg))| {
            let shade = if i % 2 == 1 { BRowShade::Alt } else { BRowShade::Flat };
            BView::list_row(vec![BView::icon(BReportLevel::icon(&lvl)), BView::label(msg)], shade)
        })
        .collect();
    if rows.is_empty() {
        return BView::from(BCard::column(BColumn::Body, vec![BView::label("No reports")]));
    }
    BView::from(BCard::column(BColumn::Body, rows))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_the_newest_reports() {
        let mut s = BState::new();
        for i in 0..REPORT_MAX + 3 {
            push(&mut s, BReportLevel::Info, format!("r{i}"));
        }
        let all = reports(&s);
        assert_eq!(all.len(), REPORT_MAX);
        assert_eq!(all.last().unwrap().1, format!("r{}", REPORT_MAX + 2));
        assert_eq!(all[0].1, "r3");
    }
}
