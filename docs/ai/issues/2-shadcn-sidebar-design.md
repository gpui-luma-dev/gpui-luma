# SDK Control Redesign: Shadcn Sidebar Component Architecture & Engineering Spec

This document specifies the technical design, state machine, eventing model, `control_group` integration, component hierarchy, builder API ergonomics, and implementation plan for lifting the **Shadcn Sidebar** specification into `gpui-luma`.

It replaces the early, monolithic `NavigationSidebar` control (`NavNode` state tree) with a modular set of primitives adhering strictly to `gpui-luma`'s LMTP (Model-Control-Template-Theme) split, `control_group` selection engine, and `crates/look-shadcn` theme architecture.

> [!IMPORTANT]
> **Zero Backwards Compatibility Required**: The legacy `NavigationSidebar` control and `NavNode` recursive tree will be cleanly removed and replaced across the workspace. No backwards-compatibility shims, adapter layers, or deprecated aliases will be retained.

---

## 1. Architectural Overview & Component Hierarchy

The Sidebar system decomposes layout into declarative sub-components matching the standard Shadcn specification:

```
SidebarControl (Root Context & State Controller)
├── Sidebar (Outer Container & Structural Shell)
│   ├── SidebarHeader (Top branding / workspace switcher slot)
│   ├── SidebarContent (Scrollable item area - backed by ScrollContainer)
│   │   ├── SidebarGroup (Section container with optional header & actions)
│   │   │   ├── SidebarGroupLabel
│   │   │   ├── SidebarGroupAction
│   │   │   ├── SidebarGroupContent
│   │   │   └── SidebarMenu (Selection engine - backed by ControlGroup)
│   │   │       ├── SidebarMenuItem
│   │   │       │   ├── SidebarMenuButton (Icon + Label + State + Badge + Action)
│   │   │       │   └── SidebarMenuSub (Nested disclosure sub-menu)
│   │   │       │       ├── SidebarMenuSubItem
│   │   │       │       └── SidebarMenuSubItem
│   │   └── SidebarGroup
│   ├── SidebarFooter (Bottom user profile / settings slot)
│   └── SidebarRail (Interactive edge drag/collapse rail trigger)
├── SidebarInset (Main application content container)
└── SidebarTrigger (Icon button for expanding/collapsing sidebar)
```

---

## 2. State Machine, Eventing & Rail Behavior Matrix

`SidebarControl` manages state transitions across three primary layout parameters: `open`, `collapsible`, and `variant`.

### 2.1 Unified `SidebarEvent` Architecture

In accordance with SDK standards (like `toolbar` and `tabs_navigation`), `SidebarControl` emits strongly-typed events via `cx.emit(SidebarEvent::...)`. This allows parent workbenches, app routing, and Theme Studio inspector event streams to subscribe cleanly.

```rust
#[derive(Clone, Debug, PartialEq)]
pub enum SidebarEvent {
    /// Fired when sidebar expands, collapses to icon rail, or slides offcanvas.
    OpenChanged { open: bool, collapsible: SidebarCollapsible },

    /// Fired when an item or sub-item is selected.
    Select { id: SharedString },

    /// Fired when item hover state changes (used for collapsed popovers).
    HoverChanged { id: SharedString, hovered: bool },

    /// Fired when a nested sub-menu branch expands or collapses.
    SubMenuToggle { id: SharedString, open: bool },

    /// Fired during interactive rail drag-resizing.
    Resized { width: Pixels },
}
```

### 2.2 Layout State Definitions

| State Kind | `open` | `collapsible` Mode | Container Width | Visual Behavior |
| :--- | :--- | :--- | :--- | :--- |
| **`Expanded`** | `true` | `Icon` \| `Offcanvas` \| `None` | `var(--sidebar-width)` (`16rem` / `256px`) | Full labels, icons, badges, expandable sub-menus, visible group section titles. |
| **`CollapsedIcon`** | `false` | `SidebarCollapsible::Icon` | `var(--sidebar-width-icon)` (`3rem` / `48px`) | Compact icon rail. Text labels, badges, and group headers hidden. Hovering items triggers an `AnchoredPanel` popover showing item label & sub-items. |
| **`CollapsedOffcanvas`** | `false` | `SidebarCollapsible::Offcanvas` | `0px` (Hidden / Out of flow) | Fully hidden from layout flow. `SidebarInset` expands to 100% viewport width. Toggling open on mobile slides a `SlidePanel` drawer over backdrop. |
| **`Fixed`** | `true` / `false` | `SidebarCollapsible::None` | `var(--sidebar-width)` | Fixed width navigation sidebar. Cannot be collapsed. |

