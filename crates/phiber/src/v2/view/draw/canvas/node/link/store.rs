//! How a host's link edits are spelled in one state value, and read back.
//!
//! One key holds the whole list rather than a key per link, because a link has
//! no id. `bNodeLink` is identified by its two ends and nothing else — there is
//! no name, no index, nothing stable to put in a path — so a key per link would
//! have to invent one. Keeping the list in a single value also means rewiring a
//! tree is one store write, which is one undo step, and that matches what
//! Blender's link operator does: it commits its whole `nldrag.links` at once.

use super::BLinkEnd;
use crate::phiphx::blend::shade::graph::BlendNodeLink;

/// The links added and removed on top of the file's own, for one host.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BLinkEdits {
    pub added: Vec<(BLinkEnd, BLinkEnd)>,
    pub removed: Vec<(BLinkEnd, BLinkEnd)>,
}

impl BLinkEdits {
    /// The file's links with the edits applied — what gets drawn.
    ///
    /// Removals are applied before additions so re-making a link that was
    /// broken in the same session comes back rather than cancelling out.
    pub fn apply(&self, file: &[BlendNodeLink]) -> Vec<BlendNodeLink> {
        let gone = |l: &BlendNodeLink| {
            self.removed.iter().any(|(f, t)| {
                f.node == l.from && f.socket == l.from_socket && t.node == l.to && t.socket == l.to_socket
            })
        };
        let mut out: Vec<BlendNodeLink> = file.iter().filter(|l| !gone(l)).cloned().collect();
        for (from, to) in &self.added {
            out.push(BlendNodeLink {
                from: from.node.clone(),
                to: to.node.clone(),
                from_socket: from.socket.clone(),
                to_socket: to.socket.clone(),
            });
        }
        out
    }
}

/// The stored spelling of one end: `node/socket`.
///
/// A node name can hold almost anything a user types, so the separators are
/// characters `bNode.name` and `bNodeSocket.identifier` do not contain: an
/// identifier is a registered C string and a node name is unique-ified with a
/// `.NNN` suffix, neither of which uses these.
const END: char = '/';
const ARROW: char = '>';
const NEXT: char = ';';

pub(super) fn write_end(end: &BLinkEnd, out: &mut String) {
    out.push_str(&end.node);
    out.push(END);
    out.push_str(&end.socket);
}

pub(super) fn read_end(text: &str) -> Option<BLinkEnd> {
    let (node, socket) = text.split_once(END)?;
    (!node.is_empty() && !socket.is_empty()).then(|| BLinkEnd::new(node, socket))
}

/// `+a/Out>b/In;-c/Out>d/In` — the made links and the broken ones.
pub fn encode(edits: &BLinkEdits) -> String {
    let mut out = String::new();
    for (sign, list) in [('+', &edits.added), ('-', &edits.removed)] {
        for (from, to) in list {
            if !out.is_empty() {
                out.push(NEXT);
            }
            out.push(sign);
            write_end(from, &mut out);
            out.push(ARROW);
            write_end(to, &mut out);
        }
    }
    out
}

/// The edits back out of the store. An entry that does not parse is dropped
/// rather than guessed at — a half-read link would draw a curve to nowhere.
pub fn decode(text: &str) -> BLinkEdits {
    let mut edits = BLinkEdits::default();
    for entry in text.split(NEXT).filter(|e| !e.is_empty()) {
        let (sign, rest) = entry.split_at(1);
        let Some((from, to)) = rest.split_once(ARROW) else { continue };
        let (Some(from), Some(to)) = (read_end(from), read_end(to)) else { continue };
        match sign {
            "+" => edits.added.push((from, to)),
            "-" => edits.removed.push((from, to)),
            _ => {}
        }
    }
    edits
}

/// This host's edits, out of the store.
pub fn edits_of(state: &crate::v2::state::BState, host: crate::v2::host::BHostType) -> BLinkEdits {
    use crate::v2::state::{BKey as _, NodeKey};
    decode(state.get(&NodeKey::Wired(host).path()).unwrap_or(""))
}


#[cfg(test)]
mod tests {
    use super::*;


    fn link(from: &str, from_socket: &str, to: &str, to_socket: &str) -> BlendNodeLink {
        BlendNodeLink {
            from: from.into(),
            to: to.into(),
            from_socket: from_socket.into(),
            to_socket: to_socket.into(),
        }
    }

