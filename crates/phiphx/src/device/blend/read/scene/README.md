# blend/scene

Scene assembly. `extract_scene`, scene properties, result types, saved UI (workspace, 3D view).

Callers enter through [`phx::Phx`](../../phx/README.md); this folder is one of its parts.

## Files

| File | What it is |
|---|---|
| `mod.rs` | Scene assembly. |
| `props.rs` | Scene-level properties Blender shows on open: frame range / fps (`Scene.r.cfra/sfra/efra/frs_sec/frs_sec_base`), output resolution… |
| `read.rs` | Scene extraction: picks `FileGlobal.curscene`, walks its collection tree (`collection.rs`), resolves world matrices (`world.rs`) and folds in the… |
| `types.rs` | Semantic result types: `BlendScene`, `BlendCollection`, `BlendSceneObject`, `BlendMaterial`. |
| `ui.rs` | "Load UI" (#28) — the workspace and 3D view Blender restores on open. |

Status and next steps: [`jobs.md`](jobs.md).
