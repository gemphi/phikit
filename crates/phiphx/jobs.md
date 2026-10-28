# Jobs — `phiphx`

| State | Open | Done |
|---|---|---|
| 🟡 in progress | 2 | 2 |

## Todo

- [ ] Everything under the folders' `jobs.md` (this file lists only crate-wide jobs)
- [ ] **Blender parity plan — `TODO.md`** (2026-10-28). A census of the user's 230 files by Blender 5.2 against phiphx, object by object. Open: Rigify pose (feet 13–16 cm low on the SHOE files), vertex parenting (50 files), Smooth by Angle geometry nodes (15), Cloth at the frame (12), Surface Deform (2), Simple Deform registered and tested (1). Each lands with a `tests/*_truth.rs` dumped by Blender and a clean `diff.py` on the real files
- [ ] Remove the `from-phi` docs once their content is folded into folder READMEs

## Done

- [x] One entry point (`Phx`); folders reorganised (`phx`, `scene`, `blend/*`, `import`, `export`, `cloud`, `geom`, `format`, `spatial`)
- [x] Loose public functions turned into methods; no warnings; 200-line rule except the files listed in folder jobs

Format: one line per job, `- [ ]` open / `- [x]` done; move a line from Todo to Done when it lands, with its test.
