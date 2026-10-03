# Why resizing and changing space feel slow

## A large scene: the Outliner, not the viewport (batch 92)

With THYMUS open (288 Outliner rows), a debug build spent about 190 ms a
frame in Layout, Animation and Shading, against 16–25 ms with nothing
loaded; the machine's slow-frame log showed 75–115 ms. A scratch test that
opens THYMUS and times twenty frames, read under callgrind, put the time
here:

| What | Fix |
|---|---|
| Listing a collection's objects looked for each one's children by scanning the whole level — 288² look-ups a frame, ~40% of it | `outliner/object_rows.rs`: the children indexed once per level (`BLevel`) |
| Every row built and laid out each frame, though a dozen are in view — about half of the rest | `view/draw/tree/window.rs` (`BRowWindow`): a long tree lays out the rows in view and eight either side; spacers keep the scroll range |
| The hidden set built six times a pass (viewport, overlay, extras, picks), each a walk of the collection tree and two keys per object | `BHidden::begin_pass` / `end_pass`: one set per kind per pass, answers kept |

| THYMUS, debug build, 1600×900 | Before | After |
|---|---|---|
| Layout, still frame | 190 ms | 36–41 ms |
| Animation, still frame | 190 ms | 40–45 ms |
| Shading, still frame | 190 ms | 43–48 ms |
| Layout, frame change | 202 ms | 66–69 ms |

