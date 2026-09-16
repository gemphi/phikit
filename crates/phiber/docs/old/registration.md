# Space Type Registration - Blender vs BUIKit

> **Reference**: `REFS/blender/blender/source/blender/editors/space_api/spacetypes.c`

## Blender C

At startup, `ED_spacetypes_init()` calls each `ED_spacetype_*()` once:

```c
void ED_spacetypes_init(void) {
    ED_spacetype_outliner();
    ED_spacetype_time();
    ED_spacetype_view3d();
    ED_spacetype_buttons();
    // ... 17 total
}
```

Each `ED_spacetype_*()` allocates a `SpaceType`, sets its name/id/icon, registers region types with their draw callbacks, and registers it globally:

```c
void ED_spacetype_view3d(void) {
    SpaceType *st = MEM_callocN(sizeof(SpaceType), "spacetype view3d");
    ARegionType *art;

    st->spaceid = SPACE_VIEW3D;
    strncpy(st->name, "View3D", BKE_ST_MAXNAME);

    st->new = view3d_new;
    st->free = view3d_free;
    st->init = view3d_init;
    st->listener = space_view3d_listener;

    // Region: main window
    art = MEM_callocN(sizeof(ARegionType), "view3d main region");
    art->regionid = RGN_TYPE_WINDOW;
    art->draw = view3d_main_region_draw;
    BLI_addhead(&st->regiontypes, art);

    // Region: header
    art = MEM_callocN(sizeof(ARegionType), "view3d header region");
    art->regionid = RGN_TYPE_HEADER;
    art->draw = view3d_main_region_draw;
    BLI_addhead(&st->regiontypes, art);

    BKE_spacetype_register(&st);
}
```

## BUIKit Rust Equivalent

In BUIKit, the `BSpaceHandler` trait defines space behavior.

```rust
// bspace.rs
pub trait BSpaceHandler: Send + Sync {
    fn space_type(&self) -> BSpaceType;

    /// Build the header region's BView tree.
    fn header(&self, ctx: &BContext) -> BView;

    /// Build a region's BView tree for the given region type.
    fn region(&self, rtype: BRegionType, ctx: &BContext) -> BView;
}
```

Registration happens once via `BSpaceRegistry::init()`:

```rust
// bspace_registry.rs
pub struct BSpaceRegistry {
    handlers: HashMap<BSpaceType, Box<dyn BSpaceHandler>>,
}

impl BSpaceRegistry {
    pub fn init() -> Self {
        let mut handlers: HashMap<BSpaceType, Box<dyn BSpaceHandler>> = HashMap::new();
        handlers.insert(BSpaceType::View3D, Box::new(spaces::view3d::View3DSpace));
        handlers.insert(BSpaceType::Outliner, Box::new(spaces::outliner::OutlinerSpace));
        handlers.insert(BSpaceType::Buts, Box::new(spaces::buts::ButsSpace));
        handlers.insert(BSpaceType::Time, Box::new(spaces::time::TimeSpace));
        Self { handlers }
    }

    pub fn get(&self, st: BSpaceType) -> &dyn BSpaceHandler {
        self.handlers.get(&st).expect("unregistered space type")
    }
}
```

## Why This Matters

- **Decoupled layout declaration**: We don't mix UI rendering logic with overall workspace management.
- **Header isolation**: Each space builds its own header dynamically, avoiding long centralized header drawing chains.
- **Consistent space lookup**: Instead of hardcoding layout grids, `buiapp` queries the registry and receives a unified hierarchy of nested `BView` nodes.
