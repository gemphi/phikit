# Held to Blender's own output — the truth tests, and what they found

Batches 180–182. Three parts of phiber that *claimed* Blender parity in a
doc comment were held to Blender's actual output instead. Each has a
generator beside its truth file (`crates/phiber/tests/truth/`), so the
comparison can be regenerated against a new Blender rather than re-read by
hand, and each found something.

The pattern, for the next one of these: generate from Blender, compare by
kind rather than by pixel, and where phiber deliberately differs make the
difference a **row in a table the test checks** rather than a silence. A
divergence nobody wrote down is indistinguishable from a bug.

| What | Generator | Truth file | Test |
|---|---|---|---|
| Channel names | `blender_channel_name.py` | `channel_name.txt` (687 rows) | `channel_name_truth.rs` |
| Topbar menus | `blender_menus.py` | `menus.txt` (71 rows) | `menu_truth.rs` |
| Workspace regions | `blender_workspaces.py` | `workspaces.txt` | `workspace_regions_truth.rs` |
---

## Batches 180–182 — what Blender's own output says, and the gaps it found

Three things were held to Blender rather than to themselves. Each has a
generator beside its truth file, so the comparison can be regenerated
against a new Blender rather than re-read by hand.

### 1. Channel names (animation) — `tests/truth/blender_channel_name.py`

A channel row's label is not stored in a `.blend`. `getname_anim_fcurve`
(`editors/animation/anim_ipo_utils.cc:48`) builds it, every draw, from the
array item's letter, `RNA_property_ui_name(prop)`, and the struct that owns
it. The middle piece is a **declared** string, not a transformation of the
identifier: `energy` displays as "Power", `spot_size` as "Beam Angle",
`mid_level` as "Midlevel", `merge_threshold` as "Merge Distance".

`row/name.rs` carried an eighteen-entry hand table and title-cased
everything else, so most of the channel list read in names Blender never
shows — a light's power read "Energy", a Displace modifier's midlevel read
"Mid Level". The tables are generated now: **389 identifiers** across
objects, pose bones, lights, materials, worlds, every constraint type,
shape keys, `ConstraintTargetBone` and the modifiers phiber evaluates,
split across `row/name/{set,data,rig,modifier}.rs` and held to every row of
`truth/channel_name.txt` by `tests/channel_name_truth.rs` (687 rows, all
passing).

Two things the first generated pass got wrong and the truth test caught:

- **The letters are a four-letter string even for a three-component
  array.** `RNA_property_array_item_char` (`rna_access.cc:1370`) only
  bounds the index at 4, so index 3 of an XYZ vector answers 'W'.
  `AXISANGLE` sits in the same `ELEM` as `QUATERNION`, so
  `rotation_axis_angle` runs `WXYZ` — index 0 is **W**.
- **Walking objects and datablocks alone loses the rig.** `influence`,
  `mute`, `value` and `weight` — the four most commonly driven properties
  in any rig — belong to constraints and shape keys, not to the object.
  The generator walks every constraint type for that reason.

Where an identifier's name depends on which struct carries it, there are
now two records rather than a silence. `OWNED` holds the three the path
answers (`matrix_basis` is "Input Matrix" on an object and "Basis Matrix"
on a pose bone; `mute` is "Disable" on a constraint and "Mute" on a shape
key; `weight` is "Weight" on an IK constraint and "Blend Weight" on an
Armature target). `AMBIGUOUS` holds the four the path **cannot** answer,
because a path carries a modifier's and a constraint's *name* but never its
*type*: `invert_vertex_group` ("Vertex Group Invert" for Solidify),
`use_curve_radius` ("Curve Radius" for Follow Path), `use_rotation`
("Rotation" for IK) and `offset` (a vector with an axis letter for Pivot).
Closing those four would mean resolving the path against the loaded model
the way Blender does, which `fcurve_name` deliberately does not — it is
handed a curve and nothing else, which is what makes it testable without a
file.

### 2. The topbar menus — `tests/truth/blender_menus.py`

A menu is not data in Blender either: `TOPBAR_MT_render` is a Python class
whose `draw(self, context)` calls `layout.operator(...)` in order. The dump
runs those draws against a recording layout — the only way to get the real
order with the real labels — and reads the shortcuts out of
`presets/keyconfig/keymap_data/blender_default.py`, matching a binding to an
entry by operator **and by properties**, because "Render Image" and "Render
Animation" are the same operator (`render.render`) and differ only in
`animation = True`. `WM_key_event_operator_string` matches strictly for the
same reason (`is_strict = true`, `wm_keymap.cc:1676`).

`topbar/set.rs` carried a `Blender ref:` comment per menu, which no test
checked. `tests/menu_truth.rs` checks it, and found the shortcut
*spelling*:

- `WM_keymap_item_raw_to_string` appends **Shift, Ctrl, Alt, OS** in that
  order (`wm_keymap.cc:1222`), so Blender writes "Shift Ctrl S" — never
  "Ctrl Shift S". phiber already had this right.
