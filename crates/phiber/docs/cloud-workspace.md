# Rebuilding the cloud workspace

The cloud sandbox is recycled after a period of inactivity, taking the build
tree, the software GPU and the Blender install with it. Nothing is lost — the
device copy under `C:\Users\phiac\Workspace\gemphi\phiace` is the source of
truth — but the environment has to be rebuilt before anything can be verified
again. This is the recipe, which takes about fifteen minutes.

## 1. Bring the source across as archives, not files

Staging ~1000 files one by one is slow and expensive. Tar each crate on the
device instead, into a folder inside the connected tree (`device_stage_files`
cannot reach `$HOME` outside `mnt/`):

```bash
mkdir -p "$HOME/mnt/phiace/_to_delete/src-sync"
cd "$HOME/mnt/phiace"
tar -czf "$HOME/mnt/phiace/_to_delete/src-sync/t_root.tgz" \
    Cargo.toml Cargo.lock rust-toolchain.toml clippy.toml rustfmt.toml patches
for c in phiray phiphx phicore phiui phigpu; do
  tar -czf "$HOME/mnt/phiace/_to_delete/src-sync/t_$c.tgz" \
      --exclude=target --exclude=assets "crates/$c"
done
tar -czf "$HOME/mnt/phiace/_to_delete/src-sync/t_phiber.tgz" \
    --exclude=target --exclude=assets --exclude=docs crates/phiber
tar -czf "$HOME/mnt/phiace/_to_delete/src-sync/t_tooling.tgz" --exclude=target tooling
tar -czf "$HOME/mnt/phiace/_to_delete/src-sync/t_assets.tgz" \
    crates/phiber/assets/icons crates/phiber/assets/brand \
    crates/phiber/assets/fonts crates/phiber/assets/mod.rs
```

Two exclusions matter, and both were found the hard way by a command that ran
past its limit: `crates/phiber/assets` is **1.4 GB** and
`crates/phiber/docs` is **548 MB** (the vendored Blender manual). Without
them the whole source is about 11 MB and each tar takes seconds.

`tooling/` is needed too — `gpui` depends on `tooling/perf` through
`util_macros`, and cargo refuses to load the workspace manifest without it.

A background job (`nohup … &`) does **not** survive a `device_bash` call: each
call is its own sandbox and the process dies with it. Everything has to finish
inside the 180 s limit, which is why this is one tar per crate.

## 2. Unpack and build

```bash
cd <scratch>/ws && for f in /mnt/user-data/uploads/phiace/_to_delete/src-sync/t_*.tgz; do tar -xzf "$f"; done
RUSTUP_TOOLCHAIN=stable cargo build -p phiber
```

`RUSTUP_TOOLCHAIN=stable` is required. `rust-toolchain.toml` pins 1.95.0 and
`static.rust-lang.org` is blocked by the egress proxy, so rustup cannot fetch
it — but the container's `stable` **is** 1.95.0, so overriding by environment
(rather than editing the file, which would then sync back) builds correctly.
A cold build is about fifteen minutes.

## 3. Software GPU

```bash
apt-get install -y mesa-vulkan-drivers
export VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.json
```

lavapipe runs every phiray and phiber GPU test, and the `color_render`
example, with no hardware adapter.

## 4. Blender and the test asset, for comparisons

```bash
pip install bpy          # Blender as a Python module, for the truth scripts
```

Stage `crates/phiber/assets/CW-Fashion/Epic-Pods-Female-01/Female-01-Chatty-Whims-Rigged-Lace.blend`
(178 MB, one call). Its textures and HDRI are **not** in the repo — they live
outside it on the user's machine — so Blender and phiber both draw the missing
textures magenta. For the world, generate a stand-in equirectangular sky and
put it at `crates/Downloads/lonely_road_afternoon_puresky_4k (2).exr`, which
is where the file's `//`-relative path resolves; write it as Radiance HDR and
convert with `bpy` (`image.file_format = 'OPEN_EXR'; image.save_render(…)`),
because Blender's EXR loader rejects HDR bytes under an `.exr` name.

