//! The account: who the person is, their wallet, the look, and what
//! it says where Phiora does not answer or lets nobody in.

use std::rc::Rc;

use appkit::pui::PApp;
use appkit::pui::core::{PEvent, PMark, PMode, PPoint};
use appkit::pui::window::{PDeed, PStage};
use appkit::theme::{PLook, PTheme};

use super::exchange::line;
use super::{AT, DESK, drawn, shown, site, staged};
use crate::Phiogy;
use crate::kept::Kept;
use crate::phiora::Phiora;

/// Open the account, and say what the window shows.
fn opened(stage: &mut PStage, look: &PLook) -> Vec<String> {
    assert!(stage.window.apply(&PDeed::Open("account".to_string())));
    stage.lay(DESK, PMode::of_width(DESK.width));
    shown(stage, look)
}

/// A press on the last line of words that reads so: the account's,
/// which is drawn over everything.
fn press(stage: &mut PStage, look: &PLook, wanted: &str) -> bool {
    let at = drawn(stage, look).iter().rev().find_map(|mark| match mark {
        PMark::Text { at, text, .. } if text == wanted => Some(at.plus(PPoint::new(2.0, 2.0))),
        _ => None,
    });
    stage.send(&PEvent::PointerDown(at.unwrap_or_else(|| panic!("{wanted} in the account"))))
}

#[test]
fn step_32_the_account_is_who_i_am_my_wallet_and_the_look() {
    let (app, mut stage, look) = staged(DESK);
    let desk = app.desk();
    let text = opened(&mut stage, &look);
    let word = |name: &str| desk.text.get(name).to_string();
    // It stands at the window's right edge, as the web's does.
    let panel = stage.panel().expect("the account is open");
    assert!(panel.rect.origin.x > DESK.width / 2.0 && panel.rect.size.width < DESK.width / 2.0);
    // Nobody to Phiora: a guest; and signing in says what it needs.
    assert!([word("account.title"), word("account.guest"), word("account.nobody"), word("account.offline"), word("account.phione")].iter().all(|line| text.contains(line)), "{text:?}");
    // The wallet a person starts with: each coin, and all of it in
    // dollars at what each coin is worth; what each is, and its sum.
    assert!(["Total", "USD", "$EGG", "$UM", "ETH", "OGY"].iter().all(|line| text.contains(&line.to_string())), "{text:?}");
    // Every sum stands on a ground of its own, in the table's right half.
    let sum = ["$7,709", "2,500", "12,000", "140", "0.42", "500"];
    assert!(sum.iter().all(|sum| super::grounded(&stage, &look, sum)), "{text:?}");
    assert!(!super::grounded(&stage, &look, "Total"));
    // What was paid in a shop is gone from it.
    let shop = desk.shop("s_one").expect("the shop");
    desk.pick("s_one", &shop.product[1]);
    assert!(desk.pay() && super::grounded(&stage, &look, "1,600"));
    // The look is chosen here too.
    assert!(press(&mut stage, &look, desk.text.get("look.dark")));
    assert_eq!(desk.shade.name(), "dark");
    // The profile is a press away.
    assert!(press(&mut stage, &look, &word("account.profile")));
    assert_eq!(stage.window.modal(), Some("profile"));
}

