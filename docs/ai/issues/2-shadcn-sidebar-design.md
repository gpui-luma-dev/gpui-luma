# SDK Control Redesign: Shadcn Sidebar Component Architecture & Engineering Spec

This document specifies the technical design, state machine, eventing model, `control_group` integration, layout primitive composition, component hierarchy, builder API ergonomics, and staged implementation plan for lifting the **Shadcn Sidebar** specification into `gpui-luma`.

> [!IMPORTANT]
> **Visual & Functional Parity Guarantee**: The existing `NavigationSidebar` presentation engine ([`crates/sdk/src/controls/navigation_sidebar/template.rs`](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/navigation_sidebar/template.rs)) is **visually and functionally correct**. 
> The redesign MUST preserve 100% of its visual quality (flush list row alignment, chevron disclosures `ChevronRight`/`ChevronDown`, depth-indented child rows, primary blue highlight fills for selected active items, section headers, and integrated scrollbars). Unstyled text stacks or card-shell wrappers (`SelectionPanel`/`Card` rounded boxes around menu groups) are strictly prohibited. The modern builder API (`SidebarControlBuilder`, `sidebar_group`, `sidebar_menu`) acts as the clean ergonomic builder surface driving this proven presentation engine.

> [!IMPORTANT]
> **Zero Public Backwards Compatibility Required**: The legacy `NavigationSidebar` control and `NavNode` recursive tree will be cleanly removed and replaced across the workspace. No public backwards-compatibility shims or adapter layers will be retained in the final public API.

---

## 1. Architectural Overview & Structural Composition

`SidebarControl` is the root layout controller (`Entity<SidebarControl>`) owning state, focus management, keybindings, and event dispatching. It composes `Sidebar` (panel shell) and `SidebarInset` (main workspace container) as sibling layout elements:

```
SidebarControl (Root Layout Controller & Entity State)
├── Sidebar (Outer Docked/Floating Panel Shell)
│   ├── SidebarHeader (Top branding / workspace switcher slot)
│   ├── SidebarContent (Scrollable item container - backed by ScrollContainer)
│   │   ├── SidebarGroup (Section container with optional label & action)
│   │   │   ├── SidebarGroupLabel
│   │   │   ├── SidebarGroupAction
│   │   │   └── SidebarMenu (Selection list - backed by ControlGroup)
│   │   │       └── SidebarMenuItem (Unified Recursive Menu Item)
│   │   │           ├── SidebarMenuButton (Icon + Label + State + Badge + Action)
│   │   │           └── SidebarMenuSub (Nested sub-menu branch)
│   │   │               └── SidebarMenuItem (Recursive child)
│   │   └── SidebarGroup
│   ├── SidebarFooter (Bottom user profile / settings slot)
│   └── SidebarRail (Interactive edge drag handle & collapse trigger)
├── SidebarInset (Main application workspace view container)
└── SidebarTrigger (Icon button for expanding/collapsing sidebar)
```

---

## 2. State Machine, Eventing & Layout Matrix

`SidebarControl` manages state transitions across three primary parameters: `open`, `collapsible`, and `variant`.

### 2.1 Complete `#[non_exhaustive]` Event Enum (`SidebarEvent`)

In accordance with SDK standards (like `toolbar` and `tabs_navigation`), `SidebarControl` emits strongly-typed semantic events via `cx.emit(SidebarEvent::...)`. Callbacks attached on builders (`.on_click(...)`) internally dispatch `SidebarEvent::Select` to ensure a single event-driven boundary for host applications and Theme Studio logs.

