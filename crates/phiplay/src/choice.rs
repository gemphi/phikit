//! A match as the picker leaves it: one of each set. The default is the
//! demo's: basketball, Ember against Fjord, the Panel strip, the Apex
//! high-top, the Bloom theme in the Lantern bowl.

use crate::kit::sneaker::BSneaker;
use crate::kit::uniform::BUniform;
use crate::sport::BSport;
use crate::team::BTeam;
use crate::theme::BTheme;
use crate::turf::BTurf;
use crate::venue::BVenue;
use crate::venue::seat::BSeat;

/// The crowd a venue holds by default.
pub const CROWD: u32 = 3000;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BChoice {
    pub sport: BSport,
    pub home: BTeam,
    pub away: BTeam,
    pub uniform: BUniform,
    pub sneaker: BSneaker,
    pub theme: BTheme,
    pub venue: BVenue,
    /// The surface; `None` lays the sport's own.
    pub turf: Option<BTurf>,
    /// The seats; `None` fits the venue's own.
    pub seat: Option<BSeat>,
    /// Fans in the stands.
    pub crowd: u32,
}

impl Default for BChoice {
    fn default() -> Self {
        Self {
            sport: BSport::Basketball,
            home: BTeam::Ember,
            away: BTeam::Fjord,
            uniform: BUniform::Panel,
            sneaker: BSneaker::Apex,
            theme: BTheme::Bloom,
            venue: BVenue::Lantern,
            turf: None,
            seat: None,
            crowd: CROWD,
        }
    }
}

impl BChoice {
    /// The surface laid: the picked one, or the sport's.
    pub fn turf(&self) -> BTurf {
        self.turf.unwrap_or(self.sport.spec().turf)
    }

    /// The seats fitted: the picked ones, or the venue's.
    pub fn seat(&self) -> BSeat {
        self.seat.unwrap_or(self.venue.spec().seat)
    }
}
