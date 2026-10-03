# Moved from `gemphi/phi/crates/phiphx`

`phi` is being retired; `phiace/crates/phiphx` is the one reader. The old
`phi` crate (named `phx`, single-file `blend_native.rs`, 1434 lines) is the
**ancestor** of this crate's `importer/blend/*` split — every function in it
exists here in evolved form, so no reader code was ported back.

What was worth bringing across:

- `tests/fixtures/*.blend` — two small real production files (see
  `tests/fixtures/README.md`); the large ones stay in `crates/phiber/assets/production`.
- `examples/inspect_blend.rs` — compiles against the current API.
- `docs/from-phi/*.md` — the Blender DNA coverage table and conversion
  guides (reference only; the live gap analysis is
  `crates/phiber/docs/blend-read-parity.md`).
- `docs/from-phi/examples/{convert_blend_to_phx,blend_to_usd}.rs` — written
  against the old `phx::importers::blend_native` / `exporters::phx_scene`
  paths; kept as reference, **not compiled**. Port when needed.

Not brought across: `phi/lib/{phicfg,phidoc,phierr,phijob,phimem,phistd}` are
3-line empty stubs; `phi/lib/phical` (quantum/spatial) differs from
`phiace/crates/phical` (heat/mobius/q64/sphere/winding) — left for you to
reconcile.
