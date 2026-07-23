# SDK Control Eventing Specification & Comprehensive Control Audit

This document defines the standardized event taxonomy for the `gpui-luma` SDK controls library (`crates/sdk/src/controls`). It establishes consistent rules for state, lifecycle, interaction, and value change events across all control families.

---

## 1. Event Invariants & Architectural Taxonomy

All SDK controls emit semantic events via GPUI's `EventEmitter` mechanism (`cx.emit(...)`). Templates render presentation element trees (`Div`) and hook element listeners to forward user actions to control entities; **templates never emit events directly**.

### 1.1 Core Event Taxonomy

Events across all control families are categorized into four standard layers:

| Layer | Event Pattern | Payload | Description / Purpose |
| :--- | :--- | :--- | :--- |
| **Action & Activation** | `Click`, `Activate { id }`, `Submit { value }` | Typed identifier / value | Primary activation (pointer click or keyboard `Enter`/`Space`). |
| **Value Mutation** | `Change { value }`, `LiveChange { value }`, `Commit { value }` | Value type `T` | State mutation. `LiveChange` fires per drag/keystroke; `Commit` fires on mouse up/blur. |
| **Focus & Keyboard Navigation** | `FocusChanged { focused: bool }` or `Focus`, `Blur` | `bool` or unit | Tracks focus ring entry and exit for form validation, accessibility, and event logging. |
| **Interactive & Lifecycle State** | `EnabledChanged { enabled: bool }`, `HoverChanged { hovered: bool }` | `bool` | Emits on programmatic state changes or pointer enter/leave. Useful for parent container layouts and tooltips. |

### 1.2 Guiding Consistency Rules

1. **User-Driven vs. Programmatic Mutations**:
   - User interactions (mouse clicks, keypresses, drags) **MUST** emit semantic events.
   - Direct programmatic setters (e.g. `control.set_value(val, cx)`) update visual state and call `cx.notify()`, but **MUST NOT** emit events, avoiding feedback loop cascades.
2. **Payload Naming Standard**:
   - Boolean toggles use `value: bool` or `selected: bool`.
   - String payloads use `value: String`.
   - Identifiers use `id: SharedString`.
   - State transition flags use `focused: bool`, `enabled: bool`, `hovered: bool`.
3. **Container Event Fan-In**:
   - Container controls (`Toolbar`, `ControlGroup`, `Form`) aggregate and fan-in item events, surfacing child events wrapped in a parent envelope (e.g. `ToolbarEvent::ItemClick { item_id }`).

---

## 2. Comprehensive SDK Control Event Audit

Below is the complete audit of all control families in `crates/sdk/src/controls/`, detailing their existing events, missing events, and proposed standardized event signature.

---

### 2.1 Command & Action Controls

#### **Button / IconButton (`command/button`, `command/icon_button`)**
* **Core Engine**: `CommandCore`
* **Current Events**: `CommandEvent::Click`
* **Audit & Gap Analysis**: Lacks focus, enabled, and hover state notifications needed for event logs and host container coordination.
* **Proposed Standardized Event (`ButtonEvent`)**:
  ```rust
  pub enum ButtonEvent {
      Click,
      FocusChanged { focused: bool },
      EnabledChanged { enabled: bool },
      HoverChanged { hovered: bool },
  }
  ```

#### **Toolbar (`toolbar`)**
* **Current Events**: `ToolbarEvent::Action { item_id }`, `ToolbarEvent::Change { item_id, selected }`
* **Audit & Gap Analysis**: Aggregates child controls. Needs focus and item enable/disable fan-in.
* **Proposed Standardized Event (`ToolbarEvent`)**:
  ```rust
  pub enum ToolbarEvent {
      Action { item_id: SharedString },
      Change { item_id: SharedString, selected: bool },
      FocusChanged { focused: bool },
      ItemEnabledChanged { item_id: SharedString, enabled: bool },
  }
  ```

---

### 2.2 Boolean & Choice Controls

