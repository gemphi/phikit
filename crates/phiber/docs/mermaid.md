# BUI Architectural Diagrams, Activity Bar & Universal Card Hierarchy

This document visualizes the complete declarative BUI architecture, featuring the **Universal Card Abstraction** (`Card`, `Header`, `Body`, `Footer`, `List`, `Item`), the **Full-Height Activity Bar**, and **Zero-Bespoke-Terms Component Assembly**.

---

## 1. Full Layout Hierarchy with Activity Bar

```mermaid
graph TD
    subgraph S["Space (Workspace Root)"]
        Z_Act["Zone: Activity Bar (Extreme Left, Full Height)"]
        Z_Top["Zone: Topbar"]
        Z_Left["Zone: Left (Sub-area)"]
        Z_Center["Zone: Center (Primary Viewport)"]
        Z_Right["Zone: Right (Stacked Properties)"]
        Z_Bottom["Zone: Bottom (Timeline / Drawers)"]
    end

    subgraph A_Act["Activity Bar Container (44px + 220px Drawer)"]
        Z_Act --> Act_Top["Top Pane: Brand / Home / Chat / Spaces"]
        Z_Act --> Act_Mid["Center Pane: Spark AI Stream"]
        Z_Act --> Act_Bot["Bottom Pane: Settings / User Profile"]
    end

    subgraph Drawer["Secondary Social Drawer (Slide-out)"]
        Act_Top --> Chat_Users["Discord Users: @alice (♀), @bob (♂)"]
        Act_Top --> Chat_Voice["Collab Viewport Channels"]
        Act_Top --> Space_List["Workspace Spaces in Secondary Palette"]
    end

    subgraph Area_Center["Center Editor Area"]
        Z_Center --> Host_View3D["Host: View3D"]
    end

    subgraph Area_Right["Right Editor Area"]
        Z_Right --> Host_Outliner["Host: Outliner"]
        Z_Right --> Host_Properties["Host: Buts"]
    end

    subgraph Cards["Universal Collapsible Cards (BView::Card)"]
        Host_View3D --> Card_View["Card: Viewport (3D Canvas + Header)"]
        Host_Outliner --> Card_Out["Card: Outliner (Collapsible)"]
        Host_Properties --> Card_Prop["Card: Properties (Collapsible Accordions)"]
    end

    classDef actStyle fill:#0f172a,stroke:#38bdf8,stroke-width:2px,color:#fff;
    classDef drawerStyle fill:#1e293b,stroke:#818cf8,stroke-width:2px,color:#fff;
    classDef cardStyle fill:#064e3b,stroke:#10b981,stroke-width:2px,color:#fff;
    classDef hostStyle fill:#312e81,stroke:#6366f1,stroke-width:2px,color:#fff;

    class Z_Act,A_Act,Act_Top,Act_Mid,Act_Bot actStyle;
    class Drawer,Chat_Users,Chat_Voice,Space_List drawerStyle;
    class Host_View3D,Host_Outliner,Host_Properties hostStyle;
    class Card_View,Card_Out,Card_Prop cardStyle;
```

---

## 2. Universal Card Slot Assembly: Zero Bespoke Structs

Every visual component is assembled from the same 6 standard primitive slots:

