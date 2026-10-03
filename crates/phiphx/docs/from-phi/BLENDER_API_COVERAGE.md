# Blender DNA API Coverage & Apple Framework Mapping

> **Historical (from `gemphi/phi`).** Status columns below are the old `phi`
> reader's. The current reader (`crates/phiphx/src/importer/blend`) covers
> much more — as of 2026-09-16: world matrix (computed; 4.x+ has no `obmat`),
> `data` → Mesh, `mat` / `totcol` material slots, per-face `material_index`,
> 4.x/5.x attribute meshes (`position`, `.corner_vert`, UVs, normals),
> `Base.flag` selection / hidden, `LayerCollection` flags, `Scene.r` frame
> range / resolution / engine, World colour. Live status:
> `crates/phiber/docs/blend-read-parity.md`.

This document tracks all Blender features detectable via DNA parsing and their mapping to Apple ecosystem frameworks.

## Blender DNA Structures to Detect

### Core Object Properties (Object Struct)
| Blender Field | Type | Status | Apple Mapping | Notes |
|--------------|------|--------|---------------|-------|
| `id.name` | char[66] | ✅ Detected | `NSString` (RealityKit Entity name) | Object identifier |
| `type` | short | ✅ Detected | `EntityType` enum | MESH, EMPTY, ARMATURE, etc. |
| `loc[3]` | float[3] | ✅ Detected | `Transform.translation` | Location in 3D space |
| `rot[3]` | float[3] | ✅ Detected | `Transform.rotation` | Euler rotation |
| `size[3]` | float[3] | ✅ Detected | `Transform.scale` | Scale factors |
| `obmat[4][4]` | float[4][4] | 🔄 TODO | `Transform.matrix` | World matrix |
| `modifiers` | ListBase | ✅ Detected | `MeshResource` modifiers | Modifier stack |
| `dup_group` | Group* | ✅ Detected | `Entity.reference` | Instance target |
| `instance_collection` | Collection* | ✅ Detected | `Entity.reference` | Collection instance |
| `data` | ID* | 🔄 TODO | `MeshResource` | Linked mesh data |
| `mat` | Material** | 🔄 TODO | `MaterialResource` | Material array |
| `totcol` | short | 🔄 TODO | Material count | Number of materials |
| `flag` | int | 🔄 TODO | `EntityFlags` | Visibility, selection, etc. |
| `lay` | int | 🔄 TODO | `LayerMask` | Scene layer membership |

### Mesh Properties (Mesh Struct)
| Blender Field | Type | Status | Apple Mapping | Notes |
|--------------|------|--------|---------------|-------|
| `id.name` | char[66] | ✅ Detected | `MeshResource.name` | Mesh identifier |
| `mvert` | MVert* | 🔄 TODO | `MeshResource.positions` | Vertex array |
| `totvert` | int | 🔄 TODO | Vertex count | Number of vertices |
| `medge` | MEdge* | 🔄 TODO | `MeshResource.edges` | Edge array |
| `totedge` | int | 🔄 TODO | Edge count | Number of edges |
| `mpoly` | MPoly* | 🔄 TODO | `MeshResource.polygons` | Polygon array |
| `totpoly` | int | 🔄 TODO | Polygon count | Number of polygons |
| `mloop` | MLoop* | 🔄 TODO | `MeshResource.loopIndices` | Loop data |
| `totloop` | int | 🔄 TODO | Loop count | Number of loops |
| `mloopuv` | MLoopUV* | 🔄 TODO | `MeshResource.textureCoordinates` | UV coordinates |
| `mloopcol` | MLoopCol* | 🔄 TODO | `MeshResource.vertexColors` | Vertex colors |
| `mat` | Material** | 🔄 TODO | `MaterialResource` | Material references |

### Vertex Data (MVert)
| Blender Field | Type | Status | Apple Mapping | Notes |
|--------------|------|--------|---------------|-------|
| `co[3]` | float[3] | 🔄 TODO | `simd_float3` position | Vertex position |
| `no[3]` | short[3] | 🔄 TODO | `simd_float3` normal | Vertex normal (packed) |
| `flag` | char | 🔄 TODO | Vertex flags | Selection, hide, etc. |
| `bweight` | char | 🔄 TODO | Blend weight | Bevel weight |

