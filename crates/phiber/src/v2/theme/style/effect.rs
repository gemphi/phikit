//! Effects surface: shadow style + `BEffects` + palette presets. Split out
//! of `style.rs` so the parent stays under the 200-line cap.

/// How far a lifted surface — a popover, a modal card — casts. Blender
/// casts none (`widget_menu_back` draws a flat backdrop); the softer themes
/// cast further.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BShadowStyle {
    None,
    Small,
    Medium,
    Large,
    ExtraLarge,
}

impl BShadowStyle {
    pub const ALL: [Self; 5] = [Self::None, Self::Small, Self::Medium, Self::Large, Self::ExtraLarge];

    /// This rung's position, counting `None` as 0.
    pub(super) const fn rung(self) -> u8 {
        match self {
            Self::None => 0,
            Self::Small => 1,
            Self::Medium => 2,
            Self::Large => 3,
            Self::ExtraLarge => 4,
        }
    }

    const fn at_rung(rung: u8) -> Self {
        match rung {
            0 => Self::None,
            1 => Self::Small,
            2 => Self::Medium,
            3 => Self::Large,
            _ => Self::ExtraLarge,
        }
    }

    /// This rung, `by` steps softer — `None` once it runs out. `None` stays
    /// `None`, so no elevation can make a flat theme cast (`style/elevation`).
    pub const fn softer(self, by: u8) -> Self {
        let rung = self.rung();
        if by >= rung { Self::None } else { Self::at_rung(rung - by) }
    }

    /// This rung or `floor`, whichever is harder — a shadow floor. The
    /// spatial coat casts at least this far without pulling a theme that
    /// already casts further down to it (`style/flavour`).
    pub const fn at_least(self, floor: Self) -> Self {
        if self.rung() >= floor.rung() { self } else { floor }
    }

    /// The value a preference stores.
    pub const fn slug(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Small => "small",
            Self::Medium => "medium",
            Self::Large => "large",
            Self::ExtraLarge => "extra_large",
        }
    }

    /// The label a preference shows.
    pub const fn label(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Small => "Small",
            Self::Medium => "Medium",
            Self::Large => "Large",
            Self::ExtraLarge => "Extra Large",
        }
    }

    pub fn from_slug(slug: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|style| style.slug() == slug)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BEffects {
    pub gradients: bool,
    pub gradient_lift: f32,
    pub sheen_lift: f32,
    pub selected_hover_lift: f32,
    pub footer_lift: f32,
    pub sidebar_lift: f32,
    pub card_lift: f32,
    pub section_lift: f32,
    pub strip_recess: f32,
    pub strip_gradient_lift: f32,
    pub tab_accent_mix: f32,
    pub tab_saturation_lift: f32,
    pub tab_top_lift: f32,
    pub viewport_lightness_delta: f32,
    /// How opaque floating chrome is over a canvas — the toolbar, the
    /// sidebar and the viewport gizmos. Near solid: a strip reads as a panel
    /// laid over the scene, with just a hint of it through. It was 0.36,
    /// which left the strips as bare buttons on the canvas with no bar
    /// behind them.
    pub floating_alpha: f32,
    /// A control's fill is the accent at an opacity, and only the opacity
    /// changes with state: resting (floating chrome only), hovered, active.
    /// One hue at three strengths is what keeps a toolbar, a sidebar and an
    /// activity rail reading as one family instead of three shades of grey.
    pub control_rest_alpha: f32,
    pub control_hover_alpha: f32,
    pub control_active_alpha: f32,
    /// The weight a control not in force paints at — its fill and its ink
    /// alike (`chrome::paint_state`, `button.rs`): on and not available are
    /// two looks, the fill still saying "on" through the fade.
    pub control_muted_alpha: f32,
    /// How long a fold, unfold or slide takes, in milliseconds. One duration
    /// for every motion in the window, so nothing feels faster than the rest.
    pub motion_ms: f32,
    pub viewport_grid_alpha: f32,
    pub axis_alpha: f32,
    /// A hint drawn beside the controls it annotates — a panel's grip.
    /// Blender draws `ICON_GRIP` at 0.5 (`panel_draw_aligned_widgets`).
    pub grip_alpha: f32,
    pub node_grid_lift: f32,
    pub node_grid_alpha: f32,
    pub image_light_lift: f32,
    pub image_dark_lift: f32,
    pub surface_shadow: BShadowStyle,
}

