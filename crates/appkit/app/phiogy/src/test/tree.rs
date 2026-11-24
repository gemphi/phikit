//! The tree of the web Phiogy's own files on this machine, every level
//! of it filled: the window's hosts, the places (servers) with their
//! rooms (channels), a room to call in, the pages (screens), the line's
//! calls, and in every place its people, no two of whom look alike or
//! are called alike. `puikit/app/phiogy/server.py` writes the files;
//! a test that fails here is fixed there, by its data step, not here.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use appkit::service::PTable;
use serde_json::Value;

use crate::comp::Limit;
use crate::site::Member;
use crate::{Phiogy, Site, data};

/// The site as the window reads it, and its rooms' file as it is
/// written, with the names `data/site.tsv` gives.
fn real() -> (Site, Value, PTable) {
    let key = PTable::read(data::SITE);
    let site = Site::open(&Phiogy::source());
    let read = |text: Option<String>| serde_json::from_str::<Value>(&text.unwrap_or_default()).unwrap_or_default();
    let doc = read(std::fs::read_to_string(&site.from).ok());
    let people = read(site.load(doc[key.get("people")].as_str().unwrap_or_default()));
    (site, people, key)
}

/// Who each place seats, by id, in the file's order; and who wrote in
/// the rooms of each place, which the data step seats first.
fn seated(people: &Value, key: &PTable) -> (Vec<(String, Vec<String>)>, HashMap<String, HashSet<String>>) {
    let every: Vec<&str> = people[key.get("member")].as_array().into_iter().flatten().filter_map(|one| one[key.get("id")].as_str()).collect();
    let place = people[key.get("space")].as_object().cloned().unwrap_or_default();
    let at = |index: &Value| index.as_u64().and_then(|index| every.get(index as usize)).map(|id| id.to_string());
    let seat = place.iter().map(|(name, list)| (name.clone(), list.as_array().into_iter().flatten().filter_map(at).collect())).collect();
    let mut wrote: HashMap<String, HashSet<String>> = HashMap::new();
    for room in people[key.get("room")].as_object().into_iter().flat_map(|room| room.values()) {
        let voice = room[key.get("kind")].as_str() == Some(key.get("voice"));
        let named = room[key.get("member")].as_array().into_iter().flatten().filter_map(Value::as_str);
        if let (false, Some(space)) = (voice, room[key.get("space")].as_str()) {
            wrote.entry(space.to_string()).or_default().extend(named.map(str::to_string));
        }
    }
    (seat, wrote)
}

