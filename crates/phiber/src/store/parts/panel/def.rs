//! Panel section constants and accordion sub-panel data.
//! Matches Blender's eSpaceButtons_Context (BCONTEXT) enum.
//! Ref: DNA_space_types.h, buttons_context.c

use super::{PanelAccordion, PanelRow};

// ── Buts (Properties) context tab IDs/labels/icons ──

pub const SECTION_RENDER_ID: &str = "render";
pub const SECTION_RENDER_LABEL: &str = "Render";
pub const SECTION_RENDER_ICON: &str = "render";

pub const SECTION_SCENE_ID: &str = "scene";
pub const SECTION_SCENE_LABEL: &str = "Scene";
pub const SECTION_SCENE_ICON: &str = "scene";

pub const SECTION_WORLD_ID: &str = "world";
pub const SECTION_WORLD_LABEL: &str = "World";
pub const SECTION_WORLD_ICON: &str = "world";

pub const SECTION_OBJECT_ID: &str = "object";
pub const SECTION_OBJECT_LABEL: &str = "Object";
pub const SECTION_OBJECT_ICON: &str = "object";

pub const SECTION_DATA_ID: &str = "data";
pub const SECTION_DATA_LABEL: &str = "Data";
pub const SECTION_DATA_ICON: &str = "data";

pub const SECTION_MATERIAL_ID: &str = "material";
pub const SECTION_MATERIAL_LABEL: &str = "Material";
pub const SECTION_MATERIAL_ICON: &str = "material";

pub const SECTION_TEXTURE_ID: &str = "texture";
pub const SECTION_TEXTURE_LABEL: &str = "Texture";
pub const SECTION_TEXTURE_ICON: &str = "texture";

pub const SECTION_PARTICLE_ID: &str = "particle";
pub const SECTION_PARTICLE_LABEL: &str = "Particle";
pub const SECTION_PARTICLE_ICON: &str = "particle";

pub const SECTION_PHYSICS_ID: &str = "physics";
pub const SECTION_PHYSICS_LABEL: &str = "Physics";
pub const SECTION_PHYSICS_ICON: &str = "physics";

pub const SECTION_BONE_ID: &str = "bone";
pub const SECTION_BONE_LABEL: &str = "Bone";
pub const SECTION_BONE_ICON: &str = "bone";

pub const SECTION_MODIFIER_ID: &str = "modifiers";
pub const SECTION_MODIFIER_LABEL: &str = "Modifiers";
pub const SECTION_MODIFIER_ICON: &str = "modifier";

pub const SECTION_CONSTRAINT_ID: &str = "constraint";
pub const SECTION_CONSTRAINT_LABEL: &str = "Constraints";
pub const SECTION_CONSTRAINT_ICON: &str = "constraint";

pub const SECTION_BONE_CONSTRAINT_ID: &str = "bone_constraint";
pub const SECTION_BONE_CONSTRAINT_LABEL: &str = "Bone Constraints";
pub const SECTION_BONE_CONSTRAINT_ICON: &str = "bone_constraint";

pub const SECTION_RENDER_LAYER_ID: &str = "render_layer";
pub const SECTION_RENDER_LAYER_LABEL: &str = "Render Layers";
pub const SECTION_RENDER_LAYER_ICON: &str = "render_layer";

// ── View3D sidebar sections ──

pub const SECTION_ITEM_ID: &str = "item";
pub const SECTION_ITEM_LABEL: &str = "Item";
pub const SECTION_ITEM_ICON: &str = "item";

pub const SECTION_TOOL_ID: &str = "tool";
pub const SECTION_TOOL_LABEL: &str = "Tool";
pub const SECTION_TOOL_ICON: &str = "tool";

pub const SECTION_VIEW_ID: &str = "view";
pub const SECTION_VIEW_LABEL: &str = "View";
pub const SECTION_VIEW_ICON: &str = "view";

// ── Accordion sub-panels per context tab ──
// Ref: Blender's Properties editor panel layouts

pub static EMPTY_ROWS: &[PanelRow] = &[];

