//! Modebar set - host-to-modebar association and tab arrays.

use super::ModeTab;
use super::def;
use crate::store::hosts::BHostType;

pub static VIEW3D_MODES: &[ModeTab] = &[
    ModeTab {
        label: def::MODE_OBJECT,
        icon: def::ICON_OBJECT,
        active: true,
    },
    ModeTab {
        label: def::MODE_EDIT,
        icon: def::ICON_EDIT,
        active: false,
    },
    ModeTab {
        label: def::MODE_SCULPT,
        icon: def::ICON_SCULPT,
        active: false,
    },
    ModeTab {
        label: def::MODE_UV_EDITING,
        icon: def::ICON_UV,
        active: false,
    },
    ModeTab {
        label: def::MODE_TEXTURE_PAINT,
        icon: def::ICON_TEXTURE,
        active: false,
    },
    ModeTab {
        label: def::MODE_WEIGHT_PAINT,
        icon: def::ICON_SLIDERS,
        active: false,
    },
    ModeTab {
        label: def::MODE_VERTEX_PAINT,
        icon: def::ICON_PAINTBRUSH,
        active: false,
    },
    ModeTab {
        label: def::MODE_POSE,
        icon: def::ICON_POSE,
        active: false,
    },
];

pub static IMAGE_MODES: &[ModeTab] = &[
    ModeTab {
        label: def::MODE_VIEW,
        icon: def::ICON_VIEW,
        active: true,
    },
    ModeTab {
        label: def::MODE_UV_EDITOR,
        icon: def::ICON_UV,
        active: false,
    },
    ModeTab {
        label: def::MODE_PAINT,
        icon: def::ICON_PAINTBRUSH,
        active: false,
    },
    ModeTab {
        label: def::MODE_MASK,
        icon: def::ICON_MASK,
        active: false,
    },
];

pub static NODE_MODES: &[ModeTab] = &[
    ModeTab {
        label: def::MODE_VIEW,
        icon: def::ICON_VIEW,
        active: true,
    },
    ModeTab {
        label: def::MODE_EDIT,
        icon: def::ICON_EDIT,
        active: false,
    },
];

pub static DEFAULT_MODES: &[ModeTab] = &[ModeTab {
    label: def::MODE_VIEW,
    icon: def::ICON_VIEW,
    active: true,
}];

/// Mode tabs for a given host.
pub fn with_idx(host: BHostType) -> &'static [super::ModeTab] {
    match host {
        BHostType::View3D | BHostType::Clip => VIEW3D_MODES,
        BHostType::Image | BHostType::Uv => IMAGE_MODES,
        BHostType::Node | BHostType::Compositor | BHostType::Shader | BHostType::GeoNode => {
            NODE_MODES
        }
        _ => DEFAULT_MODES,
    }
}
