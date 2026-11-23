//! Hierarchy model and data source trait.
//!
//! Contains:
//! - HierarchyEntity: Data structure for tree nodes
//! - HierarchyDataSource: Trait for providing data to NSHierarchyViewController

// ═══════════════════════════════════════════════════════════════════════
// Entity Model
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct HierarchyEntity {
    pub id: u64,
    pub name: String,
    pub children: Vec<HierarchyEntity>,
    pub is_expanded: bool,
    pub icon_id: String,
    pub depth: usize,
}

impl HierarchyEntity {
    pub fn new(id: u64, name: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            children: Vec::new(),
            is_expanded: false,
            icon_id: String::new(),
            depth: 0,
        }
    }

    pub fn with_depth(id: u64, name: &str, depth: usize) -> Self {
        Self {
            id,
            name: name.to_string(),
            children: Vec::new(),
            is_expanded: false,
            icon_id: String::new(),
            depth,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Data Source Trait (Apple NSOutlineViewDataSource pattern)
// ═══════════════════════════════════════════════════════════════════════

/// Data source trait for hierarchy views.
///
/// Implement this on your Scene or data model to provide tree data to the hierarchy panel.
/// Follows Apple's NSOutlineViewDataSource protocol pattern.
pub trait HierarchyDataSource: Send + Sync {
    /// Returns the root-level entities in the hierarchy.
    fn root_items(&self) -> Vec<HierarchyEntity>;

    /// Returns the children of a given entity.
    fn children_of(&self, entity_id: u64) -> Vec<HierarchyEntity>;

    /// Returns true if the entity can have children (is expandable).
    fn is_expandable(&self, entity_id: u64) -> bool;

    /// Returns the entity with the given ID, if it exists.
    fn entity_by_id(&self, entity_id: u64) -> Option<HierarchyEntity>;

    /// Returns the total number of entities in the hierarchy (for performance hints).
    fn entity_count(&self) -> usize {
        self.root_items().len()
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Callback Types (Apple delegate pattern)
// ═══════════════════════════════════════════════════════════════════════

/// Callback for selection changes.
pub type SelectionCallback = Box<dyn Fn(Option<u64>) + Send + Sync>;

/// Callback for drag/drop operations. Returns true to accept the drop.
pub type DragDropCallback = Box<dyn Fn(u64, Option<u64>) -> bool + Send + Sync>;

/// Callback for expand/collapse events.
pub type ExpandCollapseCallback = Box<dyn Fn(u64, bool) + Send + Sync>;
