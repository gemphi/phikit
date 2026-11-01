# Jingle Reality Composer Documentation Index

> Mapping of Apple Reality Composer Pro docs to Jingle/PhiPro equivalents

---

## Filename Mapping

| Apple Reality Composer Pro | Jingle/PhiPro Equivalent | Status |
|------------------------------|--------------------------|--------|
| `--configure-the-reality-composer-pro-project-window.md` | `--configure-the-jingle-project-window.md` | ✅ Created |
| `--adding-assets-into-your-reality-composer-pro-scene.md` | `--adding-assets-into-your-jingle-scene.md` | ✅ Created |
| `--applying-realitykit-materials-on-imported-assets.md` | `--applying-jingle-materials-on-imported-assets.md` | ✅ Created |
| `--build-and-run-reality-composer-pro-scenes-in-your-app.md` | `--build-and-run-jingle-scenes-in-your-app.md` | ✅ Created |
| `--building-materials.md` | `--building-jingle-materials.md` | ✅ Created |
| `--correcting-normals-after-vertex-displacement-with-shader-graph.md` | `--correcting-normals-after-vertex-displacement-with-jingle-shader-graph.md` | ✅ Created |
| `--creating-a-fresnel-outline-effect-with-shader-graph.md` | `--creating-a-fresnel-outline-effect-with-jingle-shader-graph.md` | ✅ Created |
| `--creating-a-reality-composer-pro-package-in-your-app.md` | `--creating-a-jingle-package-in-your-app.md` | ✅ Created |
| `--creating-a-vertex-displacement-material-with-shader-graph.md` | `--creating-a-vertex-displacement-material-with-jingle-shader-graph.md` | ✅ Created |
| `--editor_projectbrowser.md` | `--jingle-editor-projectbrowser.md` | ✅ Created |
| `--editor_shadergraph.md` | `--jingle-editor-shadergraph.md` | ✅ Created |
| `--loading-entities-with-shadergraph-materials.md` | `--loading-entities-with-jingle-shadergraph-materials.md` | ✅ Created |

---

## Documentation Structure

### Root Level
- `JINGLE_REALITY_COMPOSER_PRO_GUIDE.md` - Master guide
- `JINGLE_PROJECT_SETUP.md` - Project initialization
- `STEP_BY_STEP_ASSET_IMPORT_GUIDE.md` - Asset pipeline
- `STEP_BY_STEP_SHADER_GRAPH_GUIDE.md` - Material creation
- `STEP_BY_STEP_BOTANIST_USD_PRELOAD.md` - Botanist USD/USDZ preload and verification

### Subdirectories
Each subdirectory contains:
1. **Original Apple doc** (e.g., `--configure-the-reality-composer-pro-project-window.md`)
2. **Jingle equivalent** (e.g., `--configure-the-jingle-project-window.md`)

---

## Feature Coverage

### PhiPro Features Documented

| Feature | Original Apple Doc | Jingle Doc |
|---------|-------------------|------------|
| **Project Window** | Reality Composer Pro interface | `--configure-the-jingle-project-window.md` |
| **3D Viewport** | Viewport navigation | `--configure-the-jingle-project-window.md` |
| **Asset Import** | USDZ/OBJ import | `--adding-assets-into-your-jingle-scene.md` |
| **Asset Browser** | Project Browser | `--jingle-editor-projectbrowser.md` |
| **Materials** | PBR materials | `--applying-jingle-materials-on-imported-assets.md` |
| **Shader Graph** | Node editor | `--jingle-editor-shadergraph.md` |
| **Fresnel Effect** | Rim lighting | `--creating-a-fresnel-outline-effect-with-jingle-shader-graph.md` |
| **Displacement** | Vertex animation | `--creating-a-vertex-displacement-material-with-jingle-shader-graph.md` |
| **Normal Correction** | Lighting fix | `--correcting-normals-after-vertex-displacement-with-jingle-shader-graph.md` |
| **Scene Loading** | Runtime loading | `--loading-entities-with-jingle-shadergraph-materials.md` |
| **App Integration** | visionOS apps | `--build-and-run-jingle-scenes-in-your-app.md` |
| **Package Creation** | Project setup | `--creating-a-jingle-package-in-your-app.md` |
| **Material Building** | Shader authoring | `--building-jingle-materials.md` |

---

## API Mappings

| Apple API | Jingle/PhiPro API | Location |
|-----------|-------------------|----------|
| `Entity` | `jingle::reality_kit::Entity` | All scene docs |
| `Scene` | `jingle::reality_kit::Scene` | All scene docs |
| `Material` | `jingle::reality_kit::Material` | Material docs |
| `ShaderGraphMaterial` | `jingle::material::ShaderGraphMaterial` | Shader Graph docs |
| `RealityView` | `jingle::reality_kit::RealityView` | UI docs |
| `AssetManager` | `jingle::asset::AssetManager` | Asset docs |
| `ViewportRenderer` | `phipro::blenda::ViewportRenderer` | Window docs |
| `BlenderUI` | `phipro::blenda::BlenderUI` | Window docs |

---

## Image References

All Jingle docs reference the same images as Apple docs:
- `./../assets/rcpro-welcomewindow.png`
- `./../assets/rcpro-newproject.png`
- `./../assets/rcpro-heroshot.png`
- `./../assets/rcpro-viewporttoolbar.png`
- `./../assets/projectbrowser.jpg`
- `./../assets/rcpro-materialhierarchy.png`
- `./../assets/rcpro-materialinspector-pbr.png`
- `./../assets/fresnel-*.png`
- `./../assets/vertex-displacement-*.png`
- `./../assets/correcting-normals-*.png`

---

## Quick Access

### For PhiPro Users
Start with: `PHIPRO_OVERVIEW.md` in `/phipro/` directory

### For Jingle Developers
Start with: `JINGLE_REALITY_COMPOSER_PRO_GUIDE.md`

### For Scene Building
Follow: `STEP_BY_STEP_ASSET_IMPORT_GUIDE.md`

### For Botanist USD Preload
Follow: `STEP_BY_STEP_BOTANIST_USD_PRELOAD.md`

### For Materials
Follow: `STEP_BY_STEP_SHADER_GRAPH_GUIDE.md`

---

*Generated for Jingle RealityKit Implementation v0.1.0*
