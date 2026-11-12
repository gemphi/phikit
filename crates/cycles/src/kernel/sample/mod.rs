// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/sample/`: the numbers a path draws,
//! and what a kernel makes of them.
//!
//! | File | Ported from |
//! |---|---|
//! | `util.rs` | `util.h`: Owen scrambling, the Morton code |
//! | `sobol.rs` | `sobol_burley.h`: the shuffled, scrambled Sobol sampler |
//! | `pattern.rs` | `pattern.h`: `path_rng_*`, the dimension of each draw |
//! | `tabulated.rs` | `tabulated_sobol.h`: the Classic pattern, from a table |
//! | `mapping.rs` | `mapping.h`: the disk, the polygon, the cosine hemisphere |
mod mapping;
mod pattern;
mod sobol;
mod tabulated;
mod util;
pub use mapping::{make_orthonormals, regular_polygon_sample, sample_cos_hemisphere, sample_uniform_disk};
pub use pattern::{SamplerData, SamplingPattern, path_rng_1d, path_rng_2d, path_rng_3d, path_rng_pixel_init};
pub use sobol::{sobol_burley, sobol_burley_sample_1d, sobol_burley_sample_2d, sobol_burley_sample_3d, sobol_burley_sample_4d};
pub use tabulated::{tabulated_sobol_sample_1d, tabulated_sobol_sample_2d, tabulated_sobol_sample_3d};
pub use util::{expand_bits, morton2d, nested_uniform_scramble_base4, reversed_bit_owen, reversed_bit_owen_base4};
