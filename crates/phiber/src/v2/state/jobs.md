# Jobs — `phiber/v2/state`

| State | Open | Done |
|---|---|---|
| 🟡 in progress | 3 | 2 |

## Todo

- [ ] A handful of verbs: `get`, `has`, `set`, `remove` — fold `set_encoded`, `get_encoded`, `iter_keys` into them
- [ ] Persistence (see `app` jobs): transient-key list lives here

## Done

- [x] Every fixed key is a `BKey` enum in `state/key/` — `tests/key_budget.rs` (sync111–116)
- [x] Follow-ups are `BRule`s beside their levels, `state/rule.rs` is the table — `tests/rule_budget.rs` (sync117)
- [x] Area/zone size reducers and space switching moved to `area/size.rs`, `zone/size.rs`, `space/switch.rs` (sync118)
- [x] Keys for view pivot / lens, frame range / fps, links, command query, collaboration

Format: one line per job, `- [ ]` open / `- [x]` done; move a line from Todo to Done when it lands, with its test.
