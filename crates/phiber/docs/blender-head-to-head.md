# Opening a `.blend`: phiphx against Blender, measured

`frame-cost.md` has carried phiber's open times since batch 117 and never a
Blender column, so the project's whole claim — the same scene, read faster —
had never been put beside the thing it claims to beat. This is that column,
and the batch that came out of making it.

Both sides are repeatable: `crates/phiphx/tests/truth/blender_open_time.py`
produces Blender's figures, `crates/phiphx/examples/bench_open` produces
phiphx's, and `crates/phiphx/tests/truth/open_time.txt` is the dump the two
are compared through.

## Asking both sides for the same work

They are not the same by default, which is the first thing to get right:

* **Blender's read** is `wm.open_mainfile` — the file parsed, its datablocks
  built, and nothing evaluated. No modifier has run; `object.data` is still
  the *pre*-modifier mesh.
* **Blender's evaluate** is the depsgraph updated and every mesh object's
  evaluated geometry realised (`evaluated_get(dg).to_mesh()`), which is what
  runs the modifier stack. Blender does this lazily, per object, when
  something asks — so the script asks, for every object, because that is the
  state `Phx::open` has already reached when it returns.
* **phiphx** does both before it returns, so its single figure belongs
  beside Blender's **total**.

phiphx is also doing *more* than the script asks Blender for on the small
files — actions, NLA, drivers, constraints, rigs, materials, UV maps — none
of which a `to_mesh()` loop touches. That is worth saying because it is why
the single-digit files are a wash rather than a win.

**Runs matter on both sides.** The first open in a fresh process pays for
allocator arenas, thread pools and first-touch page faults. Blender pays it
on its first file and not on the rest; a one-shot Rust process pays it every
time (`LIGHTS.blend` measured 9.4 ms cold against 5.0 ms warm — most of the
number). Both tools repeat and report the **min**.

## The numbers

Container, release build, 2-core Intel Xeon @ 2.10GHz, Blender 5.0.1 (`bpy`).
Blender min-of-3, phiphx min-of-7.

| fixture | objects | vertices | Blender | b117 | b183 | **b184** | b184 ÷ Blender |
|---|---|---|---|---|---|---|---|
| THYMUS-BOLDROID-WAVE-003 | 273 | 223,988 | **621.8 ms** | 253.0 | 214.8 | **173.0 ms** | **3.6× faster** |
| FLORIS_BOLDRIOD_RIG | 331 | 85,948 | **247.9 ms** | 212.7 | 125.1 | **107.9 ms** | **2.3× faster** |
| nla_cube | 2 | 11 | 4.7 ms | 5.2 | 5.0 | 4.4 ms | a wash |
| MATERIALS-BSDF | 7 | 984 | 6.2 ms | 6.7 | 7.3 | 5.7 ms | a wash |
| PROCEDURAL | 15 | 60 | 8.5 ms | 8.7 | 9.3 | 7.4 ms | a wash |
| constraints | 2 | 0 | 6.0 ms | 6.2 | 6.5 | 5.2 ms | a wash |
| LIGHTS | 9 | 0 | 5.3 ms | 5.5 | 5.4 | 4.5 ms | a wash |

**Read the small rows as noise, not as five wins.** None of those files has an
Armature modifier, so batch 184 cannot have touched them; they moved because
the container was quieter during that run than during the one before. That is
also why the two rig figures in §184 below are quoted from an A/B inside one
binary rather than from the difference between two columns here.

The five small fixtures open in single-digit milliseconds and vary by about
a millisecond run to run on this machine, so the ±0.3 ms between the columns
there is noise and is not claimed either way. The two rig files are where a
`.blend` reader is actually exercised, and that is where the margin is.

## What the measurement found: one armature read forty times

Timing the open by stage on FLORIS put **113–139 ms of a 213 ms open inside
`rigs`**, and splitting that put it in `skin_sources` — whose own split put
**56 ms in `modifier_stack`**, called once per mesh object. Timing the
per-modifier reads inside it named the culprit exactly: **84 reads of
modifier type 8 — Armature — totalling 144 ms.**

