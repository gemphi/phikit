# A unified structure for `phiber/src/v2`

The proposal, in one sentence: **every level is one folder with one trait, one
enum, one table and one build file; every visual thing is a `view`; every
piece of state is a path.** Nothing else earns a file.

Names are four letters wherever the thing allows it (`view`, `flex`, `atom`,
`card`, `part`, `host`, `pane`, `zone`, `area`, `menu`, `rail`, `key`,
`set`, `text`, `draw`). Blender's own names stay where they are Blender's
(`space`, `region`, `modal`, `state`, `theme`) — a reader coming from
`space_file.cc` has to find `space` and `region`.

## 0. What buikit already got right

`crates/apple/src/buikit` (v1) is the shape to keep, minus AppKit:

| buikit                     | what it is                                              | v2 name         |
|----------------------------|---------------------------------------------------------|-----------------|
| `view/` — one `BView` enum | the retained tree, pure data, no native view until host | `view/mod.rs`   |
| `structure/`               | `ScrArea` / `ARegion` / `eSpace_Type` as data           | the level folders |
| `blayout/` — `classify` → `measure` → `layout` | one layout engine that looks at a view's *kind* and decides Fixed / Flex / Spacer | `view/flex/` |
| `brender/` — `BRenderer`   | the single render boundary, one file per view family    | `view/draw/`    |
| `spaces/`                  | one file per editor, declaring its regions              | `host/set.rs`  |
| `see/*.md`                 | the design record beside the code                       | `v2/*.md`       |

Two things buikit did that v2 has drifted from, and this proposal restores:

- **The layout engine decides from the type, not the call site.**
  `BLayoutEngine::classify` looks at `BView::Button` and knows it is one
  `row_h` tall; nobody writes `.h(px(row_h))` at a button. In v2 that role is
  `BFlex`, and it already has the trait for it — `BFlexed` — but only the
  levels implement it. The rule becomes: **every kind implements `BFlexed`,
  and `BFlex` is the only thing that turns a kind into layout.**
- **One render boundary.** buikit has `brender/`; v2 has `atom/`, `prim/`,
  `prim/panel/`, `modal/`, `popover/` each styling on their own. §4 merges
  them into `view/draw/`, where a file may only ask a kind for its `BFlex`
  and add children.

## 1. The three traits everything implements

