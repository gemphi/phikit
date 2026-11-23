# gpui_glass - to do

What is left, the nearest first. The steps and what moved are in
[PLAN.md](PLAN.md).

- Look at Phiber's glass mode on screen with the consolidated pass
  (`PHIBER_STATE="ctx.theme_mode=glass;ctx.theme=Phiber"`). The device
  tests pass; nobody has looked.
- A wgpu pass from the same plan: two half-size layers, the copy, the
  four draws (PLAN, step 3).
- `gpui_wgpu` draws `Backdrops` with it, which is glass in a browser and
  on Linux (step 4).
- A test that draws one panel on a GPU and compares it with the CPU
  stages (step 5).
- Phiber's theme hands the crate its numbers through `GlassMaterial`
  (step 6). Phiber's code is not touched until appkit holds.
- One table for the web apps' CSS (step 7).
- Glass on an appkit window is drawn (20 November 2026): `PMark::Glass`
  in `pui`, played as a backdrop, and a look that is glass has the
  window show the desktop blurred. Left: Phigen's panes take their
  numbers from `data/look.tsv`, not yet through `GlassMaterial`.
