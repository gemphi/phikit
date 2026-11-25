//! The shops, worked as a hand works them: picking a shop, its pages,
//! a thing's own page, the bag, and paying from the wallet.

use appkit::pui::core::{PEvent, PFit, PMark, PMode, PPoint};
use appkit::pui::window::{PDeed, PStage};
use appkit::theme::PLook;
use serde_json::json;

use super::tools::icon;
use super::{DESK, drawn, part, rect, shown, staged};
use crate::Phiogy;

/// A press on the first line of words that reads so in the shops.
fn press(stage: &mut PStage, look: &PLook, wanted: &str) -> bool {
    let inside = rect(stage, "shop");
    let at = drawn(stage, look).iter().find_map(|mark| match mark {
        PMark::Text { at, text, .. } | PMark::Head { at, text, .. } if text == wanted && inside.contains(*at) => Some(at.plus(PPoint::new(2.0, 2.0))),
        _ => None,
    });
    let pressed = stage.send(&PEvent::PointerDown(at.unwrap_or_else(|| panic!("{wanted} in the shops"))));
    stage.lay(DESK, PMode::of_width(DESK.width));
    pressed
}

/// The window on a page of the shops, laid out.
fn on(page: &str) -> (Phiogy, PStage, std::rc::Rc<PLook>) {
    let (app, mut stage, look) = staged(DESK);
    assert!(app.desk().show(page), "{page}");
    stage.send(&PEvent::KeyUp(' '));
    stage.lay(DESK, PMode::of_width(DESK.width));
    (app, stage, look)
}

/// The rows of the rail's place of that name, by their ids.
fn rung(app: &Phiogy, place: &str) -> Option<Vec<String>> {
    app.desk().rail().into_iter().find(|(label, _)| label == place).map(|(_, rung)| rung.into_iter().map(|rung| rung.id).collect())
}

#[test]
fn step_29_a_shop_is_picked_and_an_open_one_stands_in_my_rail_with_its_own_menu() {
    let (app, mut stage, look) = staged(DESK);
    let desk = app.desk();
    // From the bar, with a room open, the shops are the page they are
    // picked from.
    assert!(stage.window.apply(&PDeed::Show("shop".to_string())));
    stage.lay(DESK, PMode::of_width(DESK.width));
    assert!(part(&stage).contains(&"shop"));
    let text = shown(&stage, &look);
    let word = |name: &str| desk.text.get(name).to_string();
    assert!([word("shop.title"), "Shop One".to_string(), "Looks and pups. · 3 items".to_string(), desk.text.fill("shop.count", "n", "2")].iter().all(|line| text.contains(line)), "{text:?}");
    // One the index opens from the start is in the rail: its landing,
    // its menu under its headings, and the bag.
    assert_eq!(rung(&app, "Shop One"), Some(vec!["s_one".to_string(), "s_one.women.coats".to_string(), "s_one.pft".to_string(), "s_one.theme".to_string(), "_bag".to_string()]));
    assert!(press(&mut stage, &look, &word("shop.close")));
    assert_eq!((rung(&app, "Shop One"), desk.kept.get("shop")["phiogy/s_one"].clone()), (None, json!(false)));
    // Opened, it is there again, and its landing is shown: its own
    // picture, and the grid it names.
    assert!(press(&mut stage, &look, &word("shop.open")));
    assert_eq!((desk.open.borrow().as_str(), rung(&app, "Shop One").is_some()), ("s_one", true));
    let text = shown(&stage, &look);
    assert!(["Shop One", "Looks and pups.", "New in", "Teal Signal", "$7,000", "Bold Pup #1"].iter().all(|line| text.contains(&line.to_string())) && !text.contains(&"Plum Line".to_string()), "{text:?}");
    let shop = rect(&stage, "shop");
    let pictured = |stage: &PStage, fit: PFit| drawn(stage, &look).iter().filter(|mark| matches!(mark, PMark::Picture { rect, fit: known, .. } if *known == fit && shop.contains(rect.center()))).count();
    // The landing's picture covers; a look is whole, a pup a cell.
    assert_eq!((pictured(&stage, PFit::Cover), pictured(&stage, PFit::Whole), pictured(&stage, PFit::Cell { column: 8, index: 0 })), (1, 1, 1));
}

