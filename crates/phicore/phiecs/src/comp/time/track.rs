//! A track: one value's history, kept only where it changed.
/// Ticks and values, sparse: a value holds from its tick until the next
/// one. Ticks only grow; pushing at the last tick replaces its value.
#[derive(Clone, Debug)]
pub struct Track<A> {
    ticks: Vec<u64>,
    values: Vec<A>,
}
impl<A> Default for Track<A> {
    fn default() -> Self {
        Self { ticks: Vec::new(), values: Vec::new() }
    }
}
impl<A> Track<A> {
    pub fn new() -> Self {
        Self::default()
    }
    /// Sets the value from `tick` on. A tick before the last one is a
    /// rewrite of the past and is dropped from there: later keys go.
    pub fn push(&mut self, tick: u64, value: A) {
        let keep = self.ticks.partition_point(|t| *t < tick);
        self.ticks.truncate(keep);
        self.values.truncate(keep);
        self.ticks.push(tick);
        self.values.push(value);
    }
    /// Drops every value pushed after `tick`.
    pub fn truncate_after(&mut self, tick: u64) {
        let keep = self.ticks.partition_point(|t| *t <= tick);
        self.ticks.truncate(keep);
        self.values.truncate(keep);
    }
    /// The value at `tick`: the last pushed at or before it.
    pub fn at(&self, tick: u64) -> Option<&A> {
        let n = self.ticks.partition_point(|t| *t <= tick);
        self.values.get(n.checked_sub(1)?)
    }
    /// The tick of the value `at(tick)` gives.
    pub fn tick_at(&self, tick: u64) -> Option<u64> {
        let n = self.ticks.partition_point(|t| *t <= tick);
        self.ticks.get(n.checked_sub(1)?).copied()
    }
    pub fn last_tick(&self) -> Option<u64> {
        self.ticks.last().copied()
    }
    pub fn len(&self) -> usize {
        self.ticks.len()
    }
    pub fn is_empty(&self) -> bool {
        self.ticks.is_empty()
    }
    pub fn iter(&self) -> impl Iterator<Item = (u64, &A)> {
        self.ticks.iter().copied().zip(self.values.iter())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a_value_holds_until_the_next() {
        let mut t = Track::new();
        assert_eq!(t.at(5), None);
        t.push(2, "a");
        t.push(6, "b");
        assert_eq!((t.at(1), t.at(2), t.at(5), t.at(6), t.at(99)), (None, Some(&"a"), Some(&"a"), Some(&"b"), Some(&"b")));
        assert_eq!((t.tick_at(5), t.len()), (Some(2), 2));
    }
    #[test]
    fn the_same_tick_replaces_and_the_past_rewrites() {
        let mut t = Track::new();
        t.push(1, 1);
        t.push(4, 4);
        t.push(4, 40);
        assert_eq!((t.at(4), t.len()), (Some(&40), 2));
        t.push(2, 2);
        assert_eq!((t.at(9), t.len()), (Some(&2), 2), "tick 4 went with the rewrite");
    }
}