/// What a person is told apart by: their face (a cell of a sheet),
/// their name and their handle.
fn look(site: &Site, member: &Member) -> [(&'static str, String); 3] {
    let face = site.face(&member.id).map(|face| format!("{}#{}", face.sheet, face.index)).unwrap_or_default();
    [("face", face), ("name", member.name.clone()), ("handle", member.handle.clone())]
}

#[test]
fn every_person_has_a_name_a_handle_a_face_a_role_and_a_bio() {
    let (site, _, _) = real();
    assert!(!site.member.is_empty(), "the people of {}", site.from);
    // The person using the app writes their own bio; everyone else has one.
    let bare = |member: &&Member| {
        let bio = member.bio.is_empty() && member.id != site.me;
        member.name.is_empty() || member.handle.is_empty() || member.role.is_empty() || site.face(&member.id).is_none() || bio
    };
    let missing: Vec<&str> = site.member.iter().filter(bare).map(|member| member.id.as_str()).collect();
    assert!(missing.is_empty(), "{} people lack a part of their profile: {missing:?}", missing.len());
    let id: HashSet<&str> = site.member.iter().map(|member| member.id.as_str()).collect();
    assert_eq!(id.len(), site.member.len(), "an id is somebody's alone");
}

#[test]
fn in_every_place_no_two_people_share_a_face_a_name_or_a_handle() {
    let (site, people, key) = real();
    let member: HashMap<&str, &Member> = site.member.iter().map(|member| (member.id.as_str(), member)).collect();
    let (seat, wrote) = seated(&people, &key);
    assert!(!seat.is_empty(), "the places of {}", site.from);
    let (mut clash, mut theirs) = (Vec::new(), 0);
    for (place, list) in &seat {
        let writer = wrote.get(place).cloned().unwrap_or_default();
        let mut seen: HashMap<(&str, String), &str> = HashMap::new();
        for id in list {
            let one = member.get(id.as_str()).unwrap_or_else(|| panic!("{place} seats {id}, who is nobody"));
            for mark in look(&site, one) {
                // Two who both wrote in a place are there by Phiegg's own
                // record (its people repeat names): counted, not seated.
                match seen.get(&mark).copied() {
                    Some(other) if writer.contains(id) && writer.contains(other) => theirs += 1,
                    Some(other) => clash.push(format!("{place}: {other} and {id} share a {} ({})", mark.0, mark.1)),
                    None => _ = seen.insert(mark, id),
                }
            }
        }
    }
    assert!(clash.is_empty(), "{} repeats in places, {theirs} of Phiegg's own besides: {:?}", clash.len(), &clash[..clash.len().min(12)]);
}

#[test]
#[ignore = "Phiegg's own people repeat names and handles (puikit/app/phiegg/src/data, core.json and agents.json): fixed there, not by Phiogy"]
fn across_the_app_no_two_people_share_a_name_or_a_handle() {
    let (site, _, _) = real();
    let mut seen: HashMap<(&str, String), &str> = HashMap::new();
    let mut clash = Vec::new();
    for member in &site.member {
        for mark in look(&site, member).into_iter().filter(|(kind, _)| *kind != "face") {
            match seen.get(&(mark.0, mark.1.clone())).copied() {
                Some(other) => clash.push(format!("{other} and {} share a {} ({})", member.id, mark.0, mark.1)),
                None => _ = seen.insert(mark, &member.id),
            }
        }
    }
    assert!(clash.is_empty(), "{} repeats: {:?}", clash.len(), &clash[..clash.len().min(12)]);
}

#[test]
fn every_level_of_the_tree_has_data() {
    let (site, people, key) = real();
    // The window's hosts: what an area of it holds.
    assert!(data::row(data::HOST).iter().any(|row| row.len() > 2), "data/host.tsv");
    // The places, each with rooms; every place that is no list of
    // direct conversations has a room to call in, and its people.
    assert!(!site.venue.is_empty(), "no places in {}", site.from);
    let (seat, _) = seated(&people, &key);
    let seat: HashMap<&str, usize> = seat.iter().map(|(place, list)| (place.as_str(), list.len())).collect();
    let step = Path::new(&site.from).ancestors().nth(3).map(|folder| folder.join("server.json")).unwrap_or_default();
    let step: Value = serde_json::from_str(&std::fs::read_to_string(&step).unwrap_or_default()).unwrap_or_default();
    let least = step["space"]["least"].as_u64().unwrap_or(1) as usize;
    let mut gap = Vec::new();
    for venue in &site.venue {
        let direct = !venue.plate.is_empty() && venue.plate.iter().all(|plate| !plate.with.is_empty());
        let voice = venue.plate.iter().any(|plate| plate.voice);
        let many = seat.get(venue.label.as_str()).copied().unwrap_or_default();
        match (venue.plate.is_empty(), direct, voice, many >= least) {
            (true, _, _, _) => gap.push(format!("{} has no rooms", venue.label)),
            (_, true, _, _) => {}
            (_, _, false, _) => gap.push(format!("{} has no room to call in", venue.label)),
            (_, _, _, false) => gap.push(format!("{} seats {many} people, fewer than {least}", venue.label)),
            _ => {}
        }
    }
    assert!(gap.is_empty(), "{} gaps: {:?}", gap.len(), &gap[..gap.len().min(12)]);
    // The pages: each one the app's file gives shows a room it has.
    assert!(!site.view.is_empty() && !data::row(data::PAGE).is_empty());
    let lost: Vec<&str> = site.view.iter().flat_map(|(_, block)| block).filter(|block| (block.kind.clone(), block.variant.clone()) == site.own && site.plate(&block.room).is_none()).map(|block| block.room.as_str()).collect();
    assert!(lost.is_empty(), "pages of rooms the site does not have: {lost:?}");
    // A call: the line holds callers in each room.
    assert!(Limit::own().caller > 1, "a call holds more than one");
}
