//! Round-trip property test: print → parse produces an equivalent stage.
//!
//! **Validates: Requirements 2.4**

use proptest::prelude::*;

use crate::device::usd::read::UsdaParser;
use crate::device::usd::write::UsdaPrinter;
use crate::device::usd::stage::UsdStage;

use super::generator::arb_stage;

/// Compare two stages for equivalence in metadata, prim paths, types, and
/// attribute values. Uses structural equality via `PartialEq`.
fn stages_equivalent(a: &UsdStage, b: &UsdStage) -> bool {
    // Metadata
    if a.default_prim != b.default_prim {
        return false;
    }
    if a.up_axis != b.up_axis {
        return false;
    }
    if (a.meters_per_unit - b.meters_per_unit).abs() > 1e-10 {
        return false;
    }
    // Prim tree
    let a_prims = a.traverse();
    let b_prims = b.traverse();
    if a_prims.len() != b_prims.len() {
        return false;
    }
    for (ap, bp) in a_prims.iter().zip(b_prims.iter()) {
        if ap.path != bp.path || ap.type_name != bp.type_name {
            return false;
        }
        if ap.attributes.len() != bp.attributes.len() {
            return false;
        }
        for (aa, ba) in ap.attributes.iter().zip(bp.attributes.iter()) {
            if aa != ba {
                return false;
            }
        }
    }
    true
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// For any valid UsdStage, `parse(print(stage))` produces an equivalent
    /// stage in metadata, prim paths, types, and attribute values.
    #[test]
    fn roundtrip_print_then_parse(stage in arb_stage()) {
        let printed = UsdaPrinter::print(&stage);
        let parsed = UsdaParser::parse(&printed)
            .expect("printed USDA should always parse successfully");
        prop_assert!(
            stages_equivalent(&stage, &parsed),
            "Round-trip failed.\nOriginal: {:?}\nPrinted:\n{}\nParsed: {:?}",
            stage, printed, parsed
        );
    }
}