    fn edits() -> BLinkEdits {
        BLinkEdits {
            added: vec![(BLinkEnd::new("Image Texture", "Color"), BLinkEnd::new("Principled BSDF", "Base Color"))],
            removed: vec![(BLinkEnd::new("Principled BSDF", "BSDF"), BLinkEnd::new("Material Output", "Surface"))],
        }
    }

    #[test]
    fn the_edits_survive_a_round_trip_through_the_store() {
        assert_eq!(decode(&encode(&edits())), edits());
    }

    #[test]
    fn a_socket_name_with_a_space_in_it_survives() {
        // "Base Color" and "Material Output" both carry spaces, and a node
        // name carries whatever the user typed — so the separators have to be
        // characters neither can contain, not whitespace.
        let text = encode(&edits());
        assert!(text.contains("Base Color"), "{text}");
        assert_eq!(decode(&text).added[0].1.socket, "Base Color");
    }

    #[test]
    fn nothing_stored_is_no_edits() {
        assert_eq!(decode(""), BLinkEdits::default());
        assert_eq!(encode(&BLinkEdits::default()), "");
    }

    #[test]
    fn a_half_written_entry_is_dropped_rather_than_guessed_at() {
        // A link read back wrong would draw a curve to nowhere, which is worse
        // than one that is missing.
        for bad in ["+a/Out", "+aOut>b/In", "+a/Out>bIn", "?a/Out>b/In", "+/Out>b/In", "+a/>b/In"] {
            let read = decode(bad);
            assert_eq!(read, BLinkEdits::default(), "{bad:?} should not have parsed");
        }
        // And a bad entry does not take its neighbours with it.
        let mixed = format!("{};+a/Out>b/In", "+broken");
        assert_eq!(decode(&mixed).added.len(), 1, "the good entry survives");
    }

    #[test]
    fn the_order_within_each_list_is_kept() {
        // The removal order is Blender's own iteration order, which decides
        // WHICH link disappears from a socket at its limit — so it is not
        // something to normalise.
        let many = BLinkEdits {
            added: Vec::new(),
            removed: vec![
                (BLinkEnd::new("a", "o"), BLinkEnd::new("z", "i")),
                (BLinkEnd::new("b", "o"), BLinkEnd::new("z", "i")),
            ],
        };
        assert_eq!(decode(&encode(&many)).removed, many.removed);
    }
    #[test]
    fn the_edits_overlay_the_files_own_links() {
        let file = vec![link("Principled BSDF", "BSDF", "Material Output", "Surface")];
        let edits = BLinkEdits {
            added: vec![(
                BLinkEnd::new("Image Texture", "Color"),
                BLinkEnd::new("Principled BSDF", "Base Color"),
            )],
            removed: vec![(
                BLinkEnd::new("Principled BSDF", "BSDF"),
                BLinkEnd::new("Material Output", "Surface"),
            )],
        };
        let applied = edits.apply(&file);
        assert_eq!(applied.len(), 1, "one out, one in: {applied:?}");
        assert_eq!(applied[0].from, "Image Texture");
        // The file itself is untouched — the whole point of an overlay.
        assert_eq!(file.len(), 1);
        assert_eq!(file[0].from, "Principled BSDF");
    }
    #[test]
    fn remaking_a_broken_link_brings_it_back() {
        // Removals apply before additions, so breaking and re-making the same link
        // in one session leaves it there rather than cancelling out.
        let file = vec![link("A", "Out", "B", "In")];
        let edits = BLinkEdits {
            added: vec![(BLinkEnd::new("A", "Out"), BLinkEnd::new("B", "In"))],
            removed: vec![(BLinkEnd::new("A", "Out"), BLinkEnd::new("B", "In"))],
        };
        let applied = edits.apply(&file);
        assert_eq!(applied.len(), 1);
        assert_eq!((applied[0].from.as_str(), applied[0].to.as_str()), ("A", "B"));
    }
    #[test]
    fn no_edits_is_the_files_own_list_unchanged() {
        let file = vec![link("A", "Out", "B", "In"), link("B", "Out", "C", "In")];
        assert_eq!(BLinkEdits::default().apply(&file), file);
    }
}
