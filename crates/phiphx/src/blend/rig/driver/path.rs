//! RNA paths as a rig's drivers and curves spell them:
//! `pose.bones["thigh.L"].constraints["IK"].influence`,
//! `pose.bones["torso"]["neck_follow"]`, `pose.bones["x"].scale.y`.

use super::{BDriven, BDriverSource};
use crate::blend::rig::BRig;

/// One step of a path.
#[derive(Clone, Debug, PartialEq)]
pub enum BRnaStep {
    Field(String),
    Key(String),
    Index(usize),
}

/// A parsed path.
#[derive(Clone, Debug, PartialEq)]
pub struct BRnaPath(pub Vec<BRnaStep>);

impl BRnaPath {
    /// Split `path` into fields, quoted keys (`\"` escapes) and indices.
    pub fn parse(path: &str) -> Option<Self> {
        let c: Vec<char> = path.chars().collect();
        let (mut i, mut out) = (0, Vec::new());
        while i < c.len() {
            match c[i] {
                '.' => i += 1,
                '[' if c.get(i + 1) == Some(&'"') => {
                    let mut key = String::new();
                    i += 2;
                    while i < c.len() && c[i] != '"' {
                        if c[i] == '\\' && i + 1 < c.len() {
                            i += 1;
                        }
                        key.push(c[i]);
                        i += 1;
                    }
                    if c.get(i + 1) != Some(&']') {
                        return None;
                    }
                    i += 2;
                    out.push(BRnaStep::Key(key));
                }
                '[' => {
                    let end = i + c[i..].iter().position(|&ch| ch == ']')?;
                    out.push(BRnaStep::Index(c[i + 1..end].iter().collect::<String>().trim().parse().ok()?));
                    i = end + 1;
                }
                _ => {
                    let start = i;
                    while i < c.len() && c[i] != '.' && c[i] != '[' {
                        i += 1;
                    }
                    out.push(BRnaStep::Field(c[start..i].iter().collect()));
                }
            }
        }
        Some(Self(out))
    }

    /// `pose.bones["B"]` → the bone and the steps after it.
    fn bone<'a>(&'a self, rig: &BRig) -> Option<(usize, &'a [BRnaStep])> {
        match self.0.as_slice() {
            [BRnaStep::Field(p), BRnaStep::Field(b), BRnaStep::Key(name), rest @ ..] if p == "pose" && b == "bones" => Some((rig.bone(name)?, rest)),
            _ => None,
        }
    }

    /// What a driver on this path writes.
    pub fn driven(&self, rig: &BRig) -> Option<BDriven> {
        let (bone, rest) = self.bone(rig)?;
        let con = |name: &str| rig.bones[bone].constraints.iter().position(|c| c.name == name);
        Some(match rest {
            [BRnaStep::Field(c), BRnaStep::Key(name), BRnaStep::Field(f)] if c == "constraints" && f == "influence" => BDriven::Influence { bone, con: con(name)? },
            [BRnaStep::Field(c), BRnaStep::Key(name), BRnaStep::Field(f)] if c == "constraints" && f == "mute" => BDriven::Mute { bone, con: con(name)? },
            [BRnaStep::Field(c), BRnaStep::Key(name), BRnaStep::Field(t), BRnaStep::Index(i), BRnaStep::Field(w)] if c == "constraints" && t == "targets" && w == "weight" => {
                BDriven::TargetWeight { bone, con: con(name)?, target: *i }
            }
            [BRnaStep::Key(name)] => BDriven::Prop { bone, name: name.clone() },
            [BRnaStep::Field(p)] if Self::is_channel(p) => BDriven::Channel { bone, property: p.clone() },
            _ => return None,
        })
    }

    /// What a single-property variable on this path reads.
    pub fn source(&self, rig: &BRig) -> BDriverSource {
        let Some((bone, rest)) = self.bone(rig) else { return BDriverSource::Unsupported };
        match rest {
            [BRnaStep::Key(name)] => BDriverSource::Prop { bone, name: name.clone() },
            [BRnaStep::Field(p), BRnaStep::Index(i)] if Self::is_channel(p) => BDriverSource::Channel { bone, property: p.clone(), index: *i },
            [BRnaStep::Field(p), BRnaStep::Field(axis)] if Self::is_channel(p) => match Self::axis(p, axis) {
                Some(index) => BDriverSource::Channel { bone, property: p.clone(), index },
                None => BDriverSource::Unsupported,
            },
            _ => BDriverSource::Unsupported,
        }
    }

    fn is_channel(p: &str) -> bool {
        matches!(p, "location" | "rotation_euler" | "rotation_quaternion" | "rotation_axis_angle" | "scale")
    }

    /// A vector's `.x` / `.y` / `.z` / `.w` as an index (`w` first in a
    /// quaternion).
    fn axis(property: &str, axis: &str) -> Option<usize> {
        let quat = property == "rotation_quaternion";
        match axis {
            "w" if quat => Some(0),
            "x" => Some(usize::from(quat)),
            "y" => Some(1 + usize::from(quat)),
            "z" => Some(2 + usize::from(quat)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_split_into_fields_keys_and_indices() {
        let p = BRnaPath::parse(r#"pose.bones["a \"b\""].constraints["Armature"].targets[3].weight"#).unwrap();
        assert_eq!(
            p.0,
            vec![
                BRnaStep::Field("pose".into()),
                BRnaStep::Field("bones".into()),
                BRnaStep::Key("a \"b\"".into()),
                BRnaStep::Field("constraints".into()),
                BRnaStep::Key("Armature".into()),
                BRnaStep::Field("targets".into()),
                BRnaStep::Index(3),
                BRnaStep::Field("weight".into()),
            ]
        );
        assert_eq!(BRnaPath::parse(r#"pose.bones["x"]["IK_FK"]"#).unwrap().0.last(), Some(&BRnaStep::Key("IK_FK".into())));
    }
}
