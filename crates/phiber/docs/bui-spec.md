# BUI Declarative Specification & Universal Card Contract

## 1. Specification Principles: Direct Chaining on `card()`

1. **No Standalone Sub-Constructors**: Never access or construct separate wrapper types like `card_header(...)` or `card_footer(...)`.
2. **Direct Methods on `card()`**: All slots, titles, tools, collapse actions, bodies, and footers are chained **directly** on the `BView::card()` instance:
   ```rust
   BView::card()
       .with_header("Title")
       .with_icon("icon-name")
       .with_badge("badge-text")
       .with_tool(tool_button)
       .with_body(body_content)
       .with_footer("Footer Status")
       .with_footer_action(action_button)
   ```
3. **Composable Body Components**:
   - `BView::list().with_item(BView::item(label).with_icon(...).with_action(...))`
   - `BView::row().with_gap(...).with_child(...)`
   - `BView::column().with_gap(...).with_child(...)`
   - `BView::scroll(content)`
4. **100% Stylable & 100% Clickable**:
   - Color shades: `with_shade(CardShade)`
   - Dimensions: `with_width(f32)`, `with_height(f32)`
   - Collapse: `with_collapsible(bool)`, `with_collapsed(bool)`, `with_collapse_action(action)`
   - Every button and item accepts `.with_action(BAction)`.

---

## 2. Universal `Card` Fluent API Contract

```rust
pub enum CardShade {
    App,            // Darkest root window shade
    Activity,       // Activity bar dark shade
    Panel,          // Standard editor panel shade
    Header,         // Header bar contrast shade
    Viewport,       // Canvas background shade
    Accent,         // Highlight accent shade
}

// Universal Card Builder — all methods chain directly on card()
BView::card()
    // Styling & Dimensions
    .with_shade(shade: CardShade)
    .with_width(width: Option<f32>)
    .with_height(height: Option<f32>)
    
    // Header Slot
    .with_header(title: &str)
    .with_icon(icon: &str)
    .with_badge(badge: &str)
    .with_tool(tool_button: BView)
    
    // Collapsible Behavior
    .with_collapsible(collapsible: bool)
    .with_collapsed(collapsed: bool)
    .with_collapse_action(action: BAction)
    
    // Body Slot
    .with_body(body: BView)
    
    // Footer Slot
    .with_footer(status: &str)
    .with_footer_action(action_button: BView);

// Composable List & Item Primitives
BView::list()
    .with_gap(gap: Option<f32>)
    .with_item(item: BView)
    .with_items(items: Vec<BView>);

BView::item(label: &str)
    .with_icon(icon: &str)
    .with_badge(badge: &str)
    .with_active(active: bool)
    .with_action(action: BAction);
```

---

## 3. Composing Features Directly on `card()`

### 3.1 Theme Marketplace Item
```rust
let theme_card = BView::card()
    .with_shade(CardShade::Panel)
    .with_header("Tokyo Night")
    .with_icon("palette")
    .with_badge("★ 3.8k")
    .with_tool(
        BView::button()
            .with_icon("heart")
            .with_action(BAction::Toggle(BKey::ThemeFavorite("TokyoNight".to_string())))
    )
    .with_body(
        BView::row()
            .with_gap(Some(4.0))
            .with_child(BView::label_icon("", "square").with_color("#1a1b26"))
            .with_child(BView::label_icon("", "square").with_color("#7aa2f7"))
            .with_child(BView::label_icon("", "square").with_color("#bb9af7"))
            .with_child(BView::label_icon("", "square").with_color("#7dcfff"))
    )
    .with_footer("By @folke")
    .with_footer_action(
        BView::button()
            .with_label(if is_active { "Applied" } else { "Apply Theme" })
            .with_active(is_active)
            .with_action(BAction::Set(BKey::Theme, "TokyoNight".to_string()))
    );
```

---

### 3.2 Discord-Style Social Chat Drawer
```rust
let social_drawer = BView::card()
    .with_shade(CardShade::Activity)
    .with_width(Some(220.0))
    .with_header("DIRECT MESSAGES")
    .with_badge("3 Online")
    .with_tool(BView::button().with_icon("user-plus"))
    .with_body(
        BView::list()
            .with_gap(Some(2.0))
            .with_item(
                BView::item("@alice (3D Sculptor)")
                    .with_icon("user")
                    .with_badge("♀ Online")
                    .with_active(active_chat == "@alice")
                    .with_action(BAction::Set(BKey::ActiveChatUser, "@alice".to_string()))
            )
            .with_item(
                BView::item("@bob (Shader Artist)")
                    .with_icon("user")
                    .with_badge("♂ Online")
                    .with_active(active_chat == "@bob")
                    .with_action(BAction::Set(BKey::ActiveChatUser, "@bob".to_string()))
            )
            .with_item(
                BView::item("@carol (Animator)")
                    .with_icon("user")
                    .with_badge("♀ Away")
                    .with_action(BAction::Set(BKey::ActiveChatUser, "@carol".to_string()))
            )
    )
    .with_footer("Connected to #viewport-collab");
```