```mermaid
graph TD
    subgraph Card["Universal Card (BView::Card)"]
        direction TB
        
        subgraph Header["Card Header (BView::CardHeader)"]
            Title["Title Label / Icon"]
            Badge["Badge / Star Rating"]
            Tools["Tool Buttons / Extra Actions"]
            CollapseChevron["[▾/▸] Collapse Toggle"]
        end
        
        subgraph Body["Card Body (BView::CardBody)"]
            subgraph List["Card List (BView::CardList)"]
                Item1["Card Item (BView::CardItem)"]
                Item2["Card Item (BView::CardItem)"]
                Item3["Card Item (BView::CardItem)"]
            end
            CustomContent["3D Canvas / Color Swatches / Accordion"]
        end
        
        subgraph Footer["Card Footer (BView::CardFooter)"]
            StatusText["Status / Creator / Breadcrumb"]
            ActionBtn["Apply / Save Action Button"]
            StatsBadge["Metrics / Active Counts"]
        end
    end

    Header --> Body --> Footer

    classDef cardCont fill:#0f172a,stroke:#38bdf8,stroke-width:3px,color:#fff;
    classDef headerCont fill:#1e293b,stroke:#0ea5e9,stroke-width:2px,color:#fff;
    classDef bodyCont fill:#1e293b,stroke:#22c55e,stroke-width:2px,color:#fff;
    classDef footerCont fill:#1e293b,stroke:#eab308,stroke-width:2px,color:#fff;
    classDef itemCont fill:#334155,stroke:#94a3b8,stroke-width:1px,color:#fff;

    class Card cardCont;
    class Header headerCont;
    class Body,List bodyCont;
    class Footer footerCont;
    class Item1,Item2,Item3,Title,Badge,Tools,CollapseChevron,StatusText,ActionBtn,StatsBadge,CustomContent itemCont;
```

---

## 3. All UI Features Reusing the Exact Same Card Slots

```mermaid
graph LR
    subgraph UniversalSlots["Standard Card Slots"]
        C_Head["Header: title, icon, badge, tools"]
        C_Body["Body: list, canvas, custom rows"]
        C_Foot["Footer: status, actions, metrics"]
    end

    subgraph Examples["Composed Without New Structs"]
        ThemeCard["Theme Marketplace Item:<br/>• Header: 'Tokyo Night' + ★3.8k<br/>• Body: Color Swatch Row<br/>• Footer: 'By @folke' + [Apply]"]
        ChatCard["Discord Social Drawer:<br/>• Header: 'DIRECT MESSAGES' + 3 Online<br/>• Body: List of User Items (@alice ♀)<br/>• Footer: '#viewport-collab'"]
        PropCard["Render Properties Panel:<br/>• Header: 'Render Properties' + [▾]<br/>• Body: Prop Rows (Engine, Sampling)<br/>• Footer: 'Device: GPU'"]
        ViewportCard["3D Viewport:<br/>• Header: 'User Perspective'<br/>• Body: 3D Wireframe Canvas<br/>• Footer: 'Grid (1.0m)'"]
    end

    C_Head --> ThemeCard
    C_Body --> ThemeCard
    C_Foot --> ThemeCard

    C_Head --> ChatCard
    C_Body --> ChatCard
    C_Foot --> ChatCard

    C_Head --> PropCard
    C_Body --> PropCard
    C_Foot --> PropCard

    C_Head --> ViewportCard
    C_Body --> ViewportCard
    C_Foot --> ViewportCard

    classDef slotStyle fill:#1e293b,stroke:#38bdf8,stroke-width:2px,color:#fff;
    classDef compStyle fill:#0f172a,stroke:#10b981,stroke-width:2px,color:#fff;

    class C_Head,C_Body,C_Foot slotStyle;
    class ThemeCard,ChatCard,PropCard,ViewportCard compStyle;
```

---

## 4. Activity Bar 3-Pane Structure & Drawer Flow

```mermaid
sequenceDiagram
    autonumber
    participant User as User
    participant ActBar as Activity Bar (44px)
    participant Drawer as Social Drawer (220px)
    participant State as BState
    participant GPUI as GPUI Frame

    User->>ActBar: Click [💬] Chat Icon
    ActBar->>State: state.set(BKey::ActivityTab, "chat")
    ActBar->>State: state.set(BKey::ActivityDrawer, "true")
    State->>Drawer: Render Card::card_list([@alice (♀), @bob (♂)])
    Drawer->>GPUI: Draw side-by-side drawer

    User->>Drawer: Click @alice (3D Sculptor)
    Drawer->>State: state.set(BKey::ActiveChatUser, "@alice")
    State->>GPUI: Open direct message pane

    User->>ActBar: Click [⌂] Home Icon
    ActBar->>State: state.set(BKey::CtxSpace, "Layout")
    ActBar->>State: state.set(BKey::ActivityDrawer, "false")
    State->>GPUI: Close drawer & show 3D Viewport in Layout space
```
