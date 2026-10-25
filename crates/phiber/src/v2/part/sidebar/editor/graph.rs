//! The Graph and Drivers editors' View, F-Curve, Modifiers and Drivers tabs (`editor`).

use crate::v2::part::props::prop::{BEnumItem, BPropSpec, BUnit, b, e, en, f, i, s};
use crate::v2::part::props::set::{BSectionSpec, sec};

static GRAPH_VIEW: &[BPropSpec] = &[
    b("space_data.show_cursor", "Show Cursor"),
    f("space_data.cursor_position_x", "Cursor X", BUnit::None),
    f("space_data.cursor_position_y", "Cursor Y", BUnit::None),
];

pub static IPO_VIEW: &[BSectionSpec] = &[sec("View", GRAPH_VIEW)];

static COLOR_MODE: &[BEnumItem] = &[en("AUTO_RAINBOW", "Auto Rainbow"), en("AUTO_RGB", "Auto XYZ to RGB"), en("AUTO_YRGB", "Auto WXYZ to YRGB"), en("CUSTOM", "User Defined")];
static SMOOTHING: &[BEnumItem] = &[en("NONE", "None"), en("CONT_ACCEL", "Continuous Acceleration")];
static INTERPOLATION: &[BEnumItem] = &[en("CONSTANT", "Constant"), en("LINEAR", "Linear"), en("BEZIER", "Bézier")];
static EASING: &[BEnumItem] = &[en("AUTO", "Automatic Easing"), en("EASE_IN", "Ease In"), en("EASE_OUT", "Ease Out"), en("EASE_IN_OUT", "Ease In and Out")];
static HANDLE: &[BEnumItem] = &[en("FREE", "Free"), en("ALIGNED", "Aligned"), en("VECTOR", "Vector"), en("AUTO", "Automatic"), en("AUTO_CLAMPED", "Auto Clamped")];

static FCURVE: &[BPropSpec] = &[
    s("fcurve.data_path", "Data Path"),
    i("fcurve.array_index", "RNA Array Index", 0, 63),
    e("fcurve.color_mode", "Color Mode", COLOR_MODE),
    e("fcurve.auto_smoothing", "Auto Handle Smoothing", SMOOTHING),
];

static KEY: &[BPropSpec] = &[
    e("keyframe.interpolation", "Interpolation", INTERPOLATION),
    e("keyframe.easing", "Easing", EASING),
    f("keyframe.co_ui_x", "Key Frame", BUnit::None),
    f("keyframe.co_ui_y", "Value", BUnit::None),
    e("keyframe.handle_left_type", "Left Handle Type", HANDLE),
    e("keyframe.handle_right_type", "Right Handle Type", HANDLE),
];

pub static IPO_FCURVE: &[BSectionSpec] = &[sec("F-Curve", FCURVE), sec("Active Keyframe", KEY)];

static FMODIFIER: &[BEnumItem] = &[en("GENERATOR", "Generator"), en("FNGENERATOR", "Built-In Function"), en("ENVELOPE", "Envelope"), en("CYCLES", "Cycles"), en("NOISE", "Noise"), en("LIMITS", "Limits"), en("STEPPED", "Stepped Interpolation")];

static MODIFIER_ADD: &[BPropSpec] = &[e("fcurve.modifier_add", "Add Modifier", FMODIFIER), b("fcurve.modifiers.only_active", "Only Active")];

pub static IPO_MODIFIERS: &[BSectionSpec] = &[sec("Modifiers", MODIFIER_ADD)];

static DRIVER_TYPE: &[BEnumItem] = &[en("AVERAGE", "Averaged Value"), en("SUM", "Sum Values"), en("SCRIPTED", "Scripted Expression"), en("MIN", "Minimum Value"), en("MAX", "Maximum Value")];

static DRIVEN: &[BPropSpec] = &[s("fcurve.data_path", "Driven Property"), i("fcurve.array_index", "Index", 0, 63)];

static DRIVER: &[BPropSpec] = &[
    e("driver.type", "Type", DRIVER_TYPE),
    s("driver.expression", "Expression"),
    b("driver.use_self", "Use Self"),
];

pub static IPO_DRIVERS: &[BSectionSpec] = &[sec("Driven Property", DRIVEN), sec("Driver", DRIVER)];
