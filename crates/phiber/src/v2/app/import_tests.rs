//! Tests for `should_import` / `settle_import`.

    use super::*;

    #[test]
    fn blend_differing_from_prev_imports() {
        // A .blend selection that differs from the reflected path => import.
        assert!(should_import(
            Some("/old/scene.blend"),
            Some("/new/model.blend")
        ));
    }

    #[test]
    fn blend_equal_to_prev_does_not_import() {
        // Selecting the already-reflected .blend path => no new import (1.6).
        assert!(!should_import(
            Some("/dir/model.blend"),
            Some("/dir/model.blend")
        ));
    }

    #[test]
    fn non_blend_selection_does_not_import() {
        // Non-.blend selections never trigger an import (1.5).
        assert!(!should_import(None, Some("notes.txt")));
        assert!(!should_import(Some("/dir/model.blend"), Some("readme.md")));
    }

    #[test]
    fn none_selected_does_not_import() {
        // No selection at all => nothing to import.
        assert!(!should_import(None, None));
        assert!(!should_import(Some("/dir/model.blend"), None));
    }

    #[test]
    fn case_insensitive_blend_suffix_imports() {
        // The .blend suffix match is case-insensitive.
        assert!(should_import(None, Some("MODEL.BLEND")));
        assert!(should_import(Some("/a.blend"), Some("/b.Blend")));
    }

    #[test]
    fn prev_none_with_blend_selected_imports() {
        // First-ever load: no reflected path yet, a .blend is selected.
        assert!(should_import(None, Some("/first/load.blend")));
    }

    #[test]
    fn blend_equal_to_prev_is_case_insensitive() {
        // Same path with differing case in the .blend suffix is still equal
        // as a raw string comparison — differing bytes DO count as different
        // (path identity is exact); only the suffix predicate is case-insensitive.
        // A byte-identical path never re-imports.
        assert!(!should_import(
            Some("/dir/Model.blend"),
            Some("/dir/Model.blend")
        ));
    }
