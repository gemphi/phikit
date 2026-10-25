//! A player dressed for the match the choice picks: her side's kit in the
//! sport's cut with her number, the picked sneaker (or the sport's boots,
//! or bare feet on sand), and her own hairstyle in her own colour — every
//! piece cut from the one body, so the renderer needs nothing but the
//! garments and the body they ride on.

use super::body::BBody;
use super::kit::BGarment;
use super::part::BMat;
use super::{hair, kit, shoe};
use crate::choice::BChoice;
use crate::look::roster::BPlayer;

/// The side's shirt and trim: the home side wears its home colour, the
/// away side its own unless that clashes, then its change kit.
pub fn colours(choice: &BChoice, team: usize) -> (crate::Rgb, crate::Rgb) {
    let (home, away) = (choice.home.spec(), choice.away.spec());
    if team == 0 { (home.home, home.trim) } else { (away.shirt(home), away.trim) }
}

/// Her skin, as a material.
pub fn skin(player: &BPlayer) -> BMat {
    BMat::plain(format!("Skin {}", player.look.spec().label), player.look.spec().skin, 0.45)
}

/// Everything she wears.
pub fn dress(body: &BBody, choice: &BChoice, team: usize, player: &BPlayer) -> Vec<BGarment> {
    let sport = choice.sport.spec();
    let (shirt, trim) = colours(choice, team);
    let mut out = kit::kit(body, sport.cut, choice.uniform.spec(), shirt, trim, player.number);
    out.extend(shoe::shoes(body, sport.footwear, choice.sneaker.spec(), shirt, trim));
    let look = player.look.spec();
    out.extend(hair::hair(body, player.hair.spec(), look.hair[usize::from(player.hair_colour) % look.hair.len()]));
    out
}
