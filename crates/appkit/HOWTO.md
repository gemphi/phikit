# rust-appkit — How to Implement and Use

## Architecture Rules

1. **No dangling functions** — all behavior lives on `NS*` structs
2. **mod.rs = declarations + re-exports only** — no logic
3. **Each file < 180 lines** — split by concern
4. **Use existing components** — NSToolbar, NSTabView, NSSplitView, NSScrollView
5. **Delegates = pure state** — no UI logic, no size calculations
6. **Drawing = separate file** — implements `NSViewDrawing` trait

## File Structure (per module)

```
views/my_view/
├── mod.rs          — pub mod declarations + pub use re-exports
├── view.rs         — NSMyView struct + constructors
├── delegate.rs     — NSMyViewDelegate (pure state, forwards to drawing)
├── drawing.rs      — NSMyViewDrawing struct (implements NSViewDrawing)
├── builder.rs      — NSMyViewBuilder struct (if complex assembly needed)
├── types.rs        — enums, config structs
└── tests.rs        — #[cfg(test)] tests
```

## Creating a New View

```rust
// views/my_view/view.rs
pub struct NSMyView {
    view: NSView,
    state: Arc<Mutex<MyState>>,
}

impl NSMyView {
    pub fn new(frame: NSRect) -> Self {
        let state = Arc::new(Mutex::new(MyState::default()));
        let mut view = NSView::new(frame).with_name("NSMyView");
        view.set_delegate(MyDelegate { state: state.clone() });
        Self { view, state }
    }
    pub fn view(&self) -> &NSView { &self.view }
    pub fn view_mut(&mut self) -> &mut NSView { &mut self.view }
}
```

```rust
// views/my_view/delegate.rs — PURE STATE, no UI logic
pub struct MyDelegate {
    pub(crate) state: Arc<Mutex<MyState>>,
}

impl NSViewDelegate for MyDelegate {
    fn draw(&self, view: &NSView, ctx: &mut RenderContext) {
        NSMyViewDrawing::draw(view, ctx, &self.state.lock().unwrap());
    }
    fn mouse_down(&mut self, view: &mut NSView, event: &NSEvent) {
        // Update state only — no size calculations here
        self.state.lock().unwrap().selected = true;
        view.set_needs_display();
    }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}
```

```rust
// views/my_view/drawing.rs — ALL rendering here
pub struct NSMyViewDrawing;

impl NSMyViewDrawing {
    pub fn draw(view: &NSView, ctx: &mut RenderContext, state: &MyState) {
        let bounds = view.bounds();
        ctx.set_color(theme::color(keys::CLR_BG_PANEL));
        ctx.fill_rect(bounds);
        // ... all drawing logic
    }
}

impl NSViewDrawing for NSMyViewDrawing {
    fn draw(&self, view: &NSView, ctx: &mut RenderContext) {
        Self::draw(view, ctx, &MyState::default());
    }
}
```

## Creating a Panel (in rust-kit-pro)

```rust
// panels/my_panel/mod.rs
pub mod controller;
pub mod view;

// panels/my_panel/controller.rs
pub struct MyPanelController {
    pub view: NSView,
}

impl MyPanelController {
    pub fn new(rect: NSRect) -> Self {
        let mut view = NSView::new(rect).with_name("MyPanel");
        // Use existing components — don't reinvent
        let mut tab_view = NSTabView::new(rect);
        tab_view.add_tab(NSTabViewItem::new("Tab 1"));
        tab_view.add_tab(NSTabViewItem::new("Tab 2"));
        view.add_subview(tab_view.view_clone());
        Self { view }
    }
}

impl EditorPanel for MyPanelController {
    fn view(&self) -> &NSView { &self.view }
    fn view_mut(&mut self) -> &mut self.view }
    fn on_selection_changed(&mut self, _entity_id: Option<u64>) {}
    fn name(&self) -> &str { "MyPanel" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}
```

## Using NSToolbar (don't manually position buttons)

```rust
let mut toolbar = NSToolbar::new("MyToolbar");
toolbar.add_item(NSToolbarItem::button("save", "Save", save_btn.view_clone()));
toolbar.add_flexible_space();
toolbar.add_item(NSToolbarItem::button("search", "Search", search.view_clone()));
let toolbar_view = toolbar.build_view(width);
parent.add_subview(toolbar_view);
```

## Using NSTabView (don't manually create tab buttons)

```rust
let mut tabs = NSTabView::new(rect);
tabs.add_tab(NSTabViewItem::new("Scene"));
tabs.add_tab(NSTabViewItem::new("Assets"));
tabs.set_content_for_tab(0, scene_scroll.view_clone());
tabs.set_content_for_tab(1, assets_view);
parent.add_subview(tabs.view_clone());
```

## Using NSSplitView (don't manually calculate divider positions)

```rust
let mut split = NSSplitView::vertical();
split.add_arranged_subview(left_panel);
split.add_arranged_subview(center_panel);
split.add_arranged_subview(right_panel);
split.set_divider_at_index(0, 260.0);
split.set_divider_at_index(1, 900.0);
parent.add_subview(split.view().clone());
```

## Theme System

```rust
// Read tokens — never hardcode colors or sizes
let bg = theme::color(keys::CLR_BG_PANEL);
let radius = theme::f32_or(keys::RADIUS_PANEL, 12.0);
let font_size = theme::f32_or(keys::SIZE_FONT_UI, 13.0);

// Switch appearance at runtime
theme::set_variant("tesla");
theme::set_mode("dark");
theme::set_material("glass");
```

## Global Protocols (traits every view type should implement)

```rust
// NSViewDrawing — for rendering
pub trait NSViewDrawing: Send + Sync {
    fn draw(&self, view: &NSView, ctx: &mut RenderContext);
}

// NSViewBuilder — for construction
pub trait NSViewBuilder: Send + Sync {
    fn build(&self, frame: NSRect) -> NSView;
}

// NSViewLayout — for layout passes
pub trait NSViewLayout: Send + Sync {
    fn layout(&self, view: &mut NSView);
}
```

## What NOT to Do

- ❌ `pub fn build_thing(view: &mut NSView, ...)` — dangling function
- ❌ `let toolbar_height = 32.0;` — hardcoded size
- ❌ `let mut toolbar = NSView::new(...)` — reinventing NSToolbar
- ❌ `let mut tab_btn = NSButton::new(...)` — reinventing NSTabView
- ❌ Putting draw logic in the delegate
- ❌ Files over 180 lines
- ❌ `mod.rs` with struct definitions or impl blocks

## What TO Do

- ✅ `NSMyBuilder::build(frame)` — method on a struct
- ✅ `theme::f32_or(keys::SIZE_CONTROL_HEIGHT, 24.0)` — from theme
- ✅ `NSToolbar::new("id").build_view(width)` — use the component
- ✅ `NSTabView::new(rect)` — use the component
- ✅ Delegate = state only, Drawing = separate struct
- ✅ Each file named for what it does
- ✅ `mod.rs` = declarations + re-exports only
