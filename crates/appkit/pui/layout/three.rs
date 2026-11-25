//! Three places in a row, or down a column (the user, 25 November 2026:
//! "every row has three areas: left, centre, right, or top, centre,
//! bottom"). The start stands at the start (left, or top), the end at the
//! end (right, or bottom), and the middle always in the middle of the
//! whole, however large the sides are; a middle that would run into a
//! side keeps clear of it, between the two. Any place may hold a grid,
//! or three places again, so the same rule holds at any depth: a 3x2 or
//! a 6x3 is three places of three.

use crate::pui::core::{PPoint, PRect, PSize};

/// Which way the three run: across a row, or down a column.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PRun {
    Across,
    Down,
}

/// Where three places stand in a rect, each as large as it says (none
/// for an empty place): the start at the rect's start, the end at its
/// end, the middle centred on the whole and kept a gap clear of the
/// sides. Across the run, each stands in the middle of the rect.
pub fn places(rect: PRect, run: PRun, size: [Option<PSize>; 3], gap: f32) -> [Option<PRect>; 3] {
    // Along the run: where it begins, how long it is; across it the same.
    let (from, long, side, wide) = match run {
        PRun::Across => (rect.origin.x, rect.size.width, rect.origin.y, rect.size.height),
        PRun::Down => (rect.origin.y, rect.size.height, rect.origin.x, rect.size.width),
    };
    let along = |size: PSize| match run {
        PRun::Across => (size.width, size.height),
        PRun::Down => (size.height, size.width),
    };
    let put = |at: f32, size: PSize| {
        let (length, thick) = along(size);
        let across = side + (wide - thick) / 2.0;
        match run {
            PRun::Across => PRect { origin: PPoint::new(at, across), size: PSize { width: length, height: thick } },
            PRun::Down => PRect { origin: PPoint::new(across, at), size: PSize { width: thick, height: length } },
        }
    };
    let [start, middle, end] = size;
    let start = start.map(|size| put(from, size));
    let end = end.map(|size| put(from + long - along(size).0, size));
    let start_ends = start.map_or(from, |rect| from + match run {
        PRun::Across => rect.size.width,
        PRun::Down => rect.size.height,
    } + gap);
    let end_begins = end.map_or(from + long, |rect| match run {
        PRun::Across => rect.origin.x,
        PRun::Down => rect.origin.y,
    } - gap);
    let middle = middle.map(|size| {
        let length = along(size).0;
        let centred = from + (long - length) / 2.0;
        // Kept clear of the sides where it can be; with no room between
        // them, it begins after the start.
        let at = match end_begins - start_ends >= length {
            true => centred.clamp(start_ends, end_begins - length),
            false => start_ends,
        };
        put(at, size)
    });
    [start, middle, end]
}
