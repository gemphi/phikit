// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Cycles' own tests, ported from
//! `intern/cycles/test/integrator_tile_test.cpp`.
use crate::integrator::*;

#[test]
fn basic() {
    // The CPU-like case.
    assert_eq!(TileSize::best(false, [1920, 1080], 1, 1, 1.0), TileSize::new(1, 1, 1));
    assert_eq!(TileSize::best(false, [1920, 1080], 100, 1, 1.0), TileSize::new(1, 1, 1));
    // Enough path states for the whole image with every sample.
    assert_eq!(TileSize::best(false, [1920, 1080], 1, 1920 * 1080, 1.0), TileSize::new(1920, 1080, 1));
    assert_eq!(TileSize::best(false, [1920, 1080], 100, 1920 * 1080 * 100, 1.0), TileSize::new(1920, 1080, 100));
}

#[test]
fn extreme() {
    assert_eq!(TileSize::best(false, [32, 32], 262_144, 131_072, 1.0), TileSize::new(1, 1, 512));
    assert_eq!(TileSize::best(false, [32, 32], 1_048_576, 131_072, 1.0), TileSize::new(1, 1, 1024));
    assert_eq!(TileSize::best(false, [32, 32], 10_485_760, 131_072, 1.0), TileSize::new(1, 1, 4096));
    assert_eq!(TileSize::best(false, [32, 32], 8192 * 8192 * 2, 1024, 1.0), TileSize::new(1, 1, 1024));
}

#[test]
fn a_tile_never_asks_for_more_states_than_there_are() {
    for (image, sample, state) in [([1920, 1080], 64, 1 << 20), ([640, 480], 1024, 1 << 16), ([4096, 4096], 2, 1 << 22)] {
        let tile = TileSize::best(false, image, sample, state, 1.0);
        assert!(tile.width >= 1 && tile.height >= 1 && tile.sample_count >= 1, "{tile:?}");
        assert!(i64::from(tile.width) * i64::from(tile.height) * i64::from(tile.sample_count) <= i64::from(state), "{tile:?}");
    }
}
