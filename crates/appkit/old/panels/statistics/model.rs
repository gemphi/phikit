//! Statistics model.

#[derive(Debug, Clone, Default)]
pub struct NSStatisticsData {
    pub fps: f32,
    pub draw_calls: u32,
    pub triangles: u64,
    pub vertices: u64,
    pub textures: u32,
    pub memory_mb: f32,
    pub entity_count: u32,
    pub render_time_ms: f32,
}
