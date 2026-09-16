//! Custom set - host-to-custom-renderer association and render-key table.

use super::def;
use crate::store::hosts::BHostType;

pub static RENDER_KEYS: &[(BHostType, &str)] = &[
    (BHostType::View3D, def::RENDER_KEY_VIEWPORT_3D),
    (BHostType::Outliner, def::RENDER_KEY_OUTLINER_TREE),
    (BHostType::Buts, def::RENDER_KEY_PROPERTY_PANELS),
    (BHostType::Time, def::RENDER_KEY_TIMELINE_RULER),
    (BHostType::Action, def::RENDER_KEY_DOPESHEET_GRID),
    (BHostType::Nla, def::RENDER_KEY_NLA_TRACKS),
    (BHostType::Seq, def::RENDER_KEY_SEQUENCER_STRIPS),
    (BHostType::Node, def::RENDER_KEY_NODE_GRAPH),
    (BHostType::Shader, def::RENDER_KEY_NODE_GRAPH),
    (BHostType::GeoNode, def::RENDER_KEY_NODE_GRAPH),
    (BHostType::Compositor, def::RENDER_KEY_NODE_GRAPH),
    (BHostType::Image, def::RENDER_KEY_IMAGE_VIEW),
    (BHostType::Uv, def::RENDER_KEY_IMAGE_VIEW),
    (BHostType::Text, def::RENDER_KEY_TEXT_EDITOR),
    (BHostType::Console, def::RENDER_KEY_PYTHON_CONSOLE),
    (BHostType::Clip, def::RENDER_KEY_CLIP_VIEW),
    (BHostType::Spreadsheet, def::RENDER_KEY_SPREADSHEET_TABLE),
    (BHostType::Asset, def::RENDER_KEY_ASSET_GRID),
    (BHostType::File, def::RENDER_KEY_FILE_LIST),
    (BHostType::Info, def::RENDER_KEY_INFO_LOG),
    (BHostType::Topbar, def::RENDER_KEY_TOPBAR),
    (BHostType::Statusbar, def::RENDER_KEY_STATUSBAR),
    (BHostType::Empty, def::RENDER_KEY_EMPTY),
    (BHostType::Logic, def::RENDER_KEY_EMPTY),
    (BHostType::Drivers, def::RENDER_KEY_EMPTY),
    (BHostType::Prefs, def::RENDER_KEY_EMPTY),
];

/// Render dispatch key for a given host.
pub fn render_key_for(host: BHostType) -> &'static str {
    RENDER_KEYS
        .iter()
        .find(|(h, _)| *h == host)
        .map(|(_, k)| *k)
        .unwrap_or(def::RENDER_KEY_EMPTY)
}

// ── Per-host center content (Blender default startup scene) ────────────────

use super::def::BContent;

static VIEW3D_CONTENT: &[BContent] = &[BContent::Spatial {
    name: "Cube",
    objects: 1,
    vertices: 8,
}];

// Blender default outliner: Scene Collection → Collection → objects (+ data).
static OUTLINER_CONTENT: &[BContent] = &[BContent::Scroll(&[BContent::Section {
    label: "Scene Collection",
    icon: Some("outliner"),
    expanded: true,
    items: &[BContent::Section {
        label: "Collection",
        icon: Some("collection"),
        expanded: true,
        items: &[
            BContent::Item {
                label: "Camera",
                icon: "camera",
                active: false,
            },
            BContent::Section {
                label: "Cube",
                icon: Some("cube"),
                expanded: true,
                items: &[BContent::Item {
                    label: "Cube",
                    icon: "mesh",
                    active: true,
                }],
            },
            BContent::Item {
                label: "Light",
                icon: "light",
                active: false,
            },
        ],
    }],
}])];

static BUTS_CONTENT: &[BContent] = &[BContent::Scroll(&[
    BContent::PropRow {
        label: "Name",
        value: "Cube",
    },
    BContent::PropRow {
        label: "Type",
        value: "Mesh",
    },
    BContent::PropRow {
        label: "Vertices",
        value: "8",
    },
])];

