# Drivers, against Blender

What a `.blend`'s drivers come to when phiphx evaluates them, measured
against Blender's own evaluated depsgraph — not against a reading of the
source. Two fixtures carry the measurement:

| fixture | what it pins | test |
| --- | --- | --- |
| `OBJDRIVERS.blend` | the driver **types**: Average, Python (`evaluate_driver`), Sum, and the driver's own F-curve over its value | `tests/object_driver_truth.rs` |
| `DRIVERVARS.blend` | the driver **variables**: every `dvar_eval_*` Blender has but Context Property, all three target spaces, and the custom-property chain | `tests/driver_var_truth.rs` |

The reference in both is the **evaluated** depsgraph copy's `matrix_world`,
which is where a driver's result lands. `driver_var.txt` records the sixteen
matrix elements rather than a decomposition, because a Min driver can push a
scale negative and no decomposition of a mirrored matrix is unique.

Worst element over the whole `DRIVERVARS` sheet, five frames × eight
objects: **9.5 × 10⁻⁷**. The test's tolerance is 1 × 10⁻⁵, so there is no
slack in it.

## What is read

### Driven properties

| path | state | where |
| --- | --- | --- |
| `location`, `rotation_euler`, `rotation_quaternion`, `rotation_axis_angle`, `scale` | **evaluated** | `anim/driver/read.rs`, `own_channel` |
| `["custom_property"]` on the object | **evaluated**, and before the channel drivers, so a driver reading it resolves in the same frame | `anim/driver/pass.rs` |
| `pose.bones[…]` — influence, mute, Armature target weight, a channel, a bone custom property | **evaluated** by the rig solver | `rig/driver/path.rs` |
| a material's `diffuse_color`, `metallic`, `roughness`, a node socket's `default_value` | **listed only** (`anim::drivers`, the Drivers editor) | — |
| a modifier setting (`modifiers["Array"].count`, …) | **listed only** | — |
| a shape key's `value` (`key_blocks[…]`) | **listed only**; phiphx has no shape keys yet | — |
| `hide_viewport`, `hide_render`, `color` | **listed only** | — |

### Driver variables

Blender's `dvar_types` table has five entries. Four are evaluated:

| `eDriverVar_Types` | state | notes |
| --- | --- | --- |
| `DVAR_TYPE_SINGLE_PROP` | **evaluated** for a transform channel and for an object custom property (`["name"]`) | a path under a bone falls to the rig solver; any other path reads 0 |
| `DVAR_TYPE_TRANSFORM_CHAN` | **evaluated**, all eleven channels and all four rotation modes | `anim/driver/chan.rs` |
| `DVAR_TYPE_LOC_DIFF` | **evaluated**, both ends in any of the three spaces | `len_v3v3(loc1, loc2)` |
| `DVAR_TYPE_ROT_DIFF` | **evaluated** | world space only, as Blender's own comment says it is |
| `DVAR_TYPE_CONTEXT_PROP` | **reads 0** | it names a setting of the active scene or view layer, which a file alone does not fix |

A variable this evaluator cannot resolve reads **0**, the way a driver whose
scripts do not run does. A driver nobody can evaluate is better still than a
driver evaluated from a guess.

### Transform channels

`dvar_eval_transChan`'s tail, ported in `anim/driver/chan.rs`:

| `eDriverTarget_TransformChannels` | what it reads |
| --- | --- |
| `LOCX`/`Y`/`Z` | `mat[3][chan]` |
| `SCALEX`/`Y`/`Z` | `len_v3(mat[chan - SCALEX])`, inline `mat4_to_size` |
| `SCALE_AVG` | `cbrtf(mat4_to_volume_scale(mat))` — the geometric mean of the three scales unless the matrix shears |
| `ROTX`/`Y`/`Z`/`W` | `BKE_driver_target_matrix_to_rot_channels` with `angles = false` |

and the rotation modes under it: Auto (the target's own Euler order, with
`compatible_eul` against its saved Euler when it is on one), the six
explicit Euler orders, Quaternion, and Swing/Twist X/Y/Z — the
"Damped Track + child" decomposition, `quat_split_swing_and_twist` with the
swing left as pseudo-angles and the twist in its own slot.

### Target spaces

The Driver panel offers three (`prop_local_space_items`), and they are three
different answers, not two of one thing and a special case:

| panel | flags | what it reads |
| --- | --- | --- |
| **World Space** | — | `object_to_world`: the local matrix composed up the parent chain |
| **Transform Space** | `LOCALSPACE` | `BKE_object_to_mat4`: the object's own channels, nothing above them. For a Loc Diff end, `ob->loc` itself, not the matrix |
| **Local Space** | `LOCALSPACE \| LOCAL_CONSTS` | `BKE_constraint_mat_convertspace(WORLD → LOCAL)` |

