//! An Outliner drop — `ctx.op.outliner_drop = <dragged>><target>`, led by
//! `&` when Ctrl was held — as Blender's `outliner_dragdrop.cc` takes it:
//!
//! - an object on a collection moves there (`collection_drop_exec`, "Move
//!   inside collection"), or with Ctrl is linked there as well ("Link
//!   inside collection");
//! - an object on an object is parented to it (`parent_drop_exec`), unless
//!   that would make it its own ancestor ("Loop in parents");
//! - a collection on a collection is nested in it, last among its children
//!   (`BKE_collection_move`), unless that puts it inside itself; the Scene
//!   Collection stays where it is.

use crate::v2::scene::links::{BSceneLinks, BSceneTree};
use crate::v2::state::{BKey as _, BPath, BState, OpKey, log, rule::BRule};

/// What a drop asks for, read off its value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BOutlinerDrop {
    Move { object: String, path: String },
    Link { object: String, path: String },
    Parent { child: String, parent: String },
    Nest { child: String, parent: String },
    /// A collection Ctrl-dragged: in two parents at once.
    LinkCollection { child: String },
}

impl BOutlinerDrop {
    pub fn parse(op: &str) -> Option<Self> {
        let (ctrl, op) = op.strip_prefix('&').map_or((false, op), |rest| (true, rest));
        let (dragged, target) = op.split_once('>')?;
        let (a, b) = (dragged.to_string(), target.to_string());
        let (object, collection) = (|s: &str| s.strip_prefix("ob:").map(str::to_string), |s: &str| s.strip_prefix("col:").map(str::to_string));
        Some(match (object(&a), collection(&a), object(&b), collection(&b), ctrl) {
            (Some(object), _, _, Some(path), false) => Self::Move { object, path },
            (Some(object), _, _, Some(path), true) => Self::Link { object, path },
            (Some(child), _, Some(parent), _, _) => Self::Parent { child, parent },
            (_, Some(child), _, Some(parent), false) => Self::Nest { child, parent },
            (_, Some(child), _, Some(_), true) => Self::LinkCollection { child },
            _ => return None,
        })
    }

    fn name(path: &str) -> &str {
        path.rsplit('/').next().unwrap_or(path)
    }

    /// Make the drop in `links`, over the scene as it stands (`tree`):
    /// `Ok` with what the log says, `Err` with why it was refused, `None`
    /// when there is nothing to do.
    pub fn apply(&self, links: &mut BSceneLinks, tree: Option<&BSceneTree>) -> Option<Result<String, String>> {
        Some(match self {
            Self::Move { object, path } => {
                links.move_to(object, path);
                Ok(format!("Moved {object} to {}", Self::name(path)))
            }
            Self::Link { object, path } if tree.is_some_and(|t| t.members(path).iter().any(|(n, _)| n == object)) => Err(format!("{object} is already in {}", Self::name(path))),
            Self::Link { object, path } => {
                links.link_to(object, path);
                Ok(format!("Linked {object} to {}", Self::name(path)))
            }
            Self::Parent { child, parent } if tree.is_some_and(|t| t.descends_from(parent, child)) => Err("Loop in parents".to_string()),
            Self::Parent { child, parent } => {
                links.parent(child, parent);
                Ok(format!("Parented {child} to {parent}"))
            }
            Self::Nest { child, parent } => {
                let root = tree?.root_of(child)?;
                if root.parent_of(child) == Some(parent.as_str()) {
                    return None;
                }
                if !root.fits(child, parent) {
                    return Some(Err(match root.path == *child {
                        true => "The Scene Collection cannot be moved".to_string(),
                        false => format!("Cannot move {} inside itself", Self::name(child)),
                    }));
                }
                links.nest(child, parent);
                Ok(format!("Moved {} into {}", Self::name(child), Self::name(parent)))
            }
            Self::LinkCollection { child } => Err(format!("Linking {} into a second collection is not supported yet", Self::name(child))),
        })
    }
}

/// `ctx.op.outliner_drop` → the drop, made and logged, or refused and logged.
pub struct BOutlinerDropRule;

impl BRule for BOutlinerDropRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == OpKey::OutlinerDrop.path()
    }
    fn after(&self, _: &BPath, op: &str, state: &mut BState) {
        state.remove(OpKey::OutlinerDrop);
        let Some(drop) = BOutlinerDrop::parse(op) else { return };
        let tree = crate::v2::scene::source::with_loaded(|m| m.map(|m| BSceneTree::now(state, m)));
        let mut links = BSceneLinks::now(state);
        match drop.apply(&mut links, tree.as_ref()) {
            Some(Ok(said)) => {
                log::push(said);
                links.write(state);
            }
            Some(Err(said)) => log::push(said),
            None => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_drop_reads_what_was_dragged_where_and_whether_ctrl_was_held() {
        let parse = |op: &str| BOutlinerDrop::parse(op);
        assert_eq!(parse("ob:Cube>col:Scene Collection/Props"), Some(BOutlinerDrop::Move { object: "Cube".into(), path: "Scene Collection/Props".into() }));
        assert_eq!(parse("&ob:Cube>col:Scene Collection/Props"), Some(BOutlinerDrop::Link { object: "Cube".into(), path: "Scene Collection/Props".into() }));
        assert_eq!(parse("&ob:Camera>ob:Cube"), Some(BOutlinerDrop::Parent { child: "Camera".into(), parent: "Cube".into() }), "Ctrl does not change a parent");
        assert_eq!(parse("col:Scene Collection/Lamps>col:Scene Collection/Props"), Some(BOutlinerDrop::Nest { child: "Scene Collection/Lamps".into(), parent: "Scene Collection/Props".into() }));
        assert!(matches!(parse("&col:A>col:B"), Some(BOutlinerDrop::LinkCollection { .. })));
        assert_eq!(parse("col:A>ob:Cube"), None, "a collection on an object is no drop");
    }
}
