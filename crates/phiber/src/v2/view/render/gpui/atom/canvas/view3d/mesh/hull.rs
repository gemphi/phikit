//! Projected selection hull geometry.

pub(super) fn convex(mut points: Vec<(f32, f32)>) -> Vec<(f32, f32)> {
    if points.len() < 3 {
        return points;
    }
    points.sort_by(|left, right| {
        left.0
            .partial_cmp(&right.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(
                left.1
                    .partial_cmp(&right.1)
                    .unwrap_or(std::cmp::Ordering::Equal),
            )
    });
    let mut lower = Vec::new();
    for point in &points {
        while lower.len() >= 2
            && cross(lower[lower.len() - 2], lower[lower.len() - 1], *point) <= 0.0
        {
            lower.pop();
        }
        lower.push(*point);
    }
    let mut upper = Vec::new();
    for point in points.iter().rev() {
        while upper.len() >= 2
            && cross(upper[upper.len() - 2], upper[upper.len() - 1], *point) <= 0.0
        {
            upper.pop();
        }
        upper.push(*point);
    }
    lower.pop();
    upper.pop();
    lower.extend(upper);
    lower
}

fn cross(origin: (f32, f32), left: (f32, f32), right: (f32, f32)) -> f32 {
    (left.0 - origin.0) * (right.1 - origin.1) - (left.1 - origin.1) * (right.0 - origin.0)
}