## Known-good baselines

With the tree above, at 2026-09-23:

| Crate | Result |
|---|---|
| phiphx | 113 passed, 6 failed (fixtures that need the excluded assets; 155 + 6 at batch 47, 156 + 6 at batch 62, 157 + 6 at batch 78, 158 + 6 at batch 79, 160 + 6 at batch 80, 162 + 6 at batch 81, 163 + 6 at batches 82–84, 171 + 6 at batches 85–87, 172 + 6 at batch 88, 173 + 6 at batch 89, 176 + 6 at batch 90, 181 + 6 at batch 91, 182 + 6 at batch 92, 185 + 6 at batch 93, 187 + 6 at batches 94–95, 188 + 6 at batches 96–111) |
| phiray | 34 passed, 0 failed (38 at batches 79–82, 41 at batch 83, 43 at batches 84–93, 44 at batches 94–99, 45 at batches 100–102, 46 at batches 103–107, 54 at batch 108, 55 at batches 109–111) |
| phiber | **536 passed, 0 failed** (594 at 2026-09-26, batch 26; 722 at batch 53, 727 at batch 54, 731 at batch 55, 733 at batch 56, 735 at batch 57, 736 at batch 58, 738 at batch 59, 739 at batch 60, 741 at batches 61–62, 742 at batch 63, 744 at batch 64, 745 at batch 65, 747 at batch 66, 749 at batch 67, 752 at batch 68, 757 at batch 69, 759 at batch 70, 760 at batch 71, 764 at batch 72, 769 at batch 73, 772 at batch 74, 775 at batch 75, 778 at batch 76, 780 at batch 77, 782 at batch 78, 797 + 9 failing at batch 79, 800 + 8 failing at batch 80, 800 + 10 failing at batch 81, 800 + 11 failing at batch 82, 801 + 11 failing at batch 83, 802 + 11 failing at batch 84, 808 + 12 failing at batch 85, 811 + 14 failing at batch 86, 840 + 0 at batch 87, 844 + 3 failing at batch 88, 863 + 3 failing at batch 89, 871 + 3 failing at batch 90, 884 + 3 failing at batch 91, 910 + 1 failing at batch 92, 928 + 2 failing at batch 93, 937 + 2 failing at batch 94, 955 + 4 failing at batch 95, 963 + 8 failing at batch 96, 974 + 4 failing at batch 97, 987 + 6 failing at batch 98, 997 + 9 failing at batch 99, 1001 + 7 failing at batch 100, 1003 + 7 failing at batch 101, 1005 + 7 failing at batches 102–104, 1010 + 7 failing at batch 105, 1013 + 7 at batch 106, 1016 + 7 at batch 107, 1018 + 7 at batches 108–109, 1020 + 7 at batch 111 — the failures from work in progress on the machine) |

phiber reaching zero needed one real fix: four `_to_delete/` folders left
inside `crates/phiber/src/v2/` by the structure migration. They were not
compiled, but the source-scanning guard tests read files by path and
`key_budget` tripped on them. They now live in `_to_delete/from-src/` at the
repo root.

## 5. Sending changes back (2026-09-26)

Device deletes are off and `tar` cannot unlink on the FUSE mount, so a
batch goes over as one archive and is copied file by file:

```sh
# container: everything changed since the last batch's marker
find crates -newer /tmp/b<N-1>_marker -type f \( -name '*.rs' -o -name '*.wgsl' -o -name '*.md' -o -name '*.py' -o -name '*.txt' -o -name '*.toml' -o -name '*.svg' \) -not -path '*/target/*' | sort > /tmp/b<N>.list
tar czf /mnt/user-data/outputs/batch<N>.tgz -T /tmp/b<N>.list && touch /tmp/b<N>_marker
# device_commit_files → phiace/_to_delete/sync/batch<N>.tgz, then on the device:
mkdir -p $HOME/b<N> && tar xzf _to_delete/sync/batch<N>.tgz -C $HOME/b<N>
cd $HOME/b<N> && for f in $(find crates -type f); do mkdir -p "$HOME/mnt/phiace/$(dirname $f)"; cp -f "$f" "$HOME/mnt/phiace/$f"; done
```

