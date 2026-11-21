# reference

Trees phiphx reads and ports from. None is built. Meta (facebookresearch)
repositories, as shallow git submodules, added 2026-11-10. Descriptions are
from each repository's own summary. Read in depth so far: `momentum/`,
`MHR/` and the avatars' deformation code; what they give phiphx is drawn in
`../docs/character.md`.

Three checkouts hold less than their names promise:

- `omnifacerig/` is a dataset release with no rigging code, and every
  payload is a Git LFS pointer (LFS is skipped on clone).
- `MHR/` has its code, but the rig itself (FBX, `.model`, correctives) is a
  separate download (`mhr/download_assets.py`).
- `DiffAvatar/` holds OBJ meshes only.

## Characters, rigs and motion

- `momentum/`: foundational algorithms for human kinematic motion and the numerical solvers that apply it (C++)
- `MHR/`: Momentum Human Rig, an anatomically inspired parametric full-body model
- `omnifacerig/`: automatic face rigging, inner mouth included, across character topologies
- `actionmesh/`: video to animated mesh, importable into 3D software
- `DiffAvatar/`: garment optimisation with differentiable cloth simulation
- `sam-3d-body/`: full-body mesh recovery from one image, on MHR
- `AGRoL/`: smooth full-body motion from sparse tracking signals
- `meshtalk/`: 3D face animation from speech
- `audio2photoreal/`: body and face motion for avatars driven by audio
- `CT2Hair/`: hair strands modelled from computed tomography
- `DeepHandMesh/`, `UHM/`: hand mesh models
- `VCMeshConv/`: mesh autoencoders over registered meshes
- `banmo/`, `AnimalAvatar/`: animatable models from casual video

## Avatars

- `goliath/`: relightable Gaussian codec avatars, relightable hands, full-body avatars
- `ava-256/`: universal codec avatars
- `D3GA/`: drivable Gaussian avatars in tetrahedral cages
- `dva/`, `tava/`: drivable and template-free volumetric actors
- `ca_body/`: codec avatar body
- `multiface/`: multi-view face captures with a mesh and texture decoder
- `KeypointNeRF/`, `AutoAvatar/`: volumetric and autoregressive avatars

## Geometry and reconstruction

- `vggt/`, `vggsfm/`: cameras, depth and points from images; structure from motion
- `map-anything/`, `fast3r/`, `mvdust3r/`: feed-forward multi-view reconstruction
- `sam-3d-objects/`: one image to an object's geometry, texture and layout
- `DeepSDF/`, `iSDF/`: learned signed distance fields
- `meshrcnn/`, `meshflow/`, `AutoPartGen/`: meshes and parts, predicted and generated
- `Large-Sparse-Reconstruction-Model/`: relightable digital twins from sparse views
- `vfusion3d/`, `MCC/`: 3D from video diffusion; multiview compressive coding

## Where to start

For "the character never fractures": `momentum/` (the solver and character
model) and `MHR/` (the rig) first. The ECS side of the same question is
`../../phicore/phiecs/docs/character.md`.

Renderers sit with `phiray` in `crates/phiray/reference`; robotics with
`phiace` in `crates/phiace/reference`. Blender's own source, the
specification for everything phiphx reads, is at `gemphi/ext/phiber-refs/blender`
and `gemphi/ext/REFS/blender` today; `crates/phiber/reference/blender` is
where it is headed.