### 2.3 Component Visual & Interaction Matrix

| Sub-Component | `Expanded` State | `CollapsedIcon` Rail State | `CollapsedOffcanvas` State |
| :--- | :--- | :--- | :--- |
| **`Sidebar` Container** | `w-[16rem]` (`256px`), border-right. | `w-[3rem]` (`48px`), border-right. | `w-0` (`0px`), `overflow-hidden`. |
| **`SidebarHeader`** | Full branding layout (logo + text + dropdown). | Compact icon-only logo or workspace avatar. | Hidden. |
| **`SidebarGroupLabel`** | Section title text (uppercase/muted). | Hidden (`display: none` / 0-height spacer). | Hidden. |
| **`SidebarGroupAction`** | Right-aligned action button (e.g. "+"). | Hidden. | Hidden. |
| **`SidebarMenuButton`** | `hstack(icon, label, flex_spacer, badge, action)`. | Centered `32x32px` icon square. Text/badge hidden. Hover displays `AnchoredPanel` tooltip. | Hidden. |
| **`SidebarMenuSub`** | Indented nested list under parent item (`pl-3.5`). | Hidden inline. Opens floating dropdown inside `AnchoredPanel` on item hover/click. | Hidden. |
| **`SidebarFooter`** | Full user profile row (avatar + name + email). | Avatar icon only. Hover opens user menu popover. | Hidden. |
| **`SidebarRail`** | 4px edge handle. Hover highlights `bg-sidebar-border`. Drag resizes width; click toggles collapse. | 4px edge handle. Hover highlights. Click expands. | Hidden. |
| **`SidebarInset`** | Standard main view. Adjacent to sidebar. | Shifts left to fill saved rail margin (`ml-[3rem]`). | Full window width (`ml-0`). |

### 2.4 Collapsed Icon Rail Deep-Dive Implementation Spec

When `SidebarState::is_icon_rail()` is true (`open == false` and `collapsible == Icon`), the presentation engine enforces explicit layout transformations:

#### 1. Container Sizing & Padding Rules
* **Width**: Shrinks to `--sidebar-width-icon` (`3rem` / `px(48.0)`).
* **Inner Padding**: Changes from `px(12.0)` to `px(6.0)` (`px-1.5 py-2`), ensuring all inner touch targets fit within `36px` to `48px` bounds.
* **ScrollContainer**: `SidebarContent` maintains scrollability, but hides horizontal overflow (`overflow_x_hidden`).

#### 2. Item Layout Transformation (`SidebarMenuButton`)
* **Square Touch Target**: Button renders as a fixed `32x32px` or `36x36px` square (`w-8 h-8` or `w-9 h-9`), centered horizontally (`items_center`, `justify_center`).
* **Hidden Elements**: Text label (`SidebarMenuButtonBuilder::label`), badge (`badge`), and right action (`action`) are excluded from inline layout (`display: none`).
* **Active Indicator**: Active item renders a subtle left accent pill or background highlight fill (`bg-sidebar-accent text-sidebar-accent-foreground`).

#### 3. Group Label & Divider Behavior
* `SidebarGroupLabel` renders as a hidden 0-height spacer OR an optional subtle `1px` horizontal border divider (`w-6 h-[1px] bg-sidebar-border mx-auto my-2`), visually separating groups without text clutter.

#### 4. Collapsed Hover Popover Engine (`AnchoredPanel`)
Hovering or focusing a collapsed icon item opens an [`AnchoredPanel`](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/anchored_panel):
* **Anchor Alignment**: Target corner `AnchorCorner::TopRight` of icon button -> Anchor corner `AnchorCorner::TopLeft` of popover.
* **Horizontal Offset**: `px(8.0)` right margin gap from the rail edge.
* **Content Rendering**:
  * **Items WITHOUT Sub-menus**: Renders a compact floating label badge (`bg-sidebar-accent text-sidebar-accent-foreground shadow-md rounded-md px-3 py-1.5 text-xs font-medium`).
  * **Items WITH Sub-menus (`SidebarMenuSub`)**: Renders a floating popover card containing the item header title + the full nested `SidebarMenuSub` selection list.
