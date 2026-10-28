//! The arithmetic of Blender's `MOD_correctivesmooth.cc`, translated line by
//! line: the two smoothing loops, the per-corner tangent spaces, and the
//! deltas the reconstruction adds back.
//!
//! Matrices are kept as Blender keeps them, `m[column][row]`
//! (`float[3][3]`), and the vector helpers repeat `BLI_math_vector`'s
//! operation order, so a float rounds here as it rounds there.

/// `float[3][3]`, column-major as in `BLI_math_matrix`.
pub(super) type M3 = [[f32; 3]; 3];
type V3 = [f32; 3];

/// `MOD_CORRECTIVESMOOTH_SMOOTH_LENGTH_WEIGHT`.
pub(super) const SMOOTH_LENGTH_WEIGHT: u32 = 1;

fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn dot(a: V3, b: V3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn cross(a: V3, b: V3) -> V3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}
/// `madd_v3_v3fl`: `r += a * f`.
pub(super) fn madd(r: &mut V3, a: V3, f: f32) {
    r[0] += a[0] * f;
    r[1] += a[1] * f;
    r[2] += a[2] * f;
}
/// `normalize_v3`: zero below `1e-35` squared length, else `a * (1 / len)`.
fn normalize(a: V3) -> V3 {
    let d = dot(a, a);
    match d > 1.0e-35 {
        true => {
            let f = 1.0 / d.sqrt();
            [a[0] * f, a[1] * f, a[2] * f]
        }
        false => [0.0; 3],
    }
}
/// `mul_v3_m3v3`: `M · a`.
pub(super) fn mul_m3_v3(m: &M3, a: V3) -> V3 {
    [
        m[0][0] * a[0] + m[1][0] * a[1] + m[2][0] * a[2],
        m[0][1] * a[0] + m[1][1] * a[1] + m[2][1] * a[2],
        m[0][2] * a[0] + m[1][2] * a[1] + m[2][2] * a[2],
    ]
}
/// `invert_m3_m3`: the adjoint over the determinant; `None` where the
/// determinant is exactly zero (`calc_deltas` then transposes instead).
pub(super) fn invert_m3(m: &M3) -> Option<M3> {
    let [[m00, m01, m02], [m10, m11, m12], [m20, m21, m22]] = *m;
    let mut r = [
        [m11 * m22 - m12 * m21, -m01 * m22 + m02 * m21, m01 * m12 - m02 * m11],
        [-m10 * m22 + m12 * m20, m00 * m22 - m02 * m20, -m00 * m12 + m02 * m10],
        [m10 * m21 - m11 * m20, -m00 * m21 + m01 * m20, m00 * m11 - m01 * m10],
    ];
    let det = m00 * (m11 * m22 - m12 * m21) - m10 * (m01 * m22 - m02 * m21) + m20 * (m01 * m12 - m02 * m11);
    if det == 0.0 {
        return None;
    }
    let inv = 1.0 / det;
    r.iter_mut().flatten().for_each(|x| *x *= inv);
    Some(r)
}
pub(super) fn transpose_m3(m: &M3) -> M3 {
    [[m[0][0], m[1][0], m[2][0]], [m[0][1], m[1][1], m[2][1]], [m[0][2], m[1][2], m[2][2]]]
}

/// `math::safe_acos_approx` (`BLI_math_base.hh`), verbatim.
fn safe_acos_approx(x: f32) -> f32 {
    let f = x.abs();
    let m = match f < 1.0 {
        true => 1.0 - (1.0 - f),
        false => 1.0,
    };
    let a = (1.0 - m).sqrt() * (1.5707963267 + m * (-0.213300989 + m * (0.077980478 + m * -0.02164095)));
    match x < 0.0 {
        true => std::f32::consts::PI - a,
        false => a,
    }
}

/// `smooth_iter__simple`: each vertex moves `lambda` of the way to the mean
/// of its edge neighbours, `iterations` times. `weights` folds the vertex
/// group and the pinned boundary in, as Blender folds them into the divisor.
pub(super) fn smooth_simple(edges: &[(u32, u32)], cos: &mut [V3], weights: Option<&[f32]>, lambda: f32, iterations: u32) {
    let mut delta = vec![[0.0f32; 3]; cos.len()];
    let mut div = vec![0.0f32; cos.len()];
    for &(a, b) in edges {
        div[a as usize] += 1.0;
        div[b as usize] += 1.0;
    }
    for (i, d) in div.iter_mut().enumerate() {
        let per = match *d != 0.0 {
            true => 1.0 / *d,
            false => 1.0,
        };
        *d = match weights {
            Some(w) => w[i] * lambda * per,
            None => lambda * per,
        };
    }
    for _ in 0..iterations {
        for &(a, b) in edges {
            let dir = sub(cos[b as usize], cos[a as usize]);
            delta[a as usize] = add(delta[a as usize], dir);
            delta[b as usize] = sub(delta[b as usize], dir);
        }
        for (i, co) in cos.iter_mut().enumerate() {
            madd(co, delta[i], div[i]);
            delta[i] = [0.0; 3];
        }
    }
}

