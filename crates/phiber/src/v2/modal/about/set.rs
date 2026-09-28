//! About window links — `WM_MT_splash_about` (`bl_operators/wm.py:3527-3540`).

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BAboutLink {
    pub icon: &'static str,
    pub label: &'static str,
    /// `doc:<slug>` reads in the app; a URL leaves it.
    pub url: &'static str,
}

const fn l(icon: &'static str, label: &'static str, url: &'static str) -> BAboutLink {
    BAboutLink { icon, label, url }
}

pub static ABOUT_LINKS: &[BAboutLink] = &[
    l("release", "Release Notes", "doc:release_notes"),
    l("heart", "Credits", "doc:credits"),
    l("url", "License", "doc:license"),
    l("url", "Phiber Website", "https://phiace.dev"),
    l("heart", "Donate", "https://phiace.dev/support"),
];
