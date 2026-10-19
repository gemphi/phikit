//! The two rules the proof sheet produced, kept as guards.

use super::*;

    #[test]
    fn six_terrains_every_curve_full_width_and_inside_the_frame() {
        assert_eq!(TERRAIN.len(), 6, "six terrains");
        for t in TERRAIN {
            assert!(!t.ridges.is_empty(), "{} has no ridge", t.name);
            for r in t.ridges {
                assert_eq!(r.curve.len(), SAMPLES);
                assert!(r.curve.iter().all(|v| (0.0..=1.0).contains(v)), "{} leaves the frame", t.name);
                assert!(r.rgb.iter().all(|v| (0.0..=1.0).contains(v)));
            }
        }
    }

    #[test]
    fn a_ridge_is_smooth_and_has_relief() {
        // The two rules the proof sheet produced, kept as a guard: a flat
        // line is the sky and a cliff is a gap-fill, and neither is a
        // silhouette. If a re-extraction ever lets one back in, this fails.
        for t in TERRAIN {
            for r in t.ridges {
                let mean = r.curve.iter().sum::<f32>() / SAMPLES as f32;
                let var = r.curve.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / SAMPLES as f32;
                assert!(var.sqrt() > 0.012, "{} has a ridge with no relief", t.name);
                let step = r.curve.windows(2).map(|w| (w[1] - w[0]).abs()).fold(0.0_f32, f32::max);
                assert!(step <= 0.11, "{} has a cliff of {step}", t.name);
            }
        }
    }

