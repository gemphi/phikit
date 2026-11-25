//! Who a person is, as Phiegg's profile card has it
//! (puikit: app/phi-kit/src/ProfileView.tsx): the picture over the
//! profile, their face standing over its foot; their name; their
//! handle, role and how they stand; how many they are linked to, the
//! spaces they are in and the pictures they posted; what they say of
//! themself; their tags; since when they are here.

use appkit::pui::block::{PFace, PPill, PText};
use appkit::pui::core::light::label;
use appkit::pui::core::{PFit, PLight, PPaint, PPoint, PRect};
use appkit::pui::layout::wrap;

use super::{BANNER, FACE, Person};
use crate::part::{DIM, GAP, ICON, INK, LINE, RADIUS};
use crate::site::Member;

/// Where something that wide begins, in the middle of a width.
fn left_of(from: f32, wide: f32, size: f32) -> f32 {
    from + (wide - size).max(0.0) / 2.0
}

impl Person {
    /// The card, from `at` down; how far it goes.
    pub(super) fn card(&self, paint: &mut dyn PPaint, at: PPoint, wide: f32, one: &Member) -> f32 {
        // Their face as large as the theme says, never wider than the card:
        // the picture is what a profile is about (the user, 25 November
        // 2026: "make that picture bigger").
        let (gap, line, radius) = (paint.size(GAP), paint.size(LINE), paint.size(RADIUS));
        // The arrow at its left widens it to the whole column, and back.
        let face = match self.big.get() {
            true => wide,
            false => paint.size(FACE).min(wide),
        };
        let middle = left_of(at.x, wide, face);
        let (ink, dim, text) = (paint.color(INK), paint.color(DIM), &self.desk.text);
        let (left, mut top) = (at.x, at.y);
        let cell = self.desk.site.face(&one.id).map(|cell| (cell.sheet.as_str(), cell.column, cell.index));
        // The picture over the profile, and the face standing over its
        // foot; with none, the face at the top.
        match one.banner.is_empty() {
            true => {
                (cell, one.name.as_str()).face(paint, PRect::new(middle, top, face, face));
                top += face + gap / 2.0;
            }
            false => {
                let high = wide / paint.size(BANNER).max(1.0);
                paint.picture(&one.banner, PRect::new(left, top, wide, high), radius, PFit::Cover);
                (cell, one.name.as_str()).face(paint, PRect::new(middle, top + high / 2.0, face, face));
                top += high / 2.0 + face + gap / 2.0;
            }
        }
        // The arrow, round, at the face's left in the middle of its height.
        let arrow = paint.size(ICON) + gap * 0.75;
        let face_top = match one.banner.is_empty() {
            true => at.y,
            false => at.y + wide / paint.size(BANNER).max(1.0) / 2.0,
        };
        let place = PRect::new(middle + gap / 2.0, face_top + face / 2.0 - arrow / 2.0, arrow, arrow);
        label(paint, place, arrow / 2.0, PLight::Rest, false, "");
        let icon = match self.big.get() {
            true => "arrow-right",
            false => "arrow-left",
        };
        let size = paint.size(ICON);
        paint.icon(icon, PRect::new(place.center().x - size / 2.0, place.center().y - size / 2.0, size, size), ink);
        self.grow.set(place);
        // Their name; under it their handle, role and how they stand.
        let under = [one.handle.clone(), one.role.clone(), one.state.clone()];
        let said = under.iter().filter(|said| !said.is_empty()).cloned().collect::<Vec<_>>().join(&format!(" {} ", text.get("between")));
        paint.head(PPoint::new(left, top), &one.name.fit(wide, paint), ink);
        top += line + gap / 4.0;
        if !said.is_empty() {
            paint.text(PPoint::new(left, top), &said.fit(wide, paint), dim);
            top += line + gap / 2.0;
        }
        // How many, each a figure over its word on a ground of its own (the
        // user, 25 November 2026), side by side: Phiegg's
        // own counts of them (shots, followers, following, puts); for
        // somebody Phiegg does not count, their links, spaces, pictures.
        let own = [("linked".to_string(), one.link), ("space".to_string(), one.space.len() as u64), ("media".to_string(), self.desk.shot_of(&one.id).len() as u64)];
        let count: Vec<(String, u64)> = match one.count.is_empty() {
            true => own.to_vec(),
            false => one.count.clone(),
        };
        let step = wide / count.len().max(1) as f32;
        for (index, (word, n)) in count.iter().enumerate() {
            let x = left + index as f32 * step;
            label(paint, PRect::new(x, top - gap / 4.0, step - gap / 4.0, line * 2.0 + gap * 0.5), radius, PLight::Soft, false, "");
            let x = x + gap / 2.0;
            paint.head(PPoint::new(x, top), &crate::part::tool::compact(*n, text).fit(step - gap, paint), ink);
            paint.text(PPoint::new(x, top + line + gap / 8.0), &text.get(&format!("person.count.{word}")).fit(step - gap, paint), dim);
        }
        top += line * 2.0 + gap;
        // What they say of themself.
        for words in wrap(&one.bio, wide, &|text| paint.measure(text)).into_iter().filter(|words| !words.is_empty()) {
            paint.text(PPoint::new(left, top), &words, ink);
            top += line;
        }
        // Their tags, each a pill of its own.
        let mut across = left;
        for tag in &one.tag {
            across += ("", tag.as_str(), false).draw(paint, PPoint::new(across, top + gap / 4.0), false).size.width + gap / 4.0;
        }
        top += match one.tag.is_empty() {
            true => 0.0,
            false => line + gap,
        };
        // Since when they are here.
        if !one.since.is_empty() {
            paint.text(PPoint::new(left, top), &format!("{} {} {}", text.get("person.since"), text.get("between"), one.since), dim);
            top += line;
        }
        top
    }
}