#### **Checkbox (`checkbox`)**
* **Current Wrapper**: `Button<bool>`
* **Current Events**: Emits `CommandEvent::Click`
* **Audit & Gap Analysis**: Currently relies on command clicks. Needs explicit boolean `Change` event and focus state transitions.
* **Proposed Standardized Event (`CheckboxEvent`)**:
  ```rust
  pub enum CheckboxEvent {
      Change { value: bool },
      FocusChanged { focused: bool },
      EnabledChanged { enabled: bool },
  }
  ```

#### **Switch (`switch`)**
* **Current Events**: `SwitchEvent::Change { value: bool }` (implicit via toggle)
* **Audit & Gap Analysis**: Lacks standardized focus and interaction state events.
* **Proposed Standardized Event (`SwitchEvent`)**:
  ```rust
  pub enum SwitchEvent {
      Change { value: bool },
      FocusChanged { focused: bool },
      EnabledChanged { enabled: bool },
  }
  ```

#### **Toggle (`toggle`)**
* **Current Events**: `ToggleEvent::Change { selected: bool }`
* **Audit & Gap Analysis**: Needs focus, hover, and enabled events.
* **Proposed Standardized Event (`ToggleEvent`)**:
  ```rust
  pub enum ToggleEvent {
      Change { selected: bool },
      FocusChanged { focused: bool },
      EnabledChanged { enabled: bool },
  }
  ```

---

### 2.3 Selection & Grouping Controls

#### **ControlGroup (`control_group`)**
* **Current Events**:
  - `ControlGroupEvent::Activate { activated_id: SharedString }`
  - `ControlGroupEvent::Change { changed_id: SharedString, selected: bool, selected_ids: Vec<SharedString> }`
* **Audit & Gap Analysis**: Strong selection event model; needs parent focus notification and item focus navigation notifications.
* **Proposed Standardized Event (`ControlGroupEvent`)**:
  ```rust
  pub enum ControlGroupEvent {
      Activate { activated_id: SharedString },
      Change { changed_id: SharedString, selected: bool, selected_ids: Vec<SharedString> },
      FocusChanged { focused: bool },
      ItemFocused { item_id: SharedString },
  }
  ```

#### **RadioGroup / RadioButton (`radio_group`, `radio_button`)**
* **Current Events**: Wrapped in `ControlGroupEvent`
* **Audit & Gap Analysis**: Standardized via `ControlGroupEvent` with single-selection invariant.
* **Proposed Standardized Event (`RadioGroupEvent`)**:
  ```rust
  pub enum RadioGroupEvent {
      Change { value: SharedString },
      FocusChanged { focused: bool },
  }
  ```

#### **Selector / SelectionPanel (`selector`, `selection_panel`)**
* **Current Events**: `SelectorEvent::Select { index: usize }`, `SelectionPanelEvent::Change { selected_indices: Vec<usize> }`
* **Audit & Gap Analysis**: Needs standard focus and cancel/escape actions.
* **Proposed Standardized Event (`SelectorEvent`)**:
  ```rust
  pub enum SelectorEvent {
      Select { index: usize, id: SharedString },
      Change { selected_indices: Vec<usize> },
      FocusChanged { focused: bool },
      Dismiss,
  }
  ```

---

### 2.4 Text & Input Controls

#### **TextField (`textfield`)**
* **Current Events**:
  - `TextFieldEvent::Change { value: String }`
  - `TextFieldEvent::Submit { value: String }`
  - `TextFieldEvent::Focus`
  - `TextFieldEvent::Blur`
* **Audit & Gap Analysis**: Uses split `Focus`/`Blur` variants. Standardize focus to unified `FocusChanged { focused: bool }` or keep backwards-compatible `Focus`/`Blur` aliases alongside `EnabledChanged`.
* **Proposed Standardized Event (`TextFieldEvent`)**:
  ```rust
  pub enum TextFieldEvent {
      Change { value: String },
      Submit { value: String },
      FocusChanged { focused: bool },
      EnabledChanged { enabled: bool },
      Escape,
  }
  ```