```rust
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub enum SidebarEvent {
    /// Fired when sidebar expands, collapses to icon rail, or slides offcanvas.
    OpenChanged { open: bool, collapsible: SidebarCollapsible },

    /// Fired when the mobile drawer or offcanvas sidebar is dismissed.
    Dismissed,

    /// Fired when an item or sub-item is selected.
    Select { id: SharedString },

    /// Fired when an item action button (e.g. "+") is activated.
    Activate { id: SharedString },

    /// Fired when keyboard or pointer focus moves to an item.
    ItemFocused { id: SharedString },

    /// Fired when item hover state changes (used for collapsed popover tracking).
    HoverChanged { id: Option<SharedString> },

    /// Fired when a sub-menu branch expands or collapses.
    SubMenuToggle { id: SharedString, open: bool },

    /// Fired when rail drag resizing begins.
    ResizeStart,

    /// Fired during interactive rail drag-resizing.
    Resized { width: Pixels },

    /// Fired when rail drag resizing completes.
    ResizeEnd { width: Pixels },

    /// Fired when sidebar enabled state changes.
    EnabledChanged { enabled: bool },
}
```

### 2.2 Layout State & Mobile Drawer Policy

| State Kind | `open` | `collapsible` Mode | Width / Placement | Visual & Interaction Behavior |
| :--- | :--- | :--- | :--- | :--- |
| **`Expanded`** | `true` | `Icon` \| `Offcanvas` \| `Responsive` \| `None` | `var(--sidebar-width)` (`16rem` / `256px`) | Full labels, icons, badges, expandable sub-menus, visible section headers. |
| **`CollapsedIcon`** | `false` | `SidebarCollapsible::Icon` | `var(--sidebar-width-icon)` (`3rem` / `48px`) | Compact icon rail. Text labels and badges hidden. Hovering items opens an `AnchoredPanel` popover. |
| **`CollapsedOffcanvas`**| `false` | `SidebarCollapsible::Offcanvas` | `0px` (Out of flow) | Desktop slide-offscreen (0px width). `SidebarInset` expands to 100% window width. |
| **`MobileDrawer`** | `true` | `SidebarCollapsible::Responsive` (< 768px) | Floating Drawer Overlay | When viewport width is below responsive breakpoint, `SidebarControl` renders `Sidebar` as a modal `SlidePanel` drawer over a dimmed backdrop. |
| **`Fixed`** | `true` | `SidebarCollapsible::None` | `var(--sidebar-width)` | Non-collapsible fixed width sidebar. |

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
3. **Flush List Layout (No Card-Shell Contamination)**:
   * Menu groups render as borderless, flush item lists using `NavigationSidebarItemLook` and `NavigationSidebarSectionLook`, strictly avoiding card-shell wrappers.
4. **Consolidated Builder Ergonomics**:
   * `SidebarMenuItemBuilder` directly supports `.icon()`, `.label()`, `.badge()`, `.action()`, and `.sub()` methods.

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

### Complete Workbench Usage Example

```rust
use gpui::*;
use gpui_luma::prelude::*;
use gpui_luma_look_shadcn::{ShadcnLook, ShadcnLookControlExt};
use lucide_icons::Icon as LucideIcon;

pub fn render_workbench(cx: &mut WindowContext) -> Entity<SidebarControl> {
    let look = cx.theme::<ShadcnLook>();

    look.sidebar_control("main_app_sidebar")
        .default_open(true)
        .collapsible(SidebarCollapsible::Icon)
        .variant(SidebarVariant::Sidebar)
        .sidebar(
            look.sidebar("workbench_sidebar")
                .header(look.sidebar_header().title("Properties").subtitle("Rectangle / Prominent card"))
                .content(
                    look.sidebar_content()
                        .group(
                            look.sidebar_group()
                                .label("Pinned")
                                .menu(
                                    look.sidebar_menu("pinned_menu")
                                        .item(look.sidebar_menu_item("summary", "Summary").icon(LucideIcon::Info))
                                        .item(look.sidebar_menu_item("tokens", "Design Tokens").icon(LucideIcon::Tags))
                                )
                        )
                        .group(
                            look.sidebar_group()
                                .label("Properties")
                                .menu(
                                    look.sidebar_menu("properties_menu")
                                        .item(
                                            look.sidebar_menu_item("layout", "Layout")
                                                .icon(LucideIcon::Ruler)
                                                .sub(
                                                    look.sidebar_menu_sub()
                                                        .item(look.sidebar_menu_item("position", "Position"))
                                                        .item(look.sidebar_menu_item("dimensions", "Dimensions").active(true))
                                                        .item(look.sidebar_menu_item("constraints", "Constraints"))
                                                        .item(look.sidebar_menu_item("grid", "Grid"))
                                                )
                                        )
                                        .item(
                                            look.sidebar_menu_item("look", "Look")
                                                .icon(LucideIcon::Palette)
                                        )
                                )
                        )
                )
                .footer(
                    look.sidebar_footer()
                        .child(look.sidebar_menu_item("audit_log", "Audit Log").icon(LucideIcon::FileText))
                        .child(look.sidebar_menu_item("reset_overrides", "Reset Overrides").icon(LucideIcon::RotateCcw).disabled(true))
                )
                .rail(look.sidebar_rail())
        )
        .inset(
            look.sidebar_inset()
                .header(hstack().child(look.sidebar_trigger()).child(render_breadcrumbs(cx)))
                .content(render_main_page_body(cx))
        )
        .spawn(cx)
}
```