What is left of a frame change is mostly the viewport's extras
(`BPickExtra::list`, `camera::with_view`, the overlay's projection) and
GPUI laying the window out. Phiber itself is unoptimised in a dev build
(`profile.dev.package.phiber`), so a release build is about 5–10× faster
than these numbers.

## A rig's bones (batch 93)

THYMUS's rig draws 1366 lines in Object mode: 54 custom shapes. A first
cut listed them again for every caller: the viewport, its picks and the
extras list. It also looked up the object's name, hidden state and colour
once per line and stroked every line as its own path. That added about 10 ms
to a still frame in a debug build. Now:

| What | Fix |
|---|---|
| The extras (cameras, lights, empties, bones) rebuilt for each caller, every frame | `scene_extras_now` keeps one list (`BExtrasMemo`) until the frame, the rigs' pose or the session's edits change |
| Name, hidden state and colour asked once per line | once per object: the lines are laid out object by object (`chunk_by`) in the viewport's projection and in `BPickExtra::list` |
| A path per line | a path per run of one colour (`extra::paint`) |

With THYMUS, per frame, debug build: listing the extras for picks takes 0.6 ms,
projecting them 0.25 ms and stroking them 0.8 ms. When the frame changes, the
list is rebuilt once, which takes about 4 ms. The probe's still frame reads
47–64 ms with the bones and 49–54 ms without them, across runs on the same
container. Those ranges are within the probe's noise on this 2-core
software-GPU machine, which also varied the frame change between 77 and
102 ms with or without bones.

## Opening a file: the DNA lookups (batch 117)

Female-01 opened in 23.8 s on the user's machine and 28.1 s in the
container (release build, two cores). The open's main thread, sampled 251
times with gdb: **65 % of the samples were inside
`BlendFile::struct_field_offset_exact`** — every read of a DNA member
walked the type list (hundreds of string compares) to find the struct,
then the struct's members, calling `field_size` on each, which walked the
type list again. Called per bone per shape, per F-curve per key. Blender
works this out once at load: `DNA_sdna_from_data` fills `types_size`, and
`DNA_struct_member_offset_by_name` is answered from the SDNA.

`phiphx/blend/file/sdna.rs` does the same: `DnaType::layout(ptr_size)`
builds, once per file, a map of type name → index and, for every struct,
member name (full `idname[64]` and bare `idname`) → (offset, size), with
the members in order for the loose `contains` lookup; `struct_field_offset
_exact`, `struct_field_offset`, `field_size`, `struct_size`, `int_field`,
`uint_field` and `nested_int` read from it. The second cost was
`skin_sources` applying each clothed mesh's subdivision surface one after
another on the main thread (`rigs` → `skin_sources` → `subsurf::apply`,
54 % of the samples after the first fix); it now applies them in parallel
as `evaluate_modifiers` already did.

| Female-01, release, two cores | before | after |
|---|---|---|
| Evaluating modifiers | 6.8 s | 3.0 s |
| Reading actions | 5.7 s | 1.2 s |
| Reading motion | 9.5 s | 2.7 s |
| Building scene | 1.7 s | 1.7 s |
| **Total** | **28.1 s** | **10.0 s** |
| THYMUS / FLORIS, total | — | 0.31 s / 0.25 s |

What remains is the work itself: subdividing a dressed figure's cloth
(parallel, so a six-core machine does better than this table), reading
300 k F-curves of one-frame poses, and building 2.9 M triangles for the
scene.

## Female-01's journal (batch 114, read 2026-10-03)

A 1.9-hour release-build session on the GTX 1660 Ti with batch 112's code,
Female-01 opened at 77 minutes:

| What | Measured |
|---|---|
| Kernels compiled on first use (`Tracer::warm`) | Solid 0, Preview 3.1 s, Wire 0.6 s, **Trace 25.6 s** |
| Opening Female-01 (319 objects, 4.5 M vertices) | **23.8 s**: indexing and objects 1 s, evaluating modifiers 6.8 s, reading actions 5.2 s, reading motion 8.9 s, drivers, building the scene 1.6 s |
| Tracer scene built (2.89 M triangles, 2 MB of images) | triangles 1.36 s, tree and upload 0.38 s; first image 2.2 s after the file opened |
| A pose (vertices moved) / the trees rebuilt | 976 ms / 277 ms |
| UI thread, 3840 frames | 1.8–3.2 ms a frame, worst 18–32 ms, **0 slow frames** |
| First image after a view change (180 views) | median 49 ms, p90 260 ms, max 2.2 s |
| A Solid or Wireframe sample at ~1080 × 670, Female-01 | **44–64 ms** of GPU each (the tally's "readback" column waits for the dispatch: the BVH walk over 2.9 M triangles, 15–20 frames a second) |
| A Material Preview sample at 1325 × 668 | 13 ms |
| A Rendered sample at 1566 × 767, the default cube | 9–26 ms |

Two things follow. The viewport on this file is bound by the Solid frame's
primary rays — 2.9 M triangles through the tree — which is exactly what
batch 114's rasterized prepass replaces on hardware (this session predates
it; the next journal will show the rasterized figure). And opening is
bound by three reader stages — modifiers, actions, motion — 21 of the 24
seconds: batch 115's target. The UI thread itself is never the cost.

## Solid anti-aliased: eight samples where there was one (batch 118)

Workbench's TAA (`rendering.md` §25): a Solid or Wireframe view is now
eight dispatches of the shading kernel, each followed by the 3 × 3
gather (`Taa`, a pass over the image's pixels, cheap next to the shading),
where it was one. The first sample is unjittered — the frame of before,
shown at once — and the other seven land while the view rests, so the
cost is time to a settled edge, not time to a picture: on hardware's
raster path a sample was 0–18 ms (the 1660 Ti journal, batch 111), so the
eighth should land within a few frames; the next journal's `solid` lines
give the figure. A navigation starts the count over, as Blender's does.

## Solid's surfaces rasterized (batch 114)

| THYMUS, 436 k faces, 1200 × 700, lavapipe (software GPU) | traced | rasterized prepass |
|---|---|---|
| Solid, one full frame | 83 ms | 378 ms |
| Wireframe | 82 ms | 377 ms |

On a software adapter the BVH walk wins — 1.3 million vertices pulled from
storage through a CPU rasterizer cost more than 840 k rays — so there the
prepass stays off; it is on for hardware, where that many triangles
rasterize in under a millisecond and the Solid frame is left with its
shading. The journal's `Tracer: scene built` line says which a session
took; the 1660 Ti's figure is the next journal's (its traced Solid frame
was 0–18 ms in batch 111's).

## The Animation workspace's frame (batch 113)

THYMUS-BOLDROID-WAVE-003 (259 objects, 328 bones) in the gpui test harness,
a debug build, the playhead stepped once a frame (`PROBE_SPACE=Animation`):

| Workspace | Still | Frame step | After |
|---|---|---|---|
| Animation | 23–39 ms | **64 ms** (worst 90) | 22 / **35** (worst 52) |
| Layout | 19 ms | 39 ms | 18 / 33 |
| Rendering | 13 ms | 13 ms | — |

The UI thread's stack, sampled 250 times with gdb during the frame steps:

| What | Samples | Fix |
|---|---|---|
| `BOutlinerItem::build_rows`, `tree::flatten`, `BHidden::by`: the Outliner's rows and hidden set built again every frame — the frame counter bumps `scene_revision` | 29 + 10 (≈ 16 %) | `BState::LIST_SKIPS`: the clock's key families leave the scene revision alone |
| `time::row::set::diamond`: two tessellated paths per key per frame | 13 innermost, 27 inclusive (≈ 11 %) | three paths a row (borders, fills, picked fills), off-window keys skipped |
| `BShown::now`, `scene_objects`, `BPoser::take`: the posed object list and the viewport's shown list | ≈ 15 % | the frame's work — Blender re-evaluates its depsgraph too |
| `BState::get` + `BPath::encode`: string-keyed state reads | ≈ 10 % | not this batch |
| gpui element ids (`memcpy` under `request_layout`) | ≈ 8 % | gpui's |

After the two fixes the Animation workspace steps a frame in the time the
Layout workspace does, and the second sampling shows no Animation-specific
cost left: what remains is the pose evaluation and the whole UI tree
rebuilt per frame, shared by every workspace.

## A move is a matrix (batch 112)

Before this batch, moving, turning or scaling one object rebuilt the
scene for the tracer: every vertex of every object transformed on the
CPU, every vertex uploaded again, every tree refitted (`repose`) and,
once still, built again (`retree`) — for Female-03, 70 ms + 18 ms a step,
for a 9-million-triangle Female-01 seconds. Now each object is an
instance with its own tree in its own space, and a move uploads its
matrix and the small top tree over the objects' bounds (`place.rs`,
`Tracer::reinstance`): the cost no longer grows with the scene.

The journal the user sent after batch 111 (a release build, GTX 1660 Ti,
the default cube, ten minutes): no frame over 50 ms, no panic, the first
image 1.3 s after start — and `Tracer: kernels compiled in 21355ms`. The
three kernels the warm-up thread compiles after the first image took 21 s
between them on that driver's first run (it caches the result, so the
next start is quick); the warm-up line now names each kernel's time so the
slow one is known. The compiles run on their own thread and never hold
the window, but a change of shading inside those first 20 s still waits
for its kernel.

## The viewport that only refreshed on input (batch 111)

The journal of a release build on a GTX 1660 Ti, Female-03 open (322
objects, 203 k triangles):

| What the tracer reported | Time |
|---|---|
| Scene built: triangles, then tree and upload | 174 ms + 26 ms |
| Scene posed; tree rebuilt over the pose | 70 ms; 18 ms |
| A full-size Solid frame at 1566 × 767 | 0–18 ms (the first 1.1 s: its shader compiling) |
| **First image after the file opened** | **10.9 s** |
| First image after Shading ▸ Solid / Wireframe / X-ray | 1.1 s / 2.7 s / 1.6 s |
| First image of the session (the default cube) | **4.2 hours** — the first click |

Nothing in the tracer accounts for the right-hand column; the window did.
The ray ticker — the task that redraws the window when the render thread
lands a frame — is armed in `render` from the flag the viewport's paint
sets when its image is still on its way. `render` asked after `drawing()`
had cleared that flag for the draw in hand, so the ticker never started
for a viewport (it started for F12, which sets a different flag), and
every frame the tracer landed waited for the next input to redraw the
window. An orbit looked fine — each mouse move was an input — and the
moment the hand stopped, so did the viewport: the "lagging in refreshing
the mesh", the first image "not responding", the refinement that never
seemed to finish.

`render` now asks first, and a paint that leaves its view refining with
no ticker running asks for one more frame itself. The shader compiles that
made a shading change wait a second now run on their own thread after the
first image, and a deterministic mode whose full frame is known to be
quick starts at full size instead of an eighth.

## Where the per-object cost went, and the sampler that found it (batch 106)

callgrind under the gpui test harness lies about a frame: the tracer's
thread is as slow as the UI thread there, so the CPU painter runs every
frame and takes 95% of the profile, while natively it never runs. A stack
sampler does not lie: the probe left running on a still frame, the UI
thread's stack read with `gdb -batch -p <tid> -ex "bt 60"` 250 times, the
`phiber::` frames counted. At 500 objects, 25 ms a debug frame, one sample
is 0.1 ms:

| What | Samples | Fix |
|---|---|---|
| `BHidden::now` and `for_select`: each kind's set built again every frame — the collection tree walked (`BSceneTree::now`), two keys encoded and read per object | 30 (3 ms) | the set kept across frames by `(BState::scene_revision, model)` — `part/outliner/key.rs` |
| The shown list: 500 objects filtered and cloned, 500 names collected for the arrow toggle | ~8 | `view3d/shown.rs`: both kept while the objects list and the hidden sets are the last frame's |
| `BRayKey::of`: SipHash over 500 names and matrices to see whether the tracer holds this scene | 8 (0.8 ms) | `view3d/identity.rs`: a kept list's identity hashed once per list; `BRayKey::of_list` reads it |
| `cache::fingerprint`: the same hash again for the mesh memo | 6 (0.6 ms) | reads `BSceneId` too |
| GPUI laying out the Outliner's rows in view (~34 at this height, against 5 for the default scene) | ~25 (2.5 ms) | the visible rows — legitimate, Blender draws them too |

| Debug build, 1600 × 900, harness ≈ 13 ms of it | 3 objects | 500 objects before | 500 objects after |
|---|---|---|---|
| Orbit step | 15–16 ms | 24–28 ms | 16–17 ms |
| Orbit, object selected | 13–17 ms | 24–26 ms | 17–18 ms |
| Zoom | 15–18 ms | 24–28 ms | 18–19 ms |

The still frame at 500 objects stays a few ms over the 3-object one in the
probe because the tracer is still delivering images for it (each one an
upload) — the tracer's work, not the UI thread's.

**A release build changes the question.** The user runs `cargo run
--release -p phiber`, where all of the above is a few milliseconds. What
feels slow there is the tracer or its device, and the two logs — the Info
editor's render-device line (`app/health.rs`) and `%TEMP%\phiber-frames.log`
— say which. Checked in the meantime: Solid and Wire trace in a single
dispatch (`phiray::Mode::deterministic`), not progressively; the window is
drawn once per landed frame, not every vsync while the tracer refines
(`rendered/wake.rs`).

## Measured again after batches 99–101, and two more causes (batch 105)

A debug build, 1600 × 900, one UI frame in the test harness (whose own cost
is ~13 ms — subtract it), four scenes:

| Scene | Still | Orbit step | Orbit, object selected | Zoom |
|---|---|---|---|---|
| 500 objects, 10 M triangles, Rendered | 27 ms | 27–32 ms | 30–33 ms | 29 ms |
| 500 objects, 10 M triangles, Solid | 27–37 ms | 29–36 ms | 30–33 ms | 29–31 ms |
| 500 objects, 4 000 triangles, Rendered | 28–30 ms | 27–33 ms | 30–33 ms | 30–34 ms |
| 3 objects, 60 000 triangles, Rendered | 13 ms | **52–56 ms** | **48–50 ms** | 14 → 50 ms |

Two things stand out, and both are fixed in batch 105:

| What | Fix |
|---|---|
| A *small* scene orbited four times slower than a huge one. The CPU painter stands in for a frame the tracer has not delivered, and its triangles were projected every frame of every scene under its 400 000-triangle ceiling in case the fallback was needed — every vertex, on the UI thread, on every mouse move, with the image already on screen | `view3d/mod.rs`: the fallback meshes are projected in the paint, the moment the tracer turns out to have no image, through the same memo (`fall_back`). 3-object orbit 52 → 13 ms |
| Five hundred Outliner rows were built from the state every frame (6.3 ms of a 9.5 ms tree), though batch 92 lays out only the rows in view | `BState` counts writes per key family (`revision_except`); the Outliner keeps its rows until a family it reads or the loaded model changes, and hands them back as `BView::Shared(Rc<BView>)`. 6.3 → 1.2 ms; the 500-object frame 28 → 22 ms |

Triangle count no longer matters to the UI thread (10 M against 4 000: the
same frame). What is left of a 500-object frame over a 3-object one is about
9 ms in debug, in the per-object passes that still run each frame
(`for_select` + `hides` 1.4 ms, the hit list, the tree layout) — the next
thing to take apart.

The Info log now says, as the window opens, whether this is a debug build,
where this frame log is, and which device phiray draws on — with a warning
when it is a software adapter (`app/health.rs`). A report of slowness
should start with those three lines.

## The headline: the CPU painter ran every frame, even while the GPU drew

Found by profiling a dev build under callgrind: **74% of every frame** went to
`BMesh::paint → fills → BFrame::downsample`. With the GPU drawing the
viewport there are no CPU triangles, yet `fills` still allocated a 2×
supersampled buffer the size of the whole viewport, averaged it down, uploaded
the result — a fully transparent image — as a new texture, and painted it over
the real render. On every frame. It also never released those textures, so
each frame left one more in GPUI's atlas.

A zone drag or a window resize asks for a frame per mouse move, so that is
where it showed. Nothing in the drag code itself was slow: an idle frame cost
the same as a drag frame.

`mesh/paint.rs` now skips the painter when there is nothing to fill, and
releases the previous image when it does paint. Guarded by
`central_reuse::the_cpu_painter_stays_idle_while_the_gpu_draws`, which fails
with the fix reverted.

Frame time for the Layout space, measured in the test harness:

| Build | Before | After |
|---|---|---|
| Dev (how `run-phiber.bat` launches it) | 403 ms | 40 ms |
| Dev, dependencies optimised (new profile) | — | **11 ms** |
| Release | 20 ms | 6–8 ms |

Scripting and Shading moved the same way (335 → 11 ms, 227 → 15 ms with the new
profile).

### The dev profile

Both launch scripts run `cargo run -p phiber` — a debug build. The workspace
`Cargo.toml` now optimises every dependency in dev (`"*"` for the registry,
plus the GPUI and render crates by name, since `"*"` does not reach workspace
members), while Phiber itself stays at opt-level 0.

Measured one-file rebuild of Phiber: **2.6 s at opt-level 0**, 11 s at 1, 13 s
at 3 — which is why Phiber stays at 0. The cost is one full rebuild of the
dependencies the first time; after that, edits rebuild as fast as before.
Stepping into GPUI in a debugger becomes less exact, since it is optimised.

---


Measured in this repo, not guessed. Numbers are release-build, on the cloud
container's CPU; a debug build is roughly 10× slower across the board, which
matters — see "First, check the build" below.

## Earlier fixes, in the order they were found

### What a frame actually does

`BPhiberApp::render` runs on **every** frame, and it does the whole job from
scratch each time: record the window width into state, build the view tree from
state, walk the tree into GPUI elements. There is no diffing and no caching
between frames — by design, and mostly fine, because:

| Step | Cost |
|---|---|
| Tree build, Layout space (223 nodes) | 0.14 ms |
| Tree build, Shading space (291 nodes) | 0.23 ms |
| One state write / read | 0.0001 ms |

So the tree is **not** the problem. A resize could rebuild it 60 times a second
and spend 9 ms.


### 1. The viewport re-projected the whole scene on every frame — now fixed

`BMesh::build` transforms every vertex by the object's world matrix, projects
it, and allocates a `BTriangle` per face. It ran on every render pass, so a
window drag re-projected the entire scene per frame, as did switching space,
opening a menu, or a progress tick.

It depends on exactly three things: the camera, the palette, and the scene.
A resize changes none of them. So the built meshes now sit behind a fingerprint
of those three (`view3d/cache.rs`), keyed by mesh `Arc` pointer and world
matrix rather than vertex content, so computing the key is far cheaper than the
work it avoids. Orbiting, zooming, changing shading or loading a file still pay
for one rebuild, as they should.

Guarded by `central_reuse::resizing_and_switching_space_do_not_rebuild_the_viewport`.

Caveat worth stating plainly: this path only runs when **no GPU adapter is
available**. With a working GPU, `draw::plan` sends the scene to phiray and the
CPU projection is skipped entirely — so if the machine has a GPU, this change
buys nothing and the cause is item 2 or 3.

### 2. The scene was deep-copied into the render scope every frame — now fixed

`set_loaded_scene(self.scene_cache…m.clone())` cloned the whole `SceneModel` —
every object's name, material list and collection tree — once per frame.

| Objects | Model clone | `scene_source` |
|---|---|---|
| 100 | 0.009 ms | 0.009 ms |
| 1 000 | 0.091 ms | 0.090 ms |
| 5 000 | 0.517 ms | 0.491 ms |

The scene now lives behind an `Arc` (`scene_cache`, `set_loaded_shared`), so the
frame pays a refcount bump. In a debug build at 5 000 objects that is ~5 ms a
frame recovered.

### 3. The object list was rebuilt per reader — now fixed

`scene_source` rebuilds `Vec<BObject3D>` — a `String` and a materials `Vec` per
object — and the viewport, the overlay, the outliner and the slow-frame trace
each called it independently within one pass. That is two to four rebuilds a
frame depending on what is on screen, at ~0.5 ms each for 5 000 objects in
release (~5 ms in debug), for a list that cannot change within a pass.

It depends on nothing but the model in scope — `current_scene` takes state only
for its signature — so `scene_objects` now builds it once per model and hands
out an `Rc` after that, keyed on the scoped `Arc` pointer. The entry holds a
strong reference to the model it was built from, which is what makes the
pointer comparison sound: that address cannot be reused by another scene while
the entry is alive. `scene_source` stays as it was for the call sites that want
an owned copy.

Guarded by `central_reuse::one_frame_reads_the_scene_once` and the memo's own
unit tests (`scene::source::memo_tests`), which also cover the case that
matters most: a newly loaded file must not be served the previous file's
objects.

### 4. A window drag restarted the trace on every frame — now fixed

A correction first: an earlier version of this doc said the render queue
"backs up". It does not — the worker drains every queued job and keeps only
the newest view, so it never works through a backlog. The real cost was
different.

`rendered::paint` keys the view on `(scene, width, height, mode, camera)`, so
every new size during a drag was a new view, and every new view restarts the
trace from its first sample and reads back whole frames (up to four, ~6 MB each
at 1600 px) before it refines. A drag produced a new size every frame, so the
tracer spent the entire drag restarting at sizes that were gone before they
finished — and since the view never settled, the UI polled flat out the whole
time.

`rendered/settle.rs` now sorts size changes by how they arrive:

- **On their own** (space switch, maximise, a panel toggling) they are taken at
  once — delaying one would only show a stretched picture for nothing.
- **In a burst** (another change within 150 ms, *or on the very next frame*)
  the tracer holds the size it has and the painter stretches that image over
  the new bounds. Once the size has held still for 150 ms **and** two frames,
  the final size is traced once.

The frame condition is not decoration: the first version used time alone and
its test failed, because in a debug build frames arrive ~500 ms apart and every
drag step looked like a lone change. That is exactly the build where this is
felt, so it now counts frames as well.

Measured by `central_reuse::a_window_drag_restarts_the_trace_once_not_every_frame`:
an eight-frame drag went from **8 restarts to 2** — one where it began, one
where it stopped. The camera is untouched by any of this: orbiting or zooming
still restarts at once.

**The trade you will see:** during a drag the rendered image is stretched to the
new window shape rather than re-framed, and snaps sharp ~150 ms after you let
go. Lights, cameras and empties are drawn separately at the live shape, so for
that moment they can sit slightly off their objects. That is the usual
compromise for a traced viewport.


## The log already knows

`app::trace` writes any frame slower than 50 ms to `phiber-frames.log` in
`%TEMP%`, with the split between tree build and draw, the scene size, and
whether the GPU or the CPU painter drew it. Run the app, drag the window,
switch space, then send that file — it will say which of the four items above
is yours.