#### **TextArea (`textarea`)**
* **Current Events**: `TextAreaEvent::Change { value: String }`, `Focus`, `Blur`
* **Audit & Gap Analysis**: Needs `Submit` (e.g. `Cmd+Enter`), `FocusChanged`, and cursor selection change notifications.
* **Proposed Standardized Event (`TextAreaEvent`)**:
  ```rust
  pub enum TextAreaEvent {
      Change { value: String },
      Submit { value: String },
      FocusChanged { focused: bool },
      SelectionChanged { range: std::ops::Range<usize> },
  }
  ```

#### **Autocomplete / ComboBox / SearchSelector (`autocomplete`, `combobox`, `search_selector`)**
* **Current Events**: `ComboBoxEvent::Select { index: usize }`, `Change { query: String }`, `Open`, `Close`
* **Audit & Gap Analysis**: Needs standardized dropdown overlay visibility and item commit events.
* **Proposed Standardized Event (`ComboBoxEvent`)**:
  ```rust
  pub enum ComboBoxEvent {
      QueryChange { query: String },
      Select { id: SharedString, value: String },
      OverlayVisibilityChanged { open: bool },
      FocusChanged { focused: bool },
  }
  ```

---

### 2.5 Range & Numeric Controls

#### **Slider (`slider`, `color_slider`, `color_arc`, `color_ring`)**
* **Current Events**:
  - `SliderEvent::Change { thumb_id: ThumbId, value: f32 }`
  - `SliderEvent::Release { thumb_id: ThumbId, value: f32 }`
  - `SliderEvent::ThumbAdded`, `ThumbRemoved`, `ThumbSelected`
* **Audit & Gap Analysis**: Comprehensive value and multi-thumb event model. Missing focus and drag start/end interaction markers.
* **Proposed Standardized Event (`SliderEvent`)**:
  ```rust
  pub enum SliderEvent {
      Change { thumb_id: ThumbId, value: f32 },
      Release { thumb_id: ThumbId, value: f32 },
      DragStart { thumb_id: ThumbId },
      ThumbAdded { thumb_id: ThumbId, value: f32 },
      ThumbRemoved { thumb_id: ThumbId },
      ThumbSelected { thumb_id: ThumbId },
      FocusChanged { focused: bool },
  }
  ```

#### **Scrollbar (`scrollbar`)**
* **Current Events**: `ScrollbarEvent::Scroll { offset: f32 }`
* **Audit & Gap Analysis**: Needs drag state indicators (`DragStart`, `DragEnd`).
* **Proposed Standardized Event (`ScrollbarEvent`)**:
  ```rust
  pub enum ScrollbarEvent {
      ScrollTo { offset: f32 },
      DragStateChanged { dragging: bool },
  }
  ```

---

### 2.6 Navigation & Hierarchical Controls

#### **TabsNavigation (`tabs_navigation`)**
* **Current Events**: `TabsNavigationEvent::Select { tab_id: SharedString }`
* **Audit & Gap Analysis**: Missing tab close trigger and focus events.
* **Proposed Standardized Event (`TabsNavigationEvent`)**:
  ```rust
  pub enum TabsNavigationEvent {
      Select { tab_id: SharedString },
      Close { tab_id: SharedString },
      FocusChanged { focused: bool },
  }
  ```

#### **NavigationSidebar (`navigation_sidebar`)**
* **Current Events**: `NavigationSidebarEvent::Select { item_id: SharedString }`
* **Audit & Gap Analysis**: Missing collapse/expand state change notifications.
* **Proposed Standardized Event (`NavigationSidebarEvent`)**:
  ```rust
  pub enum NavigationSidebarEvent {
      Select { item_id: SharedString },
      CollapsedChanged { collapsed: bool },
      FocusChanged { focused: bool },
  }
  ```

#### **Accordion (`accordion`)**
* **Current Events**: `AccordionEvent::Toggle { section_id: SharedString, expanded: bool }`
* **Audit & Gap Analysis**: Complete for section toggles; add focus tracking.
* **Proposed Standardized Event (`AccordionEvent`)**:
  ```rust
  pub enum AccordionEvent {
      SectionToggled { section_id: SharedString, expanded: bool },
      FocusChanged { focused: bool },
  }
  ```