// Blender file browser list view: column header + name | date | size rows.
static FILE_CONTENT: &[BContent] = &[
    BContent::Row(&[
        BContent::Label {
            text: "Name",
            icon: Some("file"),
        },
        BContent::Label {
            text: "Date Modified",
            icon: None,
        },
        BContent::Label {
            text: "Size",
            icon: None,
        },
    ]),
    BContent::Separator,
    BContent::Scroll(&[
        BContent::Row(&[
            BContent::Item {
                label: "Female-02--Charry-Whims-Rigged--Lace.blend",
                icon: "file",
                active: true,
            },
            BContent::Label {
                text: "29 Jun 2026 23:06",
                icon: None,
            },
            BContent::Label {
                text: "77.3 MiB",
                icon: None,
            },
        ]),
        BContent::Row(&[
            BContent::Item {
                label: "Female-02--Charry-Whims-Rigged--Lace--Copy.blend",
                icon: "file",
                active: false,
            },
            BContent::Label {
                text: "29 Jun 2026 23:06",
                icon: None,
            },
            BContent::Label {
                text: "77.3 MiB",
                icon: None,
            },
        ]),
    ]),
];

static IMAGE_CONTENT: &[BContent] = &[BContent::Column(&[
    BContent::Label {
        text: "Render Result",
        icon: Some("image"),
    },
    BContent::Separator,
    BContent::PropRow {
        label: "Size",
        value: "1920 x 1080",
    },
    BContent::PropRow {
        label: "Channels",
        value: "RGBA",
    },
])];

static INFO_CONTENT: &[BContent] = &[BContent::Scroll(&[
    BContent::Item {
        label: "Ready",
        icon: "info",
        active: false,
    },
    BContent::Item {
        label: "Saved 'scene.blend'",
        icon: "check",
        active: false,
    },
    BContent::Item {
        label: "Loaded startup file",
        icon: "info",
        active: false,
    },
])];

static SEQ_CONTENT: &[BContent] = &[BContent::Scroll(&[
    BContent::Item {
        label: "Channel 3 — strip.mov",
        icon: "seq",
        active: false,
    },
    BContent::Item {
        label: "Channel 2 — title.png",
        icon: "seq",
        active: false,
    },
    BContent::Item {
        label: "Channel 1 — audio.wav",
        icon: "seq",
        active: false,
    },
])];

static TEXT_CONTENT: &[BContent] = &[BContent::Scroll(&[
    BContent::Label {
        text: "1  import bpy",
        icon: None,
    },
    BContent::Label {
        text: "2  ",
        icon: None,
    },
    BContent::Label {
        text: "3  obj = bpy.context.object",
        icon: None,
    },
    BContent::Label {
        text: "4  obj.location = (0.0, 0.0, 0.0)",
        icon: None,
    },
])];

static ACTION_CONTENT: &[BContent] = &[BContent::Scroll(&[BContent::Section {
    label: "Object Action",
    icon: None,
    expanded: true,
    items: &[
        BContent::Item {
            label: "Location",
            icon: "keyframe",
            active: false,
        },
        BContent::Item {
            label: "Rotation",
            icon: "keyframe",
            active: false,
        },
        BContent::Item {
            label: "Scale",
            icon: "keyframe",
            active: false,
        },
    ],
}])];

static NLA_CONTENT: &[BContent] = &[BContent::Scroll(&[
    BContent::Item {
        label: "Cube — Action.001",
        icon: "nla",
        active: false,
    },
    BContent::Item {
        label: "Camera — CameraAction",
        icon: "nla",
        active: false,
    },
])];

static TIME_CONTENT: &[BContent] = &[BContent::Column(&[
    BContent::Label {
        text: "1     10    20    30    40    50    60    70    80    90   100",
        icon: None,
    },
    BContent::Separator,
    BContent::PropRow {
        label: "Frame",
        value: "1 / 250",
    },
    BContent::PropRow {
        label: "Playback",
        value: "Stopped",
    },
])];