* **Hover Persistence**: The popover remains open while the cursor is over the icon button OR inside the `AnchoredPanel` popover area.

### 2.5 Scrolling Mechanics & Scroll Event Propagation Spec

Long navigation lists inside `SidebarContent` require explicit scroll handling and scroll isolation policies:

#### 1. `ScrollContainer` Engine Integration
* `SidebarContent` wraps inner groups and menus in SDK [`ScrollContainer`](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/scroll_container.rs) / [`scrollbar`](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/scrollbar).
* **Expanded & Rail Parity**: Vertical scrolling is supported identically in both `Expanded` and `CollapsedIcon` modes.
* **Horizontal Scroll Suppression**: Horizontal scrolling is strictly disabled (`overflow_x_hidden`), preventing awkward horizontal shifts during collapse animations.

#### 2. Scroll Event Isolation (Preventing Scroll Chaining)
* **Default Behavior (`scroll_propagation: false`)**: `SidebarContent` traps wheel and trackpad scroll events when the cursor is positioned over the sidebar viewport. This prevents "scroll leaking"—i.e., scrolling to the bottom of the sidebar will **not** trigger accidental scrolling of the main application page in `SidebarInset`.
* **Configurable Boundary Chaining**: `SidebarContentBuilder::scroll_propagation(bool)` allows callers to explicitly opt into scroll chaining if the sidebar is hosted within a scrollable parent page.

#### 3. Keyboard Scroll Coordination
* When `SidebarContent` is focused, `PageUp`, `PageDown`, `Home`, and `End` keys adjust the `ScrollContainer` scroll offset smoothly while keeping active `control_group` focus visible.

---

## 3. `ControlGroup` Selection & Focus Engine Integration

Instead of the legacy `NavigationSidebar`'s ad-hoc callback vectors (`NavigationSidebarTemplateHandlers`), `SidebarMenu` delegates item selection, active descendant tracking, and keyboard focus directly to [`ControlGroupBuilder`](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/control_group).

### 3.1 Benefits of `ControlGroup` Backing

1. **Roving Keyboard Focus & ARIA**:
   * Up/Down arrow keys navigate items across menu groups.
   * Home/End keys jump to the first/last menu items.
   * Left/Right arrow keys collapse or expand sub-menus (`SidebarMenuSub`).
2. **Visual Parity (`ControlGroupItemVisualContext`)**:
   * Reuses standard SDK selection fills and state resolutions (`hovered`, `pressed`, `selected`, `focused`, `disabled`).
   * Eliminates hardcoded pixel math and custom hover state logic.
3. **Consolidated Builder Ergonomics**:
   * In Rust, we avoid React's noisy 4-layer wrapper nesting (`SidebarMenuItem -> SidebarMenuButton + SidebarMenuAction + SidebarMenuBadge`).
   * `SidebarMenuButtonBuilder` directly supports `.icon()`, `.label()`, `.badge()`, `.action()`, and `.sub()` methods.

---

## 4. Integration with SDK Layout Primitives & Overlays

Rather than writing custom positioning, scrolling, or drag logic, `SidebarControl` composes established SDK primitives:

1. **[`DockPanel`](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/layouts/dock_panel.rs)** (`crates/sdk/src/layouts/dock_panel.rs`):
   * Enforces edge-docking alignment (`Left` vs `Right` sidebars) relative to `SidebarInset`.
2. **[`DockSplitter`](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/dock_splitter)** / **[`ResizablePanels`](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/resizable_panels)**:
   * `SidebarRail` delegates drag-resizing and min/max width constraints (`200px` to `480px`) directly to `dock_splitter` primitives.
3. **[`SlidePanel`](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/slide_panel)** (`crates/sdk/src/controls/slide_panel`):
   * When `SidebarCollapsible::Offcanvas` is opened on mobile viewports, rendering delegates to `SlidePanel` for smooth edge-slide animations, backdrop dimming, and Escape key dismissal.
4. **[`ScrollContainer`](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/scroll_container.rs)** (`crates/sdk/src/controls/scroll_container.rs`):
   * `SidebarContent` delegates inner scrolling to `ScrollContainer`, ensuring scrollbar visibility and keyboard scrolling.
