//! The eight venues.

use super::seat::BSeat;
use super::{BLightShow, BRoof, BShell, BVenue, BVenueSpec};

impl BVenue {
    pub fn spec(self) -> &'static BVenueSpec {
        match self {
            BVenue::Tent => &TENT,
            BVenue::Lantern => &LANTERN,
            BVenue::Sails => &SAILS,
            BVenue::Diamond => &DIAMOND,
            BVenue::Containers => &CONTAINERS,
            BVenue::Cap => &CAP,
            BVenue::Dunes => &DUNES,
            BVenue::Arches => &ARCHES,
        }
    }
}

static TENT: BVenueSpec = BVenueSpec {
    label: "Tent",
    after: "Al Bayt's Bedouin tent: a striped canopy pitched over the bowl",
    shell: BShell::Tent,
    roof: BRoof::Ring,
    show: BLightShow::Weave,
    tiers: 15,
    panels: 64,
    rise: 9.0,
    seat: BSeat::SemiHex,
};
static LANTERN: BVenueSpec = BVenueSpec {
    label: "Lantern",
    after: "Lusail's golden bowl, latticed like a lantern",
    shell: BShell::Lantern,
    roof: BRoof::Ring,
    show: BLightShow::Breathe,
    tiers: 18,
    panels: 96,
    rise: 8.0,
    seat: BSeat::Phiegg,
};
static SAILS: BVenueSpec = BVenueSpec {
    label: "Sails",
    after: "Al Janoub's dhow sails, overlapping curved shells",
    shell: BShell::Sails,
    roof: BRoof::Closed,
    show: BLightShow::Wave,
    tiers: 14,
    panels: 48,
    rise: 10.0,
    seat: BSeat::Wing,
};
static DIAMOND: BVenueSpec = BVenueSpec {
    label: "Diamond",
    after: "Education City's diamond, faceted triangles catching the light",
    shell: BShell::Diamond,
    roof: BRoof::Ring,
    show: BLightShow::Sparkle,
    tiers: 15,
    panels: 72,
    rise: 7.0,
    seat: BSeat::Blade,
};
static CONTAINERS: BVenueSpec = BVenueSpec {
    label: "Containers",
    after: "Stadium 974, stacked from shipping containers in colour blocks",
    shell: BShell::Containers,
    roof: BRoof::Open,
    show: BLightShow::Chase,
    tiers: 14,
    panels: 40,
    rise: 6.0,
    seat: BSeat::Bench,
};
static CAP: BVenueSpec = BVenueSpec {
    label: "Cap",
    after: "Al Thumama's gahfiya, a woven cap ringed round the field",
    shell: BShell::Cap,
    roof: BRoof::Ring,
    show: BLightShow::Orbit,
    tiers: 15,
    panels: 80,
    rise: 8.0,
    seat: BSeat::Pod,
};
static DUNES: BVenueSpec = BVenueSpec {
    label: "Dunes",
    after: "Ahmad bin Ali's façade, rising and falling like sand dunes",
    shell: BShell::Dunes,
    roof: BRoof::Open,
    show: BLightShow::Sweep,
    tiers: 14,
    panels: 56,
    rise: 7.0,
    seat: BSeat::Saddle,
};
static ARCHES: BVenueSpec = BVenueSpec {
    label: "Arches",
    after: "Khalifa International's twin arches spanning the bowl",
    shell: BShell::Arches,
    roof: BRoof::Ring,
    show: BLightShow::Pulse,
    tiers: 17,
    panels: 64,
    rise: 14.0,
    seat: BSeat::Shell,
};
