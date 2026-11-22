//! Guests: every page that reaches the line is somebody, a guest until
//! something says who they are. A page is known by a cookie; the first
//! time it comes it is given one, and a caller of its own. Nothing is
//! kept: a line that starts again knows nobody.

use std::collections::{HashMap, VecDeque};

use super::Set;
use crate::entity::Caller;

/// Who each cookie is.
#[derive(Debug, Default)]
pub struct Guest {
    known: HashMap<String, Caller>,
    /// The cookies, the oldest first.
    order: VecDeque<String>,
}

impl Guest {
    /// Who a page with that cookie is. A page with none, or with one
    /// the line does not know, is a new guest: the caller, and the
    /// cookie's new value to give the page. Past `most` guests the
    /// oldest is forgotten.
    pub fn of(&mut self, cookie: Option<&str>, set: &Set, most: usize) -> (Caller, Option<String>) {
        if let Some(caller) = cookie.and_then(|value| self.known.get(value)) {
            return (caller.clone(), None);
        }
        let value = format!("{:032x}", rand::random::<u128>());
        let caller = Caller::new(&format!("{}{:016x}", set.id, rand::random::<u64>()), &set.guest, None);
        self.known.insert(value.clone(), caller.clone());
        self.order.push_back(value.clone());
        while self.order.len() > most.max(1) {
            self.order.pop_front().into_iter().for_each(|old| _ = self.known.remove(&old));
        }
        (caller, Some(value))
    }
}