---

### 3.3 Collapsible Properties Panel Card
```rust
let is_collapsed = state.get(&BKey::CardCollapsed("properties".to_string())) == Some("true");

let properties_card = BView::card()
    .with_shade(CardShade::Panel)
    .with_collapsible(true)
    .with_collapsed(is_collapsed)
    .with_header("Render Properties")
    .with_icon("render")
    .with_tool(BView::button().with_icon("refresh_cw"))
    .with_collapse_action(BAction::Toggle(BKey::CardCollapsed("properties".to_string())))
    .with_body(
        if is_collapsed {
            BView::empty()
        } else {
            BView::list()
                .with_item(BView::prop_row("Render Engine", "Eevee"))
                .with_item(BView::prop_row("Sampling", "128"))
                .with_item(BView::prop_row("Ambient Occlusion", "Enabled"))
        }
    )
    .with_footer(
        if is_collapsed {
            BView::empty()
        } else {
            BView::label("Device: GPU Compute")
        }
    );
```

---

## 4. Activity Bar Direct Card Assembly

```rust
pub struct ActivityView<'a> {
    pub state: &'a BState,
}

impl<'a> ActivityView<'a> {
    pub fn new(state: &'a BState) -> Self { Self { state } }

    pub fn build(self) -> BView {
        let active = self.state.get(&BKey::ActivityTab).unwrap_or("home");
        let drawer_open = self.state.get(&BKey::ActivityDrawer) == Some("true");
        let expanded = self.state.get(&BKey::ActivityExpanded) == Some("true");

        let brand = BView::item(if expanded { "PHIBER" } else { "P" })
            .with_icon("zap")
            .with_action(BAction::Toggle(BKey::ActivityExpanded));

        let top_items = BView::column()
            .with_child(
                BView::button()
                    .with_icon("home")
                    .with_tooltip("Home (Layout Space)")
                    .with_active(active == "home")
                    .with_action(BAction::Set(BKey::CtxSpace, "Layout".to_string()))
            )
            .with_child(
                BView::button()
                    .with_icon("message-square")
                    .with_tooltip("Social Chat & Collaborators")
                    .with_active(active == "chat")
                    .with_action(BAction::Toggle(BKey::ActivityDrawer))
            )
            .with_child(
                BView::button()
                    .with_icon("grid")
                    .with_tooltip("Workspaces")
                    .with_active(active == "spaces")
                    .with_action(BAction::Toggle(BKey::ActivityDrawer))
            );

        let center_stream = BView::button()
            .with_icon("sparkles")
            .with_tooltip("AI Stream & Viewport Sharing")
            .with_active(active == "stream")
            .with_action(BAction::Set(BKey::ActivityTab, "stream".to_string()));

        let bottom_items = BView::column()
            .with_child(
                BView::button()
                    .with_icon("settings")
                    .with_tooltip("Settings & Theme Marketplace")
                    .with_action(BAction::Set(BKey::Modal, "Settings".to_string()))
            )
            .with_child(
                BView::button()
                    .with_icon("user")
                    .with_tooltip("User Profile")
                    .with_action(BAction::Set(BKey::Modal, "Profile".to_string()))
            );

        let primary_strip = BView::card()
            .with_shade(CardShade::Activity)
            .with_width(Some(if expanded { 140.0 } else { 44.0 }))
            .with_body(
                BView::column()
                    .with_child(brand)
                    .with_child(top_items)
                    .with_child(center_stream)
            )
            .with_footer_action(bottom_items);

        if drawer_open {
            BView::row()
                .with_child(primary_strip)
                .with_child(self.build_drawer(active))
        } else {
            primary_strip
        }
    }

    fn build_drawer(&self, active_tab: &str) -> BView {
        BView::card()
            .with_shade(CardShade::Activity)
            .with_width(Some(220.0))
            .with_header(active_tab)
            .with_body(BView::label(format!("{} active content", active_tab)))
    }
}
```
