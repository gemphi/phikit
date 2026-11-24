//! The people of the site, as the rooms' file has them: who each is,
//! what they say of themself, whether they are about, and the spaces
//! they are in. What each name in the file is called is
//! `data/site.tsv`.

use std::collections::HashMap;

use appkit::service::PTable;
use serde_json::Value;

use super::Face;

/// Somebody of the app.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Member {
    pub id: String,
    pub name: String,
    pub handle: String,
    /// What they are there, and what they say of themself.
    pub role: String,
    pub bio: String,
    /// Whether they are about, as a word of the file's own (`here`,
    /// `away`), and in words a person reads.
    pub presence: String,
    pub state: String,
    /// Since when they are a member, as the file writes it.
    pub since: String,
    pub tag: Vec<String>,
    /// How many people they are linked to.
    pub link: u64,
    /// The picture over their profile, by its key.
    pub banner: String,
    /// The spaces they are in, by name.
    pub space: Vec<String>,
    /// What Phiegg counts of them, in its order: the word for each, and
    /// how many (shots, followers, following, puts).
    pub count: Vec<(String, u64)>,
    /// The pictures they posted, by their keys.
    pub picture: Vec<String>,
    /// The clubs they hold, each with its mark and how many hold it.
    pub club: Vec<Club>,
    pub verified: bool,
}

/// A club somebody holds: its id, its name, its mark, its holders.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Club {
    pub id: String,
    pub name: String,
    pub mark: Face,
    pub holder: u64,
}

/// Everyone of the rooms' file, in its order, each with the spaces
/// they are in: those the file lists them under, and those of the
/// rooms it names them in.
pub fn read(people: &Value, key: &PTable) -> Vec<Member> {
    let text = |value: &Value, name: &str| value.get(key.get(name)).and_then(Value::as_str).unwrap_or_default().to_string();
    let list = |value: &Value, name: &str| value.get(key.get(name)).and_then(Value::as_array).cloned().unwrap_or_default();
    // The columns of the sheets faces are cells of, shared by every mark.
    let column = people.get(key.get("sheet")).and_then(|sheet| sheet.get(key.get("sheet.column"))).and_then(Value::as_u64).unwrap_or(1) as u32;
    let one = |one: &Value| Member {
        id: text(one, "id"),
        name: text(one, "name"),
        handle: text(one, "handle"),
        role: text(one, "role"),
        bio: text(one, "bio"),
        presence: text(one, "presence"),
        state: text(one, "state"),
        since: text(one, "since"),
        tag: list(one, "tag").iter().filter_map(Value::as_str).map(str::to_string).collect(),
        link: one.get(key.get("linked")).and_then(Value::as_u64).unwrap_or_default(),
        banner: text(one, "banner"),
        space: Vec::new(),
        count: list(one, "person.count").iter().filter_map(|pair| Some((pair.get(0)?.as_str()?.to_string(), pair.get(1)?.as_u64()?))).collect(),
        picture: list(one, "picture").iter().filter_map(Value::as_str).map(str::to_string).collect(),
        club: list(one, "club").iter().map(|club| Club {
            id: text(club, "id"),
            name: text(club, "name"),
            mark: Face { sheet: text(club, "sheet"), column, index: club.get(key.get("face")).and_then(Value::as_u64).unwrap_or_default() as u32 },
            holder: club.get(key.get("club.holder")).and_then(Value::as_u64).unwrap_or_default(),
        }).collect(),
        verified: one.get(key.get("verified")).and_then(Value::as_bool).unwrap_or_default(),
    };
    let mut member: Vec<Member> = list(people, "member").iter().map(one).collect();
    let place: HashMap<String, usize> = member.iter().enumerate().map(|(index, member)| (member.id.clone(), index)).collect();
    let mut join = |index: usize, space: &str| {
        let own = member.get_mut(index).map(|member| &mut member.space);
        own.into_iter().filter(|own| !own.iter().any(|known| known == space)).for_each(|own| own.push(space.to_string()));
    };
    // The rooms that name them, then the spaces that list them.
    let room = people.get(key.get("room")).and_then(Value::as_object).cloned().unwrap_or_default();
    for room in room.values() {
        let space = text(room, "space");
        let named = list(room, "member");
        let inside = named.iter().filter_map(Value::as_str).filter_map(|id| place.get(id)).filter(|_| !space.is_empty());
        inside.for_each(|index| join(*index, &space));
    }
    let space = people.get(key.get("space")).and_then(Value::as_object).cloned().unwrap_or_default();
    for (name, listed) in &space {
        let inside = listed.as_array().into_iter().flatten().filter_map(Value::as_u64);
        inside.for_each(|index| join(index as usize, name));
    }
    member
}