The third is worth spelling out, because guessing it wrong is silent. For a
**parented** object Blender subtracts `parent.world · parentinv`. For a
**parent-less** one it subtracts the object's own *rotation* alone —
`BKE_object_to_mat4`, normalised, translation zeroed, inverted — so the
matrix comes out in the object's own rotated axes and the result is neither
the world matrix nor the local one. `constraint.cc` owns up to this in a
comment ("this is actually an ugly hack"), and the Driver panel means it
regardless. `DRIVERVARS.blend` drives one channel from a parented child's X
in each of the three, so the fixture fails if any of them is confused for
another.

## Order

Blender's depsgraph runs the animation system for a property and then the
driver on it, so a driver wins over a key on the same channel. phiphx's pass
runs on the `raw` the actions and NLA have already written
(`BlendMotionList::raw_at`), which is that order.

Within the frame the pass is two steps:

1. **custom-property drivers**, read against the properties as saved;
2. **channel drivers**, read against the properties as step 1 left them.

That is the one driver-against-driver ordering a file leans on in practice —
a property on the rig object, read by a driver on something else — and
`DRIVERVARS.blend`'s `SOURCE → PROPHOST["gain"] → READER` chain measures it:
a pass that ran the channel drivers first would leave `READER` on the
property's saved value at every frame.

**Stated rather than hidden:** a *channel* driver chained off another
channel driver's output reads the keyed value underneath it, because within
step 2 every value comes from one snapshot. Blender's depsgraph orders
driver against driver generally; this pass does not. A driver reading a
keyed or still object — which is almost all of them — is exact.

Local Space is also meant to include what the target's constraints add. The
pass runs before constraints are solved (`BlendMotionList::solve` applies
them after the drivers), so a Local Space target *with* constraints is read
without them.

## Material drivers (batch 172)

A material's drivers live on **two** IDs, and Blender's UI hides the split.
A driver on the material's own property is on `Material.adt`:

    roughness                 metallic
    specular_intensity        diffuse_color      (component in array_index)

A driver on a shader socket is on `Material.nodetree->adt`, pathed by the
node's **name** and the socket's **index** in that node's input list:

    nodes["Principled BSDF"].inputs[2].default_value      (array_index = component)

The index counts through `bNode.inputs` as the file stores it — every
socket, `Normal` and `Weight` and `Tangent` included — so on a Blender 4.x
Principled, 0 is Base Color, 1 Metallic, 2 Roughness, 3 IOR, 28 Emission
Strength. It is not a count of the sockets the panel shows.

### The three clamps

What a driver writes is clamped on the way in, and the three write paths
disagree. This was measured against Blender's own evaluated materials
rather than read off the RNA ranges, because the ranges do not predict it:

| written through | below its range | above its range |
| --- | --- | --- |
| a float or vector socket | **kept** | **kept** |
| an RGBA socket's component | clamped to 0 | kept |
| a `Material` float property | clamped to its hard min | clamped to its hard max |

So the Principled's `Roughness` socket declares 0..1 and accepts −1 from a
driver without complaint, while `Material.roughness` declares the same and
clamps. The reason is the setter: `rna_NodeSocketStandard_value_set` writes
a float socket straight through, where a plain float property goes through
`RNA_property_float_set`'s clamp. An RGBA socket's setter clamps negatives
and nothing else.

### What reaches a pixel

A driver on a **linked** socket still writes `default_value` — Blender does
— but the link feeds the shader, so the write changes no shading. The fold
into `BlendMaterial::bsdf` therefore skips a linked socket and any node
that is not the one the Material Output reaches. A muted driver writes
nothing at all and the saved value stands.

The material-level properties land where the reader puts them:
`Material.roughness`/`metallic` in `viewport_pbr` and `r/g/b/a` in
`viewport_color` — what Solid mode shades with, not the Principled's own
values. `specular_intensity` is read, clamped and listed, and has no field
on `BlendMaterial` to land in; that is stated here rather than silently
dropped.

### Ordering

The pass runs over `BDriverState` — the objects' own driver pass, kept
instead of discarded. That gives a material driver the same scene the
object drivers read, *after* them, so the chain Blender's depsgraph orders
— a custom property on an object, driven, read by a socket's driver —
resolves in one frame. `BlendMotionList::raw_at` now routes through the
same state, so there is one pass and not two.

### Measured