5. **[`AnchoredPanel`](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/anchored_panel)** (`crates/sdk/src/controls/anchored_panel`):
   * Item tooltips and floating sub-menus in collapsed icon rail mode leverage `AnchoredPanel` for precise edge alignment and focus management.

---

## 5. Developer Usability: Complete Builder API Spec

Below is the production Builder API pattern for `gpui-luma` apps (`apps/luma-studio`, `apps/shells`).

### 5.1 Complete Application Workbench Example

```rust
use gpui::*;
use gpui_luma::prelude::*;
use gpui_luma_look_shadcn::{ShadcnLook, ShadcnLookControlExt};
use lucide_icons::Icon as LucideIcon;

pub fn render_workbench(cx: &mut WindowContext) -> Entity<SidebarControl> {
    let look = cx.theme::<ShadcnLook>();

    look.sidebar_control("main_app_sidebar")
        .default_open(true)
        .collapsible(SidebarCollapsible::Icon) // Icon | Offcanvas | None
        .variant(SidebarVariant::Sidebar)      // Sidebar | Floating | Inset
        .side(SidebarSide::Left)
        .shortcut("cmd-b")
        .sidebar(
            look.sidebar("workbench_sidebar")
                .header(
                    look.sidebar_header()
                        .child(render_workspace_switcher(cx))
                )
                .content(
                    look.sidebar_content()
                        .group(
                            look.sidebar_group()
                                .label("Platform")
                                .action(look.ghost_icon_button("add_platform_btn", LucideIcon::Plus))
                                .menu(
                                    look.sidebar_menu("platform_menu")
                                        .item(
                                            look.sidebar_menu_button("nav_playground")
                                                .icon(LucideIcon::Terminal)
                                                .label("Playground")
                                                .badge("v2.0")
                                                .active(true)
                                                .on_click(|_event, cx| {
                                                    // Navigate to playground
                                                })
                                        )
                                        .item(
                                            look.sidebar_menu_button("nav_models")
                                                .icon(LucideIcon::Bot)
                                                .label("Models")
                                                .action(look.ghost_icon_button("models_more", LucideIcon::MoreHorizontal))
                                                .sub(
                                                    look.sidebar_menu_sub()
                                                        .item("Genesis", |cx| { /* select model */ })
                                                        .item("Explorer", |cx| { /* select model */ })
                                                        .item("Quantum", |cx| { /* select model */ })
                                                )
                                        )
                                )
                        )
                        .group(
                            look.sidebar_group()
                                .label("Projects")
                                .menu(
                                    look.sidebar_menu("projects_menu")
                                        .item(
                                            look.sidebar_menu_button("proj_design_system")
                                                .icon(LucideIcon::Folder)
                                                .label("Design System")
                                        )
                                        .item(
                                            look.sidebar_menu_button("proj_analytics")
                                                .icon(LucideIcon::PieChart)
                                                .label("Analytics")
                                        )
                                )
                        )
                )
                .footer(
                    look.sidebar_footer()
                        .child(render_user_profile(cx))
                )
                .rail(look.sidebar_rail())
        )
        .inset(
            look.sidebar_inset()
                .header(
                    hstack()
                        .gap_cn("gap-2")
                        .items_center()
                        .child(look.sidebar_trigger())
                        .child(divider_vertical())
                        .child(render_breadcrumbs(cx))
                )
                .content(
                    render_main_page_body(cx)
                )
        )
        .spawn(cx)
}
```

### 5.2 Sub-Control Builder Signatures