#### **ListView / Listbox (`list_view`, `listbox`)**
* **Current Events**: `ListViewEvent::Select { index: usize }`, `Activate { index: usize }`
* **Audit & Gap Analysis**: Needs selection change, scroll offset change, and item action events.
* **Proposed Standardized Event (`ListViewEvent`)**:
  ```rust
  pub enum ListViewEvent {
      Select { indices: Vec<usize> },
      Activate { index: usize },
      ScrollChanged { top_index: usize },
      FocusChanged { focused: bool },
  }
  ```

#### **TreeView (`tree_view`)**
* **Current Events**: `TreeViewEvent::Select { node_id }`, `ToggleExpand { node_id }`
* **Audit & Gap Analysis**: Well structured; ensure standard focus payload.
* **Proposed Standardized Event (`TreeViewEvent<T>`)**:
  ```rust
  pub enum TreeViewEvent<T> {
      Select { node_id: T },
      ToggleExpand { node_id: T, expanded: bool },
      FocusChanged { focused: bool },
  }
  ```

---

### 2.7 Menus & Overlays

#### **PopupMenu / ContextMenu (`popup_menu`, `context_menu`)**
* **Current Events**: `PopupMenuEvent::Select { item_id: SharedString }`, `Dismiss`
* **Audit & Gap Analysis**: Clean overlay event model. Ensure `Open` and `Dismiss` events fire reliably on backdrop click or `Escape`.
* **Proposed Standardized Event (`PopupMenuEvent`)**:
  ```rust
  pub enum PopupMenuEvent {
      Select { item_id: SharedString },
      Open,
      Dismiss,
  }
  ```

#### **OverlayWindow / Dialog (`overlay_window`)**
* **Current Events**: `DialogEvent::Confirm`, `DialogEvent::Cancel`, `DialogEvent::Dismiss`
* **Audit & Gap Analysis**: Excellent modal lifecycle handling. Needs `OpenStateChanged { open: bool }`.
* **Proposed Standardized Event (`DialogEvent`)**:
  ```rust
  pub enum DialogEvent {
      Confirm,
      Cancel,
      Dismiss,
      OpenStateChanged { open: bool },
  }
  ```

---

### 2.8 Layout & Structure Controls

#### **SplitView / ResizablePanels / DockSplitter (`split_view`, `resizable_panels`, `dock_splitter`)**
* **Current Events**:
  - `SplitViewEvent::Resize { split_offset: f32 }`
  - `ResizablePanelsEvent::Resize { panel_sizes: Vec<f32> }`
  - `DockSplitterEvent::Resize { position: f32 }`
* **Audit & Gap Analysis**: Structural resize controls. Add `ResizeStart` and `ResizeEnd` for smooth drag performance tuning in heavy viewports.
* **Proposed Standardized Event (`SplitViewEvent`)**:
  ```rust
  pub enum SplitViewEvent {
      Resize { offset: f32 },
      ResizeStart,
      ResizeEnd,
      CollapseToggled { panel_index: usize, collapsed: bool },
  }
  ```

---

## 3. Summary & Phased Implementation Plan

1. **Phase 1: Universal State Trait & Common Variants**:
   - Introduce standardized helper payloads (`FocusChanged { focused: bool }`, `EnabledChanged { enabled: bool }`, `HoverChanged { hovered: bool }`).
2. **Phase 2: Core Control Update**:
   - Update `CommandCore` to emit `FocusChanged` and `HoverChanged` alongside `Click`.
   - Update `Checkbox`, `Switch`, `Toggle`, `TextField`, and `Slider`.
3. **Phase 3: Container Fan-In & Studio Event Log**:
   - Wire `Toolbar` and `ControlGroup` fan-in listeners.
   - Connect Luma Studio's `EventLogView` to showcase real-time event streaming across all control demos.
