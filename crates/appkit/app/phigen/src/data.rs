//! Phigen's tables, as the files in `data/` hold them. They are built
//! into the program, so the same app runs where there is no disk to read:
//! in a browser.

pub const APP: &str = include_str!("../data/app.tsv");
pub const SPACE: &str = include_str!("../data/space.tsv");
pub const LAYOUT: &str = include_str!("../data/layout.tsv");
pub const HOST: &str = include_str!("../data/host.tsv");
pub const REGION: &str = include_str!("../data/region.tsv");
pub const PART: &str = include_str!("../data/part.tsv");
pub const MODAL: &str = include_str!("../data/modal.tsv");
pub const TEXT: &str = include_str!("../data/text.tsv");
pub const THEME: &str = include_str!("../data/theme.tsv");
pub const LOOK: &str = include_str!("../data/look.tsv");
pub const ACTIVITY: &str = include_str!("../data/activity.tsv");
pub const MENU: &str = include_str!("../data/menu.tsv");
pub const USER: &str = include_str!("../data/user.tsv");
pub const MODEL: &str = include_str!("../data/model.tsv");
pub const CARD: &str = include_str!("../data/card.tsv");
pub const FIND: &str = include_str!("../data/find.tsv");