`armature::read` called `pose_deform_mats`, which walks *every pose channel
and every bone* of the armature the modifier points at. A dressed figure has
forty-odd meshes on one rig, so the same few hundred bones were read forty
times — and the modifier stack is built **twice** over an open, once by
`modifier_jobs` for the evaluated meshes and once by `skin_sources` for the
ones the app re-poses, so forty became eighty.

Blender does not do this. `BKE_modifier_deform_verts` for an Armature
modifier reads `md->object->pose`, which the depsgraph evaluated once; the
modifier only points at it.

`modifier/deform.rs` is that: the read keyed by the armature's own block
address, handed out as an `Arc`, so forty modifiers share one. The memo is
sound because the read is a pure function of its armature and the DNA —
both halves of it, `pose_shapes` and the channel walk, open with
`self.scoped(arm, …)`, so neither depends on which object's scope happened
to be ambient; and the DNA does not change for the life of a `BlendFile`.
The lock is taken to look and taken again to store, never held across the
read.

| | before b183 | after b183 |
|---|---|---|
| FLORIS_BOLDRIOD_RIG | 212.7 ms | **125.1 ms** (−41%) |
| THYMUS-BOLDROID-WAVE-003 | 253.0 ms | **214.8 ms** (−15%) |

Vertex and object totals are unchanged, which `tests/open_parity.rs` is what
checks — a faster reader that read less is not faster.

## Batch 184: the stack was also built twice

The memo above removed the duplicated armature *read*. The duplicated
**apply** was the other half, and it is now gone too (`modifier/prefix.rs`).

Two things build the same mesh over an open, and neither knew about the other:

* `modifier_jobs` captures every mesh object's **whole** stack and
  `apply_eval_jobs` applies it — the evaluated mesh the viewport draws at the
  saved pose.
* `skin_sources` splits the stack at the Armature and applies the part
  **before** it, keeping that as `base` so the rig can pose the mesh at any
  frame.

Everything before the Armature was therefore evaluated twice, from the same
vertices, with the same modifiers, under the same matrices — and on a dressed
figure that prefix is a Subdivision Surface per garment. Probing FLORIS: **28
of 32 armature-deformed meshes had a non-empty prefix, 39 modifiers in all,
and `skin_sources` spent 20.7 ms re-applying what `apply_eval_jobs` had
already applied inside its own 39.8 ms.**

Blender has no equivalent step because it does not have the two consumers: the
depsgraph evaluates each object's stack once, and the pose comes from the
armature it has already evaluated.

### Why a drop box and not a cache

The obvious fix — apply the prefix first, let both read it — needs to know
which one runs first, and that differs by how the file is opened. An eager
open applies the stack during the read, *before* the rigs are read. A
progressive open defers the stack to a worker thread, so the rigs are read
first. Picking one order would have meant reordering `scene/read.rs` and
changing the sequence of stages the progress bar reports.

So neither is the producer. Each prefix is **put by whichever consumer gets
there first and taken by the other**, which keeps the ordering out of it
entirely and left `scene/read.rs` untouched. The taker *removes* the entry, so
a subdivided garment is not held once its second reader has it, and nothing
has to sweep up afterwards.

The key is `(object, modifiers applied, weights loaded)`. That last field is
there because the two consumers decide separately whether to load the mesh's
vertex groups — `skin_sources` always does, `modifier_jobs` only when some
modifier in the stack asks — and a mesh whose groups were never loaded cannot
be skinned by them. Today an Armature always asks, so it is always true on
both sides; it is in the key so that a modifier which stops asking makes the
keys *miss* rather than hand the rig a mesh with no weights in it. The
modifier count does the same job for `PHIPHX_MODIFIERS`, which truncates one
consumer's stack and not the other's.

### What it bought

| | prefix applied twice | applied once |
|---|---|---|
| THYMUS-BOLDROID-WAVE-003 | 222.0 ms | **173.0 ms — 22% faster** |
| FLORIS_BOLDRIOD_RIG | 121.5 ms | **107.9 ms — 11% faster** |

