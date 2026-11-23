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
        template.filter(|_| !made).into_iter().for_each(|template| _ = desk.create(&name, template));
    }
}
