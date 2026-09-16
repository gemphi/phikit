//! Channels set - host-to-channels association and channel arrays.

use super::ChannelEntry;
use super::def;
use crate::store::hosts::BHostType;

pub static TIME_CHANNELS: &[ChannelEntry] = &[
    ChannelEntry {
        idx: def::CHANNEL_CUBE,
        icon: def::ICON_OBJECT,
        selected: true,
        locked: false,
    },
    ChannelEntry {
        idx: def::CHANNEL_CAMERA,
        icon: def::ICON_OBJECT,
        selected: false,
        locked: false,
    },
    ChannelEntry {
        idx: def::CHANNEL_LIGHT,
        icon: def::ICON_LIGHT,
        selected: false,
        locked: false,
    },
];

pub static NLA_CHANNELS: &[ChannelEntry] = &[
    ChannelEntry {
        idx: def::CHANNEL_CUBE_ACTION,
        icon: def::ICON_ACTION,
        selected: true,
        locked: false,
    },
    ChannelEntry {
        idx: def::CHANNEL_WALK_ACTION,
        icon: def::ICON_ACTION,
        selected: false,
        locked: false,
    },
];

// File browser source list: System bookmarks, then volumes, then recent.
pub static FILE_CHANNELS: &[ChannelEntry] = &[
    ChannelEntry {
        idx: def::BOOKMARK_HOME,
        icon: def::ICON_BOOKMARK,
        selected: false,
        locked: false,
    },
    ChannelEntry {
        idx: def::BOOKMARK_DESKTOP,
        icon: def::ICON_FOLDER,
        selected: false,
        locked: false,
    },
    ChannelEntry {
        idx: def::BOOKMARK_DOCUMENTS,
        icon: def::ICON_FOLDER,
        selected: false,
        locked: false,
    },
    ChannelEntry {
        idx: def::BOOKMARK_DOWNLOADS,
        icon: def::ICON_FOLDER,
        selected: true,
        locked: false,
    },
    ChannelEntry {
        idx: def::BOOKMARK_MUSIC,
        icon: def::ICON_FOLDER,
        selected: false,
        locked: false,
    },
    ChannelEntry {
        idx: def::BOOKMARK_PICTURES,
        icon: def::ICON_FOLDER,
        selected: false,
        locked: false,
    },
    ChannelEntry {
        idx: def::BOOKMARK_VIDEOS,
        icon: def::ICON_FOLDER,
        selected: false,
        locked: false,
    },
    ChannelEntry {
        idx: def::BOOKMARK_FONTS,
        icon: def::ICON_FOLDER,
        selected: false,
        locked: false,
    },
    ChannelEntry {
        idx: def::VOLUME_C,
        icon: def::ICON_DRIVE,
        selected: false,
        locked: false,
    },
    ChannelEntry {
        idx: def::VOLUME_D,
        icon: def::ICON_DRIVE,
        selected: false,
        locked: false,
    },
];

pub static DEFAULT_CHANNELS: &[ChannelEntry] = &[];

/// Channel entries for a given host.
pub fn with_idx(host: BHostType) -> &'static [super::ChannelEntry] {
    match host {
        BHostType::Time => TIME_CHANNELS,
        BHostType::Nla | BHostType::Action => NLA_CHANNELS,
        BHostType::File => FILE_CHANNELS,
        _ => DEFAULT_CHANNELS,
    }
}