#[test]
fn where_phiora_answers_the_account_says_who_it_says_i_am() {
    // The line answers who a page is as Phiora does: a guest, whom it
    // lets be nobody else.
    let line = line();
    let kept = Kept::open(None, Some(Phiora::at(&line.at())));
    assert_eq!(kept.at, Some(line.at()));
    assert!(kept.me()["id"].as_str().is_some_and(|id| id.starts_with("guest_")), "{}", kept.me());
    let look = Rc::new(Phiogy::look(""));
    let app = Phiogy::holding(Rc::clone(&look), site(), Phiogy::board(), Ok(AT.to_string()), None).keeping(kept);
    let mut stage = app.stage();
    stage.lay(DESK, PMode::of_width(DESK.width));
    let text = opened(&mut stage, &look);
    let word = |name: &str| app.desk().text.get(name).to_string();
    // A note is broken to fit: its words are there, over two lines.
    let all = text.join(" ");
    assert!(text.contains(&word("account.guest")) && all.contains(&word("account.nodemo")) && !all.contains(&word("account.offline")), "{text:?}");
    // Being somebody it does not offer changes nothing.
    assert!(!app.desk().kept.sign(Some("u_nobody")));
    assert!(app.desk().kept.me()["id"].as_str().is_some_and(|id| id.starts_with("guest_")));
    // Being nobody again is a new guest, with a session of their own.
    let was = app.desk().kept.me()["id"].clone();
    assert!(!app.desk().kept.sign(None) || app.desk().kept.me()["id"] != was);
}

#[test]
fn step_59_the_profile_sets_which_of_four_ways_the_light_runs_how_strong_how_far_and_its_colour() {
    let (app, mut stage, look) = staged(DESK);
    let desk = app.desk();
    let user = appkit::service::PTable::read(crate::data::USER);
    let word = |name: &str| user.get(name).to_string();
    assert!(stage.window.apply(&PDeed::Open("profile".to_string())));
    stage.lay(DESK, PMode::of_width(DESK.width));
    // Across at first; from the foot up once chosen, and kept.
    assert_eq!(desk.shade.size("surface.turn"), Some(90.0));
    // Any of four ways: from the right, from the top, from the foot;
    // each scrolled into sight where the panel is short.
    let body = stage.panel().expect("the profile").body;
    let sight = |stage: &mut PStage, said: &str| {
        let at = |stage: &PStage| drawn(stage, &look).iter().rev().find_map(|mark| match mark {
            PMark::Text { at, text, .. } if text == said => Some(at.y),
            _ => None,
        });
        for _ in 0..20 {
            if at(stage).is_some_and(|y| y + 40.0 < body.origin.y + body.size.height) {
                break;
            }
            stage.send(&PEvent::Scroll(body.center(), PPoint::new(0.0, -60.0)));
        }
        press(stage, &look, said)
    };
    assert!(sight(&mut stage, &word("prefer.back")));
    assert_eq!(desk.shade.size("surface.turn"), Some(270.0));
    assert!(sight(&mut stage, &word("prefer.down")));
    assert_eq!(desk.shade.size("label.turn"), Some(180.0));
    assert!(sight(&mut stage, &word("prefer.up")));
    assert_eq!((desk.shade.size("surface.turn"), desk.shade.size("label.turn")), (Some(0.0), Some(0.0)));
    stage.send(&PEvent::KeyUp(' '));
    assert!(desk.choice.get("prefer").contains("surface.turn 0"), "{}", desk.choice.get("prefer"));
    // Fainter, and half the way, further down the profile.
    let body = stage.panel().expect("the profile").body;
    stage.send(&PEvent::Scroll(body.center(), PPoint::new(0.0, -100000.0)));
    assert!(press(&mut stage, &look, &word("prefer.faint")) && press(&mut stage, &look, &word("prefer.half")));
    assert_eq!((desk.shade.size("shade.rest"), desk.shade.size("light.to")), (Some(0.45), Some(0.5)));
    // The light's colour from the wheel: a dot pressed.
    // The wheel, scrolled to where it is in sight.
    let red = |stage: &PStage| drawn(stage, &look).iter().rev().find_map(|mark| match mark {
        PMark::Fill { rect, color, .. } if color.r > 0.9 && color.g < 0.2 && color.b < 0.2 && rect.size.width < 20.0 && body.contains(rect.center()) => Some(rect.center()),
        _ => None,
    });
    for _ in 0..20 {
        if red(&stage).is_some() {
            break;
        }
        stage.send(&PEvent::Scroll(body.center(), PPoint::new(0.0, -120.0)));
    }
    let dot = red(&stage).expect("a red dot of the wheel");
    assert!(stage.send(&PEvent::PointerDown(dot)));
    let light = desk.shade.color("shade.light").expect("the light");
    assert!(light.r > 0.9 && light.g < 0.2);
    // Kept, the next window shows it so.
    stage.send(&PEvent::KeyUp(' '));
    let kept = desk.choice.get("prefer");
    let other = appkit::theme::PShade::new(vec![("one".to_string(), appkit::theme::PLook::read("size\tsurface.turn\t90\n"))]);
    other.prefer(&kept);
    assert_eq!(other.size("surface.turn"), Some(0.0));
}