pub static RENDER_ACCORDIONS: &[PanelAccordion] = &[
    PanelAccordion {
        label: "Render Properties",
        icon: "render",
        expanded: true,
        rows: &[
            PanelRow {
                label: "Engine",
                value: "Eevee",
            },
            PanelRow {
                label: "Sampling",
                value: "128",
            },
        ],
    },
    PanelAccordion {
        label: "Dimensions",
        icon: "ratio",
        expanded: false,
        rows: &[
            PanelRow {
                label: "Resolution X",
                value: "1920",
            },
            PanelRow {
                label: "Resolution Y",
                value: "1080",
            },
            PanelRow {
                label: "Frame Rate",
                value: "24 fps",
            },
        ],
    },
    PanelAccordion {
        label: "Output",
        icon: "output",
        expanded: false,
        rows: &[
            PanelRow {
                label: "Path",
                value: "//render/",
            },
            PanelRow {
                label: "File Format",
                value: "PNG",
            },
        ],
    },
    PanelAccordion {
        label: "Post Processing",
        icon: "sliders",
        expanded: false,
        rows: &[
            PanelRow {
                label: "Glare",
                value: "Off",
            },
            PanelRow {
                label: "Bloom",
                value: "On",
            },
        ],
    },
];

pub static SCENE_ACCORDIONS: &[PanelAccordion] = &[
    PanelAccordion {
        label: "Scene Properties",
        icon: "scene",
        expanded: true,
        rows: &[
            PanelRow {
                label: "Unit System",
                value: "Metric",
            },
            PanelRow {
                label: "Scale",
                value: "1.0",
            },
        ],
    },
    PanelAccordion {
        label: "Color Management",
        icon: "palette",
        expanded: false,
        rows: &[
            PanelRow {
                label: "Display Device",
                value: "sRGB",
            },
            PanelRow {
                label: "View Transform",
                value: "Filmic",
            },
        ],
    },
    PanelAccordion {
        label: "Audio",
        icon: "audio",
        expanded: false,
        rows: &[
            PanelRow {
                label: "Volume",
                value: "1.0",
            },
            PanelRow {
                label: "Speed of Sound",
                value: "343.0",
            },
        ],
    },
];

pub static WORLD_ACCORDIONS: &[PanelAccordion] = &[
    PanelAccordion {
        label: "World Properties",
        icon: "world",
        expanded: true,
        rows: &[PanelRow {
            label: "Color",
            value: "#404040",
        }],
    },
    PanelAccordion {
        label: "Surface",
        icon: "surface",
        expanded: false,
        rows: &[
            PanelRow {
                label: "Background",
                value: "#808080",
            },
            PanelRow {
                label: "Strength",
                value: "1.0",
            },
        ],
    },
    PanelAccordion {
        label: "Volume",
        icon: "cloud",
        expanded: false,
        rows: &[PanelRow {
            label: "Density",
            value: "0.0",
        }],
    },
];

pub static OBJECT_ACCORDIONS: &[PanelAccordion] = &[
    PanelAccordion {
        label: "Object Properties",
        icon: "object",
        expanded: true,
        rows: &[
            PanelRow {
                label: "Name",
                value: "Cube",
            },
            PanelRow {
                label: "Type",
                value: "Mesh",
            },
        ],
    },
    PanelAccordion {
        label: "Transform",
        icon: "move",
        expanded: true,
        rows: &[
            PanelRow {
                label: "Location",
                value: "0, 0, 0",
            },
            PanelRow {
                label: "Rotation",
                value: "0, 0, 0",
            },
            PanelRow {
                label: "Scale",
                value: "1, 1, 1",
            },
        ],
    },
    PanelAccordion {
        label: "Delta Transform",
        icon: "move",
        expanded: false,
        rows: &[PanelRow {
            label: "Delta Location",
            value: "0, 0, 0",
        }],
    },
    PanelAccordion {
        label: "Relations",
        icon: "link",
        expanded: false,
        rows: &[PanelRow {
            label: "Parent",
            value: "None",
        }],
    },
    PanelAccordion {
        label: "Collections",
        icon: "folder",
        expanded: false,
        rows: &[PanelRow {
            label: "Scene Collection",
            value: "Yes",
        }],
    },
    PanelAccordion {
        label: "Instancing",
        icon: "copy",
        expanded: false,
        rows: &[PanelRow {
            label: "Instance Type",
            value: "None",
        }],
    },
];