#[test]
fn step_30_a_page_of_a_shop_is_its_things_and_a_thing_has_a_page_of_its_own() {
    let (app, mut stage, look) = on("s_one.women.coats");
    let desk = app.desk();
    let word = |name: &str| desk.text.get(name).to_string();
    let text = shown(&stage, &look);
    assert!(["Coats", "Teal Signal", "Plum Line", "$900"].iter().all(|line| text.contains(&line.to_string())) && !text.contains(&"Bold Pup #1".to_string()), "{text:?}");
    // How a thing stands is a press away from the next thing.
    assert!(press(&mut stage, &look, &word("shop.put")));
    assert_eq!(desk.bag().iter().map(|line| (line.id.clone(), line.shop.clone(), line.cost)).collect::<Vec<_>>(), [("lk1".to_string(), "s_one".to_string(), 7000.0)]);
    assert!(shown(&stage, &look).contains(&word("shop.inbag")));
    assert_eq!(desk.rail().into_iter().flat_map(|(_, rung)| rung).find(|rung| rung.id == "_bag").map(|rung| rung.count), Some(1));
    assert!(press(&mut stage, &look, &word("shop.inbag")) && desk.bag().is_empty());
    // A press on a thing opens its own page, the kit's form: its name and
    // what it is at the head, the way back and the bag in its bar, its
    // picture held at the left, its tabs at the right.
    assert!(press(&mut stage, &look, "Teal Signal"));
    assert_eq!(desk.now.product.borrow().as_deref(), Some("s_one/lk1"));
    let text = shown(&stage, &look);
    let between = format!(" {} ", word("between"));
    assert!([["Outerwear", "$7,000", "Shop One", "Common"].join(&between), "Teal Signal".to_string(), word("thing.info"), word("thing.stats"), word("thing.related"), "Cut for after the show.".to_string(), "Colour".to_string(), "Fabric".to_string(), "silk".to_string()].iter().all(|line| text.contains(line)), "{text:?}");
    // A thing with no ledger has no trail and nobody who held it.
    assert!(!text.contains(&word("thing.trail")) && !text.contains(&word("thing.owner")), "{text:?}");
    // Its picture stands at the top of the side column; its facts at the
    // right of it, each a row: its name, then its value in the right half.
    let whole = |stage: &PStage| drawn(stage, &look).iter().find_map(|mark| match mark {
        PMark::Picture { name, fit: PFit::Top, rect, .. } if rect.size.width > 200.0 => Some((name.clone(), *rect)),
        _ => None,
    });
    let (first, frame) = whole(&stage).expect("its picture");
    assert_eq!(first, "phiegg:worlds/w_brocade.png");
    let column = |wanted: &str| super::written(&stage, &look, wanted).map(|at| at.x).expect("drawn");
    let edge = frame.origin.x + frame.size.width;
    assert!(column("Fabric") > edge && column("silk") > column("Fabric") && column(&word("thing.info")) > edge, "{} {} {edge}", column("Fabric"), column("silk"));
    // Its views are a band under it: a press on one shows it.
    assert!(press(&mut stage, &look, "Back"));
    assert_eq!(whole(&stage).map(|(name, _)| name).as_deref(), Some("phiegg:worlds/w_carousel.png"));
    // What can be done with it floats down its picture's corner, as on
    // Phiegg's item page, round icons over it: I like it, and that is
    // kept, and no more.
    let tap = |stage: &mut PStage, name: &str| {
        let pressed = super::tools::tool(stage, &look, name);
        stage.lay(DESK, PMode::of_width(DESK.width));
        pressed
    };
    let corner = super::tools::icon(&stage, &look, "heart").expect("its like");
    assert!(frame.contains(corner.center()) && corner.center().x < frame.center().x && corner.center().y < frame.center().y, "{corner:?} {frame:?}");
    assert!(tap(&mut stage, "heart") && desk.liked("s_one/lk1"));
    assert!(tap(&mut stage, "heart") && !desk.liked("s_one/lk1"));
    // Full screen: its views are the viewer's album, the one shown first.
    assert!(tap(&mut stage, "maximize"));
    assert_eq!((stage.window.modal(), desk.view.get(), desk.now.album.borrow().len()), (Some("view"), Some(1), 2));
    assert!(stage.send(&PEvent::KeyDown(PEvent::ESCAPE)));
    stage.lay(DESK, PMode::of_width(DESK.width));
    // The bag is at the end of its bar.
    assert!(press(&mut stage, &look, &word("shop.put")) && desk.bag().len() == 1);
    assert!(press(&mut stage, &look, &word("shop.inbag")) && desk.bag().is_empty());
    // Its figures are a tab of their own.
    assert!(press(&mut stage, &look, &word("thing.stats")));
    let text = shown(&stage, &look);
    assert!(text.contains(&word("thing.stat.price")) && !text.contains(&"Fabric".to_string()), "{text:?}");
    // Related: the same thing in its other colours, and more like it from
    // its page of the menu; a press goes to one, and back.
    assert!(press(&mut stage, &look, &word("thing.related")));
    let text = shown(&stage, &look);
    assert!([word("shop.colours"), "Plum Signal".to_string(), word("shop.more"), "Plum Line".to_string()].iter().all(|line| text.contains(line)), "{text:?}");
    assert!(press(&mut stage, &look, "Plum Signal"));
    assert_eq!(desk.now.product.borrow().as_deref(), Some("s_one/lk3"));
    assert!(press(&mut stage, &look, "Teal Signal"));
    assert_eq!(desk.now.product.borrow().as_deref(), Some("s_one/lk1"));
    // The way back is in its bar: its page of the menu, and its shop.
    assert!(press(&mut stage, &look, "Coats"));
    assert_eq!((desk.open.borrow().as_str(), desk.now.product.borrow().clone()), ("s_one.women.coats", None));
}

