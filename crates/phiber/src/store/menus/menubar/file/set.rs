//! File menu sets - labels, icons, shortcuts, and the top-level tree.

use super::def::*;
use crate::store::menus::MenuItem;

// New templates
static NEW_ITEMS: &[MenuItem] = &[
    MenuItem::node(FILE_NEW_GENERAL, FILE_NEW_GENERAL_ICON),
    MenuItem::node(FILE_NEW_2D_ANIMATION, FILE_NEW_2D_ANIMATION_ICON),
    MenuItem::node(FILE_NEW_SCULPTING, FILE_NEW_SCULPTING_ICON),
    MenuItem::node(FILE_NEW_VFX, FILE_NEW_VFX_ICON),
    MenuItem::node(FILE_NEW_VIDEO_EDITING, FILE_NEW_VIDEO_EDITING_ICON),
];

// Import formats
static IMPORT_ITEMS: &[MenuItem] = &[
    MenuItem::node(FILE_IMPORT_FBX, FILE_IMPORT_FBX_ICON),
    MenuItem::node(FILE_IMPORT_OBJ, FILE_IMPORT_OBJ_ICON),
    MenuItem::node(FILE_IMPORT_GLTF, FILE_IMPORT_GLTF_ICON),
    MenuItem::node(FILE_IMPORT_USD, FILE_IMPORT_USD_ICON),
    MenuItem::node(FILE_IMPORT_ALEMBIC, FILE_IMPORT_ALEMBIC_ICON),
];

// Export formats
static EXPORT_ITEMS: &[MenuItem] = &[
    MenuItem::node(FILE_EXPORT_FBX, FILE_EXPORT_FBX_ICON),
    MenuItem::node(FILE_EXPORT_OBJ, FILE_EXPORT_OBJ_ICON),
    MenuItem::node(FILE_EXPORT_GLTF, FILE_EXPORT_GLTF_ICON),
    MenuItem::node(FILE_EXPORT_USD, FILE_EXPORT_USD_ICON),
    MenuItem::node(FILE_EXPORT_ALEMBIC, FILE_EXPORT_ALEMBIC_ICON),
    MenuItem::node(FILE_EXPORT_ALL, FILE_EXPORT_ALL_ICON),
];

// External data management
static EXTERNAL_DATA_ITEMS: &[MenuItem] = &[
    MenuItem::node(FILE_PACK_AUTOMATICALLY, FILE_PACK_AUTOMATICALLY_ICON),
    MenuItem::node(FILE_PACK_ALL, FILE_PACK_ALL_ICON),
    MenuItem::node(FILE_UNPACK_ALL, FILE_UNPACK_ALL_ICON),
    MenuItem::node(FILE_PACK_LINKED, FILE_PACK_LINKED_ICON),
    MenuItem::node(FILE_UNPACK_LINKED, FILE_UNPACK_LINKED_ICON),
    MenuItem::node(FILE_MAKE_PATHS_RELATIVE, FILE_MAKE_PATHS_RELATIVE_ICON),
    MenuItem::node(FILE_MAKE_PATHS_ABSOLUTE, FILE_MAKE_PATHS_ABSOLUTE_ICON),
    MenuItem::node(FILE_REPORT_MISSING_FILES, FILE_REPORT_MISSING_FILES_ICON),
];

// Top-level File menu
pub static ITEMS: &[MenuItem] = &[
    MenuItem::node(FILE_NEW, FILE_NEW_ICON).with_items(NEW_ITEMS),
    MenuItem::separator(),
    MenuItem::node(FILE_OPEN, FILE_OPEN_ICON).with_shortcut(FILE_OPEN_SHORTCUT),
    MenuItem::node(FILE_OPEN_RECENT, FILE_OPEN_RECENT_ICON),
    MenuItem::separator(),
    MenuItem::node(FILE_RECOVER_LAST_SESSION, FILE_RECOVER_LAST_SESSION_ICON),
    MenuItem::node(FILE_RECOVER_AUTO_SAVE, FILE_RECOVER_AUTO_SAVE_ICON),
    MenuItem::node(FILE_REVERT, FILE_REVERT_ICON),
    MenuItem::separator(),
    MenuItem::node(FILE_SAVE, FILE_SAVE_ICON).with_shortcut(FILE_SAVE_SHORTCUT),
    MenuItem::node(FILE_SAVE_AS, FILE_SAVE_AS_ICON).with_shortcut(FILE_SAVE_AS_SHORTCUT),
    MenuItem::node(FILE_SAVE_COPY, FILE_SAVE_COPY_ICON),
    MenuItem::node(FILE_SAVE_INCREMENTAL, FILE_SAVE_INCREMENTAL_ICON),
    MenuItem::separator(),
    MenuItem::node(FILE_LINK, FILE_LINK_ICON),
    MenuItem::node(FILE_APPEND, FILE_APPEND_ICON),
    MenuItem::separator(),
    MenuItem::node(
        FILE_BATCH_GENERATE_PREVIEWS,
        FILE_BATCH_GENERATE_PREVIEWS_ICON,
    ),
    MenuItem::node(FILE_CLEAR_PREVIEWS, FILE_CLEAR_PREVIEWS_ICON),
    MenuItem::separator(),
    MenuItem::node(FILE_IMPORT, FILE_IMPORT_ICON).with_items(IMPORT_ITEMS),
    MenuItem::node(FILE_EXPORT, FILE_EXPORT_ICON).with_items(EXPORT_ITEMS),
    MenuItem::separator(),
    MenuItem::node(FILE_EXTERNAL_DATA, FILE_EXTERNAL_DATA_ICON).with_items(EXTERNAL_DATA_ITEMS),
    MenuItem::separator(),
    MenuItem::node(FILE_SAVE_STARTUP, FILE_SAVE_STARTUP_ICON),
    MenuItem::node(FILE_LOAD_FACTORY, FILE_LOAD_FACTORY_ICON),
    MenuItem::separator(),
    MenuItem::node(FILE_QUIT, FILE_QUIT_ICON).with_shortcut(FILE_QUIT_SHORTCUT),
];