static NODE_CONTENT: &[BContent] = &[BContent::Section {
    label: "Node Tree",
    icon: None,
    expanded: true,
    items: &[
        BContent::Item {
            label: "Texture Coordinate",
            icon: "node",
            active: false,
        },
        BContent::Item {
            label: "Principled BSDF",
            icon: "node",
            active: false,
        },
        BContent::Item {
            label: "Material Output",
            icon: "node",
            active: false,
        },
    ],
}];

static LOGIC_CONTENT: &[BContent] = &[BContent::Scroll(&[
    BContent::Section {
        label: "Sensors",
        icon: None,
        expanded: true,
        items: &[BContent::Item {
            label: "Always",
            icon: "logic",
            active: false,
        }],
    },
    BContent::Section {
        label: "Controllers",
        icon: None,
        expanded: true,
        items: &[BContent::Item {
            label: "And",
            icon: "logic",
            active: false,
        }],
    },
    BContent::Section {
        label: "Actuators",
        icon: None,
        expanded: true,
        items: &[BContent::Item {
            label: "Motion",
            icon: "logic",
            active: false,
        }],
    },
])];

static CONSOLE_CONTENT: &[BContent] = &[BContent::Scroll(&[
    BContent::Label {
        text: "PYTHON INTERACTIVE CONSOLE 3.11",
        icon: None,
    },
    BContent::Label {
        text: ">>> import bpy",
        icon: None,
    },
    BContent::Label {
        text: ">>> bpy.context.object",
        icon: None,
    },
    BContent::Label {
        text: "<bpy_struct, Object(\"Cube\")>",
        icon: None,
    },
    BContent::Label {
        text: ">>> ",
        icon: None,
    },
])];

static UV_CONTENT: &[BContent] = &[BContent::Column(&[
    BContent::Label {
        text: "UV Editor",
        icon: Some("uv"),
    },
    BContent::Separator,
    BContent::PropRow {
        label: "UV Islands",
        value: "8",
    },
    BContent::PropRow {
        label: "Selected",
        value: "0",
    },
])];

static COMPOSITOR_CONTENT: &[BContent] = &[BContent::Section {
    label: "Compositor Nodes",
    icon: None,
    expanded: true,
    items: &[
        BContent::Item {
            label: "Render Layers",
            icon: "node",
            active: false,
        },
        BContent::Item {
            label: "Composite",
            icon: "node",
            active: false,
        },
    ],
}];

static SHADER_CONTENT: &[BContent] = &[BContent::Section {
    label: "Shader Nodes",
    icon: None,
    expanded: true,
    items: &[
        BContent::Item {
            label: "Principled BSDF",
            icon: "node",
            active: false,
        },
        BContent::Item {
            label: "Material Output",
            icon: "node",
            active: false,
        },
    ],
}];

static GEONODE_CONTENT: &[BContent] = &[BContent::Section {
    label: "Geometry Nodes",
    icon: None,
    expanded: true,
    items: &[
        BContent::Item {
            label: "Group Input",
            icon: "node",
            active: false,
        },
        BContent::Item {
            label: "Group Output",
            icon: "node",
            active: false,
        },
    ],
}];

static CLIP_CONTENT: &[BContent] = &[BContent::Column(&[
    BContent::Label {
        text: "Movie Clip Editor",
        icon: Some("clip"),
    },
    BContent::Separator,
    BContent::PropRow {
        label: "Trackers",
        value: "0",
    },
    BContent::PropRow {
        label: "Frame",
        value: "1 / 250",
    },
])];

static DRIVERS_CONTENT: &[BContent] = &[BContent::Scroll(&[
    BContent::PropRow {
        label: "Driver",
        value: "Cube.location[0]",
    },
    BContent::PropRow {
        label: "Expression",
        value: "frame / 24.0",
    },
    BContent::PropRow {
        label: "Type",
        value: "Scripted Expression",
    },
])];

