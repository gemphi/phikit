// SPDX-License-Identifier: Apache-2.0
//! The layout of the path state as `data/state.tsv` gives it (written by
//! `tool/state.py` from `state_template.h` and `shadow_state_template.h`):
//! for each struct, the members that every path has, in the templates'
//! order, each at a word. A member takes a word for each 32-bit part of
//! its C++ type, as `kernel/record.rs` does.
use std::sync::OnceLock;

use crate::util::row_list;

/// One member of a struct of the path state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StateMember {
    pub name: &'static str,
    pub kind: &'static str,
    /// Its first word in the struct.
    pub word: usize,
    pub word_count: usize,
}

/// The words a C++ type of the templates takes.
fn words_of(kind: &str) -> usize {
    match kind {
        "packed_float3" | "PackedSpectrum" => 3,
        "uint64_t" => 2,
        _ => 1,
    }
}

/// `KERNEL_FEATURE_PATH_TRACING`, which every path has.
const ALWAYS: &str = "KERNEL_FEATURE_PATH_TRACING";

/// The members of one struct (`path`, `ray`, `isect`) that every path has.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateLayout {
    pub name: &'static str,
    pub member: Vec<StateMember>,
}

impl StateLayout {
    /// The layout of the struct the templates call so.
    pub fn of(name: &str) -> Option<&'static StateLayout> {
        static LAYOUT: OnceLock<Vec<StateLayout>> = OnceLock::new();
        let all = LAYOUT.get_or_init(|| {
            let mut layout: Vec<StateLayout> = Vec::new();
            for row in row_list(include_str!("../../../data/state.tsv")).filter(|row| row[0] == "state" && row[6] == ALWAYS) {
                let place = match layout.iter().position(|l| l.name == row[1]) {
                    Some(place) => place,
                    None => {
                        layout.push(StateLayout { name: row[1], member: Vec::new() });
                        layout.len() - 1
                    }
                };
                let word = layout[place].member.last().map_or(0, |last| last.word + last.word_count);
                layout[place].member.push(StateMember { name: row[2], kind: row[3], word, word_count: words_of(row[3]) });
            }
            layout
        });
        all.iter().find(|layout| layout.name == name)
    }

    /// The member named so.
    pub fn member(&self, name: &str) -> Option<&StateMember> {
        self.member.iter().find(|member| member.name == name)
    }

    /// The words of the struct.
    pub fn word_count(&self) -> usize {
        self.member.last().map_or(0, |last| last.word + last.word_count)
    }
}
