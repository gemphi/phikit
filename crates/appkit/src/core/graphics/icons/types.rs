//! Icon type definitions

/// Icon identifier — every icon in the app has a variant here.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum IconId {
    // Primitives
    Sphere,
    Cube,
    Cylinder,
    Cone,
    Capsule,
    Plane,
    Torus,
    // Lights
    Light,
    Sun,
    PointLight,
    SpotLight,
    DirectionalLight,
    // UI
    Add,
    Remove,
    Edit,
    Settings,
    Search,
    Filter,
    Sort,
    // Actions
    Play,
    Pause,
    Stop,
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    Delete,
    Select,
    Move,
    Rotate,
    Scale,
    ViewPan,
    ViewZoom,
    ViewCamera,
    ZoomAll,
    ZoomIn,
    ZoomOut,
    ShadingSolid,
    MenuPanel,
    // Navigation
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    ChevronRight,
    ChevronDown,
    TriaRight,
    TriaDown,
    // Layout
    LayoutBottom,
    LayoutLeft,
    LayoutRight,
    LayoutTop,
    LayoutCenter,
    // Toolbar
    Plus,
    Minus,
    Camera,
    Microphone,
    Speaker,
    // Media
    Image,
    Video,
    Audio,
    Mesh,
    Material,
    Shader,
    Object,
    // States
    Visible,
    Hidden,
    Locked,
    Unlocked,
    Selected,
    // Tabs/Views
    ShaderGraph,
    Behaviors,
    Statistics,
    File,
    Folder,
    Project,
    BottomPanel,
    Package,
    Scene,
    World,
    Grid,
    SnapOn,
    Overlay,
    Lingerie,
    // Activity Bar / Misc
    Clock,
    Chat,
    // Generic
    Other(String),
}

/// Trait for types that can provide an icon
pub trait Iconable {
    fn icon_id(&self) -> IconId;
}
