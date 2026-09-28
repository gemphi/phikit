# v2/view

`BView` — declarative view tree. Seven store-level variants plus primitives.
Primitives (`Row`/`Column`/`Panel`/`Icon`/`Label`/`Button`) legal ONLY
inside the `body` of a `Part` node.

## Files

| File | What it is |
|---|---|
| `mod.rs` | `BView` — declarative view tree. |
| `prim/` | Primitives — constructors legal only inside `Part.body`. Split by widget family — one file per primitive (row, table, tree, panel, rail, menu, …),… |
| `render/` | `BRenderer` — the single seam that turns store-level `BView` nodes into pixels. Backend-agnostic: `BAsciiRenderer` is included for headless walks… |

Status and next steps: [`jobs.md`](jobs.md).