```rust
pub struct SidebarControlBuilder {
    id: SharedString,
    default_open: bool,
    collapsible: SidebarCollapsible,
    variant: SidebarVariant,
    side: SidebarSide,
    shortcut: Option<KeyBinding>,
    sidebar: Option<SidebarBuilder>,
    inset: Option<SidebarInsetBuilder>,
}

impl SidebarControlBuilder {
    pub fn default_open(mut self, open: bool) -> Self;
    pub fn collapsible(mut self, mode: SidebarCollapsible) -> Self;
    pub fn variant(mut self, variant: SidebarVariant) -> Self;
    pub fn side(mut self, side: SidebarSide) -> Self;
    pub fn shortcut(mut self, shortcut: impl Into<SharedString>) -> Self;
    pub fn sidebar(mut self, sidebar: SidebarBuilder) -> Self;
    pub fn inset(mut self, inset: SidebarInsetBuilder) -> Self;
    pub fn spawn(self, cx: &mut WindowContext) -> Entity<SidebarControl>;
}

pub struct SidebarMenuButtonBuilder {
    id: SharedString,
    icon: Option<LucideIcon>,
    label: Option<SharedString>,
    badge: Option<SharedString>,
    action: Option<AnyElement>,
    sub: Option<SidebarMenuSubBuilder>,
    active: bool,
    disabled: bool,
    tooltip: Option<SharedString>,
    on_click: Option<Box<dyn Fn(&ClickEvent, &mut WindowContext)>>,
}

impl SidebarMenuButtonBuilder {
    pub fn icon(mut self, icon: LucideIcon) -> Self;
    pub fn label(mut self, label: impl Into<SharedString>) -> Self;
    pub fn badge(mut self, badge: impl Into<SharedString>) -> Self;
    pub fn action(mut self, action: impl IntoElement) -> Self;
    pub fn sub(mut self, sub: SidebarMenuSubBuilder) -> Self;
    pub fn active(mut self, active: bool) -> Self;
    pub fn disabled(mut self, disabled: bool) -> Self;
    pub fn tooltip(mut self, tooltip: impl Into<SharedString>) -> Self;
    pub fn on_click(mut self, handler: impl Fn(&ClickEvent, &mut WindowContext) + 'static) -> Self;
}
```

### 5.3 Direct Migration Comparison: Luma Studio Properties Tree

