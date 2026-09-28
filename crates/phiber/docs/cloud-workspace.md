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
| phiphx | 113 passed, 6 failed (fixtures that need the excluded assets; 155 + 6 at batch 47, 156 + 6 at batch 62) |
| phiray | 34 passed, 0 failed |
| phiber | **536 passed, 0 failed** (594 at 2026-09-26, batch 26; 722 at batch 53, 727 at batch 54, 731 at batch 55, 733 at batch 56, 735 at batch 57, 736 at batch 58, 738 at batch 59, 739 at batch 60, 741 at batches 61–62, 742 at batch 63, 744 at batch 64, 745 at batch 65, 747 at batch 66, 749 at batch 67, 752 at batch 68, 757 at batch 69, 759 at batch 70, 760 at batch 71, 764 at batch 72, 769 at batch 73, 772 at batch 74, 775 at batch 75, 778 at batch 76, 780 at batch 77) |

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
the archive lets the device run `md5sum -c`). Batches 17–73 are on the
device; `_to_delete/sync/` holds every archive. The latest marker is
`/tmp/b77_marker`. The `find` pattern lists source and text only — a
binary asset (batch 47's `crates/phiber/assets/blend/startup-{3,4,5}.blend`) is
appended to the list by hand. Logs go to `/home/claude/logs/` — the `/tmp` filesystem
filled once with test logs and stale test binaries, and the root disk once
with 37 000 stale files in `target/debug/deps`; deleting every hash but the
newest per name freed 4.5 GB (cargo rebuilds what it still needed, ~5 min).
Between suites, `phiber-*` / `phiphx-*` outputs in `deps` older than the
current build (`! -newermt`, not `.rlib/.rmeta/.d`) are safe to drop and
free ~2 GB, and so are the `*.rcgu.o` / `*.dwo` codegen leftovers in `deps` older than the current build (100 000 files, ~1–4 GB); never touch `target/debug/incremental` while a build runs. Docs go straight to
`crates/phiber/docs/` with `device_commit_files` (`force: true` when the
file already exists) and are mirrored to the project as `claude/<name>.md`.

`pip install bpy==4.2.23` (a Python 3.11 venv) and the installed 5.0.1
write factory startup files (`assets/blend/make_startup.py`); PyPI has no
3.x `bpy`, and `download.blender.org` is blocked on both sides, but
`REFS/blender/blender` is a full git clone: `git show
v3.6.23:release/datafiles/startup.blend` is the 3.6 factory file (4.x and
5.x tags hold LFS pointers there). Blender source for porting lives at `phiace/REFS/blender/blender`; a file
more than seven folders deep cannot be staged directly — copy it to
`_to_delete/` on the device first (`view2d_draw.cc`, `UI_view2d.hh` are
there already).

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
