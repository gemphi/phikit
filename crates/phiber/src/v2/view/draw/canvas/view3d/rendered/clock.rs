//! How long the tracer takes to answer a view: from the last view posted to
//! its first image, and to its last — the two numbers a slow viewport is
//! about, written to the session journal as each view settles.
//!
//! An orbit posts a view per frame; only the one the hand rests on is
//! answered to the end. So the clock restarts at every post and speaks
//! once, when a posted view's image is done: how many views the burst held,
//! how long it lasted, and how long the final one took to arrive and to
//! finish.

use web_time::Instant;

/// The clock of the view being traced.
#[derive(Default)]
pub struct BViewClock {
    /// The key last posted, when, and how many posts the burst has had
    /// since the last settled view, since when.
    posted: Option<(u64, Instant)>,
    burst: (usize, Option<Instant>),
    /// When the posted view's first image landed.
    first: Option<Instant>,
}

impl BViewClock {
    /// A view was posted to the tracer.
    pub fn posted(&mut self, key: u64) {
        let now = Instant::now();
        self.posted = Some((key, now));
        self.first = None;
        self.burst.0 += 1;
        self.burst.1.get_or_insert(now);
    }

    /// An image for `view` landed, `done` when it is the view's last. The
    /// line to journal when that settles the posted view.
    pub fn landed(&mut self, view: u64, done: bool) -> Option<String> {
        let now = Instant::now();
        let (key, at) = self.posted?;
        if view != key {
            return None;
        }
        let first = *self.first.get_or_insert(now);
        if !done {
            return None;
        }
        let (posts, began) = (self.burst.0, self.burst.1.unwrap_or(at));
        self.posted = None;
        self.burst = (0, None);
        Some(Self::line(posts, at.saturating_duration_since(began), first.saturating_duration_since(at), now.saturating_duration_since(at)))
    }

    fn line(posts: usize, burst: std::time::Duration, first: std::time::Duration, done: std::time::Duration) -> String {
        let ms = |d: std::time::Duration| d.as_secs_f64() * 1000.0;
        match posts {
            1 => format!("Tracer: view settled — first image after {:.0}ms, finished after {:.0}ms", ms(first), ms(done)),
            n => format!(
                "Tracer: view settled — {n} views posted over {:.1}s; the last one's first image after {:.0}ms, finished after {:.0}ms",
                burst.as_secs_f64(),
                ms(first),
                ms(done)
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_clock_speaks_once_when_the_posted_view_is_done() {
        let mut c = BViewClock::default();
        c.posted(1);
        c.posted(2);
        assert!(c.landed(1, true).is_none(), "a stale view's image says nothing");
        assert!(c.landed(2, false).is_none(), "a first image is noted, not spoken");
        let line = c.landed(2, true).expect("the posted view settled");
        assert!(line.contains("2 views posted"), "{line}");
        assert!(c.landed(2, true).is_none(), "spoken once");
        c.posted(3);
        let line = c.landed(3, true).unwrap();
        assert!(line.contains("first image after") && !line.contains("views posted"), "{line}");
    }
}
