//! State definitions and data types.

#[derive(Clone, Debug)]
pub struct WireframeMesh {
    pub vertices: Vec<[f32; 3]>,
    pub faces: Vec<[u32; 3]>,
    pub bounds_min: [f32; 3],
    pub bounds_max: [f32; 3],
}