- The key's own text is the event enum's `name` under **`compact = false`**
  (`interface.cc:1678`), not the short description. So Blender writes
  "Ctrl ," and "Ctrl Page Down". phiber wrote "Ctrl Comma" and "Ctrl
  PageDown", in the topbar and in the animation editors' Channel ▸ Move.

Those labels are relabelled, and `BKeystroke::parse` was rewritten to match
`WM_keymap_item_raw_to_string`'s shape — the modifiers are a **prefix** and
everything after them is **one** key name. It used to read word by word,
which turned "Page Down" into the Down arrow; it accepts both spellings
now, because a parser has no reason to be strict about an input it can
understand, while the labels are held to Blender's.

phiber's deliberate differences are rows in the test rather than silences:
`EXTRA` (Render ▸ Render Turnaround, which Blender has no menu entry for;
Edit ▸ Operator Search…, which Blender dropped in 2.9x) and `RENAMED` (the
three entries that open the file browser and so take an ellipsis).

### 3. The project import moved to its own crate — `crates/phipro`

`phiphx/src/import/project.rs` was dead: gated behind an `appkit` feature
that was never switched on, and the gate hid a `use
crate::appkit::project::scaffold` whose module does not exist, so it could
not have compiled if the feature were turned on. A dead module inside a live
crate is invisible; a crate of its own is built and tested like everything
else.

`phipro` writes a project bundle from any asset `phiphx` can read:
`bundle::BBundle` owns the layout (nothing else joins a path or chooses a
file name), `bundle::BProjectName` is the naming rule, `manifest::BManifest`
is `project.json`, `scene::BScene` is the entity graph, and
`import::BProjectWrite` is the one call. It depends on `phiphx` with
`default-features = false`, so a crate that writes files pulls in no
toolkit and tests anywhere. 37 tests.

What the port fixed on the way, each with a test naming it:

- **The JSON was built by interpolation.** Both earlier versions were one
  quotation mark from writing a file nothing could read — an object named
  `He said "hi"` closed the `"name"` string early. One half knew it and did
  `obj_name.replace('"', "'")`, which silently renames the user's objects
  and still leaves `\` and the control characters. A serialiser escapes all
  of it and renames nothing.
- **The name was `file_stem().unwrap_or_default()`.** A stem of `.` or `..`
  put the bundle above its base; a stem with `:` or `?` in it could not be
  created at all and failed two calls later as an I/O error. The name is a
  type now, refusing with the character it objected to. (`file_stem` of
  `/a/..` is `None`, not `".."` — the reachable paths are `/a/...`,
  `/a/..x` and `/a/. `, which is what the test uses.)
- **The root's child list and the children's parents were built by two
  loops** that could disagree, so an off-by-one made the scene refer to an
  entity that was not there. One walk now builds both.
- **The order parsed first and named second**, so a mistyped path cost a
  full `.blend` parse before failing on the name. The bundle is decided
  first.
- **The progress bar stopped at 0.80** and the caller had to know to finish
  it. `BImportStep` declares the fractions in one table that reaches 1.0.
- **`phiphx`'s asset reader reports progress that goes backwards** — the
  `.blend` reader restarts its count per data block, so a run reports 0.125
  and then 0.0875. Forwarded straight through that is a bar the user
  watches slide backwards, so `BProgress` never reports a fraction already
  passed. The note still arrives for every report, so the line beside the
  bar keeps moving while the fraction holds.

### Found and **not** fixed: a render larger than the GPU binds still panics

`MATERIALS-BSDF.blend` through `examples/camera_render` dies with

    wgpu error: Validation Error
    Buffer binding 14 range 215654400 exceeds `max_*_buffer_binding_size` limit 134217728

Binding 14 is `px`, the accumulation image (`trace/wgsl/binds4.wgsl:15`), at
`Tracer::PX_WORDS = 26` words a pixel. At 128 MiB a binding that is about
1.29 million pixels — **less than 1080p**, so on a device with the default
limit phiray cannot allocate a 1920×1080 image at all.

The guard already exists and its doc comment describes this exact failure:

> `Tracer::max_pixels` — "The most pixels one image may hold on `gpu` … past
> which creating the pass is a validation error that ends the render
> thread." (`phiray/src/trace/mod.rs:530`)

**Nothing calls it.** `grep max_pixels` finds the definition and no use. The
banding batch 100 added (`Camera.tile`) covers the F12 path; this is the
`still::still` path (`rendered/still.rs:58`), which allocates the whole
image in one binding. Wiring the existing guard there — band like F12, or
refuse with a message naming the limit — is the fix. Left for whoever owns
phiray's render path rather than taken here, since `still.rs`,
`rendered/mod.rs`, `rendered/region.rs` and `rendered/lamp.rs` are all under
active edit.

On a real GPU `adapter.limits().max_storage_buffer_binding_size` is
typically 2 GiB, so this may not reproduce on the machine — but a software
adapter, a headless CI runner and a render at 4K all hit it.