pub static DATA_ACCORDIONS: &[PanelAccordion] = &[
    PanelAccordion {
        label: "Mesh Properties",
        icon: "data",
        expanded: true,
        rows: &[
            PanelRow {
                label: "Vertex Count",
                value: "8",
            },
            PanelRow {
                label: "Face Count",
                value: "6",
            },
        ],
    },
    PanelAccordion {
        label: "UV Maps",
        icon: "uv",
        expanded: false,
        rows: &[PanelRow {
            label: "UV Map",
            value: "UVMap",
        }],
    },
    PanelAccordion {
        label: "Vertex Groups",
        icon: "group",
        expanded: false,
        rows: &[PanelRow {
            label: "Groups",
            value: "0",
        }],
    },
    PanelAccordion {
        label: "Shape Keys",
        icon: "shapes",
        expanded: false,
        rows: &[PanelRow {
            label: "Basis",
            value: "Basis",
        }],
    },
];

pub static MATERIAL_ACCORDIONS: &[PanelAccordion] = &[
    PanelAccordion {
        label: "Material Properties",
        icon: "material",
        expanded: true,
        rows: &[
            PanelRow {
                label: "Name",
                value: "Material",
            },
            PanelRow {
                label: "Preview",
                value: "Surface",
            },
        ],
    },
    PanelAccordion {
        label: "Surface",
        icon: "surface",
        expanded: true,
        rows: &[
            PanelRow {
                label: "Base Color",
                value: "#ffffff",
            },
            PanelRow {
                label: "Metallic",
                value: "0.0",
            },
            PanelRow {
                label: "Roughness",
                value: "0.5",
            },
        ],
    },
    PanelAccordion {
        label: "Volume",
        icon: "cloud",
        expanded: false,
        rows: &[PanelRow {
            label: "Density",
            value: "0.0",
        }],
    },
    PanelAccordion {
        label: "Settings",
        icon: "settings",
        expanded: false,
        rows: &[PanelRow {
            label: "Blend Mode",
            value: "Opaque",
        }],
    },
];

pub static TEXTURE_ACCORDIONS: &[PanelAccordion] = &[
    PanelAccordion {
        label: "Texture Properties",
        icon: "texture",
        expanded: true,
        rows: &[PanelRow {
            label: "Type",
            value: "Image",
        }],
    },
    PanelAccordion {
        label: "Image",
        icon: "image",
        expanded: false,
        rows: &[PanelRow {
            label: "Source",
            value: "File",
        }],
    },
    PanelAccordion {
        label: "Mapping",
        icon: "map",
        expanded: false,
        rows: &[
            PanelRow {
                label: "Coordinates",
                value: "UV",
            },
            PanelRow {
                label: "Projection",
                value: "Flat",
            },
        ],
    },
];

pub static PARTICLE_ACCORDIONS: &[PanelAccordion] = &[
    PanelAccordion {
        label: "Particle Properties",
        icon: "particle",
        expanded: true,
        rows: &[PanelRow {
            label: "Type",
            value: "Emitter",
        }],
    },
    PanelAccordion {
        label: "Emission",
        icon: "sparkles",
        expanded: false,
        rows: &[
            PanelRow {
                label: "Number",
                value: "1000",
            },
            PanelRow {
                label: "Lifetime",
                value: "50",
            },
        ],
    },
    PanelAccordion {
        label: "Render",
        icon: "render",
        expanded: false,
        rows: &[PanelRow {
            label: "Render As",
            value: "Halo",
        }],
    },
];

pub static PHYSICS_ACCORDIONS: &[PanelAccordion] = &[
    PanelAccordion {
        label: "Physics Properties",
        icon: "physics",
        expanded: true,
        rows: &[PanelRow {
            label: "Field Type",
            value: "None",
        }],
    },
    PanelAccordion {
        label: "Collision",
        icon: "collision",
        expanded: false,
        rows: &[PanelRow {
            label: "Damping",
            value: "0.0",
        }],
    },
    PanelAccordion {
        label: "Rigid Body",
        icon: "rigid",
        expanded: false,
        rows: &[
            PanelRow {
                label: "Type",
                value: "Active",
            },
            PanelRow {
                label: "Mass",
                value: "1.0",
            },
        ],
    },
];