```rust
// v2/view/mod.rs

/// A tree of declared UI. Levels build it; `view/draw` paints it.
pub enum BView { Space{..}, Zone{..}, Area{..}, Host{..}, Region{..}, Pane{..},
                 Part{..}, Modal{..}, Card(BCard), Row{..}, Column{..}, Label{..},
                 Prose{..}, Button(BCtrl), Tab(BCtrl), Menu(BCtrl), Field{..},
                 Icon{..}, Canvas{..}, Empty }

/// One thing that becomes one view.
pub trait BItem { fn view(&self, s: &BState) -> BView; }

/// One thing that becomes many views (a table's rows, a strip's caps).
pub trait BList { fn views(&self, s: &BState) -> Vec<BView>; }

/// One view that becomes pixels. Only `view/draw/*` implements it.
pub trait BDraw { fn draw(&self, c: &BCtx) -> AnyElement; }
```

`BViewItem::build` → `BItem::view`; `BViewList::items` → `BList::views`;
`BFocusedRenderer::render` (dead today) → `BDraw::draw`, and it is the *only*
render trait.

## 2. One trait per level

Each level has one trait that says what the level *is*, one enum of kinds,
one declared table. The trait carries the enum and the table as associated
items, so a level is queryable the same way whichever level it is:

```rust
// v2/spec.rs
pub trait BSpec: Copy + Debug + 'static {
    fn label(&self) -> &'static str;
    fn icon(&self) -> &'static str { "" }
}

/// What every level has in common: a kind, a set of kinds, a spec per kind.
pub trait BLevel {
    type Kind: Copy + Eq + Hash + Debug + 'static;
    type Spec: BSpec;
    const ALL: &'static [Self::Kind];
    fn spec(kind: Self::Kind) -> &'static Self::Spec;
    fn slug(kind: Self::Kind) -> &'static str;
}
```

and then, per level, the trait that names it:

```rust
pub trait BSpace: BLevel + BItem {}     // space/space.rs
pub trait BZone:  BLevel + BItem {}     // zone/zone.rs
pub trait BArea:  BLevel + BItem {}     // area/area.rs
pub trait BHost:  BLevel + BItem {}     // host/host.rs
pub trait BRegion:BLevel + BItem {}     // region/region.rs
pub trait BPane:  BLevel + BItem {}     // pane/pane.rs
pub trait BPart:  BLevel + BItem {}     // part/part.rs
pub trait BModal: BLevel + BItem {}     // modal/modal.rs
```

A level is then exactly (sync132):

```
part.rs         BPartType, BPartSpec, the item and the list — one file
part/
    set.rs      the declared table: one spec per kind, Blender's order
    menubar.rs  a kind: its item and list (was menubar/{mod,item,list}.rs)
    menubar/
        set.rs  its declared content
        tree.rs content named for what it is
    caption.rs  a kind with no table is just its file
    …
```

No `mod.rs` anywhere in the level tree: a module with children is
`<name>.rs` beside `<name>/`. The same shape for `host`, `zone`, `area`,
`pane`, `region`, `space`, `modal`.

A kind folder with real content may add files named for the content, never
for the mechanism: `part/file/table.rs`, `part/file/thumbs.rs`,
`modal/reader/store/*.rs`. No `helpers.rs`, no `util.rs`, no `impl.rs`.

## 3. The card

Every panel role today (`Header`, `Body`, `Footer`, `Card`, `CardHeader`,
`Section`, `Sidebar`, `Toolbar`, `Viewport`, `Modal`, `Banner`) is one type
with three slots and a handful of traits:

```rust
// v2/view/card.rs
pub struct BCard {
    pub kind: BCardKind,          // Card | Section | Sheet | Strip | Bare
    pub head: Vec<BView>,         // the heading strip (title, controls)
    pub body: Vec<BView>,         // the content
    pub foot: Vec<BView>,         // the footer strip (actions)
    pub traits: BCardTraits,
}

#[derive(Default)]
pub struct BCardTraits {
    pub scroll: Option<BScroll>,  // Scroll::Y / Scroll::Both, id from the card's kind+title
    pub size:   Option<BSize>,    // resizable: the key the drag writes, min, axis
    pub shrink: bool,             // may give up height/width below its content
    pub fold:   Option<BPath>,    // collapsible: the key that holds open/closed
}

impl BCard {
    pub fn new(kind: BCardKind) -> Self;
    pub fn head(self, views: impl Into<Vec<BView>>) -> Self;
    pub fn body(self, views: impl Into<Vec<BView>>) -> Self;
    pub fn foot(self, views: impl Into<Vec<BView>>) -> Self;
    pub fn scroll(self, how: BScroll) -> Self;
    pub fn size(self, key: BPath, axis: BAxis) -> Self;   // resizable
    pub fn shrink(self) -> Self;
    pub fn fold(self, key: BPath) -> Self;               // accordion
}
```

`BView::Card(BCard)` replaces `BView::Panel { role, children }`,
`BView::AccordionHeader`, and the ad-hoc `CardHeader`/`Footer` pairs. The
file browser's source list is `BCard::new(Section).head([title]).body(rows)
.fold(key)`; the reader is `BCard::new(Sheet).body(prose).scroll(Y)`; a host
header region is a `Strip` card with only a `head`.

**Resize is standalone.** `view/size.rs` owns the grab handle, the drag, and
the key it writes; a card that is `.size(key, axis)` gets the handle laid
over its edge by `view/draw/card.rs`. Nothing about resizing lives in the
card, the sidebar or the region — today it is spread over
`prim/panel/resize.rs`, `prim/panel/sidebar.rs`, `zone/seam.rs`,
`space/seam.rs` and `state/area_resize.rs`. One `BSize` and one `size.rs`.

Likewise **scroll is standalone** (`view/scrl.rs`: the id, the offset, the
overflow) and **fold** is standalone (`view/fold.rs`: the chevron, the key).

### 3-done. First step (sync119) — checked against Blender's source

Read before writing: `REFS/blender/.../editors/interface/interface_panel.cc`
(`draw_aligned_panel`, `panel_draw_aligned_backdrop`,
`panel_draw_aligned_widgets`, `handle_panel_header`, `panel_drag_collapse`),
`blenkernel/BKE_screen.hh` (`PanelType`, `PANEL_TYPE_*`, `LayoutPanels`),
`makesdna/DNA_screen_types.h` (`Panel`, `PNL_*`), `interface_intern.hh`
(`PNL_HEADER = 1.25 units`), `screen/screen_ops.cc` (`SCREEN_OT_region_scale`)
and `UI_view2d.hh` (`V2D_SCROLL_*`).

`view/prim/card.rs` is `BCard { kind, head, body, foot, fold, active }` —
field for field the parts of a `PanelType` a view needs (`label`/`icon` →
`head`, `draw_header` → `head.controls`, `draw_header_preset` →
`head.preset`, `draw` → `body`, `PNL_CLOSED` → `fold`,
`PANEL_TYPE_NO_HEADER` → `head: None`, `parent` → `BCardKind::Sub`). Its
renderer `view/draw/card.rs` is Blender's header layout: the fold
triangle at the left (`triangle_right` / `triangle_down` for
`ICON_RIGHTARROW` / `ICON_DOWNARROW_HLT`), the title, header controls, the
preset pushed right, then the grip (`ICON_GRIP`) at the theme's
`grip_alpha` (0.5, as Blender draws it); a closed panel is its header alone
rounded on every corner, an open one rounds the header's top. It is built
entirely from `BFlex` — zero raw GPUI style calls — with three new tokens
(`BType::Body`, `BSpace::CardInset`, `BFlex::dimmed`).

`BView::AccordionHeader` and its renderer are gone. The Properties panels
(`part/props/item.rs`) and the file browser's bookmark panels
(`part/custom/file/sidebar.rs`) are `BCard`s. The geometry moved to
Blender's: row labels start under the fold triangle and row values end under
the grip (`tests/layout_gutters.rs`); the 26px disclosure chip that Phiber
drew at the right is gone.

sync120 finished the move: every card in the app is a `BCard`.
`BPanelRole::{Card, CardHeader}` and `prim/panel/{card,heading}.rs` are
gone. Preferences follows `space_userpref.py`: each style section is a
folding panel keyed like Properties' (`ctx.prefs.<category>.<section>.expanded`),
and the theme picker is headless (`USERPREF_PT_theme`, `HIDE_HEADER`). The
dialogs (welcome, about, help, profile, collab) use cards that don't fold, and
headless where they had no heading. Every pixel of those dialogs is unchanged
(zero quads differ in the golden). Row, column, body and pane now treat
`BView::Card` as the content column `Panel` was — missing that is what
shrank the About brand card on the first pass. Doing this found a bug that
was already there: the Preferences window region declared `BPartType::Panel`
(the Properties list, which has no Preferences sections), so the dialog body
was empty. It now declares `BPartType::Custom` (`part::custom::prefs`) —
`tests/prefs_panels.rs`.

sync121 declared panels the way Blender does. `BSectionSpec` gained
`closed` (`DEFAULT_CLOSED`), `headless` (`HIDE_HEADER`), `header`
(`draw_header` properties, drawn before the title) and `children`
(sub-panels, drawn as `Sub` cards). Every Properties panel Phiber carries now
opens or starts closed as its Blender class says: 39 start closed, checked by
`tests/properties_panel_parity.rs` against a table read from
`properties_*.py` and Cycles' `ui.py`. The Render tab is Cycles' tree: a
headless context panel (engine, device), then Sampling ▸ Viewport / Render ▸
Denoise, Light Paths ▸ Max Bounces / Caustics, Film ▸ Transparent / Pixel
Filter, Performance ▸ Threads / Final Render. Delta Transform is Transform's
child. `BPanelItem::expand_key` used to `Box::leak` two strings on every
build (unbounded growth per frame); its segments are now interned. Next: the
header gestures (drag-collapse, Ctrl-click), then the standalone `size` and
`scroll` traits; the missing Cycles properties are listed in `panel-parity.md`.

## 3a. Controls: one vocabulary over one value category — *implemented*

Every widget that edits a value is a `BCtrl` (`view/prim/ctrl.rs`,
drawn by `view/draw/ctrl.rs`, in sync108):

```rust
pub enum BCtrlKind {
    Check, Toggle,                       // Bool
    Radio, Select,                       // Enum   (options: Vec<BOption>)
    Int { min, max },                    // Int
    Number { min, max, step },           // Float
    Slider { min, max },                 // Float
    Text, Search,                        // Text
    Color,                               // Color  [r,g,b,a]
    Path,                                // Path
}
impl BCtrlKind { const fn value_kind(self) -> BValueKind }

pub struct BCtrl { kind, key: BPath, value: BValue, label, tooltip, option, editing, open, open_key }
impl BCtrl {
    fn read(kind, key, default, s: &BState) -> Self;   // the value from the state, as its category
    fn set(&self, BValue) -> BAction;                  // Set(key, value.encode())
    fn flip(&self) -> BAction;                         // Bool
    fn nudge(&self, steps: f32) -> Option<BAction>;    // Int / Number / Slider, clamped
    fn focus(&self) -> BAction;                        // typed kinds: the caret
    fn chosen(&self) -> Option<&BOption>;              // Radio / Select
}
```

and every state value has a category (`state/val.rs`):

```rust
pub enum BValueKind { Bool, Int, Float, Enum, Text, Color, Path }
pub enum BValue { Bool(bool), Int(i64), Float(f32), Enum(String), Text(String), Color([f32;4]), Path(String) }
impl BValue { fn encode(&self) -> String; fn parse(BValueKind, &str) -> Option<Self>; fn display(&self) -> String }
impl BState { fn value(&self, key, BValueKind) -> Option<BValue> }
```

The state stays strings; the *category* travels with the control that
edits the key, so one `encode`/`parse` serves every widget. What this buys:

- A Properties row is `BPropKind → (BCtrlKind, default)`; Bool, Int, Float,
  Enum, Str, Color, Vec3/Vec4 (one control per axis) are all real controls
  over `ctx.buts.<key>` — no more placeholder fields.
- **Render Engine → Phiray** is `Select` over `ctx.buts.engine`; choosing it
  is `Set(ctx.buts.engine, "PHIRAY")`, the log says "Render Engine: Phiray"
  (`prop_named` looks the label up), and anything else — a menu item, a
  script, a keymap — sets the engine with the same write.
- The file browser's "Load UI" is the same `Check` as a Properties bool.
- A `Button`, `Tab`, `MenuItem` stay `BControl` (an action, no value); a
  menu is `BMenuKind::{Op, Toggle, Sub, Sep}` with declared `modal` /
  `purpose` / `url` targets — already one typed table (`part/menubar/set`).

Still to do in this area: drag on `Number`/`Slider` (today: step buttons and
typing), a colour picker popover behind the swatch, and moving the
`open_key` of a `Select` into the control's own key namespace.

## 3b. `BFlex` takes the types and decides

```rust
// v2/view/flex/flex.rs
/// A kind that knows how it is laid out. Every level kind, every card kind,
/// every control kind implements it — it is the only way into BFlex.
pub trait BFlexed { fn flex(&self, c: &BCtx) -> BFlex; }

impl BFlexed for BZoneKind   { … }   // zone/zone.rs:   axis, fill, size
impl BFlexed for BRegionKind { … }   // region/region.rs: strip height, rail width, padding
impl BFlexed for BPaneKind   { … }
impl BFlexed for BCardKind   { … }   // view/card.rs: Card / Section / Sheet / Strip
impl BFlexed for BCtrlKind   { … }   // view/ctrl.rs: Button / Tab / Cap / Field / Menu row
impl BFlexed for BProse      { … }
```

and `BFlex` itself gains what buikit's engine had — a classification —
so a draw file never asks "is this a strip?":

```rust
impl BFlex {
    pub fn of(kind: &impl BFlexed, c: &BCtx) -> Self;   // the one entry
    pub fn div(&self, c: &BCtx) -> Div;                  // container
    pub fn scroller(&self, id, c) -> Stateful<Div>;      // + BScroll
    pub fn unfolding(&self, id, c) -> AnimationElement;  // + motion
    pub fn class(&self) -> BFlexClass;                   // Fixed | Flex | Spacer | Strip | Rail
}
```

`view/draw/<x>.rs` then reads as: take the kind, `BFlex::of(kind, c).div(c)`,
add the children, tag the id. Nothing else. `tests/style_budget.rs` pins
every draw file at zero raw style calls from the start, not "no worse than
today".

## 3c. A dialog is a space — *implemented (sync109)*

A modal was a "peer of Space" with its own body builder, which is why the
file dialog, Preferences and the render window each laid themselves out
differently from the editors in a workspace. Now `BModalSpec` declares
`areas: &[BAreaSpec]` exactly as `BSpaceSpec` does, and `BModalItem` builds
them through the same `BZoneList` → `BZoneItem` → `BAreaItem` → `BHostItem`
chain, scoped to the modal (`area::BScope::Modal`) so the editor chosen in
the dialog's area never overwrites the workspace's slot:

```rust
static FILE_AREAS:   &[BAreaSpec] = &[editor(BHostType::File,  &[BHostType::File, BHostType::Asset])];
static PREFS_AREAS:  &[BAreaSpec] = &[editor(BHostType::Prefs, &[BHostType::Prefs])];
static RENDER_AREAS: &[BAreaSpec] = &[editor(BHostType::Image, &[BHostType::Image])];
```

So the file dialog *is* Zone::Center ▸ Area ▸ Host::File, with the same
header (switcher icon + View/Select on the left, display / hidden / filter
icons on the right — Blender's `FILEBROWSER_HT_header`), the same path
strip, source list, table and execute bar as the File editor in the Shading
workspace. Dialogs with an empty `areas` (Splash, About, Help, Reader,
Profile, Collab, Command, Spaces, Progress) are Blender *menus*
(`WM_MT_splash`, `WM_MT_splash_about`…), not editors; they keep their own
`BViewItem` until §4's regions-for-everything step.

Sizes: every card dialog is half the window wide (`BLen::Share(0.5)`); one
that raises an editor is the window's height, a form keeps its declared
height; the reader sheet is half the window too. Declared `width`s in
`modal/set.rs` are now unused and will go.

Known gap: `zone_size` / `area_fraction` still key by `ctx.space`, so a
dialog's zone reads the active workspace's drag sizes (harmless for a
single Center zone; to fix when `BRenderCtx` carries the scope).

## 4. `view/` is the whole visual layer — no `render/`

```
view/
    mod.rs      BView, BItem, BList, BDraw
    card.rs     BCard + traits (§3)
    ctrl.rs     BCtrl: icon/label/action/hover/active (was prim/control.rs)
    prim.rs     the constructors: row, column, label, field, icon, table_row, tree_row…
    text.rs     the text dump of a BView (was render/ascii.rs) — the test oracle
    flex/       BFlex, the one layout element (as today)
        flex.rs token.rs apply.rs
    draw/       BDraw impls, one file per view kind, no level knows GPUI but these
        space.rs zone.rs area.rs host.rs region.rs pane.rs part.rs modal.rs
        card.rs  row.rs  table.rs  tree.rs  menu.rs  rail.rs  tab.rs
        ctrl.rs  text.rs  icon.rs  prose.rs  log.rs
        size.rs  scrl.rs  fold.rs      ← the standalone traits' painters
        ctx.rs   color.rs  brand.rs
    canvas/     the viewport and node graph — a renderer of its own, not an atom
        view3d/ node/ image/ gizmo/
    seam.rs     the drag line between two zones or areas (one, not two)
```

`render/gpui/atom/` and `render/gpui/prim/` merge into `draw/`: an atom and a
prim were the same thing (a `BDraw` for one `BView` variant). The
`B…Layout` traits in `render/*.rs` (axis, placement) fold into each level's
`<level>.rs` as methods on the kind — `BZoneKind::axis()` is a fact about
zones, not about rendering.

## 5. `state/` is two files

```
state/
    mod.rs      BState: get / set / has / toggle / remove; BPath; BKey; BAction; apply
    key/        one enum per domain, each `impl BKey` — the only place a path is spelt
        space.rs zone.rs area.rs host.rs region.rs pane.rs part.rs modal.rs
        file.rs view3d.rs menu.rs edit.rs theme.rs log.rs
```

```rust
/// A key is a path: `ctx.{domain}.{owner}.{field}`, spelt once, in an enum.
pub trait BKey { fn path(&self) -> BPath; }

// state/key/zone.rs
pub enum ZoneKey { Shown(BSpaceKind, BZoneKind), Size(BSpaceKind, BZoneKind) }
impl BKey for ZoneKey {
    fn path(&self) -> BPath { match self {
        Self::Shown(s, z) => BPath::of(["zone", s.slug(), z.slug(), "shown"]),
        Self::Size(s, z)  => BPath::of(["zone", s.slug(), z.slug(), "size"]),
    }}
}

// anywhere
s.get(ZoneKey::Size(space, zone));  s.set(ZoneKey::Shown(space, zone), true);
```

`BState::get/set/has/toggle/remove` take `impl BKey`. The 37 `K_…`
constants and the `k_…()` functions once scattered through
`part/custom/file/keys.rs`, `part/custom/outliner/key.rs`, `scene/key.rs`,
`part/strip.rs` and `prim/panel/resize.rs` are now variants of a domain enum
in `state/key/` (done, §5a).
A reader looking for what the file browser stores opens `key/file.rs` and
sees every key it has.

What is *not* in `state/`: the reducers. `area_resize.rs`, `zone_size.rs`,
`file_nav.rs`, `space_switch.rs`, `follow.rs`, `menu.rs`, `edit.rs`,
`log.rs`, `seed.rs` are rules about a level, so they live with the level and
register through one trait:

```rust
/// One follow-up of a `Set`.
pub trait BRule: Sync {
    fn wants(&self, path: &BPath) -> bool;
    fn after(&self, path: &BPath, value: &str, s: &mut BState);
}
```

Each follow-up is a unit struct beside the level it belongs to;
`state/rule.rs` is only the ordered table, and `state/action.rs` only
dispatches (store → `edit::before_set` → rules → close overlays). The drag
reducers (`area_resize.rs`, `zone_size.rs`) are not after-set rules — the
app calls them with a pointer delta — so they move to `area/size.rs` and
`zone/size.rs` as plain functions; `space_switch.rs` joins `space/`.

### 5a. Done so far (sync111)

`state/key/mod.rs` has the trait; `BState::get / set / has / remove / value`
take `impl BKey`, and `BPath` implements it, so the old constants and the
new enums meet at the store. The file browser is the first domain moved:
`state/key/file.rs` — `FileKey::{Path, Selected, Filename, Search, Display,
Sort, SortInvert, ShowHidden, UseFilter, Filter(flag), Nav, HistBack,
HistFwd, Purpose, Error, SidebarExpanded(label), Bookmark(slot),
BookmarkAdd, FilterOpen, LoadUi, RecoverLastSession}` — and
`part/custom/file/keys.rs` is gone. `tests/key_budget.rs` lists the moved
domains and fails on any `vec!["file", …]` or `"ctx.file.…"` spelt outside
the keys file.

sync112 moved two more: `state/key/outliner.rs` —
`OutlinerKey::{Expanded(id), Search, DisplayMode, SortAlpha,
Restrict(object, field), Column(field), FilterOpen}` (the outliner's
`keys.rs` keeps only its predicates: `is_hidden`, `BHidden`,
`is_expanded`, `search_text`, `display_mode`, `sort_alpha`) — and
`state/key/buts.rs` — `ButsKey::{Value(prop), Axis(prop, axis),
Dropdown(prop), NavbarTab}`, so a Properties value is `ButsKey::Value("engine")`
wherever it is read. The budget caught one spelling the grep had missed
(`modebar/set.rs`), which is the point of it. sync113: `View3dKey::{Yaw, Pitch, Zoom, Perspective, PanX, PanY, Shading,
View, Pivot, Lens, GizmoCorner}` and `SceneKey::{Loading, LoadStage,
LoadProgress, LoadCancel, Error, LoadedPath}`; `scene/key.rs`'s `*_key()`
functions now delegate to them (the ~200 call sites keep the function
names; only the spelling moved). sync114: `ZoneKey::{Shown(zone), Size(space, zone)}` and
`AreaKey::{Host(scope, slot, host), Switcher(scope, slot, host),
Size(space, zone, index), DragSpace, DragZone, DragIndex}` with
`size_prefix()` / `switcher_prefix()` for the sweeps; the `K_ZONE_*`
constants and `P_AREA` are gone, the seed writes typed keys, and the
"close every menu and switcher" sweeps use prefixes from the key modules
rather than string literals. Eleven domain prefixes are now under the
budget.

sync115 finished the sweep: **the `K_…` constants are gone.** Four enums
took the rest — `CtxKey::{Space, Scene, ViewLayer, Object, Mode, Tool,
Frame, Playing, FrameStep, FrameStart, FrameEnd, Fps}` (the one-segment
scene context), `ModalKey::{Open, Maximized, Minimized, ReaderDoc,
ReaderReturn}`, `ActivityKey::{Tab, Group, Pick}` and `AppKey::{WindowWidth,
Theme, UseGradients, Link, Edit, EditUndo, CommandQuery, StatusLogOpen,
Profile(field), CollabStatus, CollabServer, CollabPeer(n)}`; the navbar's
expanded flag moved to `ButsKey::NavbarExpanded`, where its path already
lived. `state/key.rs` keeps only `int_or` / `float_or` and the `ui_style`
catalogue. `state/follow.rs`'s rule table is `(fn() -> BPath,
Rule)` so its keys are typed too; `edit::is_editing / start` take `impl
BKey`. The budget now covers 36 prefixes, including the one-segment keys
(`vec!["space"]`, `"ctx.link"`), and it caught the one hand-spelt
comparison left in `app/handle.rs`.

sync116 closed step 1: `state/key/menu.rs` — `MenuKey::{Top(host, label),
Nested(host, labels)}` with `prefix()`, `host_prefix(host)`,
`is_top_level()`, `is_top_level_encoded()` and `owns()` — replaces
`k_menu`, `k_menu_path`, `k_menu_prefix` and `P_MENU`. `state/menu.rs` is
now only the open/close reducer; the menubar tree passes `(host, labels)`
down instead of raw segments, so the command palette builds a real menu
key for a plain operator too. Every fixed key in the app is a variant of
one of thirteen enums, and the budget guards thirty-seven prefixes.

sync117 did the rules: `state/rule.rs` holds `trait BRule { wants, after }`
and the table; `state/follow.rs` and `state/file_nav.rs` are gone. The
twelve follow-ups now live with their owners —
`scene/viewpoint.rs` (`BViewpointRule`, and `view_angles`),
`part/activity/choose.rs` (`BActivityTabRule`, `BActivityPickRule`),
`part/custom/time/step.rs` (`BFrameStepRule`, with the wrap as a pure
`stepped()`), `modal/reader/back.rs` (`BReaderOpenRule`,
`BReaderCloseRule`), `modal/workspace/pick.rs` (`BSpacePickedRule`) and
`part/custom/file/op.rs` — Blender's `file_ops.cc` — (`BSortRule`,
`BNavRule`, `BPurposeRule`, `BBookmarkAddRule`, `BSelectRule`, plus the
`NAV_*` operator names). `action.rs` went from 91 lines of `if *path ==`
to a 30-line dispatch; the caret's undo moved to `edit::before_set`; the
overlay sweep that `action.rs` and `space_switch.rs` each carried is one
`menu::close_overlays`. `tests/rule_budget.rs` fails if `action.rs`
compares a path or names a key, or if any `impl BRule` is missing from the
table.

sync118 moved the rest out: `state/area_resize.rs` → `area/size.rs`,
`state/zone_size.rs` → `zone/size.rs`, `state/space_switch.rs` →
`space/switch.rs`, each re-exported from its level (`area::area_fraction`,
`zone::zone_size`, `space::switch_space`) and no longer from `state`.
`state/` now holds the store, `keys/`, `rule.rs`, the dispatcher and five
small helpers (`edit`, `menu`, `log`, `seed`, `val`, `key`);
`tests/rule_budget.rs` lists them and fails if a new module appears there.
Still to go from §5: `log.rs` → `app/log.rs`, `seed.rs` → `app/seed.rs`,
`menu.rs` → `part/menubar/`. Next: §3's card.

## 6. The whole tree

```
v2/
    mod.rs  spec.rs                     BSpec, BLevel

    space/  space.rs set.rs            BSpaceKind, the 11 workspaces (set.rs holds all 11
                                        tables — today's eleven files are eleven tables)
            switch.rs                   BRule: picking a space
    zone/   zone.rs set.rs size.rs     size.rs: BRule for the dragged zone size
    area/   area.rs set.rs size.rs     size.rs: BRule for the area fractions
    host/   host.rs set.rs             BHostKind, 36 specs; each host's regions are a
                                        table in set.rs, not 36 folders of one mod.rs
    region/ region.rs set.rs           BRegionKind with axis()/placement(); the Header
                                        pane template lives here (was host/header.rs)
    pane/   pane.rs set.rs
    part/   part.rs set.rs
            menubar/  part.rs set.rs
            switcher/ part.rs set.rs
            search/   part.rs
            filter/   part.rs set.rs
            toolbar/  part.rs set.rs
            sidebar/  part.rs set.rs
            navbar/   part.rs set.rs
            channels/ part.rs set.rs
            statusbar/part.rs set.rs
            log/      part.rs
            zonebar/  part.rs
            activity/ part.rs set.rs
            caption/  part.rs
            execute/  part.rs
            hud/ assetshelf/ toolprops/ spacebar/ modebar/ …
            props/    part.rs set.rs   the Properties tabs (was part/props/, 17 folders →
                      object.rs material.rs modifier.rs … one file per tab)
            file/     part.rs set.rs   the browser: table.rs thumbs.rs path.rs list.rs
                      nav.rs (BRule)    (was part/custom/file, 19 files → 7)
            outliner/ part.rs set.rs tree.rs
            time/     part.rs
            text/ node/ image/ seq/ clip/ action/ buts/ prefs/ spreadsheet/ view3d/
                      part.rs each      (was part/custom/*: mod+item+list → part.rs)
    modal/  modal.rs set.rs
            splash/   modal.rs set.rs
            help/     modal.rs set.rs
            about/    modal.rs set.rs
            reader/   modal.rs set.rs store/*.rs
            settings/ profile/ collab/ command/ spaces/ progress/ render/ file/
                      modal.rs (+ set.rs where declared content exists)
    view/   (§4)
    state/  (§5)
    theme/  theme.rs set.rs            BTheme (was semantic.rs + style.rs + resolver.rs)
            color.rs size.rs fx.rs      the three token families (colors, metric, effects)
            table.rs                    the descriptor table → Preferences
            palette/  set.rs           the shipped palettes
    scene/  scene.rs camera.rs …        the loaded file and the viewport state (as today,
                                        minus keys.rs → state/key/scene.rs, tests → tests/)
    app/    app.rs load.rs log.rs       the window, the file load, the action log (BRule)
```

Counts: today 499 files, 36 host folders of one file each, 17 property-tab
folders of two files each, four "level mechanism" files per level. The tree
above is ~190 files, with no folder that exists only to hold a `mod.rs`.

## 7. The rules, so it stays this way

1. A folder is a level or a kind. A file is `<level>.rs`, `set.rs`, or
   named for content. Nothing named for a mechanism (`item`, `list`,
   `helpers`, `impl`, `render`).
2. `BItem::view` builds; `BDraw::draw` paints; nothing does both. Only
   `view/draw/` and `view/flex/apply.rs` may name a GPUI type.
3. Every path is a `BKey` variant in `state/key/`. A string with a dot in
   it outside `state/key/` is a bug (`tests/style_budget.rs` gets a sibling
   `tests/key_budget.rs` that greps for it).
4. Scroll, size, fold and seam are standalone (`view/size.rs` etc.) and are
   *asked for* through `BCard`'s traits, never built inside a part.
5. Declared content is a `static` table in a `set.rs`. A `Vec` built at
   runtime from literals is a table that forgot to be declared.
6. Tests live in `tests/`. A `mod tests` at the foot of a file is for
   private items only; no `*_tests.rs` in `src/`.
7. **No plural names.** A name is the thing, singular; many of it is its
   *list*, and the list is the holder. One file per thing, never a plural
   folder (`prim/columns/` → `prim/body.rs`, `prim/sidebar.rs`); a type has
   `view()` for one and `list()` for many; a field or parameter holding
   `Vec<T>` is `list` (or `<thing>_list` when a type holds two:
   `cell_list`, `peer_list`), never `children`, `cells`, `views`. The same
   goes for the set table (`set.rs`), the key folder (`state/key/`) and
   module names. No one- or two-letter names for parameters: `ctx`, not
   `c`; `element`, not `el`; `view`, not `v`.

## 8. Migration order

Each step keeps the layout fingerprint identical or explains every moved
line, as the BFlex refactor did.

1. `state/`: introduce `BKey` and `state/key/*`; replace the `K_…`/`k_…()`
   uses; move reducers to their levels as `BRule`s. (Mechanical; the
   fingerprint does not move.)
2. ~~`view/card.rs` + `view/size.rs`/`scrl.rs`/`fold.rs`; migrate `BView::Panel`
   and `AccordionHeader` to `BCard`. Delete `prim/panel/*`, `resize.rs`.~~ —
   **done (sync122–128).** Every panel role is a `BCard` kind: `Panel`/`Sub`/
   `Bare` cards, `Strip(Header | Footer | Toolbar)` (`prim/bar.rs`),
   `Column(Body | Sidebar)` (`prim/body.rs`, `prim/sidebar.rs`); the property row is
   `BView::Split` (`prim/split.rs`). `BView::Panel`, `BPanelRole` and
   `prim/panel/` are gone. Standalone traits: `view/size.rs` (region scale,
   hide below a unit), `view/scroll.rs` (drawn View2D scroller),
   `prim/fold.rs` (click, Ctrl-click solo, drag-sweep). Left for later: the
   seams' own resize (`zone/seam.rs`, `space/seam.rs`) still use
   `prim/resize.rs`'s overlay rather than `view/size.rs`.
2a. **Naming (sync129).** `BView`'s `children` fields are `list` and the
   table rows' `cells` are `cell_list`, everywhere in `src/v2` and
   `tests/`. `prim/columns/` is gone: `prim/body.rs` (`BBody`),
   `prim/sidebar.rs` (`BSidebar`), `prim/resize.rs` (edge drag),
   `prim/surface.rs` (surface tone and gutter, was `columns/style.rs`); the
   column scroll id lives in `prim/scroller.rs`, and `prim/card.rs` hands a
   `Column` card straight to its renderer. Those files take `ctx`, not `c`.
   **Then (sync130):** every renderer takes `ctx: &BRenderCtx`, not `c`;
   the context's one-letter fields are `ctx.state` and `ctx.handler`
   (were `s`, `h`); `el` is `element`. Plural modules renamed:
   `option`, `op`, `thumb` (file), `option` (activity), `pref` (host,
   part/custom, part/props), `key` (state — `state/key.rs` merged into
   `state/key/mod.rs` —, scene, outliner), `extra` (scene, view3d),
   `color`/`effect`/`metric`/`radius` (theme/style), `credit`/`tutorial`/
   `release_note` (reader store), `setting` (modal), `workspace` (the
   space-picker modal, was `modal/spaces`), `mesh/triangle.rs` (was
   `types.rs`), `rendered/vert.rs`, `surface/corner.rs`. Kept, because they
   are Blender's own names: `buts`, `channels`, `particles`, `drivers`,
   `toolprops`, `geometry_nodes`, `physics`. Left: plural *item* names
   (`CATEGORIES`, `REGIONS`, `BCorners`, `options_for`, locals like
   `verts`, `entries`) and `*_tests.rs`/`tests.rs` modules in `src/`
   (rule 6) — handled level by level in step 3.
3. **Done for the level tree (sync131–132).** No `mod.rs` and no
   `item.rs`/`list.rs`/`spec.rs`/`type.rs` left under `region`, `pane`,
   `zone`, `area`, `space`, `host`, `part`, `modal`. A level is one file
   beside its folder — `v2/part.rs` (kind enum, spec, item, list) and
   `v2/part/set.rs` (the table) — and a kind is one file,
   `part/channels.rs`, with `part/channels/set.rs` only when it declares
   content. Hosts' regions are `host/set.rs::region_set`, workspaces' areas
   `space/set.rs::area_set`; the fifteen Properties contexts are single
   files (`part/props/object.rs`, their tables inlined). `part/menubar`'s
   `item/tree.rs` is `part/menubar/tree.rs`. 487 → 353 source files, the
   layout fingerprint unchanged.
   Originally: Collapse each level's four files into `<level>.rs` + `set.rs`; hosts'
   36 folders into `host/set.rs`; spaces' 11 files into `space/set.rs`.
4. ~~`render/` → `view/draw/`; `atom/` + `prim/` merged; `B…Layout` traits
   into the kinds; `BFocusedRenderer` deleted.~~ — **done (sync133).**
   `view/render/gpui/` is `view/draw/`, and its `atom/` and `prim/` files
   sit flat in it (`draw/button.rs`, `draw/card.rs`, …; `prim/surface.rs`
   is `draw/tone.rs` beside the host `draw/surface.rs`). `draw/atom.rs` and
   `draw/prim.rs` keep only the walkers for their `BView` variants.
   `view/render/` is gone: `ascii.rs` is `view/text.rs`, the layout
   vocabulary (`BAxis`, `BPlacement`, the `B…Layout` traits, `BRenderer`) is
   `view/layout.rs`, and each level implements its trait in its own file
   (`zone.rs`: `impl BZoneLayout for BZoneType`). `BFocusedRenderer` and
   `render_contract.rs` are deleted with the two tests that only checked the
   trait existed. The dispatcher line cap no longer counts `mod x;` lines.
   Left: `canvas/` stays a subtree of `draw/` (the doc's `view/canvas/`), and
   the rest of `view/draw/` and `state/`, `theme/`, `app/`, `scene/` still
   use `mod.rs`.
5. ~~`part/custom/*` → `part/<name>`; `part/panel/` → `part/props/`;
   `part/custom/file/` split into view and state.~~ — **done (sync134).** The
   thirteen host contents are parts of their own: `part/view3d.rs`,
   `part/outliner.rs`, `part/file.rs`, … (`part::custom::file` →
   `part::file`); `part/custom.rs` keeps only the dispatcher that picks one
   by host, and its `set.rs`. `part/panel` is `part/props` (the Properties
   editor's contexts). The file browser's state already lives apart from its
   view: every key in `state/key/file.rs`, the reducers in `part/file/op.rs`
   (Blender's `file_ops.cc`) and the back/forward stacks in
   `part/file/history.rs` (`folder_history.cc`) — kept under Blender's names
   rather than folded into one `nav.rs`.
6. Modals: every body a `BView::Host` or a list of `BView::Region`; the title
   bar a Header region.
7. `theme/set.rs` deleted; tests read `DEFAULT_*`.
