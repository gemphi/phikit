//! The site: the web Phiogy's files, read by the names of
//! `data/site.tsv`.

use appkit::service::PTable;
use serde_json::Value;

use super::site;
use crate::{Phiogy, Site, data};

#[test]
fn the_site_is_read_from_files_written_as_the_web_phiogy_writes_its_own() {
    let site = site();
    assert_eq!((site.app.as_str(), site.start.as_str()), ("phiogy", "lora"));
    // The rail's places first, then every host of the hosts' file; the
    // ones the person joined are the rail's and those the file names.
    let venue: Vec<(&str, bool)> = site.venue.iter().map(|venue| (venue.label.as_str(), venue.joined)).collect();
    assert_eq!(venue, [("Direct", true), ("Phiace", true), ("Opal Atelier", true), ("The Swan Salon", false)]);
    assert_eq!(site.joined().count(), 3);
    // A place's rooms are what it holds and what its headings hold.
    let room: Vec<&str> = site.venue[1].plate.iter().map(|plate| plate.id.as_str()).collect();
    assert_eq!(room, ["lobby", "lounge"]);
    // What the rooms' file adds: the topic, the kind, the people by name.
    let (venue, lobby) = site.plate("lobby").expect("the lobby");
    assert_eq!((venue.label.as_str(), lobby.topic.as_str(), lobby.voice), ("Phiace", "Say hello, tell us what you are building.", false));
    assert_eq!(lobby.member, ["You", "Lora Vale", "Tavi Reyes"]);
    assert!(site.plate("lounge").is_some_and(|(_, plate)| plate.voice && plate.icon == "volume"));
    // One nobody named is shown by its id; a room the file does not
    // have is its label alone.
    assert_eq!(site.plate("n_front").map(|(_, plate)| plate.member.clone()), Some(vec!["u_ines".to_string()]));
    assert!(site.plate("n_swan").is_some_and(|(venue, plate)| venue.count == 90 && plate.topic.is_empty()));
    assert!(site.plate("nowhere").is_none());
}

#[test]
fn the_web_phiogy_s_own_files_on_this_machine_are_read_whole() {
    let key = PTable::read(data::SITE);
    let path = Phiogy::source();
    let site = Site::open(&path);
    // What the files hold, counted from the files themselves.
    let read = |path: &str| serde_json::from_str::<Value>(&std::fs::read_to_string(path).unwrap_or_else(|_| panic!("{path} is there"))).expect("JSON");
    let doc = read(&path);
    let count = |value: &Value, name: &str| value[key.get(name)].as_array().map_or(0, Vec::len);
    let root = std::path::Path::new(&path).parent().and_then(|folder| folder.parent()).expect("a folder over it");
    let host = read(&root.join(doc[key.get("host")].as_str().expect("named")).to_string_lossy());
    assert!(count(&doc, "nav") > 0 && count(&host, "host") > 0);
    assert_eq!(site.venue.len(), count(&doc, "nav") + count(&host, "host"));
    assert_eq!(site.joined().count(), count(&doc, "nav") + count(&doc, "joined"));
    // It begins in a room it has, and every room has a name.
    assert!(site.plate(&site.start).is_some(), "{}", site.start);
    assert!(site.venue.iter().flat_map(|venue| &venue.plate).all(|plate| !plate.id.is_empty() && !plate.label.is_empty()));
    assert_eq!(site.app, doc[key.get("app")].as_str().unwrap_or_default());
}

#[test]
fn a_site_that_cannot_be_read_has_nothing_and_says_where_it_looked() {
    let site = Site::open("nowhere/data/phiogy.json");
    assert!(site.venue.is_empty() && site.start.is_empty());
    assert_eq!(site.from, "nowhere/data/phiogy.json");
}

#[test]
fn a_host_is_as_discord_shows_a_server_its_categories_its_channels_and_their_sub_channels() {
    let site = Site::open(&Phiogy::source());
    // A host with categories: each category a heading over its channels,
    // as Phiegg keeps them (Start here, Community, Live, World); no room
    // under a made-up Text or Voice heading.
    let host = site.venue.iter().find(|venue| venue.label == "Chrome Cherubs Host").expect("a host with categories");
    let heads: Vec<&str> = host.plate.iter().map(|plate| plate.head.as_str()).collect();
    assert!(["Start here", "Community", "Live", "World"].iter().all(|head| heads.contains(head)), "{heads:?}");
    assert!(site.venue.iter().filter(|venue| venue.joined).chain(site.venue.iter()).flat_map(|venue| &venue.plate).all(|plate| plate.head != "Text" || plate.depth == 0));
    // A world's cities are its sub-channels, a step in, under the same
    // heading as the world.
    let opal = site.venue.iter().find(|venue| venue.label == "Opal Atelier").expect("Opal Atelier");
    let world = opal.plate.iter().position(|plate| plate.label == "Opalia").expect("its world");
    let city: Vec<&str> = opal.plate[world + 1..].iter().take_while(|plate| plate.depth == 1).map(|plate| plate.label.as_str()).collect();
    assert_eq!(city, ["Couture Row", "Marquee Market"]);
    assert_eq!(opal.plate[world].depth, 0);
    // A host of no category stands in one group: no heading at all.
    assert!(opal.plate.iter().all(|plate| plate.head.is_empty()));
}
