# Why resizing and changing space feel slow

Measured in this repo, not guessed. Numbers are release-build, on the cloud
container's CPU; a debug build is roughly 10× slower across the board, which
matters — see "First, check the build" below.

## What a frame actually does

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

## Where the time really goes

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

## First, check the build

If the app is being run with `cargo run` (no `--release`), every number above is
about 10× larger and no amount of caching will make it feel fast. Worth
confirming before chasing anything further.

## The log already knows

`app::trace` writes any frame slower than 50 ms to `phiber-frames.log` in
`%TEMP%`, with the split between tree build and draw, the scene size, and
whether the GPU or the CPU painter drew it. Run the app, drag the window,
switch space, then send that file — it will say which of the four items above
is yours.
