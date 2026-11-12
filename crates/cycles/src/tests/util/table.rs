// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use crate::util::*;

#[test]
fn rows_are_cut_at_tabs_and_comments_and_blank_lines_are_left_out() {
    let row: Vec<Vec<&str>> = row_list("# kind\tname\n\npass\tAlbedo\t1\n   \ntype\tOptiX\n").collect();
    assert_eq!(row, [vec!["pass", "Albedo", "1"], vec!["type", "OptiX"]]);
}

#[test]
fn an_empty_cell_is_kept() {
    let row: Vec<Vec<&str>> = row_list("pass\t\t64\t").collect();
    assert_eq!(row, [vec!["pass", "", "64", ""]]);
}