### Material Properties (Material Struct)
| Blender Field | Type | Status | Apple Mapping | Notes |
|--------------|------|--------|---------------|-------|
| `id.name` | char[66] | 🔄 TODO | `MaterialResource.name` | Material name |
| `r` | float | 🔄 TODO | `diffuse.color.r` | Diffuse red |
| `g` | float | 🔄 TODO | `diffuse.color.g` | Diffuse green |
| `b` | float | 🔄 TODO | `diffuse.color.b` | Diffuse blue |
| `specr` | float | 🔄 TODO | `specular.color.r` | Specular red |
| `specg` | float | 🔄 TODO | `specular.color.g` | Specular green |
| `specb` | float | 🔄 TODO | `specular.color.b` | Specular blue |
| `alpha` | float | 🔄 TODO | `opacity` | Transparency |
| `roughness` | float | 🔄 TODO | `roughness` | Surface roughness |
| `metallic` | float | 🔄 TODO | `metallic` | Metallic factor |
| `use_nodes` | short | 🔄 TODO | Shader graph | Node-based material |
| `nodetree` | bNodeTree* | 🔄 TODO | `ShaderGraph` | Node tree reference |

### Modifier Types (ModifierData)
| Modifier Type | DNA Name | Status | Apple Mapping | Notes |
|--------------|----------|--------|---------------|-------|
| Subdivision Surface | `SubsurfModifierData` | 🔄 TODO | `MeshResource.subdivision` | Subdiv levels |
| Mirror | `MirrorModifierData` | 🔄 TODO | `MeshResource.symmetry` | Axis mirroring |
| Array | `ArrayModifierData` | 🔄 TODO | `Entity.instancing` | Array instances |
| Bevel | `BevelModifierData` | 🔄 TODO | `MeshResource.bevel` | Edge bevel |
| Boolean | `BooleanModifierData` | 🔄 TODO | CSG operations | Union/difference |
| Armature | `ArmatureModifierData` | 🔄 TODO | `Skinning` | Bone deformation |
| Displace | `DisplaceModifierData` | 🔄 TODO | `MeshResource.displacement` | Vertex displacement |
| Solidify | `SolidifyModifierData` | 🔄 TODO | `MeshResource.thickness` | Shell thickness |
| Cast | `CastModifierData` | 🔄 TODO | Deform shapes | Cast to shape |
| Wave | `WaveModifierData` | 🔄 TODO | Animation | Wave deformation |
| Curve | `CurveModifierData` | 🔄 TODO | Path following | Follow curve |
| Lattice | `LatticeModifierData` | 🔄 TODO | `LatticeDeformer` | Lattice warp |
| MeshDeform | `MeshDeformModifierData` | 🔄 TODO | Cage deformation | Mesh cage |
| UVProject | `UVProjectModifierData` | 🔄 TODO | UV projection | Project from camera |
| VertexWeight | `VertexWeightModifierData` | 🔄 TODO | Weight maps | Vertex weights |

### Collection Properties (Collection Struct)
| Blender Field | Type | Status | Apple Mapping | Notes |
|--------------|------|--------|---------------|-------|
| `id.name` | char[66] | ✅ Detected | `Entity.name` | Collection name |
| `gobject` | ListBase | 🔄 TODO | `Entity.children` | Objects in collection |
| `layer` | int | 🔄 TODO | `LayerMask` | Collection layer |
| `flag` | short | 🔄 TODO | `EntityFlags` | Collection flags |
| `instance_offset` | float[3] | 🔄 TODO | `Transform.offset` | Instance offset |
| `color_tag` | char | 🔄 TODO | `Entity.color` | Outliner color |

### Scene Properties (Scene Struct)
| Blender Field | Type | Status | Apple Mapping | Notes |
|--------------|------|--------|---------------|-------|
| `id.name` | char[66] | ✅ Detected | `Scene.name` | Scene name |
| `camera` | Object* | 🔄 TODO | `CameraEntity` | Active camera |
| `world` | World* | 🔄 TODO | `WorldResource` | Environment/world |
| `basact` | Base* | 🔄 TODO | `Entity.active` | Active object |
| `lay` | int | 🔄 TODO | `LayerMask` | Scene layers |
| `collection` | SceneCollection* | 🔄 TODO | `Scene.root` | Master collection |

### Armature/Rig Properties (Armature Struct)
| Blender Field | Type | Status | Apple Mapping | Notes |
|--------------|------|--------|---------------|-------|
| `id.name` | char[66] | ✅ Detected | `Skeleton.name` | Armature name |
| `bonebase` | ListBase | 🔄 TODO | `Skeleton.bones` | Bone hierarchy |
| `edbo` | ListBase | 🔄 TODO | Edit bones | Edit mode bones |
| `flag` | short | 🔄 TODO | Skeleton flags | Armature options |

