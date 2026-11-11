// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/util/time.h` (`time_dt`). Cycles reads one
//! global clock; here whoever needs the time holds a [`Clock`], so a test or
//! a browser, which has no system clock, can supply its own.

use std::cell::Cell;
use std::rc::Rc;

/// `time_dt`: the time now, in seconds. Never zero: the scheduler reads a
/// zero time as "not started".
pub trait Clock {
    fn now(&self) -> f64;
}

impl<T: Clock + ?Sized> Clock for &T {
    fn now(&self) -> f64 {
        (**self).now()
    }
}

impl<T: Clock + ?Sized> Clock for Rc<T> {
    fn now(&self) -> f64 {
        (**self).now()
    }
}

/// The system's clock: seconds since 1970.
#[cfg(not(target_family = "wasm"))]
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

#[cfg(not(target_family = "wasm"))]
impl Clock for SystemClock {
    fn now(&self) -> f64 {
        let since = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH);
        since.map(|span| span.as_secs_f64()).unwrap_or(1.0)
    }
}

/// A clock its holder moves: a browser sets it from the page's timer, a test
/// steps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualClock {
    second: Cell<f64>,
}

impl ManualClock {
    pub fn at(second: f64) -> ManualClock {
        ManualClock { second: Cell::new(second) }
    }

    pub fn set(&self, second: f64) {
        self.second.set(second);
    }

    pub fn advance(&self, second: f64) {
        self.second.set(self.second.get() + second);
    }
}

impl Clock for ManualClock {
    fn now(&self) -> f64 {
        self.second.get()
    }
}
