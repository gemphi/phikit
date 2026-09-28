//! Probe: the dark-gold metal of THYMUS's torso (F0 0.058/0.05/0.017, rough
//! 0.2) as a card facing a front camera, lit by the studio forest HDRI, next
//! to the analytic expectation — F0 × the HDRI's radiance in the mirror
//! direction, averaged over the GGX lobe's footprint by point-sampling it.
use phiray::{Atlas, Camera, Ray, Tracer, Tri};

fn main() {
    let bytes = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/studiolight/forest.hdr")).expect("forest.hdr");
    let img = image::load_from_memory(&bytes).expect("decode").to_rgb32f();
    let (w, h) = (img.width(), img.height());
    let rgb = img.into_raw();
    let mut atlas = Atlas::default();
    atlas.add_hdr(w, h, &rgb);
    atlas.add_env_cdf(w, h, &rgb);
    // Radiance of the HDRI in direction d, Blender's equirectangular mapping.
    let at = |d: [f32; 3]| -> [f32; 3] {
        let u = -d[1].atan2(d[0]) / (2.0 * std::f32::consts::PI) + 0.5;
        let v = 0.5 - d[2].atan2((d[0] * d[0] + d[1] * d[1]).sqrt()) / std::f32::consts::PI;
        let (x, y) = (((u.rem_euclid(1.0)) * w as f32) as usize % w as usize, ((v.clamp(0.0, 0.9999)) * h as f32) as usize);
        let i = (y * w as usize + x) * 3;
        [rgb[i], rgb[i + 1], rgb[i + 2]]
    };
    for (label, dir) in [("−Y (behind a front camera)", [0.0, -1.0, 0.0]), ("+Z (up)", [0.0, 0.0, 1.0]), ("+Y", [0.0, 1.0, 0.0]), ("+X", [1.0, 0.0, 0.0])] {
        let c = at(dir);
        println!("HDRI radiance toward {label:28} {:.3} {:.3} {:.3}", c[0], c[1], c[2]);
    }
    let gpu = Ray::gpu().expect("gpu");
    let n = [0.0, -1.0, 0.0];
    let q = [[-9.0, 2.0, -9.0], [9.0, 2.0, -9.0], [9.0, 2.0, 9.0], [-9.0, 2.0, 9.0]];
    let f0 = [0.058, 0.05, 0.017, 1.0];
    let tri = |p: [[f32; 3]; 3], rough: f32| {
        let mut t = Tri::new(p, [n; 3], f0);
        t.rough = rough;
        t.metal = 1.0;
        t
    };
    let cam = Camera {
        right: [1.0, 0.0, 0.0], up: [0.0, 0.0, 1.0], back: [0.0, -1.0, 0.0], pivot: [0.0, -1.0, 0.0],
        dist: 1.0, units: 1.0, persp: 0, scale: 0.2, aspect: 1.0, sky: [0.24, 0.24, 0.24, 1.0],
        env: [1.0, 1.0, 1.0, 1.0], envmap: 1, env_cdf: 2, env_hidden: 1, count: 0, sun: [0.0, -0.7, 0.7],
        bounces: 4, ..Default::default()
    };
    let size = 16u32;
    for rough in [0.05f32, 0.2, 0.5] {
        let card = [tri([q[0], q[1], q[2]], rough), tri([q[0], q[2], q[3]], rough)];
        let mut tracer = Tracer::new(&gpu, &card, &atlas).unwrap();
        tracer.set_samples(1024);
        loop {
            let (_, done) = tracer.sample(&gpu, cam, size, size);
            if done { break; }
        }
        let rad = tracer.radiance(&gpu);
        let mid = &rad[(size * size / 2) as usize..(size * size / 2 + size) as usize];
        let mean: [f32; 3] = [0, 1, 2].map(|k| mid.iter().map(|c| c[k]).sum::<f32>() / mid.len() as f32);
        let mirror = at([0.0, -1.0, 0.0]);
        println!("rough {rough:.2}: card {:.4} {:.4} {:.4}   F0 × mirror radiance {:.4} {:.4} {:.4}", mean[0], mean[1], mean[2], f0[0] * mirror[0], f0[1] * mirror[1], f0[2] * mirror[2]);
    }
}
