//! Radius scale, tokens, and role-map. Split out of `style.rs` so the parent
//! stays under the 200-line cap.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BRadiusScale {
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
}

pub const APPLE_RADII: BRadiusScale = BRadiusScale {
    xs: 4.0,
    sm: 7.0,
    md: 10.0,
    lg: 14.0,
    xl: 18.0,
};
pub const TIGHT_RADII: BRadiusScale = BRadiusScale {
    xs: 1.0,
    sm: 2.0,
    md: 4.0,
    lg: 6.0,
    xl: 8.0,
};

/// A semantic reference to one rung of [`BRadiusScale`].
///
/// Roles store tokens rather than copied numbers so later resolution can apply
/// a changed radius scale without rewriting the role catalog.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BRadiusToken {
    None,
    Xs,
    Sm,
    Md,
    Lg,
    Xl,
}

/// Visual purposes understood by the centralized UI-style model.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BStyleRole {
    Workspace,
    Zone,
    Area,
    Host,
    Region,
    Panel,
    Card,
    Button,
    Field,
    MenuItem,
    Tab,
    Popover,
    Modal,
}

impl BStyleRole {
    /// Exhaustive semantic-role catalog, in hierarchy-to-overlay order.
    pub const ALL: [Self; 13] = [
        Self::Workspace,
        Self::Zone,
        Self::Area,
        Self::Host,
        Self::Region,
        Self::Panel,
        Self::Card,
        Self::Button,
        Self::Field,
        Self::MenuItem,
        Self::Tab,
        Self::Popover,
        Self::Modal,
    ];
}

impl BRadiusScale {
    /// Resolves a declared radius token against this scale.
    pub const fn resolve(&self, token: BRadiusToken) -> f32 {
        match token {
            BRadiusToken::None => 0.0,
            BRadiusToken::Xs => self.xs,
            BRadiusToken::Sm => self.sm,
            BRadiusToken::Md => self.md,
            BRadiusToken::Lg => self.lg,
            BRadiusToken::Xl => self.xl,
        }
    }
}

/// Default radius-token ownership for each semantic visual role.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BRoleMap {
    pub workspace: BRadiusToken,
    pub zone: BRadiusToken,
    pub area: BRadiusToken,
    pub host: BRadiusToken,
    pub region: BRadiusToken,
    pub panel: BRadiusToken,
    pub card: BRadiusToken,
    pub button: BRadiusToken,
    pub field: BRadiusToken,
    pub menu_item: BRadiusToken,
    pub tab: BRadiusToken,
    pub popover: BRadiusToken,
    pub modal: BRadiusToken,
}

pub const DEFAULT_ROLES: BRoleMap = BRoleMap {
    // The workspace is the unrounded root. Region/panel are nested surfaces;
    // zone/area/host/card are outer surfaces. Controls, medium overlays, and
    // modal cards retain their deliberately distinct sm/md/xl treatments.
    workspace: BRadiusToken::None,
    zone: BRadiusToken::Lg,
    area: BRadiusToken::Lg,
    host: BRadiusToken::Lg,
    region: BRadiusToken::Md,
    panel: BRadiusToken::Md,
    card: BRadiusToken::Lg,
    button: BRadiusToken::Sm,
    field: BRadiusToken::Sm,
    menu_item: BRadiusToken::Sm,
    tab: BRadiusToken::Md,
    popover: BRadiusToken::Md,
    modal: BRadiusToken::Xl,
};

impl BRoleMap {
    /// Returns the declared token for a semantic role.
    pub const fn token_for(&self, role: BStyleRole) -> BRadiusToken {
        match role {
            BStyleRole::Workspace => self.workspace,
            BStyleRole::Zone => self.zone,
            BStyleRole::Area => self.area,
            BStyleRole::Host => self.host,
            BStyleRole::Region => self.region,
            BStyleRole::Panel => self.panel,
            BStyleRole::Card => self.card,
            BStyleRole::Button => self.button,
            BStyleRole::Field => self.field,
            BStyleRole::MenuItem => self.menu_item,
            BStyleRole::Tab => self.tab,
            BStyleRole::Popover => self.popover,
            BStyleRole::Modal => self.modal,
        }
    }

    /// Resolves a role through the supplied scale instead of copying a radius.
    pub const fn radius_for(&self, role: BStyleRole, scale: &BRadiusScale) -> f32 {
        scale.resolve(self.token_for(role))
    }
}