The existing Luma Studio Navigation Sidebar exposition ([`navigation_sidebar.rs`](file:///Users/scg/Developer/GitHub/gpui-luma/apps/luma-studio/src/studio/controls/control_exposition/navigation_sidebar.rs)) currently constructs a monolithic `NavNode` tree:

```rust
// Legacy NavNode approach in Luma Studio:
let sidebar = look
    .navigation_sidebar("controls-doc-navigation-sidebar")
    .title("Properties")
    .subtitle("Rectangle / Prominent card")
    .collapsible(true)
    .selected_id("dimensions")
    .items(vec![
        NavNode::section("pinned-label", "Pinned"),
        NavNode::new("summary").label("Summary").icon(LucideIcon::Info),
        NavNode::new("tokens").label("Design Tokens").icon(LucideIcon::Tags),
        NavNode::section("properties-label", "Properties"),
        NavNode::new("layout").label("Layout").icon(LucideIcon::Ruler).expanded(true).children(vec![
            NavNode::new("position").label("Position"),
            NavNode::new("dimensions").label("Dimensions"),
        ]),
    ])
    .footer_nodes(vec![
        NavNode::new("audit-log").label("Audit Log").icon(LucideIcon::FileText),
    ])
    .spawn(cx);
```

Under the new **Shadcn Sidebar Builder API**, this refactors to explicit, composable components:

```rust
// Modern Shadcn Sidebar approach in Luma Studio:
let sidebar = look
    .sidebar("controls-doc-navigation-sidebar")
    .header(
        look.sidebar_header()
            .title("Properties")
            .subtitle("Rectangle / Prominent card")
    )
    .content(
        look.sidebar_content()
            .group(
                look.sidebar_group()
                    .label("Pinned")
                    .menu(
                        look.sidebar_menu("pinned_menu")
                            .item(look.sidebar_menu_button("summary").label("Summary").icon(LucideIcon::Info))
                            .item(look.sidebar_menu_button("tokens").label("Design Tokens").icon(LucideIcon::Tags))
                    )
            )
            .group(
                look.sidebar_group()
                    .label("Properties")
                    .menu(
                        look.sidebar_menu("properties_menu")
                            .item(
                                look.sidebar_menu_button("layout_btn")
                                    .label("Layout")
                                    .icon(LucideIcon::Ruler)
                                    .sub(
                                        look.sidebar_menu_sub()
                                            .item("Position", |_cx| { ... })
                                            .item("Dimensions", |_cx| { ... })
                                    )
                            )
                    )
            )
    )
    .footer(
        look.sidebar_footer().child(
            look.sidebar_menu_button("audit-log").label("Audit Log").icon(LucideIcon::FileText)
        )
    )
    .spawn(cx);
```

### 5.4 Deeper Node Hierarchies & Recursive Sub-Menus

**Yes, the architecture supports arbitrarily deep node hierarchies.**

While standard sidebar UX typically stays within 1–3 levels, `SidebarMenuSubItem` supports **recursive sub-menu nesting**:

```rust
// 3+ Levels Deep Hierarchy Example:
look.sidebar_menu("docs_menu")
    .item(
        look.sidebar_menu_button("api_btn")
            .label("API Reference")
            .icon(LucideIcon::Code)
            .sub(
                look.sidebar_menu_sub()
                    .item(
                        look.sidebar_menu_sub_item("v1_docs")
                            .button(look.sidebar_menu_sub_button("v1_btn").label("v1 REST API"))
                            .sub(
                                look.sidebar_menu_sub()
                                    .item("Authentication", |_cx| { /* nav */ })
                                    .item("Endpoints", |_cx| { /* nav */ })
                            )
                    )
            )
    )
```

---

## 6. CSS Token Architecture (`crates/look-shadcn/assets/native.css`)

```css
:root {
  --sidebar-background: hsl(var(--background));
  --sidebar-foreground: hsl(var(--foreground));
  --sidebar-primary: hsl(var(--primary));
  --sidebar-primary-foreground: hsl(var(--primary-foreground));
  --sidebar-accent: hsl(var(--accent));
  --sidebar-accent-foreground: hsl(var(--accent-foreground));
  --sidebar-border: hsl(var(--border));
  --sidebar-ring: hsl(var(--ring));
  
  --sidebar-width: 16rem;         /* 256px expanded */
  --sidebar-width-icon: 3rem;     /* 48px icon rail */
  --sidebar-width-mobile: 18rem;  /* 288px mobile drawer */
}

.dark {
  --sidebar-background: hsl(240 5.9% 10%);
  --sidebar-foreground: hsl(240 4.8% 95.9%);
  --sidebar-primary: hsl(224.3 76.3% 48%);
  --sidebar-primary-foreground: hsl(0 0% 100%);
  --sidebar-accent: hsl(240 3.7% 15.9%);
  --sidebar-accent-foreground: hsl(240 4.8% 95.9%);
  --sidebar-border: hsl(240 3.7% 15.9%);
  --sidebar-ring: hsl(217.2 91.2% 59.8%);
}
```

---

## 7. Implementation & Verification Plan (Single Integrated Phase)

Because there is nothing interactive to verify until Luma Studio and app shells are updated, the entire refactor is executed in **Phase 1** as a single end-to-end atomic delivery with **zero backwards compatibility**:

1. **Clean Removal & Core Primitives (`crates/sdk/src/controls/sidebar/`)**:
   * Delete legacy `NavigationSidebar` / `NavNode` files.
   * Implement `SidebarControl`, `SidebarControlBuilder`, `SidebarState`, `Sidebar`, `SidebarHeader`, `SidebarContent`, `SidebarGroup`, `SidebarMenu` (backed by `control_group`), `SidebarMenuItem`, `SidebarMenuButton`, `SidebarMenuSub`, `SidebarFooter`, `SidebarRail`, `SidebarInset`, and `SidebarTrigger`.

2. **Theme Binding & CSS (`crates/look-shadcn`)**:
   * Add `cx.theme::<ShadcnLook>().sidebar_control(...)` factory and builder styling extensions on `ShadcnLookControlExt`.
   * Update CSS token mappings in `crates/look-shadcn/assets/native.css`.

3. **Complete Dev-Site & App Integration (Theme Studio & Shells)**:
   * **Dashboard Tab**: Migrate `apps/luma-studio/src/studio/panels/dashboard.rs` to render the new `SidebarControl` builder layout.
   * **Controls Exposition Tab**: Migrate `apps/luma-studio/src/studio/controls/control_exposition/navigation_sidebar.rs` (and associated inspector adapters) to demonstrate `SidebarControl` features, icon collapse mode, and sub-menu branches.
   * **Style Guide Section**: Migrate `apps/luma-studio/src/studio/style/sections/sidebar/mod.rs` & `style_guide.rs` to render `SidebarControl` previews.
   * **Shell Reference Apps**: Update `apps/shells/` to consume `SidebarControl`.

4. **Verification**:
   * Build workspace (`cargo check`, `cargo clippy`).
   * Launch `apps/luma-studio` to visually and interactively verify sidebar behaviors in the Dashboard, Controls Exposition, and Style Guide.