#[test]
fn step_30b_a_thing_phiegg_sold_has_its_trail_its_figures_and_who_held_it() {
    let (app, mut stage, look) = on("s_one.women.coats");
    let desk = app.desk();
    let word = |name: &str| desk.text.get(name).to_string();
    *desk.now.product.borrow_mut() = Some("s_one/g_c_010_lk11".to_string());
    stage.lay(DESK, PMode::of_width(DESK.width));
    // What is said of it, its points, and its facts, a list as pills.
    let text = shown(&stage, &look);
    assert!(["Ivory Glass, cut for midsummer.", "Swap into any pool", "Occasion", word("thing.trail").as_str(), word("thing.owner").as_str()].iter().all(|line| text.contains(&line.to_string())), "{text:?}");
    // Its trail: each sale or transfer in Phiegg's ledger, the newest
    // first, who it went from and to by name, what and when, the price.
    assert!(press(&mut stage, &look, &word("thing.trail")));
    let text = shown(&stage, &look);
    let between = format!(" {} ", word("between"));
    assert!([word("thing.went").replace("{from}", "Gemma Laurent").replace("{to}", "Gemma Vane"), [word("thing.event.transfer"), "2026-11-06".to_string()].join(&between), "$23,574".to_string(), [word("thing.event.sale"), "2026-08-15".to_string()].join(&between)].iter().all(|line| text.contains(line)), "{text:?}");
    // Its figures, from the ledger.
    assert!(press(&mut stage, &look, &word("thing.stats")));
    let text = shown(&stage, &look);
    assert!([word("thing.stat.sales"), word("thing.stat.transfers"), word("thing.stat.high"), word("thing.stat.holders"), "2026-11-06".to_string()].iter().all(|line| text.contains(line)), "{text:?}");
    // Who held it, each once, the newest first, named (a person or an
    // agent); a press opens their record.
    assert!(press(&mut stage, &look, &word("thing.owner")));
    let text = shown(&stage, &look);
    assert!(!text.iter().any(|line| line.starts_with("ag_") || line.starts_with("u_")), "{text:?}");
    assert!(text.contains(&"Anouk Valois".to_string()), "{text:?}");
}

#[test]
fn step_31_the_bag_is_paid_from_the_wallet_and_what_was_bought_stays_mine() {
    let (app, mut stage, look) = on("s_one.pft");
    let desk = app.desk();
    let word = |name: &str| desk.text.get(name).to_string();
    let shop = desk.shop("s_one").expect("the shop's own file");
    desk.pick("s_one", &shop.product[1]);
    desk.pick("s_one", &shop.product[2]);
    // The bag: its lines, the total, and what the wallet holds.
    assert!(desk.show("_bag"));
    stage.send(&PEvent::KeyUp(' '));
    stage.lay(DESK, PMode::of_width(DESK.width));
    let text = shown(&stage, &look);
    assert!(["Plum Line", "Bold Pup #1", "Subtotal", "Total", "In your wallet"].iter().all(|line| text.contains(&line.to_string())), "{text:?}");
    // Its sums stand on grounds of their own: the subtotal, the total
    // and what the wallet holds are each that much.
    assert_eq!(text.iter().filter(|line| *line == "$2,500").count(), 3, "{text:?}");
    assert!(super::grounded(&stage, &look, "$2,500"));
    // Paid: the bag is empty, the things are mine, the wallet has less,
    // and its history says for what.
    assert!(press(&mut stage, &look, &word("shop.pay")));
    let text = shown(&stage, &look);
    assert!([word("shop.empty"), word("shop.yours"), "Plum Line".to_string(), "In your wallet".to_string(), "$0".to_string()].iter().all(|line| text.contains(line)), "{text:?}");
    assert_eq!((desk.bag().len(), desk.bought().len(), desk.wallet()), (0, 2, 0.0));
    let history = desk.kept.get("history");
    assert_eq!((&history[0]["amt"], &history[0]["what"], &history[0]["coin"]), (&json!(-2500.0), &json!("Shop - 2 bought"), &json!("usd")));
    // The other coins a person starts with are as they were.
    assert_eq!(desk.kept.get("wallet")["egg"], json!(12000.0));
    // What is bought stays bought, and says so on its card.
    desk.pick("s_one", &shop.product[1]);
    assert!(desk.bag().is_empty() && desk.stands("s_one", &shop.product[1]) == "shop.owned");
    // A bag the wallet cannot pay is not paid, and nothing is taken.
    desk.pick("s_one", &shop.product[0]);
    stage.lay(DESK, PMode::of_width(DESK.width));
    assert!(press(&mut stage, &look, &word("shop.pay")));
    assert!(shown(&stage, &look).contains(&word("shop.short")));
    assert_eq!((desk.bag().len(), desk.bought().len(), desk.wallet()), (1, 2, 0.0));
    // A line is taken out of the bag at its end.
    let out = icon(&stage, &look, "x").expect("the way to take it out").center();
    assert!(stage.send(&PEvent::PointerDown(out)) && desk.bag().is_empty());
}
