//! The Properties editor's panels open and close the way Blender's do.
//!
//! Each row is a Blender panel class Phiber carries — read from
//! `scripts/startup/bl_ui/properties_*.py` and Cycles'
//! `intern/cycles/blender/addon/ui.py` — and whether it declares
//! `bl_options = {'DEFAULT_CLOSED'}`. A section that drifts from its
//! Blender panel fails here.

use phiber::v2::host::BHostType;
use phiber::v2::part::props::{BSectionSpec, sections_for};

/// (tab, `bl_label`, Blender class, `DEFAULT_CLOSED`)
const BLENDER: &[(&str, &str, &str, bool)] = &[
    ("Render", "Sampling", "CYCLES_RENDER_PT_sampling", false),
    ("Render", "Light Paths", "CYCLES_RENDER_PT_light_paths", true),
    ("Render", "Volumes", "CYCLES_RENDER_PT_volumes", true),
    ("Render", "Motion Blur", "CYCLES_RENDER_PT_motion_blur", true),
    ("Render", "Film", "CYCLES_RENDER_PT_film", true),
    ("Render", "Performance", "CYCLES_RENDER_PT_performance", true),
    ("Render", "Curves", "CYCLES_RENDER_PT_curves", true),
    ("Render", "Color Management", "RENDER_PT_color_management", true),
    ("Output", "Format", "RENDER_PT_format", false),
    ("Output", "Stereoscopy", "RENDER_PT_stereoscopy", true),
    ("Output", "Output", "RENDER_PT_output", false),
    ("Output", "Metadata", "RENDER_PT_stamp", true),
    ("Output", "Post Processing", "CYCLES_PT_post_processing", true),
    ("View Layer", "View Layer", "VIEWLAYER_PT_layer", false),
    ("View Layer", "Passes", "CYCLES_RENDER_PT_passes", false),
    ("View Layer", "Filter", "CYCLES_RENDER_PT_filter", true),
    ("View Layer", "Override", "VIEWLAYER_PT_override", true),
    ("Scene", "Scene", "SCENE_PT_scene", false),
    ("Scene", "Units", "SCENE_PT_unit", true),
    ("Scene", "Keying Sets", "SCENE_PT_keying_sets", true),
    ("Scene", "Audio", "SCENE_PT_audio", true),
    ("Scene", "Rigid Body World", "SCENE_PT_rigid_body_world", true),
    ("World", "Surface", "CYCLES_WORLD_PT_surface", false),
    ("World", "Volume", "CYCLES_WORLD_PT_volume", true),
    ("World", "Mist Pass", "CYCLES_WORLD_PT_mist", true),
    ("World", "Viewport Display", "WORLD_PT_viewport_display", true),
    ("Collection", "Line Art", "COLLECTION_PT_lineart_collection", false),
    ("Collection", "Instancing", "COLLECTION_PT_instancing", false),
    ("Object", "Transform", "OBJECT_PT_transform", false),
    ("Object", "Relations", "OBJECT_PT_relations", true),
    ("Object", "Collections", "OBJECT_PT_collections", true),
    ("Object", "Instancing", "OBJECT_PT_instancing", true),
    ("Object", "Visibility", "CYCLES_OBJECT_PT_visibility", true),
    ("Object", "Viewport Display", "OBJECT_PT_display", true),
    ("Object", "Line Art", "OBJECT_PT_lineart", true),
    ("Particles", "Emission", "PARTICLE_PT_emission", false),
    ("Particles", "Physics", "PARTICLE_PT_physics", true),
    ("Particles", "Render", "PARTICLE_PT_render", true),
    ("Particles", "Viewport Display", "PARTICLE_PT_draw", true),
    ("Physics", "Rigid Body", "PHYSICS_PT_rigid_body", false),
    ("Physics", "Collision", "PHYSICS_PT_collision", false),
    ("Physics", "Cloth", "PHYSICS_PT_cloth", false),
    ("Data", "UV Maps", "DATA_PT_uv_texture", true),
    ("Data", "Vertex Groups", "DATA_PT_vertex_groups", false),
    ("Data", "Shape Keys", "DATA_PT_shape_keys", false),
    ("Data", "Attributes", "DATA_PT_mesh_attributes", true),
    ("Data", "Remesh", "DATA_PT_remesh", true),
    ("Material", "Preview", "CYCLES_MATERIAL_PT_preview", true),
    ("Material", "Surface", "CYCLES_MATERIAL_PT_surface", false),
    ("Material", "Volume", "CYCLES_MATERIAL_PT_volume", true),
    ("Material", "Settings", "CYCLES_MATERIAL_PT_settings", true),
    ("Material", "Viewport Display", "MATERIAL_PT_viewport", true),
    ("Texture", "Preview", "TEXTURE_PT_preview", false),
    ("Texture", "Image", "TEXTURE_PT_image", false),
    ("Texture", "Colors", "TEXTURE_PT_colors", true),
    ("Texture", "Mapping", "TEXTURE_PT_mapping", false),
    ("Texture", "Influence", "TEXTURE_PT_influence", true),
    ("Object", "Motion Paths", "OBJECT_PT_motion_paths", true),
];

