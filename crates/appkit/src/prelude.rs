//! Convenience re-exports for common AppKit types.

pub use crate::alert::NSAlert;
pub use crate::application::{NSAppDelegate, NSApplication};
pub use crate::box_view::NSBox;
pub use crate::button::{NSBezelStyle, NSButton, NSButtonType};
pub use crate::collection_view::NSCollectionView;
pub use crate::ecs::*;
pub use crate::graphics::NSImage;
pub use crate::graphics::{
    NSAutoresizingMask, NSColor, NSEdgeInsets, NSLayoutAttribute, NSLayoutRelation, NSPoint,
    NSRect, NSSize, NSUserInterfaceLayoutOrientation,
};
pub use crate::image_view::NSImageView;
pub use crate::math::*;
pub use crate::outline_view::{NSOutlineView, NSOutlineViewDataSource};
pub use crate::render::RenderContext;
pub use crate::scroll_view::NSScrollView;
pub use crate::search_field::NSSearchField;
pub use crate::segmented_control::NSSegmentedControl;
pub use crate::split_view::{NSSplitView, NSSplitViewDividerStyle};
pub use crate::stack_view::{NSStackView, NSStackViewDistribution};
pub use crate::tab_view::{NSTabView, NSTabViewItem};
pub use crate::text_field::{NSTextAlignment, NSTextField};
pub use crate::timeline::NSTimelineView;
pub use crate::toolbar::{NSToolbar, NSToolbarItem};
pub use crate::view::{NSLayoutConstraint, NSLayoutMode, NSView, NSViewDelegate};
pub use crate::window::{NSWindow, NSWindowDelegate, NSWindowStyle};
pub use crate::{NSError, NSResult, NSLayoutConstants};
// Panel view controllers
pub use crate::panels::activity_bar::{
    NSActivityBarItem, NSActivityBarState, NSActivityBarViewController,
};
pub use crate::panels::asset_library::{
    NSAssetCategory, NSAssetItem, NSAssetLibraryState, NSAssetLibraryViewController,
};
pub use crate::panels::hierarchy::{HierarchyEntity, NSHierarchyViewController};
pub use crate::panels::material_inspector::NSMaterialInspectorViewController;
pub use crate::panels::node_editor::NSNodeEditorViewController;
pub use crate::panels::statistics::{NSStatisticsData, NSStatisticsViewController};
pub use crate::panels::timeline::{NSTimelineState, NSTimelineViewController};
pub use crate::panels::tools_sidebar::{
    NSToolDefinition, NSToolsSidebarViewController,
};
pub use crate::panels::viewport::{NSShadingMode, NSViewportState, NSViewportViewController};
pub use crate::panels::EditorPanel;
// Panel defaults for easy styling
// pub use crate::panels::{
//     ButtonDefaults, GridItemDefaults, LabelDefaults, PanelDefaults, PanelKind, SeparatorDefaults,
//     StyledPanel,
// };  // TODO: Missing defaults module
pub use crate::shader_graph_view::{
    NSShaderConnection, NSShaderGraphState, NSShaderGraphView, NSShaderNode, NSShaderPort,
    PortDirection,
};
