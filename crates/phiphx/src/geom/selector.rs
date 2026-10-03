//! Primary subject selector and source abstraction traits.

/// Subject selection logic helper.
pub struct SubjectSelector;

impl SubjectSelector {
    /// Shared primary/active-subject selection used by the blend, USD, and glTF
    /// importers so all three resolve a scene's "primary" subject through one
    /// deterministic implementation (rather than each duplicating the algorithm).
    ///
    /// Resolution order:
    /// 1. The active subject, when recorded and in range (`active = Some(i)` with
    ///    `i < subjects.len()`).
    /// 2. Otherwise the subject with the most vertices, breaking ties
    ///    deterministically by the smaller name (independent of iteration order).
    /// 3. `None` only when `subjects` is empty.
    pub fn select_primary_subject<T>(
        subjects: &[T],
        active: Option<usize>,
        vertex_count: impl Fn(&T) -> usize,
        name: impl Fn(&T) -> &str,
    ) -> Option<usize> {
        if subjects.is_empty() {
            return None;
        }

        if let Some(i) = active {
            if i < subjects.len() {
                return Some(i);
            }
        }

        let mut best = 0usize;
        for i in 1..subjects.len() {
            let vi = vertex_count(&subjects[i]);
            let vb = vertex_count(&subjects[best]);
            if vi > vb || (vi == vb && name(&subjects[i]) < name(&subjects[best])) {
                best = i;
            }
        }
        Some(best)
    }
}

/// Trait for objects that can provide vertex and index data for rendering.
pub trait MeshSource {
    fn name(&self) -> &str;
    fn vertices(&self) -> &[[f32; 3]];
    fn indices(&self) -> &[[u32; 3]];
    fn matrix_world(&self) -> [[f32; 4]; 4];

    /// Calculate the Axis-Aligned Bounding Box (AABB)
    fn compute_aabb(&self) -> ([f32; 3], [f32; 3]) {
        let verts = self.vertices();
        if verts.is_empty() {
            return ([0.0, 0.0, 0.0], [0.0, 0.0, 0.0]);
        }
        let mut min = [f32::INFINITY; 3];
        let mut max = [f32::NEG_INFINITY; 3];
        for v in verts {
            for i in 0..3 {
                min[i] = min[i].min(v[i]);
                max[i] = max[i].max(v[i]);
            }
        }
        (min, max)
    }
}