#[test]
fn auto_at_the_end_of_the_mode_row_follows_the_system_s_light_and_dark_until_a_mode_is_chosen() {
    let (app, mut stage, look) = staged(DESK);
    let desk = app.desk();
    let user = appkit::service::PTable::read(crate::data::USER);
    let word = |name: &str| user.get(name).to_string();
    assert!(stage.window.apply(&PDeed::Open("profile".to_string())));
    stage.lay(DESK, PMode::of_width(DESK.width));
    // Auto stands at the right end of the Mode row, after Light and Dark.
    let at = |stage: &PStage, said: &str| drawn(stage, &look).iter().rev().find_map(|mark| match mark {
        PMark::Text { at, text, .. } if *text == said => Some(*at),
        _ => None,
    });
    let (light, auto) = (at(&stage, &word("value.light")).expect("Light"), at(&stage, &word("value.auto")).expect("Auto"));
    assert!(auto.x > light.x + 100.0 && (auto.y - light.y).abs() < 1.0, "{light:?} {auto:?}");
    // On: the look follows the system, light and dark, with the surface it had.
    let first = desk.shade.name().to_string();
    assert!(press(&mut stage, &look, &word("value.auto")));
    let (day, night) = desk.shade.following().expect("on Auto");
    assert_eq!(night, first);
    assert!(desk.shade.system(false) && desk.shade.name() == day);
    assert!(desk.shade.system(true) && desk.shade.name() == night);
    stage.send(&PEvent::KeyUp(' '));
    assert_eq!(desk.choice.get("look"), format!("auto {day} {night}"));
    // A mode chosen by hand stops it.
    assert!(press(&mut stage, &look, &word("value.light")));
    assert_eq!(desk.shade.following(), None);
    assert!(!desk.shade.system(true));
}

#[test]
fn the_profile_offers_silver_and_sixteen_glass_looks_as_vision_pro_and_horizon_wear_glass() {
    let (app, mut stage, look) = staged(DESK);
    let desk = app.desk();
    let user = appkit::service::PTable::read(crate::data::USER);
    let word = |name: &str| user.get(name).to_string();
    assert!(stage.window.apply(&PDeed::Open("profile".to_string())));
    stage.lay(DESK, PMode::of_width(DESK.width));
    // Its third choice is the glass; Silver is a surface.
    let text = shown(&stage, &look);
    assert!([word("axis.lens"), word("value.silver"), word("value.vision"), word("value.horizon"), word("value.graphite")].iter().all(|said| text.contains(said)), "{text:?}");
    // A glass chosen is that look, as dark or light as it is; Silver is
    // brushed steel. (The profile is scrolled to its Glass row.)
    let surface = super::written(&stage, &look, &word("axis.surface")).expect("the surface row");
    stage.send(&PEvent::Scroll(surface, PPoint::new(0.0, -400.0)));
    stage.lay(DESK, PMode::of_width(DESK.width));
    assert!(press(&mut stage, &look, &word("value.vision")));
    assert_eq!(desk.shade.name(), "vision");
    assert!(press(&mut stage, &look, &word("value.silver")));
    assert_eq!(desk.shade.name(), "silver");
}