`tests/mat_driver_truth.rs` against `MATDRIVERS.blend`: 195 values over 5
frames, worst off by **5.4 × 10⁻⁷**, tolerance 1e-5. The fixture carries a
driver of every storage and clamp kind, a linked socket, a muted driver, a
driver whose own F-curve has keys (so the driver's value is the curve's X,
not the frame), a Sum over two Transform Channel variables, a second
material slot, and the custom-property chain.

## Light and world drivers (batch 173)

A light keeps every driver on the `Light` ID; it has no node tree, so there
is no second datablock to read. (Its DNA struct is **`Lamp`** — RNA renamed
it in 2.80 and the DNA name never followed, so Blender 5.0.1 still writes
`Lamp`. The struct does carry a vestigial `*nodetree` pointer; nothing
drives through it.)

A world splits like a material: `color` on the `World` ID, the Background
node's `Strength` and `Color` on `World.nodetree`.

### The ranges are the content

Unlike a material's, a light's properties do not share one clamp. Each has
its own RNA hard range, and three of them are surprising:

| property | hard range | what a driver can do |
| --- | --- | --- |
| `energy` | **unbounded** | drive a lamp to −100 W, and it stays there |
| `color` | 0 .. ∞ | clamps below, free above 1 |
| `shadow_soft_size` | 0 .. ∞ | a radius cannot go negative |
| `spot_size` | **0.01745** .. π | a cone driven closed stays **1° open** |
| `spot_blend` | 0 .. 1 | clamps both ends |
| `size`, `size_y` | 0 .. ∞ | an area light's edges |
| `angle` | 0 .. π | a sun's angular diameter |

`energy` being unbounded and `spot_size` stopping at one degree are the two
no uniform rule would give, and both are measured.

The world's socket clamps are the material's, because they are the same
sockets: `Strength` is a float socket and takes a negative, `Color` is RGBA
and clamps each component at 0.

### An area light's energy is not its watts

`BlendLightData::energy` holds the value the **file** stores, and for an
area light that is 4/π of the watts the UI shows — Blender keeps it
pre-multiplied so its own radiance works out to `energy / (4 A)` instead of
`watts / (π A)`. Point, spot and sun lights store the watts directly.

A driver writes the **RNA** property, so its value is watts; Blender's RNA
setter converts on the way to the field. `BLightDrivers::apply` therefore
scales an area light's driven energy by `AREA_ENERGY_SCALE` (4/π) and
nothing else, which keeps the field meaning one thing whether or not a
driver touched it. Measured across all four area shapes: the factor is
exactly 4/π and independent of shape and size.

**Open question, recorded not answered.** The renderer consumes the stored
number (`view3d/rendered/lamp.rs` multiplies colour by `energy`), while
`phiray`'s own `lamp_power.rs` measured the kernel against Cycles by
feeding it **100 W** by hand and matching. If the kernel wants watts, an
area light read from a file reaches it 4/π — 27% — too bright. That is a
one-line change in the reader and a visible change to every render with an
area light, so it belongs in a batch of its own with a Cycles render
through the full path, not folded into a driver batch. Flagged in
`shade/jobs.md`.

### Measured

`tests/light_driver_truth.rs` against `LIGHTDRIVERS.blend`: 145 values over
5 frames, worst off by **3.8 × 10⁻⁶** (the 4/π round-trip), tolerance 1e-5.
Breaking either the `spot_size` minimum or the unbounded `energy` fails the
sheet and names the rule.

## Modifier drivers (batch 174)

A modifier driver sits on the **object's** `AnimData`, beside its transform
and custom-property drivers, pathed

    modifiers["<name>"].<property>

with a vector's component in the curve's `array_index`. Two things make it
unlike the three passes above.

### The modifier's type is not in the path

Only its name is, and the same property name means different things on
different modifiers — `strength` on Displace and on Lattice,
`merge_threshold` on Weld and on Mirror, `offset` on Solidify with no Mirror
equivalent at all. So the setting table is keyed by `(type, property)`, and
the type is resolved from the object's own modifier list by name. That list
is read in full, including modifiers the evaluation stack filters out,
because Blender writes a driven value whether or not the modifier is
switched on. A driver naming a modifier the object has not got, or a
setting the table has no entry for, is read and dropped rather than
guessed at.

### Integer settings truncate toward zero

A driver's value is always a float. An integer property takes it with a C
`(int)` cast — **not** a round, and **not** a floor:

| driver value | Blender's integer | a round would give | a floor would give |
| --- | --- | --- | --- |
| 0.7 | **0** | 1 | 0 |
| 2.5 | **2** | 3 | 2 |
| 11.9 | **11** | 12 | 11 |
| −0.7 | **0** | −1 | −1 |
| −1.5 | **−1** | −2 | −2 |
| −2.5 | **−2** | −3 | −3 |

The positive rows rule out rounding; the negative rows rule out floor. The
hard range clamps afterwards, and its minimum is not always zero —
`Subsurf.quality` starts at 1 and `Triangulate.min_vertices` at 4, so a
driver cannot take either to zero. `Solidify.material_offset` is the only
setting modelled whose range reaches below zero, which is why it is in the
fixture: without it, truncation and floor are indistinguishable.

Float settings are mostly unbounded both ways (`Displace.strength`,
`Solidify.thickness`); the thresholds (`Weld.merge_threshold`,
`Mirror.merge_threshold` and `bisect_threshold`) clamp at 0.

### What this does not do

The values are read, typed, clamped and measured. They are **not** written
into the modifier stack, so an evaluated mesh does not yet reflect them.
That is a separate change for two reasons: `modifier_jobs` /
`evaluate_modifiers` would have to carry the frame's overrides into each
reader, and a mesh is currently evaluated **once at open** rather than per
frame, so a driven modifier would still not animate without a
re-evaluation path. The semantics being settled makes that change
mechanical — but it lands in `modifier/mod.rs`, where another agent is
active, so it is queued rather than started. Filed in
`blend/modifier/jobs.md`.

### Measured

`tests/mod_driver_truth.rs` against `MODDRIVERS.blend`: 85 values over 5
frames — exact on every integer, worst float off by **6 × 10⁻⁸**. The test
compares the *type* as well as the number, because an integer setting read
as a float would pass a numeric compare at whole values and hide the
truncation entirely.

## Files

| file | what |
| --- | --- |
| `anim/driver/mod.rs` | `BlendDriver` — `evaluate_driver` and the F-curve over its value — and `BlendObjectDriver` |
| `anim/driver/read.rs` | `AnimData.drivers` → `BlendDriverSet`: what each curve writes, what each variable reads |
| `anim/driver/target.rs` | one `DriverTarget` out of the file, with the stride for `targets[1]` taken from the DNA |
| `anim/driver/var.rs` | `BlendDriverRead` and `dvar_eval_*` over the scene snapshot |
| `anim/driver/chan.rs` | `dvar_eval_transChan`'s tail and `BKE_driver_target_matrix_to_rot_channels` |
| `anim/driver/space.rs` | the three target spaces, and the world matrices over a snapshot |
| `anim/driver/pass.rs` | the frame's two-step pass |
| `anim/driver/state.rs` | `BDriverState` — a frame's pass kept rather than dropped, so a driver on something that is not an object reads the same scene, after the objects |
| `shade/driver.rs` | what a material driver writes, and the clamp each write path takes |
| `shade/driver_read.rs` | `Material.adt` and `Material.nodetree->adt` → `BMaterialDrivers` |
| `anim/driver/socket.rs` | the socket path and socket list every datablock with a node tree shares — material, world, and whatever comes next |
| `object/light_driver.rs` | a light's properties and each one's own hard range |
| `object/light_driver_read.rs` | `Lamp.adt` and `World.adt` / `World.nodetree->adt` out of the file |
| `shade/world_driver.rs` | the world's `color` and its Background node's sockets |
| `modifier/driver.rs` | the per-type setting table, each range, and the integer truncation |
| `modifier/driver_read.rs` | `modifiers["…"].property` off the object's `adt`, with the type resolved from its stack |
| `anim/drivers.rs` | the Drivers *editor*'s list, including the 5.0 bone-hide versioning step |
| `rig/driver/` | the rig's own drivers, over its bones |

## Next

1. **Write driven modifier settings into the stack.** The read is done and
   measured; see "What this does not do" above for the two parts and why it
   is queued rather than started.
2. **An area light's energy through the renderer.** Not a driver job — see
   batch 173's open question. Needs one Cycles render of an area light read
   from a file, compared with `phiray`'s.
3. **Shape-key drivers.** `Key.adt`, `key_blocks["Name"].value`. Blocked on
   shape keys themselves, which phiphx does not read yet.
4. **Context Property variables.** `DTAR_CONTEXT_PROPERTY_ACTIVE_SCENE` and
   `ACTIVE_VIEW_LAYER`; small, once there is a scene context to resolve
   against.
5. **Driver-against-driver ordering.** A dependency sort over the channel
   drivers would close the one limit stated above. Worth doing only on a
   file that needs it.