static ASSET_CONTENT: &[BContent] = &[BContent::Scroll(&[
    BContent::Section {
        label: "Poses",
        icon: None,
        expanded: true,
        items: &[
            BContent::Item {
                label: "Pose 01",
                icon: "user",
                active: false,
            },
            BContent::Item {
                label: "Pose 02",
                icon: "user",
                active: false,
            },
        ],
    },
    BContent::Section {
        label: "Materials",
        icon: None,
        expanded: true,
        items: &[
            BContent::Item {
                label: "Gold",
                icon: "circle",
                active: false,
            },
            BContent::Item {
                label: "Glass",
                icon: "circle",
                active: false,
            },
        ],
    },
])];

static SPREADSHEET_CONTENT: &[BContent] = &[BContent::Scroll(&[
    BContent::PropRow {
        label: "Name",
        value: "Value",
    },
    BContent::Separator,
    BContent::PropRow {
        label: "Position",
        value: "0.0, 0.0, 0.0",
    },
    BContent::PropRow {
        label: "Rotation",
        value: "0.0, 0.0, 0.0",
    },
    BContent::PropRow {
        label: "Scale",
        value: "1.0, 1.0, 1.0",
    },
    BContent::PropRow {
        label: "Vertices",
        value: "8",
    },
])];

static IPO_CONTENT: &[BContent] = &[BContent::Scroll(&[BContent::Section {
    label: "Cube — Action",
    icon: None,
    expanded: true,
    items: &[
        BContent::Item {
            label: "X Location",
            icon: "graph",
            active: false,
        },
        BContent::Item {
            label: "Y Location",
            icon: "graph",
            active: false,
        },
        BContent::Item {
            label: "Z Location",
            icon: "graph",
            active: false,
        },
    ],
}])];

static PREFS_CONTENT: &[BContent] = &[BContent::Scroll(&[
    BContent::Section {
        label: "Interface",
        icon: None,
        expanded: true,
        items: &[
            BContent::PropRow {
                label: "Theme",
                value: "Blender Dark",
            },
            BContent::PropRow {
                label: "UI Scale",
                value: "1.0",
            },
        ],
    },
    BContent::Section {
        label: "Keymap",
        icon: None,
        expanded: true,
        items: &[BContent::PropRow {
            label: "Preset",
            value: "Blender",
        }],
    },
])];

/// Center content for a given host — pure data match.
pub fn content_for(host: BHostType) -> &'static [BContent] {
    match host {
        BHostType::View3D => VIEW3D_CONTENT,
        BHostType::Outliner => OUTLINER_CONTENT,
        BHostType::Buts => BUTS_CONTENT,
        BHostType::File => FILE_CONTENT,
        BHostType::Image => IMAGE_CONTENT,
        BHostType::Info => INFO_CONTENT,
        BHostType::Seq => SEQ_CONTENT,
        BHostType::Text => TEXT_CONTENT,
        BHostType::Action => ACTION_CONTENT,
        BHostType::Nla => NLA_CONTENT,
        BHostType::Time => TIME_CONTENT,
        BHostType::Node => NODE_CONTENT,
        BHostType::Logic => LOGIC_CONTENT,
        BHostType::Console => CONSOLE_CONTENT,
        BHostType::Uv => UV_CONTENT,
        BHostType::Compositor => COMPOSITOR_CONTENT,
        BHostType::Shader => SHADER_CONTENT,
        BHostType::GeoNode => GEONODE_CONTENT,
        BHostType::Clip => CLIP_CONTENT,
        BHostType::Drivers => DRIVERS_CONTENT,
        BHostType::Asset => ASSET_CONTENT,
        BHostType::Spreadsheet => SPREADSHEET_CONTENT,
        BHostType::Ipo => IPO_CONTENT,
        BHostType::Prefs => PREFS_CONTENT,
        _ => &[],
    }
}