/// `smooth_iter__length_weight`: as the simple loop with each neighbour
/// weighted by its edge's length, at twice the factor ("approx half as
/// strong as the simple-smooth").
pub(super) fn smooth_length_weight(edges: &[(u32, u32)], cos: &mut [V3], weights: Option<&[f32]>, lambda: f32, iterations: u32) {
    let eps = f32::EPSILON * 10.0;
    let lambda = lambda * 2.0;
    let mut delta = vec![[0.0f32; 3]; cos.len()];
    let mut length_sum = vec![0.0f32; cos.len()];
    let mut count = vec![0.0f32; cos.len()];
    for &(a, b) in edges {
        count[a as usize] += 1.0;
        count[b as usize] += 1.0;
    }
    for _ in 0..iterations {
        for &(a, b) in edges {
            let mut dir = sub(cos[b as usize], cos[a as usize]);
            let dist = dot(dir, dir).sqrt();
            dir = [dir[0] * dist, dir[1] * dist, dir[2] * dist];
            delta[a as usize] = add(delta[a as usize], dir);
            delta[b as usize] = sub(delta[b as usize], dir);
            length_sum[a as usize] += dist;
            length_sum[b as usize] += dist;
        }
        for (i, co) in cos.iter_mut().enumerate() {
            let div = length_sum[i] * count[i];
            if div > eps {
                let l = match weights {
                    Some(w) => lambda * w[i],
                    None => lambda,
                };
                madd(co, delta[i], l / div);
            }
            delta[i] = [0.0; 3];
            length_sum[i] = 0.0;
        }
    }
}

/// `calc_tangent_loop`: an orthonormal frame from a corner's two edge
/// directions; `false` (and the identity) when they coincide.
fn tangent_loop(prev: V3, next: V3) -> (M3, bool) {
    let limit = f32::EPSILON * 10.0;
    if (0..3).all(|k| (prev[k] - next[k]).abs() <= limit) {
        return ([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]], false);
    }
    let z = normalize(cross(prev, next));
    let x = normalize(add(prev, next));
    ([x, cross(z, x), z], true)
}

/// `calc_tangent_spaces`: one frame per face corner, and — when `weights` is
/// asked for — each corner's angle weight and their sum per vertex.
pub(super) fn tangent_spaces(
    faces: &crate::blend::BFaces,
    cos: &[V3],
    weights: Option<(&mut [f32], &mut [f32])>,
) -> Vec<M3> {
    let mut spaces = vec![[[0.0f32; 3]; 3]; faces.corners()];
    let (mut tw, mut twv) = match weights {
        Some((a, b)) => (Some(a), Some(b)),
        None => (None, None),
    };
    if let Some(v) = twv.as_deref_mut() {
        v.fill(0.0);
    }
    for f in 0..faces.len() {
        let verts = faces.get(f);
        let start = faces.start(f);
        let n = verts.len();
        if n == 0 {
            continue;
        }
        let at = |c: usize| cos[verts[c] as usize];
        // Corners walked as Blender walks them: `curr` starts at the last.
        let mut prev_dir = normalize(sub(at((n + n - 2) % n), at(n - 1)));
        let mut curr = n - 1;
        for next in 0..n {
            let next_dir = normalize(sub(at(curr), at(next)));
            let (ts, used) = tangent_loop(prev_dir, next_dir);
            spaces[start + curr] = ts;
            if let (Some(tw), Some(twv)) = (tw.as_deref_mut(), twv.as_deref_mut()) {
                let w = match used {
                    true => safe_acos_approx(dot(next_dir, prev_dir)).abs(),
                    false => 0.0,
                };
                tw[start + curr] = w;
                twv[verts[curr] as usize] += w;
            }
            prev_dir = next_dir;
            curr = next;
        }
    }
    spaces
}