fn section(tab: &str, title: &str) -> BSectionSpec {
    *sections_for(BHostType::Buts, tab)
        .iter()
        .find(|s| s.title == title)
        .unwrap_or_else(|| panic!("{tab} has no {title} panel"))
}

#[test]
fn every_panel_starts_open_or_closed_as_blender_declares_it() {
    let mut wrong = Vec::new();
    for &(tab, title, class, closed) in BLENDER {
        if section(tab, title).closed != closed {
            wrong.push(format!("{tab} ▸ {title} ({class}): Blender DEFAULT_CLOSED={closed}"));
        }
    }
    assert!(wrong.is_empty(), "panels that open differently from Blender:\n  {}", wrong.join("\n  "));
}

/// `properties_render.py` + Cycles `ui.py`: the engine sits in a headless
/// context panel; Sampling is Viewport and Render, Render holds Denoise
/// (closed, its toggle in the header); Delta Transform is Transform's child.
#[test]
fn sub_panels_hang_where_blender_parents_them() {
    let render = sections_for(BHostType::Buts, "Render");
    assert!(render[0].headless && render[0].props.iter().any(|p| p.key == "engine"), "RENDER_PT_context leads, headless");
    let sampling = section("Render", "Sampling");
    let names: Vec<&str> = sampling.list.iter().map(|c| c.title).collect();
    assert_eq!(names, ["Viewport", "Render"]);
    let denoise = sampling.list[1].list[0];
    assert_eq!(denoise.title, "Denoise");
    assert!(denoise.closed, "CYCLES_RENDER_PT_sampling_render_denoise is DEFAULT_CLOSED");
    assert_eq!(denoise.header[0].key, "cycles.use_denoising", "its toggle is its header");
    assert_eq!(section("Render", "Motion Blur").header[0].key, "render.use_motion_blur");
    let light: Vec<&str> = section("Render", "Light Paths").list.iter().map(|c| c.title).collect();
    assert_eq!(light, ["Max Bounces", "Caustics"]);
    let object = sections_for(BHostType::Buts, "Object");
    assert!(object.iter().all(|s| s.title != "Delta Transform"), "not a top-level panel in Blender");
    let delta = section("Object", "Transform").list[0];
    assert_eq!((delta.title, delta.closed), ("Delta Transform", true));
}

mod painted {
    use gpui::{TestAppContext, px, size};
    use phiber::v2::app::BPhiberApp;
    use phiber::v2::space::BSpaceType;
    use phiber::v2::state::{BState, ButsKey, CtxKey};

    fn id(what: &str, key: &str) -> &'static str {
        Box::leak(format!("card-{what}-ctx-buts-render-{key}-expanded").into_boxed_str())
    }

    /// The Render tab as it paints: Sampling open with its Viewport and Render
    /// sub-panels, Denoise closed inside Render, Motion Blur closed with its
    /// checkbox before its title.
    #[gpui::test]
    async fn the_render_tab_paints_blenders_panel_tree(cx: &mut TestAppContext) {
        let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
        cx.simulate_resize(size(px(1600.0), px(1000.0)));
        app.update(cx, |app, cx| {
            app.state = BState::new();
            app.state.set(CtxKey::Space, BSpaceType::Layout.slug());
            app.state.set(ButsKey::NavbarTab, "Render");
            cx.notify();
        });
        cx.run_until_parked();

        for sub in ["sampling_viewport", "sampling_render"] {
            assert!(cx.debug_bounds(id("head", sub)).is_some(), "Sampling ▸ {sub} sub-panel is painted");
        }
        assert!(cx.debug_bounds(id("head", "sampling_render_denoise")).is_some(), "Denoise sits in Render");
        assert!(cx.debug_bounds(id("content", "sampling_render_denoise")).is_none(), "and starts closed");
        assert!(cx.debug_bounds(id("content", "light_paths")).is_none(), "Light Paths starts closed");

        let blur = cx.debug_bounds(id("title", "motion_blur")).expect("Motion Blur title");
        let volumes = cx.debug_bounds(id("title", "volumes")).expect("Volumes title");
        assert!(
            blur.origin.x > volumes.origin.x,
            "Motion Blur's header checkbox sits before its title, pushing it right"
        );
    }
}