A file turned into a folder (`x.rs` → `x/mod.rs`, batch 70) leaves the old
`x.rs` on the device, which rustc refuses beside the folder: move it to
`_to_delete/stale/b<N>/` in the same step. Compare an `md5sum` on both sides afterwards (`base<N>_<name>.md5` beside
the archive lets the device run `md5sum -c`). Batches 17–111 are on the
device; `_to_delete/sync/` holds every archive. The latest marker is
`/tmp/b111_marker`. The `find` pattern lists source and text only — a
binary asset (batch 47's `crates/phiber/assets/blend/startup-{3,4,5}.blend`) is
appended to the list by hand. A release build (`--release`, for timing phiray's examples) adds ~4 GB
under `target/release`: delete it before a full debug suite. When clearing
`target/debug/deps`, delete only test and example binaries — never `.rlib`,
`.rmeta` or `.so`: batch 84 cleared every file over 20 MB there, dependency
libraries included, and the next suite rebuilt every dependency (~15 min).
The expected machine checksums before a batch are the last full container
list overlaid with the last batch's `base<N>_crates.md5`
(`expect_b<N>.md5`, beside the archives). Unpack the machine's changed
files with `tar xzmf` (or `touch` them after): tar keeps the machine's
modification times, and a file older than the container's last build is
not rebuilt — batch 85's merge compiled the machine's new test against a
library that did not have the code it tested yet. A machine file can also be
a rewrite of an older copy: batch 90's `view3d/rendered/mod.rs` began
before batch 84 and no longer compiled. Check every machine-changed file
against the last `expect_b<N>.md5`, recover the copy it replaced from the
archives (`tar xzOf batch<M>.tgz <path>`, matched by checksum), and keep
the machine's change on top of it. Likewise, a layout golden recorded on
another tree: re-record it here (`UPDATE_LAYOUT_GOLDEN=1`) and check that
its diff from the last golden is exactly what the machine describes. Both
happened again at batch 91 (`part/filter/mod.rs` without the X-ray
button) and at batch 92 (nine files, among them `scene/mod.rs` without
the posing thread's module and phiphx's `write/mod.rs`, which stopped it
compiling on the machine): the other agent seems to work from an older
copy and bring files across, so expect it every batch. Check the machine
once more right before the archive goes over — at batch 92 files landed
twice more while the suite ran, three of them old copies; `batch92b.tgz`
is the one that went in. Logs go to `/home/claude/logs/` — the `/tmp` filesystem
filled once with test logs and stale test binaries, and the root disk once
with 37 000 stale files in `target/debug/deps`; deleting every hash but the
newest per name freed 4.5 GB (cargo rebuilds what it still needed, ~5 min).
Between suites, `phiber-*` / `phiphx-*` outputs in `deps` older than the
current build (`! -newermt`, not `.rlib/.rmeta/.d`) are safe to drop and
free ~2 GB, and so are the `*.rcgu.o` / `*.dwo` codegen leftovers in `deps` older than the current build (100 000 files, ~1–4 GB); never touch `target/debug/incremental` while a build runs. Docs go straight to
`crates/phiber/docs/` with `device_commit_files` (`force: true` when the
file already exists) and are mirrored to the project as `claude/<name>.md`.

Blender's C sources for the pose solver (constraints, the legacy IK
solver, drivers, the math library) are staged under
`_to_delete/refs85/` on the machine; `truth/blender_rig.py` regenerates
THYMUS's rig truth with the installed `bpy` 5.0.1.

`pip install bpy==4.2.23` (a Python 3.11 venv) and the installed 5.0.1
write factory startup files (`assets/blend/make_startup.py`); PyPI has no
3.x `bpy`, and `download.blender.org` is blocked on both sides, but
`REFS/blender/blender` is a full git clone: `git show
v3.6.23:release/datafiles/startup.blend` is the 3.6 factory file (4.x and
5.x tags hold LFS pointers there). Blender source for porting lives at `phiace/REFS/blender/blender`; a file
more than seven folders deep cannot be staged directly — copy it to
`_to_delete/` on the device first (`view2d_draw.cc`, `UI_view2d.hh` are
there already).

**The machine's tree is edited outside these batches too** (batch 79 found
another agent's work landed since batch 78). Before a batch goes over:
checksum the machine's source tree against the container's
(`md5sum -c` in chunks per folder — one pass over the FUSE mount outruns the
180 s limit), pull every file that changed there into the container, and
build and test on the merged tree; copy only the batch's own files, each
guarded by the checksum of the version it was built on (`guard<N>.md5`),
and commit the layout golden with `expectedMtimeMs`. A golden the machine
re-recorded can predate the tree it sits in (batches 80 and 81 each found
one without batch 79's X-ray button): re-record it on the merged tree and diff it
against both sides — each diff should be only the other side's changes.
Commit the golden in a call of its own: `force` (needed for the docs)
would override its `expectedMtimeMs` guard. When the golden differs only
by the machine's own work in progress, leave it for that work. Files eight or more
folders deep cannot be staged — read them through `device_bash` (`base64`)
or compare checksums there.

A file that *moved* (`ctrl.rs` → `ctrl/mod.rs`) leaves its old copy on the
device, and Rust refuses a module found in two places: `mv -n` the old file
into `_to_delete/moved/` on the device before copying the batch in
(`theme/semantic.rs` → `theme/semantic/{mod,set}.rs` in batch 29).

The mock-ups the user approves against live in the *Sphere Shades* artifact
(`scratchpad/sphere_shades.html` here, with `screens.js` inlined into it —
edit the `.js`, then splice the function back into the page); its snapshots
are rendered with Playwright (`shot2.py`, `shot3.py` for the Preferences
mocks) and delivered as PNGs. The user's own screenshots of the app are the
truth to sample: `PIL` on the upload, compared pixel by pixel against the
ladder's hexes, is what found the shade-2 area cards in batch 31.

**Two traps found in batch 87.**

- `device_commit_files` sends a staged path's *first* upload again when the
  same path is committed later in the session. Batch 86's docs never
  reached the machine this way, and batch 87's first archive went over
  stale; `fix87.tgz` put the right files back. Stage every commit under a
  name not used before in the session (`batch<N>b.tgz`,
  `b<N>_ui-parity-status.md`), then check the result on the machine with
  `md5sum`.
- To find what the machine changed, compare against the time the last
  archive of the machine's files was made (`find -newer
  _to_delete/sync/machine<N>.tgz`), not against a doc commit. Files the
  other agent saved between those two moments were missed until the
  guard's `md5sum -c` caught them.

**Three more traps found in batch 93.**

- The machine deletes and renames files too: `part/filter/overlay.rs`
  became `overlay/{mod,set}.rs`, a test was renamed, and a module file
  was dropped. Those old copies stay in the container and break things
  there. A file and a folder with one module name do not compile, an
  unwired file fails `every_file_is_reachable`, and a retired test fails
  against the new code. Each batch, list both trees (`find crates/*/src
  crates/*/tests -type f | sort`, about 1300 files, sent back through
  `gzip | base64`) and `comm` them. Every file only in the container that
  is not the batch's own goes to `/home/claude/stale<N>/`.
- Regenerate `/tmp/targets.txt` *after* the merge, or a test the machine
  added is never run.
- A test that fails to compile stops its whole chunk of 45 targets, and
  none of that chunk's tests run: batch 93's `extras_follow_edits` did
  this when `scene_extras_now` began returning a shared `Rc`. Grep the log
  for `error[` and rerun that chunk once the error is fixed.

**A recycled container (batch 95).** The recipe above rebuilt it in about
25 minutes: twelve tars staged in one call (`_to_delete/src-sync95/`, with
`crates/phiber/assets/blend` and `studiolight` added — the startup files
and the studio HDRI the tests read), a cold `cargo build -p phiber` of 12
minutes, and `apt-get update` before `mesa-vulkan-drivers` (the install
fails without it). Take the full snapshot *before* the first edit, write
its checksums (`find crates … | xargs md5sum`), and touch the marker right
after unpacking (`tar xzmf` stamps every file with the unpack time): the
snapshot's checksums are the guard for the next batch. A machine file that
began from a copy older than the last batch is merged with `git merge-file
mine base theirs`, the base being the copy in the archive whose checksum
the expected list holds.

**Measuring a frame (batch 92).** `scratchpad/keep/zz_frame_cost_probe.rs`
is a scratch test — never shipped — that opens THYMUS through `Phx::open`
and `settle_import`, puts it in `scene_cache`, and times still frames and
frame changes. Copy it into `crates/phiber/tests/`, run
`PROBE_SPACE=layout PROBE_N=20 cargo test -p phiber --test zz_frame_cost_probe -- --nocapture`
(`PROBE_EMPTY=1` for no scene), and delete it and its binary before
packing. For where the time goes, run its binary under
`valgrind --tool=callgrind --toggle-collect='*probe_frame_change*'` (or
`*probe_still_frames*`), three frames, and read `callgrind_annotate
--inclusive=yes`; ignore the `'2` recursive entries. Build one test at a
time: `cargo build -p phiber --tests` links every test binary at once and
fills the disk (a linker bus error); then clear `target/debug/deps` of test
binaries and `target/debug/incremental`.

**Batch 96: the batch goes over as its own files only.** The archive
(`batch96.tgz`) held the seventeen files this batch wrote, not every file
newer than the marker: the machine's own files, merged in before the suite,
are already on the machine, and copying them back only risks overwriting a
newer save. `guard96.md5` was checked on the machine before the copy and
`base96_crates.md5` after it. `expect_b97.md5` beside them is the whole
container tree after the batch — the machine's expected state for the next
merge. Two traps, again: a machine file landed a minute after
`machine96.tgz` was made (`find -newer` that archive caught it), and a
source-reading test pins a renderer's spelling — `viewport_click_selects`
reads `orbit.rs` for `.on_click(hit.on_click(ctx))` — so a new handler goes
where it leaves that text intact. A long sleep in the container's shell
(over two minutes) ends the call; poll the detached suite in shorter waits.

**Batch 97: checking a saved file in Blender without a scratch file in the
tree for long.** A save that writes something new (here `r.xsch / ysch /
size`) is checked by a throwaway phiphx test that applies `BlendEdits` to a
3.6, a 4.2 and a 5.0 file and writes them to `/tmp`, then `bpy` opens each
and prints the fields. Delete the test and its binary
(`target/debug/deps/zz_*`) before packing. The layout golden the machine
recorded at batch 96 is still in the tree; re-recorded here it is identical
to batch 96's own, so batch 97 draws nothing new.

**Batch 98: a long `find` over the mount times out.** `find crates/phiber
crates/phiphx/src … -newer …` with the guard check in one `device_bash`
call ran past 120 s; split the guard (`md5sum -c`), the existence checks and
the `-newer` scan into separate calls with `timeout_ms` 170000, and keep the
scan to `src` and `tests`. A machine test can also arrive with an old
modification time (`on_state_budget.rs` was not newer than the last archive
but was missing here): the side-by-side file list (`comm`) is what finds it.

**Batch 99: finding where a frame goes, and two traps.**

- A heavy scene is made in a scratch test (`zz_heavy_probe.rs`, deleted
  before packing): N objects sharing one `Arc<SceneMesh>` of an
  M-segment sphere, pushed into `SceneModel.objects` with one collection
  over them, then `scene_cache` set and `ModalKey::Open` cleared. It
  prints a still frame, an orbit frame, a zoom frame and a click. Under
  `valgrind --tool=callgrind --separate-threads=yes
  --toggle-collect='*gpui*window*Window*draw*'` (the toggle on
  `BPhiberApp::render` alone catches only the element build, not the layout
  and paint that follow it) the biggest file is the UI thread's; with the
  software GPU under valgrind the CPU painter ran, so read that profile for
  the shape of the work, not the numbers.
- Never edit source while the chunked suite runs: a chunk compiles when it
  starts, so a file moved under it fails that chunk's 45 targets and the
  lib's tests with it (two `error[` lines, 500 tests unrun). Check
  `grep -a "error\[" phiber<N>.log` before trusting a total, and rerun the
  chunk (`grep -l <target> /tmp/tchunk_*`).
- A machine test can be older than the last archive and still differ from
  here (`semantic_style_derivations.rs` at batch 99): `find -newer` misses
  it. Compare the md5 of every test a compile error names against the
  machine before deciding whose it is.
- The style budget counts `.size(` in any renderer file, whatever type it is
  called on: name a method that returns a size something else (`pixels`).

**Batch 100: the checksum pass is the merge check, not `-newer`.**

- Eight machine files differed from the batch 99 record with modification
  times older than batch 99's archive, so `find -newer` listed none of them;
  only `md5sum -c base<N>_crates.md5` after the copy found them. Run that
  full pass before calling a sync done, stage whatever it names, and diff
  each against the tree here: a file the other agent restored from an old
  copy (`part/filter/mod.rs`, from before batch 94's X-ray toggle) needs a
  three-way merge, the rest are taken whole. A merged file goes back in a
  supplement (`batch100b.tgz`, guarded by the machine's current md5), and
  the baseline is recomputed after the merge so `expect_b<N+1>.md5` is the
  tree both sides hold.
- A test that polls a worker which keeps only its newest result cannot
  assert which result it sees first — nor that it sees any but the last:
  under the full suite's load `render_thread_refines_latest_view` saw only
  the full-size frame (batch 101). Assert what a poll can know (the sizes
  it sees never shrink) and put the exact sequence in a unit test of the
  state machine.
- Blender truth for a save: `python3 crates/phiphx/tests/truth/blender_saved.py
  target/tmp/blender_opens_saves /tmp/saved.txt` after the test has written
  the files (it fails first, leaving them), then copy the output over
  `tests/truth/saved.txt`. `bpy` 5.0.1 is installed here; a 5.0 scene's
  registered settings are under `scene.bl_system_properties_get()`, not
  `scene.keys()` (batch 102).
- When the bridge is slow (batch 104: a bare `ls` over two minutes), split
  the baseline check by crate and by phiber subtree (`grep` the md5 list
  into parts of a few hundred files) and run each in its own call; record
  which parts passed and leave the rest for the next batch rather than
  retrying the whole list.
- An add-on's `EnumProperty` is stored as the item's number, not its
  identifier (batch 104): `bpy`'s `bl_rna.properties[name].enum_items`
  gives `(identifier, value)` for a static enum; a dynamic one (the
  denoiser) lists nothing and the numbers come from the add-on's source.
- The disk fills (batch 105: 252 GB, 100 % used): the chunked suite's
  cleanup deletes each chunk's binaries, but a `cargo test --test x`
  outside it leaves a 100–300 MB binary each in `target/debug/deps`.
  `find target/debug/deps -maxdepth 1 -type f -name '*-*' ! -name '*.d'
  ! -name '*.rlib' ! -name '*.rmeta' -size +20M -delete` and
  `rm -rf target/debug/incremental/*` freed 23 GB; the next build remakes
  them.
- **A gpui test has no frame loop (batch 111):** `window.request_animation_frame()`
  queues a next-frame callback that nothing delivers; a test that depends
  on it calls `window.simulate_next_frame(cx)` (test-support) then
  `run_until_parked`. The start-up frames alone (`add_window_view`,
  `simulate_resize`, a notify) are several draws, so a flag set in one
  paint is seen by the next render.
- **The user's session journal is in the workspace (batch 107):**
  `target/release/phiber-session.log` under the connected folder — read it
  with `device_bash` (`head`, `grep -n "Tracer:\|frames in\|slow frame\|
  Opened"`), never ask the user to paste it. It names the build, the render
  device, every action with its time and gap, read stages, frame summaries,
  slow frames, the tracer's settle lines and, since batch 108, the tracer's
  scene-build, pose and sample-tally lines. Batch 108's reading of it is in
  `rendering.md` §22.
- The scene-build probe is kept at `scratchpad/keep/zz_build_probe.rs`
  (batch 108): it opens THYMUS and times `rendered::still` at 8 × 8 with
  one sample — the scene build, essentially — per shading
  (`PROBE_SHADING=material|rendered|solid`). Debug: 1.1 s for 436 k
  triangles in Material mode, the IBL precompute included.
- **Profiling a frame (batch 106): callgrind lies under the gpui harness,
  a stack sampler does not.** Under valgrind the tracer thread is as slow
  as the UI thread, so the CPU painter (`BMesh::paint → fills`) runs every
  frame and dominates the profile — natively it never runs. There is no
  `perf` in the container; `gdb` is there. The sampler: start the probe
  with `PROBE_FRAMES=100000` (a long still run), find the UI thread by
  `/proc/<pid>/task/*/comm == probe`, then `gdb -batch -p <tid> -ex "bt
  60"` in a loop (about 1.2 s a sample; 250 samples ≈ 0.1 ms each at a
  25 ms frame) and aggregate the `phiber::` frames inclusively and by
  innermost frame (`/tmp/pmp.sh`, `/tmp/agg.py` in this container; the
  recipe is in `frame-cost.md`). Run it detached with `setsid nohup …
  < /dev/null & disown`: a plain `(nohup … &)` from the tool's shell did
  not survive the call. Compare a 3-object run with a 500-object one; the
  difference is the per-object cost.
- The heavy probe is kept at `scratchpad/keep/zz_heavy_probe.rs` (batch
  106): copy it into `crates/phiber/tests/`, run the binary by name —
  `<bin> probe --nocapture`; `--list` shows no tests under the gpui
  harness — and delete it and `target/debug/deps/zz_heavy_probe-*` before
  packing. On this 2-core container a frame's time varies by ±5 ms run to
  run: use `PROBE_FRAMES=40` and compare several runs, not one.
- Two probes for a frame's cost (batch 105, both deleted before packing):
  `zz_heavy_probe.rs` times a still, an orbit, an orbit with a selection
  and a zoom over `PROBE_N` objects sharing a `PROBE_SEGS`-segment sphere
  in `PROBE_SHADING`; `zz_parts_probe.rs` times the view-model pieces
  (`scene_objects`, `tree::flatten`, the Outliner build, `BHidden`,
  `scene_metrics`) on 500 cubes. Subtract the 3-object numbers from the
  500-object ones: that difference is the per-object cost, and the
  harness's own ~13 ms falls out.
- A gpui test drives the app's timers itself:
  `cx.executor().advance_clock(8 ms)` then `run_until_parked()` fires one
  tick of `ensure_ray_ticker`; a render on the real GPU thread meanwhile
  needs a `std::thread::sleep` between ticks
  (`the_rendering_workspace_opens_on_a_render.rs`).