One binary with `PHIPHX_NOSHARE=1` flipped, three interleaved rounds of five
runs each, best of each taken. FLORIS's unshared runs scattered from 121 to
150 ms where THYMUS's sat at 222–223, so 11% is a conservative floor on that
file and 22% is the figure to trust.

### Proving it changed nothing

A faster read that reads *differently* is not faster, and matching vertex
counts would not have caught a mistake here: the same count with a prefix
applied twice in a different order would pass that check. So
`examples/dump_eval.rs` hashes the evaluated positions themselves, and the two
paths are **byte-identical over 310,559 vertices in 613 objects**, across
every fixture with geometry in it. `tests/progressive_open.rs` — which asserts
a progressive open equals an eager one — and `tests/skinner.rs` both stay
green, and those two exercise the hand-off in opposite directions.

```
cargo run --release -p phiphx --example dump_eval -- f.blend > on.txt
PHIPHX_NOSHARE=1 cargo run --release -p phiphx --example dump_eval -- f.blend > off.txt
diff on.txt off.txt     # must be empty
```

## Still on the table

**`modifier/mod.rs` is 429 lines** against the project's 240-line cap. It was
378 before this batch and the prefix hand-off added the rest. The split is the
reading side (`KINDS`, `modifier_stack`) away from the evaluating side
(`modifier_jobs`, `skin_sources`, `apply_eval_jobs`) — but it is the file
another agent is active in, so it wants coordinating rather than doing.

**A driven modifier setting still does not reach the mesh.** Batch 174 read
the values and measured them (`blend/modifier/jobs.md`); they need threading
through that same `mod.rs`, and a mesh is currently evaluated once at open
rather than per frame, so a driven modifier would not animate even once the
values arrive.

## Two parity gaps the comparison also turned up

Measuring vertex totals per object on both sides found phiphx reading **192
fewer vertices in both rig files, on the same six objects**, and 48 fewer in
THYMUS. Both are recorded exactly in `tests/open_parity.rs`'s `SHORT`, so
closing one makes that test fail and the row gets removed.

* **A Subdivision Surface does not subdivide loose edges.** The six objects
  are Rigify widgets — `WGT-rig_palm.L/R` at 16 vertices against Blender's
  64, `WGT-rig_hand_ik.L/R` and `WGT-rig_foot_ik.L/R` at 8 against 32. They
  are edge loops with **no faces at all**, and `subsurf.rs`'s `refine`
  builds its topology from faces, so a wire-only mesh passes straight
  through. What Blender does was checked against its own output for
  `WGT-rig_hand_ik.L`, to six decimal places: with Use Limit Surface on, a
  valence-two vertex goes to `(prev + 4·v + next) / 6`, and the new points
  are the uniform cubic B-spline through the chain, which at a quarter of a
  segment is `[(1−t)³·P₀ + (3t³−6t²+4)·P₁ + (−3t³+3t²+3t+1)·P₂ + t³·P₃] / 6`.
  Every vertex in these widgets is valence two, so chains and loops are the
  whole case for them; valence-one ends and junctions would need deciding
  too.
* **A curve object is not tessellated into geometry.** THYMUS's `Path`:
  Blender evaluates it to 48 vertices, phiphx reports zero.

A third difference is not a gap: phiphx does not list Empties as objects,
which is the whole of the 273-against-272 and 2-against-1 object counts.

## How to run it again

```
# Blender's column (writes truth/open_time.txt)
blender --background --factory-startup \
    --python crates/phiphx/tests/truth/blender_open_time.py \
    -- -n 3 crates/phiphx/tests/fixtures/*.blend

# phiphx's column
cargo run --release -p phiphx --example bench_open -- \
    -n 7 crates/phiphx/tests/fixtures/*.blend
```

`examples/dump_eval.rs` is the third one: it digests the evaluated geometry so
two reads can be proved to agree (see §184).

The first two print the same tab-separated shape. Numbers from a different machine
are not comparable with the table above — re-run both sides together.