---

## 6. Typed Look Metric Scale (`SidebarMetricScale`)

Zero hardcoded pixel constants. All dimensions are resolved dynamically via `SidebarMetricScale` from `ShadcnLook`:

```rust
pub struct SidebarMetricScale {
    pub width_expanded: Pixels,    // 256px
    pub width_icon_rail: Pixels,   // 48px
    pub width_mobile: Pixels,      // 288px
    pub padding_expanded: Pixels,  // 12px
    pub padding_icon_rail: Pixels, // 6px
    pub item_height: Pixels,       // 32px
    pub icon_size: Pixels,         // 16px
    pub rail_hit_width: Pixels,    // 6px
    pub popover_offset: Pixels,    // 8px
}
```

---

## 7. Staged Implementation & Side-by-Side Comparison Plan

To guarantee visual correctness and allow direct verification, the legacy `NavigationSidebar` will **NOT** be removed upfront. Implementation follows four staged steps with explicit side-by-side comparison in Luma Studio:

### Step 1: Connect `SidebarControl` to `NavigationSidebar`'s Flush Layout Engine (`crates/sdk/src/controls/sidebar/`)
* Build `SidebarControl` in `crates/sdk/src/controls/sidebar/` alongside `NavigationSidebar` (preserving `NavigationSidebar` intact).
* Update `SidebarControl` template rendering (`template.rs`) to delegate item row rendering directly to `ThemedNavigationSidebarTemplate` / `NavigationSidebarItemLook`.
* Eliminate all `Card` and `SelectionPanel` card-shell wrappers around menu groups, ensuring flush list rows with chevron disclosures (`ChevronRight`/`ChevronDown`), depth-indentation, and primary blue selection pills (`bg-primary`).

### Step 2: Wire Modern Builder Ergonomics (`crates/look-shadcn`)
* Wire `ShadcnLookControlExt::sidebar_control(...)`, `sidebar_group(...)`, `sidebar_menu(...)`, and `sidebar_menu_item(...)`.
* Connect `SidebarMetricScale` to `ShadcnLook`.

### Step 3: Luma Studio Side-by-Side Old vs New Comparison
* **Controls Exposition Tab** (`apps/luma-studio/src/studio/controls/control_exposition/navigation_sidebar.rs`): Update exposition to render **both Old (`NavigationSidebar`) and New (`SidebarControl`) side-by-side** (or via a side-by-side comparison toggle) for direct visual and behavioral verification.
* **Dashboard Tab** (`apps/luma-studio/src/studio/panels/dashboard.rs`): Support side-by-side / toggle comparison mode between `NavigationSidebar` and `SidebarControl`.
* Perform visual and functional verification ensuring 100% parity with the existing exposition (header, subtitle, section labels, chevrons, blue selection highlight, scrollbar, rail collapse).

### Step 4: Final Cleanup & Architecture Guide Update (Post-Verification Only)
* Only after `SidebarControl` is verified and approved as 100% proper and working, remove legacy `NavigationSidebar` structs (`crates/sdk/src/controls/navigation_sidebar/`).
* Update [`docs/architecture.md`](file:///Users/scg/Developer/GitHub/gpui-luma/docs/architecture.md) replacing `NavigationSidebar` references with `SidebarControl`.

