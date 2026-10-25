//! What a player wears: the uniform's pattern and the shoes. The cut of the
//! uniform is the sport's — a vest for the court sports, a fitted short
//! sleeve for volleyball, a crop top on the sand, a jersey and socks on
//! grass (`BCut`) — and the pattern on it is the picker's.

pub mod footwear;
pub mod sneaker;
pub mod uniform;

/// How a sport's uniform is cut.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BCut {
    /// Sleeveless vest and shorts: basketball, netball.
    Vest,
    /// A fitted short-sleeved top and shorts: volleyball.
    Fitted,
    /// A crop top and briefs: beach volleyball.
    Crop,
    /// A short-sleeved jersey, shorts and long socks: soccer.
    Jersey,
}

/// What a sport is played in on the feet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BFootwear {
    /// Court shoes, the picker's sneaker.
    Shoes,
    /// Studded boots cut under the ankle, in the sneaker's colours.
    Boots,
    /// Nothing: the sand.
    Barefoot,
}
