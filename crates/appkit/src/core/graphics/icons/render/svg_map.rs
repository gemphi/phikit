//! SVG file mapping — each IconId → embedded .svg bytes + human-readable names.

use super::super::IconId;

/// Returns the embedded SVG bytes for an icon, or None if no SVG exists yet.
pub fn svg_data(id: &IconId) -> Option<&'static [u8]> {
    NSIconSvgMap::svg_data(id)
}

/// Human-readable name for debug logging.
pub fn icon_name(id: &IconId) -> &'static str {
    NSIconSvgMap::icon_name(id)
}

pub struct NSIconSvgMap;

impl NSIconSvgMap {

/// Returns the embedded SVG bytes for an icon, or None if no SVG exists yet.
pub fn svg_data(id: &IconId) -> Option<&'static [u8]> {
    // All paths: ../../../../assets/icons/ (4 levels up from src/core/graphics/icons/)
    match id {
        // Transform tools
        IconId::Select => Some(include_bytes!("../../../../../assets/icons/select.svg")),
        IconId::Move => Some(include_bytes!("../../../../../assets/icons/move.svg")),
        IconId::Rotate => Some(include_bytes!("../../../../../assets/icons/rotate.svg")),
        IconId::Scale => Some(include_bytes!("../../../../../assets/icons/scale.svg")),
        IconId::ViewPan => Some(include_bytes!("../../../../../assets/icons/hand.svg")),
        IconId::ViewZoom => Some(include_bytes!("../../../../../assets/icons/search.svg")),
        IconId::ViewCamera => Some(include_bytes!("../../../../../assets/icons/camera.svg")),
        IconId::ZoomAll => Some(include_bytes!("../../../../../assets/icons/maximize.svg")),
        IconId::ZoomIn => Some(include_bytes!("../../../../../assets/icons/zoom-in.svg")),
        IconId::ZoomOut => Some(include_bytes!("../../../../../assets/icons/zoom-out.svg")),

        // Navigation
        IconId::ArrowUp => Some(include_bytes!("../../../../../assets/icons/arrow-up.svg")),
        IconId::ArrowDown => Some(include_bytes!("../../../../../assets/icons/arrow-down.svg")),
        IconId::ArrowLeft => Some(include_bytes!("../../../../../assets/icons/arrow-left.svg")),
        IconId::ArrowRight => Some(include_bytes!("../../../../../assets/icons/arrow-right.svg")),
        IconId::ChevronRight => Some(include_bytes!("../../../../../assets/icons/chevron-right.svg")),
        IconId::ChevronDown => Some(include_bytes!("../../../../../assets/icons/chevron-down.svg")),
        IconId::TriaRight => Some(include_bytes!("../../../../../assets/icons/triangle-right.svg")),
        IconId::TriaDown => Some(include_bytes!("../../../../../assets/icons/triangle-down.svg")),

        // UI Controls
        IconId::Add | IconId::Plus => Some(include_bytes!("../../../../../assets/icons/plus.svg")),
        IconId::Remove | IconId::Minus => Some(include_bytes!("../../../../../assets/icons/minus.svg")),
        IconId::Edit => Some(include_bytes!("../../../../../assets/icons/pencil.svg")),
        IconId::Settings => Some(include_bytes!("../../../../../assets/icons/settings.svg")),
        IconId::Search => Some(include_bytes!("../../../../../assets/icons/search.svg")),
        IconId::Filter => Some(include_bytes!("../../../../../assets/icons/filter.svg")),
        IconId::Sort => Some(include_bytes!("../../../../../assets/icons/sort.svg")),

        // Actions
        IconId::Play => Some(include_bytes!("../../../../../assets/icons/play.svg")),
        IconId::Pause => Some(include_bytes!("../../../../../assets/icons/pause.svg")),
        IconId::Stop => Some(include_bytes!("../../../../../assets/icons/stop.svg")),
        IconId::Undo => Some(include_bytes!("../../../../../assets/icons/undo.svg")),
        IconId::Redo => Some(include_bytes!("../../../../../assets/icons/redo.svg")),
        IconId::Cut => Some(include_bytes!("../../../../../assets/icons/scissors.svg")),
        IconId::Copy => Some(include_bytes!("../../../../../assets/icons/copy.svg")),
        IconId::Paste => Some(include_bytes!("../../../../../assets/icons/clipboard.svg")),
        IconId::Delete => Some(include_bytes!("../../../../../assets/icons/trash.svg")),

        // Shading / Display
        IconId::ShadingSolid => Some(include_bytes!("../../../../../assets/icons/shading-solid.svg")),
        IconId::MenuPanel => Some(include_bytes!("../../../../../assets/icons/panel-left.svg")),

        // Layout
        IconId::LayoutLeft => Some(include_bytes!("../../../../../assets/icons/panel-left.svg")),
        IconId::LayoutRight => Some(include_bytes!("../../../../../assets/icons/panel-right.svg")),
        IconId::LayoutBottom => Some(include_bytes!("../../../../../assets/icons/panel-bottom.svg")),
        IconId::LayoutTop => Some(include_bytes!("../../../../../assets/icons/panel-top.svg")),
        IconId::LayoutCenter => Some(include_bytes!("../../../../../assets/icons/layout-dashboard.svg")),

        // Toolbar
        IconId::Camera => Some(include_bytes!("../../../../../assets/icons/camera.svg")),
        IconId::Microphone => Some(include_bytes!("../../../../../assets/icons/mic.svg")),
        IconId::Speaker => Some(include_bytes!("../../../../../assets/icons/volume.svg")),

        // Media / Assets
        IconId::Image => Some(include_bytes!("../../../../../assets/icons/image.svg")),
        IconId::Video => Some(include_bytes!("../../../../../assets/icons/video.svg")),
        IconId::Audio => Some(include_bytes!("../../../../../assets/icons/music.svg")),
        IconId::Mesh => Some(include_bytes!("../../../../../assets/icons/box.svg")),
        IconId::Material => Some(include_bytes!("../../../../../assets/icons/palette.svg")),
        IconId::Shader => Some(include_bytes!("../../../../../assets/icons/sparkles.svg")),
        IconId::Object => Some(include_bytes!("../../../../../assets/icons/component.svg")),

        // 3D Primitives
        IconId::Sphere => Some(include_bytes!("../../../../../assets/icons/circle.svg")),
        IconId::Cube => Some(include_bytes!("../../../../../assets/icons/box.svg")),
        IconId::Cylinder => Some(include_bytes!("../../../../../assets/icons/cylinder.svg")),
        IconId::Cone => Some(include_bytes!("../../../../../assets/icons/cone.svg")),
        IconId::Capsule => Some(include_bytes!("../../../../../assets/icons/pill.svg")),
        IconId::Plane => Some(include_bytes!("../../../../../assets/icons/square.svg")),
        IconId::Torus => Some(include_bytes!("../../../../../assets/icons/donut.svg")),

        // Lights
        IconId::Light | IconId::Sun => Some(include_bytes!("../../../../../assets/icons/sun.svg")),
        IconId::PointLight => Some(include_bytes!("../../../../../assets/icons/lightbulb.svg")),
        IconId::SpotLight => Some(include_bytes!("../../../../../assets/icons/flashlight.svg")),
        IconId::DirectionalLight => Some(include_bytes!("../../../../../assets/icons/sun-dim.svg")),

        // States
        IconId::Visible => Some(include_bytes!("../../../../../assets/icons/eye.svg")),
        IconId::Hidden => Some(include_bytes!("../../../../../assets/icons/eye-off.svg")),
        IconId::Locked => Some(include_bytes!("../../../../../assets/icons/lock.svg")),
        IconId::Unlocked => Some(include_bytes!("../../../../../assets/icons/unlock.svg")),
        IconId::Selected => Some(include_bytes!("../../../../../assets/icons/check-circle.svg")),

        // Tabs / Views
        IconId::ShaderGraph => Some(include_bytes!("../../../../../assets/icons/workflow.svg")),
        IconId::Behaviors => Some(include_bytes!("../../../../../assets/icons/brain.svg")),
        IconId::Statistics => Some(include_bytes!("../../../../../assets/icons/bar-chart.svg")),
        IconId::File => Some(include_bytes!("../../../../../assets/icons/file.svg")),
        IconId::Folder => Some(include_bytes!("../../../../../assets/icons/folder.svg")),
        IconId::Project => Some(include_bytes!("../../../../../assets/icons/folder-tree.svg")),
        IconId::BottomPanel => Some(include_bytes!("../../../../../assets/icons/panel-bottom.svg")),
        IconId::Package => Some(include_bytes!("../../../../../assets/icons/package.svg")),
        IconId::Scene => Some(include_bytes!("../../../../../assets/icons/layers.svg")),
        IconId::World => Some(include_bytes!("../../../../../assets/icons/globe.svg")),
        IconId::Grid => Some(include_bytes!("../../../../../assets/icons/grid.svg")),
        IconId::SnapOn => Some(include_bytes!("../../../../../assets/icons/magnet.svg")),
        IconId::Overlay => Some(include_bytes!("../../../../../assets/icons/blend.svg")),

        _ => None,
    }
}

