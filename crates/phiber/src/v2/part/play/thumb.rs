//! The picker's card pictures by key: `play:<row>:<card>:<home>:<theme>`
//! — the row's slug, the card, and the two picks the pictures are painted
//! in (the kit and the shoes wear the home side's colours, the venue and
//! the seats the theme's). The thumbnail renderer caches by key, so a
//! picture is painted once per look it can have.

use phiplay::BChoice;
use phiplay::build::BImage;
use phiplay::pick::{BPick, thumb};

const PREFIX: &str = "play:";

pub fn key(pick: BPick, i: usize, choice: &BChoice) -> String {
    format!("{PREFIX}{}:{i}:{}:{}", pick.spec().slug, BPick::Home.chosen(choice), BPick::Theme.chosen(choice))
}

/// The picture a key names; `None` for a key that is not the picker's.
pub fn image(key: &str) -> Option<BImage> {
    let mut parts = key.strip_prefix(PREFIX)?.split(':');
    let pick = BPick::by_slug(parts.next()?)?;
    let mut at = || parts.next().and_then(|p| p.parse::<usize>().ok());
    let (i, home, theme) = (at()?, at()?, at()?);
    let mut choice = BChoice::default();
    BPick::Home.choose(&mut choice, home);
    BPick::Theme.choose(&mut choice, theme);
    (i < pick.count()).then(|| thumb::thumb(pick, i, &choice))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_key_names_its_card_in_its_colours() {
        let choice = BChoice::default();
        let k = key(BPick::Uniform, 3, &choice);
        assert_eq!(image(&k), Some(thumb::thumb(BPick::Uniform, 3, &choice)));
        assert_eq!(image("play:uniform:99:0:0"), None);
        assert_eq!(image("/tmp/a.blend"), None);
    }
}
