//! Reading a property's value from a stage, whatever precision it was
//! written in.

use openusd::Stage;
use openusd::sdf::Value;
use phiecs::comp::Matrix;

/// A property's value, where it has one.
pub fn value(stage: &Stage, prim: &str, name: &str) -> Option<Value> {
    stage.field::<Value>(format!("{prim}.{name}").as_str(), "default").ok().flatten()
}

/// Points, however precise.
pub fn points(stage: &Stage, prim: &str, name: &str) -> Vec<[f32; 3]> {
    match value(stage, prim, name) {
        Some(Value::Vec3fVec(v)) => v,
        Some(Value::Vec3dVec(v)) => v.into_iter().map(|p| p.map(|c| c as f32)).collect(),
        Some(Value::Vec3hVec(v)) => v.into_iter().map(|p| p.map(f32::from)).collect(),
        _ => Vec::new(),
    }
}

/// Whole numbers.
pub fn ints(stage: &Stage, prim: &str, name: &str) -> Vec<i32> {
    match value(stage, prim, name) {
        Some(Value::IntVec(v)) => v,
        Some(Value::Int64Vec(v)) => v.into_iter().map(|n| n as i32).collect(),
        Some(Value::UintVec(v)) => v.into_iter().map(|n| n as i32).collect(),
        _ => Vec::new(),
    }
}

/// Tokens or strings.
pub fn tokens(stage: &Stage, prim: &str, name: &str) -> Vec<String> {
    match value(stage, prim, name) {
        Some(Value::TokenVec(v)) | Some(Value::StringVec(v)) => v,
        _ => Vec::new(),
    }
}

/// A token or string.
pub fn text(stage: &Stage, prim: &str, name: &str) -> String {
    match value(stage, prim, name) {
        Some(Value::Token(t)) | Some(Value::String(t)) => t,
        _ => String::new(),
    }
}

/// USD's matrix, written for a row on its left, as phiecs's: for a
/// column on its right, its move in its last column.
pub fn matrix(m: &[f64; 16]) -> Matrix {
    let mut out = [[0.0f32; 4]; 4];
    for (r, row) in out.iter_mut().enumerate() {
        for (c, cell) in row.iter_mut().enumerate() {
            *cell = m[c * 4 + r] as f32;
        }
    }
    Matrix(out)
}

/// A list of matrices.
pub fn matrices(stage: &Stage, prim: &str, name: &str) -> Vec<Matrix> {
    match value(stage, prim, name) {
        Some(Value::Matrix4dVec(v)) => v.iter().map(matrix).collect(),
        _ => Vec::new(),
    }
}

/// Three numbers, however precise.
pub fn three(v: Value) -> Option<[f32; 3]> {
    match v {
        Value::Vec3f(p) => Some(p),
        Value::Vec3d(p) => Some(p.map(|c| c as f32)),
        Value::Vec3h(p) => Some(p.map(f32::from)),
        Value::Float(s) => Some([s; 3]),
        Value::Double(s) => Some([s as f32; 3]),
        _ => None,
    }
}

/// A turn, real part first.
pub fn turn(v: Value) -> Option<[f32; 4]> {
    match v {
        Value::Quatf(q) => Some(q),
        Value::Quatd(q) => Some(q.map(|c| c as f32)),
        Value::Quath(q) => Some(q.map(f32::from)),
        _ => None,
    }
}
