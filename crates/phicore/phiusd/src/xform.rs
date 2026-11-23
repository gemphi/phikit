//! Where a prim stands in its layer's world: its transform ops in the
//! order `xformOpOrder` gives, after its parents'.

use openusd::Stage;
use openusd::sdf::Value;
use phiecs::comp::{Matrix, Pose};

use super::read::{matrix, three, tokens, turn, value};

/// A prim's own transform: each op in order, a point taking the last
/// first, as USD composes them.
pub fn local(stage: &Stage, prim: &str) -> Matrix {
    tokens(stage, prim, "xformOpOrder").iter().fold(Matrix::default(), |m, op| m.then(&one(stage, prim, op)))
}

/// One op's matrix; `!invert!` before it inverts a move or a size.
fn one(stage: &Stage, prim: &str, op: &str) -> Matrix {
    let (invert, name) = match op.strip_prefix("!invert!") {
        Some(name) => (true, name),
        None => (false, op),
    };
    let Some(v) = value(stage, prim, name) else { return Matrix::default() };
    let kind = name.trim_start_matches("xformOp:").split(':').next().unwrap_or_default();
    let sign = match invert {
        true => -1.0,
        false => 1.0,
    };
    match (kind, v) {
        ("transform", Value::Matrix4d(m)) => matrix(&m),
        ("translate", v) => three(v).map_or(Matrix::default(), |t| Pose::at(t.map(|c| c * sign)).matrix()),
        ("scale", v) => three(v).map_or(Matrix::default(), |s| Pose { scale: s.map(|c| if invert { 1.0 / c } else { c }), ..Pose::default() }.matrix()),
        ("orient", v) => turn(v).map_or(Matrix::default(), |q| Pose { orient: q, ..Pose::default() }.matrix()),
        ("rotateXYZ", v) => three(v).map_or(Matrix::default(), |r| about(2, r[2]).then(&about(1, r[1])).then(&about(0, r[0]))),
        ("rotateX", v) => three(v).map_or(Matrix::default(), |r| about(0, r[0])),
        ("rotateY", v) => three(v).map_or(Matrix::default(), |r| about(1, r[0])),
        ("rotateZ", v) => three(v).map_or(Matrix::default(), |r| about(2, r[0])),
        _ => Matrix::default(),
    }
}

/// A turn of so many degrees about an axis (0 x, 1 y, 2 z).
fn about(axis: usize, degree: f32) -> Matrix {
    let half = degree.to_radians() / 2.0;
    let mut q = [half.cos(), 0.0, 0.0, 0.0];
    q[axis + 1] = half.sin();
    Pose { orient: q, ..Pose::default() }.matrix()
}

/// Where a prim stands in its layer's world: its parents' transforms,
/// then its own.
pub fn world(stage: &Stage, prim: &str) -> Matrix {
    let mut chain = Vec::new();
    let mut at = prim.to_string();
    while !at.is_empty() && at != "/" {
        chain.push(at.clone());
        at = at.rsplit_once('/').map(|(above, _)| above.to_string()).unwrap_or_default();
    }
    chain.iter().rev().fold(Matrix::default(), |m, p| m.then(&local(stage, p)))
}
