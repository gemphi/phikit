# Inspector File Needs Clean Rewrite

## Problem
The inspector/view_controller.rs file has:
- Duplicate code from incomplete refactoring
- Orphaned code blocks (lines 177-306)
- Brace mismatches
- Two `format_value` functions

## Solution
Rewrite the file cleanly with:
1. Clean impl block structure
2. No duplicate functions
3. Proper use of NSStackView (already partially implemented)
4. Remove all legacy manual positioning code

## Current Status
File won't compile due to brace mismatch. Need to either:
1. Manually fix the brace issues by removing orphaned code
2. Rewrite the entire file cleanly

## Recommendation
Skip inspector for now, move to next panel. Come back to inspector with fresh eyes.

## Next Panels to Fix
1. ✅ Toolbar - DONE (now uses NSToolbar)
2. ⏭️ Hierarchy - Use NSOutlineView
3. ⏭️ Activity Bar - Create NSActivityBar component
4. ⏸️ Inspector - Fix later (has compilation issues)
