//! `BLevelType` — the one trait every *cataloged* level enum implements, the
//! peer of [`BSpec`](crate::v2::spec::BSpec) one tier up: where `BSpec`
//! unifies the static spec structs, this unifies the type enums that index
//! them.
//!
//! A cataloged level is a `Copy` enum with a static catalog of variants, a
//! stable per-variant slug, and a static spec (carrying label and icon). Space,
//! Zone, Host, Modal and Part are cataloged and implement this. Area, Region
//! and Pane are instance-shaped — their specs are declared inline inside each
//! host's `set.rs`, they have no catalog of their own — so they share only
//! `BSpec`, not this.
//!
//! The point is declarative: a level states its identity once. `slug` and
//! `spec` are the two per-variant matches a level cannot avoid (its stable name
//! and its static data); everything else — the inverse `from_slug`, and the
//! `label`/`icon` that are just the spec's fields — is a default here, written
//! once rather than re-matched in every enum. Generic code can then take any
//! `T: BLevelType` instead of one function per level.
//!
//! This trait is additive: each level keeps its inherent `slug`/`spec`/`ALL`,
//! which the impls delegate to, so no existing call site changes and direct
//! calls keep resolving to the inherent methods. The defaults exist for generic
//! callers and for the next level added, which gets `from_slug`/`label`/`icon`
//! for free.

use crate::v2::host::{BHostSpec, BHostType};
use crate::v2::modal::{BModalSpec, BModalType};
use crate::v2::space::{BSpaceSpec, BSpaceType};
use crate::v2::spec::BSpec;
use crate::v2::zone::{BZoneSpec, BZoneType};

pub trait BLevelType: Copy + Sized + 'static {
    /// This level's static spec struct — the `BSpec` that carries its label and
    /// icon, keyed one-to-one with the variant.
    type Spec: BSpec;

    /// Every variant that resolves from a slug — the FULL catalog, not only the
    /// subset shown in a strip. Space returns `EVERY` (which still includes the
    /// hosts dropped from the tab strip, so a file written with one loads);
    /// the others return their `ALL`.
    fn catalog() -> &'static [Self];

    /// The variant's stable identity, independent of its spec: what a state key
    /// persists and what `from_slug` matches against. One match arm per variant.
    fn slug(self) -> &'static str;

    /// The variant's static spec.
    fn spec(self) -> &'static Self::Spec;

    /// The inverse of [`slug`](Self::slug) over [`catalog`](Self::catalog).
    /// Defaulted — no level needs its own copy of the find-loop.
    fn from_slug(s: &str) -> Option<Self> {
        Self::catalog().iter().copied().find(|v| v.slug() == s)
    }

    /// The variant's label, from its spec — stated once in the spec table, not
    /// a second time as a match.
    fn label(self) -> &'static str {
        self.spec().label()
    }

    /// The variant's icon, from its spec.
    fn icon(self) -> &'static str {
        self.spec().icon()
    }
}

// The cataloged levels. Each impl states only the two matches a level cannot
// avoid — its catalog and its spec — plus `slug`, and delegates those to the
// level's own inherent methods (inherent resolution wins over the trait, so the
// calls below are not recursive). `from_slug`, `label` and `icon` come from the
// defaults above. Collected here, beside the trait, so every level's membership
// reads in one place rather than scattered across the mod.rs files.

impl BLevelType for BSpaceType {
    type Spec = BSpaceSpec;
    // The full catalog, not the tab subset (`ALL`): `from_slug` must still
    // resolve the hosts dropped from the strip (Modeling, Sculpting), which is
    // what `EVERY` is for.
    fn catalog() -> &'static [Self] {
        Self::EVERY
    }
    fn slug(self) -> &'static str {
        self.slug()
    }
    fn spec(self) -> &'static BSpaceSpec {
        self.spec()
    }
}

impl BLevelType for BZoneType {
    type Spec = BZoneSpec;
    fn catalog() -> &'static [Self] {
        Self::ALL
    }
    fn slug(self) -> &'static str {
        self.slug()
    }
    fn spec(self) -> &'static BZoneSpec {
        self.spec()
    }
}

impl BLevelType for BModalType {
    type Spec = BModalSpec;
    fn catalog() -> &'static [Self] {
        Self::ALL
    }
    fn slug(self) -> &'static str {
        self.slug()
    }
    fn spec(self) -> &'static BModalSpec {
        self.spec()
    }
}

impl BLevelType for BHostType {
    type Spec = BHostSpec;
    fn catalog() -> &'static [Self] {
        Self::ALL
    }
    fn slug(self) -> &'static str {
        self.slug()
    }
    fn spec(self) -> &'static BHostSpec {
        self.spec()
    }
}

// Not every level is cataloged. Area, Region and Pane are instance-shaped —
// their specs are declared inline inside each host's `set.rs`, with no catalog
// or stable slug of their own. Part is the same in spirit: `BPartType` has no
// `&'static` spec (a `BPartSpec` is built on demand, carrying only its kind) and
// no slug, and it already has its own `BPart` trait (KIND / LABEL / make). All
// four share `BSpec` at the spec tier; none belongs here.

#[cfg(test)]
mod tests {
    use super::*;

    /// A round-trip over any cataloged level: every variant is found by its own
    /// slug, and the trait's `label`/`icon` defaults agree with the spec. The
    /// body is generic — written once, run for each level, which is the whole
    /// reason the trait exists. Because `v` is a bare `T: BLevelType` here, the
    /// method calls resolve to the TRAIT (no inherent methods on a type param),
    /// so this exercises the defaults, not the inherent copies they mirror.
    fn round_trips<T: BLevelType + PartialEq + core::fmt::Debug>() {
        for &v in T::catalog() {
            assert_eq!(T::from_slug(v.slug()), Some(v), "{:?} did not round-trip its slug", v);
            assert_eq!(v.label(), v.spec().label(), "trait label disagrees with spec for {v:?}");
            assert_eq!(v.icon(), v.spec().icon(), "trait icon disagrees with spec for {v:?}");
        }
        assert_eq!(T::from_slug("\u{0}not-a-real-slug"), None, "an unknown slug is None");
    }

    #[test]
    fn every_cataloged_level_round_trips_through_the_trait() {
        round_trips::<BSpaceType>();
        round_trips::<BZoneType>();
        round_trips::<BModalType>();
        round_trips::<BHostType>();
    }
}
