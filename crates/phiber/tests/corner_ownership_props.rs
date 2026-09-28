//! Generated coverage for coherent rounded-surface corner ownership.
//!
//! Feature: phiber-resolved-ui-style-settings, Property 8
//! **Validates: Requirements 5.2, 5.3, 5.4, 5.5, 13.5**

use phiber::v2::view::draw::surface::{
    BCellPosition, BChildCornerSupport, BCorners, BRowPosition, BSurfaceOpacity,
    child_surface_plan, compatible_inner_radius,
};
use proptest::prelude::*;

fn arb_composition()
-> impl Strategy<Value = (BCorners, usize, usize, Vec<bool>, Vec<bool>, f32, f32, f32)> {
    (
        (any::<bool>(), any::<bool>(), any::<bool>(), any::<bool>()),
        1usize..=8,
        1usize..=8,
        0.0f32..=64.0,
        0.0f32..=32.0,
        0.0f32..=16.0,
    )
        .prop_flat_map(
            |((tl, tr, bl, br), rows, columns, radius, row_inset, cell_inset)| {
                (
                    Just(BCorners::new(tl, tr, bl, br)),
                    Just(rows),
                    Just(columns),
                    prop::collection::vec(any::<bool>(), rows),
                    prop::collection::vec(any::<bool>(), rows * columns),
                    Just(radius),
                    Just(row_inset),
                    Just(cell_inset),
                )
            },
        )
}

fn opacity(opaque: bool) -> BSurfaceOpacity {
    if opaque {
        BSurfaceOpacity::Opaque
    } else {
        BSurfaceOpacity::Transparent
    }
}

fn bits(corners: BCorners) -> [bool; 4] {
    [corners.tl, corners.tr, corners.bl, corners.br]
}

fn expected_row_claim(inherited: BCorners, row: usize, rows: usize, opaque: bool) -> BCorners {
    let first = row == 0;
    let last = row + 1 == rows;
    BCorners::new(
        inherited.tl && first && opaque,
        inherited.tr && first && opaque,
        inherited.bl && last && opaque,
        inherited.br && last && opaque,
    )
}

fn expected_cell_claim(
    inherited: BCorners,
    row: usize,
    rows: usize,
    column: usize,
    columns: usize,
    opaque: bool,
) -> BCorners {
    let first = row == 0;
    let last = row + 1 == rows;
    let leftmost = column == 0;
    let rightmost = column + 1 == columns;
    BCorners::new(
        inherited.tl && first && leftmost && opaque,
        inherited.tr && first && rightmost && opaque,
        inherited.bl && last && leftmost && opaque,
        inherited.br && last && rightmost && opaque,
    )
}

fn is_exact_partition(inherited: BCorners, surface: BCorners, child: BCorners) -> bool {
    bits(inherited)
        .into_iter()
        .zip(bits(surface))
        .zip(bits(child))
        .all(|((was_inherited, kept_by_surface), claimed_by_child)| {
            !(kept_by_surface && claimed_by_child)
                && was_inherited == (kept_by_surface || claimed_by_child)
        })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    /// Property 8: Corner ownership is coherent.
    ///
    /// Feature: phiber-resolved-ui-style-settings, Property 8
    /// **Validates: Requirements 5.2, 5.3, 5.4, 5.5, 13.5**
    #[test]
    fn corner_ownership_is_unique_touching_opaque_and_radius_compatible(
        (parent, rows, columns, row_opacity, cell_opacity, ancestor_radius, row_inset, cell_inset)
            in arb_composition(),
    ) {
        let parent_bits = bits(parent);
        let mut parent_remaining = parent;
        let mut row_claims = [0u8; 4];
        let mut row_residual_owners = [0u8; 4];
        let mut cell_claims = [0u8; 4];

        for row in 0..rows {
            let row_is_opaque = row_opacity[row];
            let before_row = parent_remaining;
            let row_ownership = before_row.assign_row(
                BRowPosition::at(row, rows),
                opacity(row_is_opaque),
            );
            let expected_row = expected_row_claim(before_row, row, rows, row_is_opaque);

            prop_assert_eq!(row_ownership.child, expected_row);
            prop_assert!(is_exact_partition(
                before_row,
                row_ownership.surface,
                row_ownership.child,
            ));
            for (count, claimed) in row_claims.iter_mut().zip(bits(row_ownership.child)) {
                *count += u8::from(claimed);
            }
            parent_remaining = row_ownership.surface;

            let row_plan = child_surface_plan(
                row_ownership.child,
                opacity(row_is_opaque),
                BChildCornerSupport::Propagated,
                ancestor_radius,
                row_inset,
            );
            if row_ownership.child.any() {
                prop_assert_eq!(row_plan.corners, row_ownership.child);
                prop_assert_eq!(
                    row_plan.radius,
                    compatible_inner_radius(ancestor_radius, row_inset),
                );
                prop_assert_eq!(row_plan.inset, 0.0);
                prop_assert!(row_plan.radius <= ancestor_radius);
            } else {
                prop_assert_eq!(row_plan.corners, BCorners::NONE);
                prop_assert_eq!((row_plan.radius, row_plan.inset), (0.0, 0.0));
            }

            let mut row_remaining = row_ownership.child;
            for column in 0..columns {
                let cell_is_opaque = cell_opacity[row * columns + column];
                let before_cell = row_remaining;
                let cell_ownership = before_cell.assign_cell(
                    BCellPosition::at(row, rows, column, columns),
                    opacity(cell_is_opaque),
                );
                let expected_cell = expected_cell_claim(
                    before_cell,
                    row,
                    rows,
                    column,
                    columns,
                    cell_is_opaque,
                );

                prop_assert_eq!(cell_ownership.child, expected_cell);
                prop_assert!(is_exact_partition(
                    before_cell,
                    cell_ownership.surface,
                    cell_ownership.child,
                ));
                for (count, claimed) in cell_claims.iter_mut().zip(bits(cell_ownership.child)) {
                    *count += u8::from(claimed);
                }
                row_remaining = cell_ownership.surface;

                let cell_plan = child_surface_plan(
                    cell_ownership.child,
                    opacity(cell_is_opaque),
                    BChildCornerSupport::Propagated,
                    row_plan.radius,
                    cell_inset,
                );
                if cell_ownership.child.any() {
                    prop_assert_eq!(cell_plan.corners, cell_ownership.child);
                    prop_assert_eq!(
                        cell_plan.radius,
                        compatible_inner_radius(row_plan.radius, cell_inset),
                    );
                    prop_assert_eq!(cell_plan.inset, 0.0);
                    prop_assert!(cell_plan.radius <= row_plan.radius);
                } else {
                    prop_assert_eq!(cell_plan.corners, BCorners::NONE);
                    prop_assert_eq!((cell_plan.radius, cell_plan.inset), (0.0, 0.0));
                }
            }

            for (count, retained) in row_residual_owners.iter_mut().zip(bits(row_remaining)) {
                *count += u8::from(retained);
            }
        }

        let parent_residual = bits(parent_remaining);
        for corner in 0..4 {
            prop_assert!(
                row_claims[corner] <= 1,
                "corner {} was claimed by multiple rows",
                corner,
            );
            prop_assert!(
                cell_claims[corner] <= 1,
                "corner {} was claimed by multiple cell_list",
                corner,
            );
            let final_owner_count = u8::from(parent_residual[corner])
                + row_residual_owners[corner]
                + cell_claims[corner];
            prop_assert_eq!(
                final_owner_count,
                u8::from(parent_bits[corner]),
                "corner {} did not have exactly one final owner",
                corner,
            );
        }
    }
}