const fn effects(gradients: bool, gradient_lift: f32, surface_shadow: BShadowStyle) -> BEffects {
    BEffects {
        gradients,
        gradient_lift,
        sheen_lift: 0.02,
        selected_hover_lift: 0.06,
        footer_lift: -0.02,
        sidebar_lift: -0.04,
        card_lift: -0.03,
        section_lift: 0.035,
        strip_recess: 2.0,
        strip_gradient_lift: -0.02,
        tab_accent_mix: 0.25,
        tab_saturation_lift: 0.06,
        tab_top_lift: 0.02,
        // No lift: the canvas is the palette's own viewport tone. Brightening
        // it washed the floor grid and the axes out, and the separation that
        // lift was buying is better bought with `viewport_grid_alpha`.
        viewport_lightness_delta: 0.0,
        floating_alpha: 0.96,
        // Half: a resting control's ground is half the weight of the shade
        // it is cut from, so a row reads as a row and not as a second panel.
        control_rest_alpha: 0.5,
        control_hover_alpha: 0.22,
        control_active_alpha: 0.42,
        control_muted_alpha: 0.45,
        motion_ms: 180.0,
        // The grid is drawn on the canvas, which is a near neighbour of it in
        // tone, so it needs enough alpha to separate.
        viewport_grid_alpha: 0.16,
        axis_alpha: 0.65,
        grip_alpha: 0.5,
        node_grid_lift: 0.12,
        node_grid_alpha: 0.18,
        image_light_lift: 0.09,
        image_dark_lift: -0.03,
        surface_shadow,
    }
}

/// One set per theme: whether its surfaces sweep, how far, and how far its
/// lifted surfaces cast. The shadow is the theme's *ceiling* — what a modal
/// casts; every level under it is a rung softer (`style/elevation`). Graphite,
/// Ember, Silver and Cyberpunk are flat, as Blender is.
pub const SPHERE_EFFECTS: BEffects = effects(true, 0.035, BShadowStyle::ExtraLarge);
pub const GRAPHITE_EFFECTS: BEffects = effects(false, 0.0, BShadowStyle::None);
pub const TOKYO_EFFECTS: BEffects = effects(true, 0.025, BShadowStyle::Medium);
pub const MOSS_EFFECTS: BEffects = effects(true, 0.05, BShadowStyle::Large);
pub const EMBER_EFFECTS: BEffects = effects(false, 0.0, BShadowStyle::Small);
pub const PLUM_EFFECTS: BEffects = effects(true, 0.03, BShadowStyle::ExtraLarge);
pub const ONYX_EFFECTS: BEffects = effects(true, 0.02, BShadowStyle::Large);
pub const GREY_EFFECTS: BEffects = effects(true, 0.03, BShadowStyle::Medium);
pub const SILVER_EFFECTS: BEffects = effects(false, 0.0, BShadowStyle::Small);
pub const APPLE_EFFECTS: BEffects = effects(false, 0.0, BShadowStyle::Medium);
pub const APPLE_DARK_EFFECTS: BEffects = effects(false, 0.0, BShadowStyle::Medium);
pub const SWIFT_EFFECTS: BEffects = effects(true, 0.03, BShadowStyle::Large);
pub const BARBIE_EFFECTS: BEffects = effects(true, 0.04, BShadowStyle::ExtraLarge);
pub const FENTY_EFFECTS: BEffects = effects(true, 0.035, BShadowStyle::Large);
pub const CYBERPUNK_EFFECTS: BEffects = effects(false, 0.0, BShadowStyle::None);
pub const CHANEL_EFFECTS: BEffects = effects(true, 0.02, BShadowStyle::Medium);