pub static BONE_ACCORDIONS: &[PanelAccordion] = &[
    PanelAccordion {
        label: "Bone Properties",
        icon: "bone",
        expanded: true,
        rows: &[PanelRow {
            label: "Name",
            value: "Bone",
        }],
    },
    PanelAccordion {
        label: "Transform",
        icon: "move",
        expanded: true,
        rows: &[
            PanelRow {
                label: "Location",
                value: "0, 0, 0",
            },
            PanelRow {
                label: "Rotation",
                value: "0, 0, 0",
            },
        ],
    },
    PanelAccordion {
        label: "Relations",
        icon: "link",
        expanded: false,
        rows: &[PanelRow {
            label: "Parent",
            value: "None",
        }],
    },
];

pub static MODIFIER_ACCORDIONS: &[PanelAccordion] = &[
    PanelAccordion {
        label: "Modifiers Properties",
        icon: "modifier",
        expanded: true,
        rows: &[PanelRow {
            label: "Add Modifier",
            value: "...",
        }],
    },
    PanelAccordion {
        label: "Subdivision Surface",
        icon: "subdivision",
        expanded: false,
        rows: &[
            PanelRow {
                label: "Levels",
                value: "2",
            },
            PanelRow {
                label: "Render Levels",
                value: "3",
            },
        ],
    },
    PanelAccordion {
        label: "Mirror",
        icon: "mirror",
        expanded: false,
        rows: &[PanelRow {
            label: "Axis",
            value: "X",
        }],
    },
];

pub static CONSTRAINT_ACCORDIONS: &[PanelAccordion] = &[
    PanelAccordion {
        label: "Constraints Properties",
        icon: "constraint",
        expanded: true,
        rows: &[PanelRow {
            label: "Add Constraint",
            value: "...",
        }],
    },
    PanelAccordion {
        label: "Copy Location",
        icon: "copy",
        expanded: false,
        rows: &[PanelRow {
            label: "Target",
            value: "None",
        }],
    },
];

pub static BONE_CONSTRAINT_ACCORDIONS: &[PanelAccordion] = &[PanelAccordion {
    label: "Bone Constraints Properties",
    icon: "bone_constraint",
    expanded: true,
    rows: &[PanelRow {
        label: "Add Constraint",
        value: "...",
    }],
}];

pub static RENDER_LAYER_ACCORDIONS: &[PanelAccordion] = &[
    PanelAccordion {
        label: "Render Layers Properties",
        icon: "render_layer",
        expanded: true,
        rows: &[PanelRow {
            label: "Layers",
            value: "ViewLayer",
        }],
    },
    PanelAccordion {
        label: "Passes",
        icon: "layers",
        expanded: false,
        rows: &[
            PanelRow {
                label: "Combined",
                value: "Yes",
            },
            PanelRow {
                label: "Z",
                value: "Yes",
            },
        ],
    },
];

// ── View3D sidebar accordions ──

pub static ITEM_ACCORDIONS: &[PanelAccordion] = &[
    PanelAccordion {
        label: "Item",
        icon: "item",
        expanded: true,
        rows: &[
            PanelRow {
                label: "Name",
                value: "Cube",
            },
            PanelRow {
                label: "Type",
                value: "Mesh",
            },
        ],
    },
    PanelAccordion {
        label: "Transform",
        icon: "move",
        expanded: true,
        rows: &[
            PanelRow {
                label: "Location",
                value: "0, 0, 0",
            },
            PanelRow {
                label: "Rotation",
                value: "0, 0, 0",
            },
            PanelRow {
                label: "Scale",
                value: "1, 1, 1",
            },
        ],
    },
];

pub static TOOL_ACCORDIONS: &[PanelAccordion] = &[
    PanelAccordion {
        label: "Tool",
        icon: "tool",
        expanded: true,
        rows: &[PanelRow {
            label: "Active Tool",
            value: "Select Box",
        }],
    },
    PanelAccordion {
        label: "Options",
        icon: "settings",
        expanded: false,
        rows: &[PanelRow {
            label: "Snap",
            value: "Off",
        }],
    },
];

pub static VIEW_ACCORDIONS: &[PanelAccordion] = &[
    PanelAccordion {
        label: "View",
        icon: "view",
        expanded: true,
        rows: &[
            PanelRow {
                label: "Camera",
                value: "User",
            },
            PanelRow {
                label: "Lens",
                value: "50mm",
            },
        ],
    },
    PanelAccordion {
        label: "Display",
        icon: "display",
        expanded: false,
        rows: &[PanelRow {
            label: "Show Overlays",
            value: "On",
        }],
    },
];

pub static EMPTY_ACCORDIONS: &[PanelAccordion] = &[];
