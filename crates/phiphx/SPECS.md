# phiphx — specs

What the new lanes must do. Blender is the specification for everything a
`.blend` shows, and that is tracked in [TODO.md](TODO.md) against Blender
itself; these rows cover only what the references add. One row is one
requirement, with the Apple source it answers and the reference it was
read from.

## Sources

| Short | What | Where (from `gemphi/`) |
|---|---|---|
| **RF** | RealityFoundation: `Skeleton`, mesh resources, loading | `phi/crates/kitpro/src/examples/headers/MacOSX-20260419T054331Z-3-001/MacOSX/RealityFoundation/SwiftInterfaces/arm64e-apple-macos.swiftinterface` |
| **HD** | Model I/O headers: meshes, materials, assets | `phi/apple-source/ruskit/Interfaces/MacOSX/ModelIO/` |
| **BL** | Blender's source | `ext/phiber-refs/blender`, `ext/REFS/blender` |
| **MOM** | momentum | `phikit/crates/phiphx/reference/momentum/momentum/` |
| **REF** | the other Meta repositories | `phikit/crates/phiphx/reference/<repo>/` |
| index | every reference, and which checkouts hold less than their name | [reference/README.md](reference/README.md) |
| index | the ECS side of a character | [../phicore/phiecs/docs/character.md](../phicore/phiecs/docs/character.md) |

## Character check

Run once a rig at open. A failure is reported in the open report; it never
panics and never silently repairs.

| Part | Must | Apple | Reference | Today |
|---|---|---|---|---|
| joint order | hold: every bone's parent comes before it | `Skeleton.joints`, `parentIndex` RF:8030-8045 | MOM `character/skeleton.cpp:16-22` | true by construction (`BRig`, `blend/rig/mod.rs:112`), not checked |
| names | hold: bone names are unique, and every vertex group that deforms names a bone | - | MOM `io/motion/mmo_io.cpp:174-219` | `by_name` index, not checked |
| pose length | hold: a pose has one entry a bone | - | MOM `character/skeleton_state.cpp:92-98` | not checked |
| influences | hold: every influence index is a bone of this rig | - | MOM `character/linear_skinning.cpp:82-88` | not checked |
| weights | report a deformed vertex whose total weight is under the threshold: Blender leaves it where it was | - | MOM `io/fbx/openfbx_loader.cpp:926-949`; BL `armature_deform.cc` | deform divides by the total above 0.0001 (`blend/modifier/armature.rs:6-7`); nothing reported |
| rest matrices | derive the inverse rest matrix from the rig's own rest pose | `inverseBindPoseMatrix` RF:8030-8045 | MOM `character/character.cpp:783-792` | `chan_mat = pose_mat x inverse(arm_mat)` |

## Rig as asset, pose as vector

| Part | Must | Apple | Reference | Today |
|---|---|---|---|---|
| asset | hand out a rig's immutable half once: joints, rest transforms, inverse rest matrices, weights, pins | `Skeleton` RF:8030-8045 | MOM `character/character.h:32-79` | inside `BlendRigSource`, per scene |
| pose | hand out a frame's pose as one vector by joint, with nothing derived in it | `SkeletalPosesComponent` RF:5182 | MOM `character/types.h:156-173` | `SceneRigs::bones_at` gives matrices |
| pins | name points on joints that other objects attach to | `GeometricPin` RF:9088 | MOM `character/locator.h:19-26` | parents are name pairs in `SceneModel.parents` |
| the CPU path stays | keep the deform checked against Blender as the reference; a GPU skin is accepted only where it matches | - | - | met: CPU only |

## Writers and readers

| Part | Must | Apple | Reference | Today |
|---|---|---|---|---|
| glTF skins | write skins, joints and inverse bind matrices | - | MOM `io/gltf` | meshes and point clouds only |
| shape keys | read a mesh's shape keys and their values a frame | blend-shape weights (not read in RF) | actionmesh `actionmesh/io/glb_export.py:239-261` | no reader found |
| mesh in place | replace a mesh's vertices without a new mesh | `LowLevelMesh` RF:9585 | - | met by `Tracer::repose` in phiray |
| materials | cover Model I/O's PBR vocabulary | HD `MDLMaterial.h:130-153` | - | met, after Blender's Principled |

## Splats

| Part | Must | Apple | Reference | Today |
|---|---|---|---|---|
| read | read position, scale, rotation, opacity and harmonics a splat from a PLY, into flat arrays | - | vkraygs `src/vkgs/engine/engine.cc:1730-1741` (in `phiray/reference`) | PLY reader for points |
| bind | let a splat follow the triangle or cage cell it was fitted on: mean moved, covariance = J x covariance x J transposed | - | D3GA `lib/cage.py:339-342`, `models/cage_net.py:220-230` | no |

## Parametric body

| Part | Must | Apple | Reference | Today |
|---|---|---|---|---|
| pose to vertices | follow the order: identity and expression to rest shape, parameters to joints, forward pass, correctives, skinning | - | MHR `mhr/mhr.py:192-263` | no |
| levels of detail | pick a level by one number; all levels share one parameter file | - | MHR `mhr/io.py:35-53` | no |
| the rig files | be supplied by the user: they are a separate download and are not in the checkout | - | MHR `mhr/download_assets.py` | - |

## Not taken

Decoders that need trained weights (ava-256, the goliath head), every
training loop, and momentum's solver as a dependency: phiphx keeps
Blender's own IK (`blend/rig/ik`).
