// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use crate::kernel::*;

#[test]
fn a_work_tile_is_eleven_words_as_the_kernel_reads_it() {
    assert_eq!(std::mem::size_of::<KernelWorkTile>(), 11 * 4);
    assert_eq!(KernelWorkTile::default().work_size, 0);
}