/// Human-readable name for debug logging.
pub fn icon_name(id: &IconId) -> &'static str {
    match id {
        IconId::Select => "select",
        IconId::Move => "move",
        IconId::Rotate => "rotate",
        IconId::Scale => "scale",
        IconId::ViewPan => "hand",
        IconId::ViewZoom => "search",
        IconId::ViewCamera => "video",
        IconId::ZoomAll => "maximize",
        IconId::ZoomIn => "zoom-in",
        IconId::ZoomOut => "zoom-out",
        IconId::ArrowUp => "arrow-up",
        IconId::ArrowDown => "arrow-down",
        IconId::ArrowLeft => "arrow-left",
        IconId::ArrowRight => "arrow-right",
        IconId::ChevronRight => "chevron-right",
        IconId::ChevronDown => "chevron-down",
        IconId::TriaRight => "triangle-right",
        IconId::TriaDown => "triangle-down",
        IconId::Add | IconId::Plus => "plus",
        IconId::Remove | IconId::Minus => "minus",
        IconId::Edit => "pencil",
        IconId::Settings => "settings",
        IconId::Search => "search",
        IconId::Filter => "filter",
        IconId::Sort => "sort",
        IconId::Play => "play",
        IconId::Pause => "pause",
        IconId::Stop => "stop",
        IconId::Undo => "undo",
        IconId::Redo => "redo",
        IconId::Cut => "scissors",
        IconId::Copy => "copy",
        IconId::Paste => "clipboard",
        IconId::Delete => "trash",
        IconId::ShadingSolid => "shading-solid",
        IconId::MenuPanel => "panel-left",
        IconId::LayoutLeft => "panel-left",
        IconId::LayoutRight => "panel-right",
        IconId::LayoutBottom => "panel-bottom",
        IconId::LayoutTop => "panel-top",
        IconId::LayoutCenter => "layout-dashboard",
        IconId::Camera => "camera",
        IconId::Microphone => "mic",
        IconId::Speaker => "volume",
        IconId::Image => "image",
        IconId::Video => "video",
        IconId::Audio => "music",
        IconId::Mesh => "box",
        IconId::Material => "palette",
        IconId::Shader => "sparkles",
        IconId::Object => "component",
        IconId::Sphere => "circle",
        IconId::Cube => "box",
        IconId::Cylinder => "cylinder",
        IconId::Cone => "cone",
        IconId::Capsule => "pill",
        IconId::Plane => "square",
        IconId::Torus => "donut",
        IconId::Light | IconId::Sun => "sun",
        IconId::PointLight => "lightbulb",
        IconId::SpotLight => "flashlight",
        IconId::DirectionalLight => "sun-dim",
        IconId::Visible => "eye",
        IconId::Hidden => "eye-off",
        IconId::Locked => "lock",
        IconId::Unlocked => "unlock",
        IconId::Selected => "check-circle",
        IconId::ShaderGraph => "workflow",
        IconId::Behaviors => "brain",
        IconId::Statistics => "bar-chart",
        IconId::File => "file",
        IconId::Folder => "folder",
        IconId::Project => "folder-open",
        IconId::BottomPanel => "folder-tree",
        IconId::Package => "package",
        IconId::Scene => "layers",
        IconId::World => "globe",
        IconId::Grid => "grid",
        IconId::SnapOn => "magnet",
        IconId::Overlay => "blend",
        IconId::Lingerie => "lingerie",
        _ => "unknown",
    }
}

} // impl NSIconSvgMap
