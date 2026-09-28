# v2/state

State — a flat `HashMap<String, String>` keyed by encoded `BPath`.

Every key is `ctx.<seg>.<seg>...`. Host state naturally nests under the
host's slug: `ctx.<host>.<tail>`. Read via `state.host(host).get(tail)`.
Three-letter accessors: `get`, `set`, `has`.

## Files

| File | What it is |
|---|---|
| `mod.rs` | `BState` — the flat store; `get / set / has / remove` take `impl BKey`. |
| `path.rs` | `BPath` — hierarchical state key. |
| `keys/` | One enum per domain, each `impl BKey` — the only place a path is spelt (`tests/key_budget.rs`). |
| `rule.rs` | `trait BRule` and the ordered table of follow-ups; each rule lives with its level (`tests/rule_budget.rs`). |
| `action.rs` | Dispatch of a rendered `BAction`: store, `edit::before_set`, rules, close overlays. Compares no paths. |
| `edit.rs` | The caret: which field is being typed into, and its value for Escape. |
| `menu.rs` | Menu open/close reducer and `close_overlays`. |
| `log.rs` | The action log — one sentence per action, for the status bar. |
| `seed.rs` | Boot defaults — the initial state a fresh `BState` starts with. |
| `key.rs` | `int_or` / `float_or` and the `ui_style` catalogue. |
| `val.rs` | Static value constants — flags, modal slugs, Blender's factory defaults. |

Moved out to their levels: area sizes → `area/size.rs`, zone sizes →
`zone/size.rs`, switching space → `space/switch.rs`, file-browser history →
`part/custom/file/op.rs`, the other follow-ups → beside their owners
(see `rule.rs`). `tests/rule_budget.rs` keeps this list from growing.

Status and next steps: [`jobs.md`](jobs.md).
