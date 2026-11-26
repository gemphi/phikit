//! What a command line has the person own before the window opens: a
//! theme put on, and hosts made.

use super::given;
use crate::part::Desk;

/// `--wear s_phiogy/theme_sphere` puts that theme on. `--host NAME`
/// makes a host of that name for the person, as the hosts' page does,
/// from the template `--from ID` names or else the site's first; a
/// name they already made a host of is left alone, so the same words
/// can be given again.
pub(super) fn own(desk: &Desk, word: &[String]) {
    given(word, "--wear").iter().for_each(|place| desk.wear(place));
    let wanted = given(word, "--from").pop();
    let template = desk.site.extra.template.iter().find(|template| wanted.as_ref().is_none_or(|id| template.id == *id));
    for name in given(word, "--host") {
        let made = desk.made().iter().any(|venue| venue.label == name.trim());
        // Made, and published on Phiora where the person is signed in, so others can join it; what Phiora said is said.
        if let Some((id, template)) = template.filter(|_| !made).and_then(|template| desk.create_with(&name, template).map(|(id, _)| (id, template))) {
            println!("{}", desk.publish_host(&id, &name, template));
        }
    }
    // `--bond u_ines=sent` says how the person stands with somebody (`sent` asks, `with` accepts or befriends,
    // `none` cancels or removes), as the Friends page's buttons do; `--stand u_ines` says how they stand now.
    for one in given(word, "--bond") {
        if let Some((id, bond)) = one.split_once('=') {
            desk.bind(id, bond);
        }
    }
    for id in given(word, "--stand") {
        println!("{id} {}", desk.bond(&id));
    }
    // `--follow ID` follows that person, or stops following them.
    for id in given(word, "--follow") {
        desk.follow(&id);
    }
    // `--give ID` gives the message of that id in the room it opens on a like, or takes it back.
    for id in given(word, "--give") {
        desk.give(&id, &desk.like());
    }
    // `--adopt NAME` joins the host of that name that somebody published on Phiora.
    for name in given(word, "--adopt") {
        desk.refresh_published();
        let found = desk.published().into_iter().find(|place| place.label == name.trim());
        match found.as_ref().and_then(|place| desk.adopt(place).map(|_| place)) {
            Some(place) => println!("{}", desk.text.fill("host.adopted", "name", &place.label).replace("{rooms}", &place.plate.len().to_string())),
            None => println!("{}", desk.text.get("host.none")),
        }
    }
}