### Bone Properties (Bone Struct)
| Blender Field | Type | Status | Apple Mapping | Notes |
|--------------|------|--------|---------------|-------|
| `name` | char[64] | 🔄 TODO | `Bone.name` | Bone name |
| `head[3]` | float[3] | 🔄 TODO | `Transform.head` | Bone head position |
| `tail[3]` | float[3] | 🔄 TODO | `Transform.tail` | Bone tail position |
| `arm_head[3]` | float[3] | 🔄 TODO | Arm space head | Armature space |
| `arm_tail[3]` | float[3] | 🔄 TODO | Arm space tail | Armature space |
| `arm_roll` | float | 🔄 TODO | Roll angle | Bone roll |
| `roll` | float | 🔄 TODO | Edit roll | Edit mode roll |
| `bone_mat[4][4]` | float[4][4] | 🔄 TODO | `Transform.matrix` | Bone matrix |
| `parent` | Bone* | 🔄 TODO | `Bone.parent` | Parent bone |
| `childbase` | ListBase | 🔄 TODO | `Bone.children` | Child bones |

## Apple Framework Mapping

### RealityKit (Primary Target)
```
Blender Object          → RealityKit Entity
Blender Mesh            → MeshResource
Blender Material        → MaterialResource / PhysicallyBasedMaterial
Blender Modifier        → MeshResource modifiers / Entity components
Blender Armature        → Skeleton / SkinningComponent
Blender Collection      → Entity hierarchy / Anchors
Blender Scene           → Scene
Blender Camera          → CameraComponent
Blender Light           → LightComponent
```

### VisualKit Integration
```
Blender Animation       → VisualKit AnimationTrack
Blender Keyframes       → VisualKit KeyframeSequence
Blender Drivers         → VisualKit AnimationController
Blender Constraints     → VisualKit Constraints
```

### AppKit/Foundation Bridge
```
Blender ID.name         → NSString
Blender Collections     → NSArray<Entity>
Blender Scene Data      → NSData (PHX binary)
Blender Metadata        → NSDictionary
```

## Implementation Priority

### Phase 1: Core Metadata (✅ Done)
- [x] Object names and types
- [x] Transforms (loc, rot, scale)
- [x] Collection hierarchy
- [x] Basic scene structure
- [x] Modifier presence detection
- [x] Instance presence detection

### Phase 2: Geometry Data (🔄 Next)
- [ ] Vertex positions (MVert.co)
- [ ] Face indices (MPoly + MLoop)
- [ ] Normals (MVert.no)
- [ ] UV coordinates (MLoopUV)
- [ ] Vertex colors (MLoopCol)

### Phase 3: Materials & Textures
- [ ] Basic material colors
- [ ] Material slots per object
- [ ] Image texture references
- [ ] PBR material properties
- [ ] Node tree structure

### Phase 4: Animation & Rigging
- [ ] Armature bones
- [ ] Bone weights (vertex groups)
- [ ] Action/keyframe data
- [ ] Shape keys (morph targets)

### Phase 5: Advanced Features
- [ ] Modifier stack details
- [ ] Particle systems
- [ ] Physics (cloth, softbody, rigid)
- [ ] Curves & surfaces
- [ ] Metaballs
- [ ] Grease pencil
- [ ] Volumes

## DNA Field Detection Status

### Currently Detected
- ✅ Object transforms (loc, rot, size)
- ✅ Object type
- ✅ Object ID name
- ✅ Modifier field presence
- ✅ Instance field presence (dup_group, instance_collection)

### Ready to Implement
- 🔄 Object world matrix (obmat)
- 🔄 Mesh vertex count (totvert)
- 🔄 Mesh polygon count (totpoly)
- 🔄 Material count (totcol)

### Requires Deeper Parsing
- 🔄 Actual vertex data (mvert pointer)
- 🔄 Face loop data (mpoly, mloop)
- 🔄 UV data (mloopuv pointer)
- 🔄 Material slot assignments

## Notes on DNA Versions

Blender DNA changes between versions:
- **2.7x**: 64-byte ID names, different struct layouts
- **2.8x-3.x**: 66-byte ID names, Collection-based hierarchy
- **4.x**: Extended ID padding, new fields added

Our parser currently handles:
- ✅ 66-byte ID names (Blender 3.x/4.x)
- ✅ 64-byte fallback for older files
- ✅ Pointer size detection (4/8 byte)
- ✅ Endianness detection

Future improvements:
- Version-specific struct layouts
- DNA compatibility layers
- Legacy file support (2.7x)

---

Last Updated: Apr 27, 2026
Status: Phase 1 Complete, Phase 2 Ready
Next: Implement geometry extraction from ME blocks
