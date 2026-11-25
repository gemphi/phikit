//! The app's server and the places over the network: a place read as
//! it is written, a picture of Phiegg's called by its key on the
//! server, as puikit's own rules call it, and where a picture is there.

use crate::hosted::{Hosted, key};
use crate::net::{Place, far};

#[test]
fn a_place_is_read_as_it_is_written_with_its_scheme_port_and_path() {
    let here = Place::read("localhost:4310");
    assert!(!here.secure && here.host == "localhost" && here.port == 4310 && here.base.is_empty() && here.near(), "{here:?}");
    assert_eq!(here.address(), "http://localhost:4310");
    let server = Place::read("https://phiegg.com/wrt/");
    assert!(server.secure && server.host == "phiegg.com" && server.port == 443 && server.base == "/wrt" && !server.near(), "{server:?}");
    // Its own port is not said; another one is.
    assert_eq!((server.authority(), server.address()), ("phiegg.com".to_string(), "https://phiegg.com/wrt".to_string()));
    assert_eq!(Place::read("http://192.168.1.4:4300/").authority(), "192.168.1.4:4300");
    // Over the internet it is waited for longer than a place here.
    let little = std::time::Duration::from_millis(400);
    assert!(server.patience(little) > little && here.patience(little) == little);
    assert!(far("https://phiegg.com/phiogy/data/phiogy.json") && !far("C:/Users/x/phiogy.json"));
}

#[test]
fn a_picture_of_phiegg_s_is_called_by_its_key_on_the_server() {
    // The keys the live CDN serves (phiegg.com/cdn/<key>).
    assert_eq!(key("worlds/w_aria.png"), "assets/scenes/w_aria.png");
    assert_eq!(key("portraits/sheet-01.webp"), "assets/portraits/sheet-01.webp");
    assert_eq!(key("logos/w_aria.svg"), "worlds/w_aria/logo.svg");
    assert_eq!(key("world/w_aria/hero.png"), "worlds/w_aria/hero.png");
    // A logo that is not drawn is no rule's: its path is its key.
    assert_eq!(key("logos/w_aria.png"), "logos/w_aria.png");
    assert_eq!(key("look/l_01.png"), "look/l_01.png");
    // Where an app's picture is on the server; never out of its folder.
    let hosted = Hosted::own();
    assert_eq!(hosted.address("phiegg:/worlds/w_aria.png").as_deref(), Some("https://phiegg.com/cdn/assets/scenes/w_aria.png"));
    assert_eq!(hosted.address("phiogy:logo/phiogy.png").as_deref(), Some("https://phiegg.com/phiogy/logo/phiogy.png"));
    assert!(hosted.address("phiegg:../secret.png").is_none() && hosted.address("file:///C:/x.png").is_none());
}

/// Every rule of puikit's own file has its row in `data/media.tsv`, and
/// every row's words are in that file, where it is on this disk.
#[test]
fn the_keys_are_puikit_s_own_rules() {
    let file = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../../../puikit/app/phiegg/src/data/media-key.json");
    let Ok(text) = std::fs::read_to_string(file) else { return };
    let rule: serde_json::Value = serde_json::from_str(&text).expect("the rules");
    let rule = rule["rule"].as_array().expect("its rules");
    let row = crate::data::row(crate::data::MEDIA);
    for one in row.iter() {
        let from = one[0].trim_end_matches('/');
        assert!(rule.iter().any(|rule| rule["from"].as_str().is_some_and(|said| said.contains(from))), "{from} is no rule of {file}");
        assert!(rule.iter().any(|rule| rule["to"].as_str().is_some_and(|said| said.contains(one[1].trim_end_matches('/')) || said.starts_with("assets/"))), "{}", one[1]);
    }
    // The rule of many folders is a row for each.
    let many = rule.iter().filter_map(|rule| rule["from"].as_str()).map(|from| from.matches('|').count() + 1).sum::<usize>();
    assert_eq!(row.len(), many);
}
