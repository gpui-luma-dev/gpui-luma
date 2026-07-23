# SDK Control Eventing Specification & Comprehensive Control Audit

This document defines the standardized event taxonomy for the `gpui-luma` SDK controls library (`crates/sdk/src/controls`). It establishes consistent rules for state, lifecycle, interaction, and value change events across all control families.

---

## 0. Implementation Status

Status as of 2026-07-23:

* **Complete**: `CommandCore` / `ButtonEvent` now emits `Click`, `FocusChanged`, `EnabledChanged`, and `HoverChanged`. Activation-only app handlers have been migrated to filter click events.
* **Complete**: `Checkbox`, `Switch`, `Toggle`, and `RadioButton` are SDK-owned boolean choice controls. They emit typed `Change` events with domain payloads (`checked`, `on`, `selected`) plus focus/enabled/hover lifecycle events. Their inner `Button<bool>` entities are private implementation details and are no longer exposed to apps.
* **Complete**: `Toolbar` fans in command button clicks and toggle changes through sourced child controls. Toggle toolbar items now consume `ToggleEvent::Change` instead of flipping raw button data.
* **Complete**: `ControlGroupEvent` now includes `FocusChanged { focused }` and `ItemFocused { item_id }` alongside existing `Activate` and `Change` events. Active-descendant and roving-item focus strategies both contribute to semantic focus notifications.
* **Complete**: `RadioGroup` is a semantic wrapper over `ControlGroup` that emits dedicated `RadioGroupEvent` values rather than exposing raw `ControlGroupEvent` payloads.
* **Complete**: `SelectorEvent` now includes semantic selection, focus, open-state, and dismiss events. Consumers that only handle selection now filter `SelectorEvent::Change` instead of assuming it is the only variant.
* **Complete**: `SelectionPanelEvent` now includes standard focus and open-state lifecycle events alongside row hover, active-index, and row activation events.
* **Complete**: `SearchSelectorEvent` now carries query and selected-item payloads and emits focus/open/dismiss lifecycle events. Consumers use selection payloads directly instead of reading current selection back out of the control.
* **Complete**: `ComboBoxEvent` and `AutocompleteTextBoxEvent` now carry query and selected-item payloads and emit focus/open/dismiss lifecycle events.
* **Complete**: `TextFieldEvent` and `TextAreaEvent` now use `FocusChanged { focused }` and emit `EnabledChanged { enabled }` for programmatic enabled transitions. Selector-family text adapters filter only the text events they consume.
* **Complete**: `SliderEvent` now includes drag, focus, hover, and enabled lifecycle events alongside existing value and multi-thumb events.
* **Complete**: Color slider, arc, and ring controls use the unified `SliderEvent` surface. `ColorFieldEvent` now includes change, release, drag, hover, and enabled lifecycle events for color surface controls.
* **Complete**: `ScrollbarEvent` now includes drag, focus, hover, and enabled lifecycle events alongside value changes. Scrollbar consumers that only need offsets filter `Change`. `Scrollbar` also exposes abstract viewport/range support so callers can provide content start/end and visible start/end values instead of manually deriving track range, value, and thumb fraction.
* **Complete**: `TabsNavigationEvent` fans out selection, activation, group focus, and item focus events from its inner `ControlGroup` without exposing raw group events.
* **Complete**: `AccordionEvent` now includes focus, item-focus, item-hover, and enabled lifecycle events alongside section expansion changes.
* **Complete**: `PagerEvent` now includes page-size popover open-state and enabled lifecycle events alongside page/page-size changes. SDK list paging filters only page mutation events.
* **Complete**: `NavigationSidebarEvent` now includes focus, item-focus, item-hover, rail submenu open-state, and enabled lifecycle events alongside activation, branch expansion, and collapse events.
* **Complete**: `ListViewEvent` now includes scroll, focus, row-hover, and enabled lifecycle events alongside selection, active-index, page, and page-size events. `Listbox` is a typed `ControlGroupControl<ListBoxItem>` surface and uses the already-standardized `ControlGroupEvent`.
* **Complete**: `TreeViewEvent` now includes active-node, scroll, focus, row-hover, and enabled lifecycle events alongside node expand/collapse and selection events.
* **Complete**: `PopupMenuEvent` and `ContextMenuEvent` now include select, open-state, dismiss, focus, hover, and enabled lifecycle events. `FloatingMenu` remains shared state/template infrastructure with no standalone public event emitter. `OverlayWindowEvent` was already complete as a non-exhaustive opened/dismissed lifecycle surface.
* **Complete**: Structural layout controls now expose resize lifecycle events. `SplitViewEvent` covers sidebar width, resize start/end, collapse, separator hover, and enabled transitions; `ResizablePanelsEvent` covers size changes, resize start/end, panel visibility, handle focus/hover, and enabled transitions; `DockSplitterEvent` covers resize start/change/end plus focus/hover/enabled lifecycle events.
* **Complete**: Public SDK control event enums are marked `#[non_exhaustive]`. Downstream consumers now use explicit filters or wildcard arms so future event additions do not require a full workspace refactor.
* **Complete**: App usages touched by this migration now subscribe to semantic SDK events instead of depending on the old one-event button assumption or raw boolean button state.
* **Open**: Luma Studio's event log exists as app-local infrastructure, but comprehensive per-control event showcase wiring is not complete.

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
   - Boolean controls use domain-specific payload names (`checked`, `on`, `selected`) rather than reusing raw button payloads.
   - String payloads use `value: String`.
   - Identifiers use `id: SharedString`.
   - State transition flags use `focused: bool`, `enabled: bool`, `hovered: bool`.
3. **Container Event Fan-In**:
   - Container controls (`Toolbar`, `ControlGroup`, `Form`) aggregate and fan-in item events, surfacing child events wrapped in a parent envelope (e.g. `ToolbarEvent::ItemClick { item_id }`).
4. **Forward-Compatible Event Enums**:
   - Public SDK control event enums **MUST** be `#[non_exhaustive]`.
   - Downstream code **MUST NOT** exhaustively match SDK events unless it includes a wildcard arm. Prefer `if let`, `let ... else { return; }`, or an explicit `_ => {}` for event variants the consumer does not handle.

---

## 2. Comprehensive SDK Control Event Audit

Below is the complete audit of all control families in `crates/sdk/src/controls/`, detailing their existing events, missing events, and proposed standardized event signature.

---

### 2.1 Command & Action Controls

#### **Button / IconButton (`command/button`, `command/icon_button`)**
* **Core Engine**: `CommandCore`
* **Implemented Events**: `ButtonEvent::Click`, `ButtonEvent::FocusChanged`, `ButtonEvent::EnabledChanged`, `ButtonEvent::HoverChanged`
* **Audit Result**: Implemented through `CommandCore`; callers that only care about activation must filter `ButtonEvent::Click`.
* **Standardized Event (`ButtonEvent`)**:
  ```rust
  pub enum ButtonEvent {
      Click,
      FocusChanged { focused: bool },
      EnabledChanged { enabled: bool },
      HoverChanged { hovered: bool },
  }
  ```

#### **Toolbar (`toolbar`)**
* **Implemented Events**: `ToolbarEvent::Click { id }`, `ToolbarEvent::Change { id, value }`
* **Audit Result**: Aggregates sourced child controls. Toolbar toggle items now consume `ToggleEvent::Change` rather than flipping raw `Button<bool>` state internally.
* **Current Event (`ToolbarEvent`)**:
  ```rust
  pub enum ToolbarEvent {
      Click { id: SharedString },
      Change { id: SharedString, value: ToolbarValue },
  }
  ```

---

### 2.2 Boolean & Choice Controls

#### **Checkbox (`checkbox`)**
* **Current Wrapper**: `CheckboxControl` owns an inner `Button<bool>` and does not expose raw button access.
* **Implemented Events**: `CheckboxEvent::Change { checked }`, `FocusChanged`, `EnabledChanged`, `HoverChanged`
* **Audit Result**: Click-to-checked transition lives inside the SDK control. Programmatic `set_data` updates visual state and does not emit `Change`.
* **Standardized Event (`CheckboxEvent`)**:
  ```rust
  pub enum CheckboxEvent {
      Change { checked: bool },
      FocusChanged { focused: bool },
      EnabledChanged { enabled: bool },
      HoverChanged { hovered: bool },
  }
  ```

#### **Switch (`switch`)**
* **Current Wrapper**: `SwitchControl` owns an inner `Button<bool>` and does not expose raw button access.
* **Implemented Events**: `SwitchEvent::Change { on }`, `FocusChanged`, `EnabledChanged`, `HoverChanged`
* **Audit Result**: Click-to-on transition lives inside the SDK control. Programmatic `set_data` updates visual state and does not emit `Change`.
* **Standardized Event (`SwitchEvent`)**:
  ```rust
  pub enum SwitchEvent {
      Change { on: bool },
      FocusChanged { focused: bool },
      EnabledChanged { enabled: bool },
      HoverChanged { hovered: bool },
  }
  ```

#### **Toggle (`toggle`)**
* **Current Wrapper**: `Toggle` is `Entity<ToggleControl>`; `ToggleControl` owns an inner `Button<bool>` and does not expose raw button access.
* **Implemented Events**: `ToggleEvent::Change { selected }`, `FocusChanged`, `EnabledChanged`, `HoverChanged`
* **Audit Result**: Click-to-selected transition lives inside the SDK control. Programmatic `set_data` updates visual state and does not emit `Change`.
* **Standardized Event (`ToggleEvent`)**:
  ```rust
  pub enum ToggleEvent {
      Change { selected: bool },
      FocusChanged { focused: bool },
      EnabledChanged { enabled: bool },
      HoverChanged { hovered: bool },
  }
  ```

---

### 2.3 Selection & Grouping Controls

#### **ControlGroup (`control_group`)**
* **Implemented Events**:
  - `ControlGroupEvent::Activate { activated_id: SharedString }`
  - `ControlGroupEvent::Change { changed_id: SharedString, selected: bool, selected_ids: Vec<SharedString> }`
  - `ControlGroupEvent::FocusChanged { focused: bool }`
  - `ControlGroupEvent::ItemFocused { item_id: SharedString }`
* **Audit Result**: Strong selection event model with parent focus and item focus navigation notifications. Programmatic selection and active-item setters update state without emitting value/focus events.
* **Standardized Event (`ControlGroupEvent`)**:
  ```rust
  pub enum ControlGroupEvent {
      Activate { activated_id: SharedString },
      Change { changed_id: SharedString, selected: bool, selected_ids: Vec<SharedString> },
      FocusChanged { focused: bool },
      ItemFocused { item_id: SharedString },
  }
  ```

#### **RadioButton (`radio_button`)**
* **Current Wrapper**: `RadioButton` is `Entity<RadioButtonControl>`; `RadioButtonControl` owns an inner `Button<bool>` and does not expose raw button access.
* **Implemented Events**: `RadioButtonEvent::Change { selected }`, `FocusChanged`, `EnabledChanged`, `HoverChanged`
* **Audit Result**: Click-to-selected transition lives inside the SDK control. Clicking an already-selected radio button is a no-op. Programmatic `set_data` updates visual state and does not emit `Change`.
* **Standardized Event (`RadioButtonEvent`)**:
  ```rust
  pub enum RadioButtonEvent {
      Change { selected: bool },
      FocusChanged { focused: bool },
      EnabledChanged { enabled: bool },
      HoverChanged { hovered: bool },
  }
  ```

#### **RadioGroup (`radio_group`)**
* **Current Wrapper**: `RadioGroup` is `Entity<RadioGroupControl<T>>`; `RadioGroupControl<T>` owns an inner `ControlGroupControl<T>` and maps control-group events to radio-specific events.
* **Implemented Events**: `RadioGroupEvent::Change { value }`, `Activate`, `FocusChanged`, `ItemFocused`
* **Audit Result**: Radio consumers no longer depend on raw `ControlGroupEvent` payloads. `Change { value }` uses `Option<SharedString>` so radio-like allow-none groups can represent an empty selection.
* **Standardized Event (`RadioGroupEvent`)**:
  ```rust
  pub enum RadioGroupEvent {
      Change { value: Option<SharedString> },
      Activate { value: SharedString },
      FocusChanged { focused: bool },
      ItemFocused { item_id: SharedString },
  }
  ```

#### **Selector (`selector`)**
* **Implemented Events**: `SelectorEvent::Change { item_id, label }`, `FocusChanged`, `OpenChanged`, `Dismiss`
* **Audit Result**: Selection changes are domain-specific and keyed by item id. Programmatic `set_selected_id` updates visual state without emitting `Change`. Selection-only consumers must filter `SelectorEvent::Change` because focus and overlay lifecycle events share the same semantic event stream.
* **Standardized Event (`SelectorEvent`)**:
  ```rust
  pub enum SelectorEvent {
      Change { item_id: SharedString, label: SharedString },
      FocusChanged { focused: bool },
      OpenChanged { open: bool },
      Dismiss,
  }
  ```

#### **SelectionPanel (`selection_panel`)**
* **Implemented Events**: `SelectionPanelEvent::HoverChanged { visible_index }`, `ActivateRow { source_index, visible_index, item_id }`, `ActiveIndexChanged { visible_index }`, `FocusChanged`, `OpenChanged`
* **Audit Result**: Event model covers row hover, row activation, active-index keyboard navigation, parent focus, and open-state lifecycle. Programmatic selected-index updates do not emit activation events.
* **Standardized Event (`SelectionPanelEvent`)**:
  ```rust
  pub enum SelectionPanelEvent {
      HoverChanged { visible_index: Option<usize> },
      ActivateRow { source_index: usize, visible_index: usize, item_id: SharedString },
      ActiveIndexChanged { visible_index: Option<usize> },
      FocusChanged { focused: bool },
      OpenChanged { open: bool },
  }
  ```

---

### 2.4 Text & Input Controls

#### **TextField (`textfield`)**
* **Implemented Events**:
  - `TextFieldEvent::Change { value: String }`
  - `TextFieldEvent::Submit { value: String }`
  - `TextFieldEvent::FocusChanged { focused: bool }`
  - `TextFieldEvent::EnabledChanged { enabled: bool }`
* **Audit Result**: Value mutation and submit payloads are explicit. Focus uses the standard boolean transition event, and programmatic enabled changes emit a lifecycle event. Selector-family text adapters ignore lifecycle variants they do not consume.
* **Standardized Event (`TextFieldEvent`)**:
  ```rust
  pub enum TextFieldEvent {
      Change { value: String },
      Submit { value: String },
      FocusChanged { focused: bool },
      EnabledChanged { enabled: bool },
  }
  ```

#### **TextArea (`textarea`)**
* **Implemented Events**: `TextAreaEvent::Change { value: String }`, `FocusChanged { focused }`, `EnabledChanged { enabled }`
* **Audit Result**: Value mutation has payloads, focus uses the standard boolean transition event, and programmatic enabled changes emit a lifecycle event. Submit shortcuts and cursor/selection notifications remain deferred behavior/API decisions.
* **Standardized Event (`TextAreaEvent`)**:
  ```rust
  pub enum TextAreaEvent {
      Change { value: String },
      FocusChanged { focused: bool },
      EnabledChanged { enabled: bool },
  }
  ```

#### **SearchSelector (`search_selector`)**
* **Implemented Events**: `SearchSelectorEvent::Change { query }`, `Select { item_id, label }`, `Complete { item_id, label }`, `Clear`, `FocusChanged`, `OpenChanged`, `Dismiss`
* **Audit Result**: Selection consumers can use the event payload directly. Query mutation, committed selection, exact completion, clear, focus, and popup visibility are distinct semantic events.
* **Standardized Event (`SearchSelectorEvent`)**:
  ```rust
  pub enum SearchSelectorEvent {
      Change { query: SharedString },
      Select { item_id: SharedString, label: SharedString },
      Complete { item_id: SharedString, label: SharedString },
      Clear,
      FocusChanged { focused: bool },
      OpenChanged { open: bool },
      Dismiss,
  }
  ```

#### **Autocomplete / ComboBox (`autocomplete`, `combobox`)**
* **Implemented Events**: `Change { query }`, `Select { item_id, label }`, `Complete { item_id, label }`, `Clear`, `FocusChanged`, `OpenChanged`, `Dismiss`
* **Audit Result**: Query mutation, committed selection, exact completion, clear, focus, and popup visibility are distinct semantic events. Selection consumers can use the event payload directly instead of reading the control state.
* **Standardized Events (`ComboBoxEvent`, `AutocompleteTextBoxEvent`)**:
  ```rust
  pub enum ComboBoxEvent {
      Change { query: SharedString },
      Select { item_id: SharedString, label: SharedString },
      Complete { item_id: SharedString, label: SharedString },
      Clear,
      FocusChanged { focused: bool },
      OpenChanged { open: bool },
      Dismiss,
  }
  ```

---

### 2.5 Range & Numeric Controls

#### **Slider (`slider`, `color_slider`, `color_arc`, `color_ring`)**
* **Implemented Events**:
  - `SliderEvent::Change { thumb_id: ThumbId, value: f32 }`
  - `SliderEvent::Release { thumb_id: ThumbId, value: f32 }`
  - `SliderEvent::DragStart { thumb_id: ThumbId }`
  - `SliderEvent::DragEnd { thumb_id: ThumbId, value: f32 }`
  - `SliderEvent::ThumbAdded { thumb_id: ThumbId, value: f32 }`
  - `SliderEvent::ThumbRemoved { thumb_id: ThumbId }`
  - `SliderEvent::ThumbSelected { thumb_id: ThumbId }`
  - `SliderEvent::FocusChanged { focused: bool }`
  - `SliderEvent::HoverChanged { hovered: bool }`
  - `SliderEvent::EnabledChanged { enabled: bool }`
* **Audit Result**: Value changes, keyboard commits, pointer drag lifecycle, multi-thumb mutation, active-thumb selection, focus, hover, and enabled state now have distinct semantic events. `Release` remains the value-commit event and can still come from keyboard adjustments; `DragEnd` identifies pointer drag completion.
* **Standardized Event (`SliderEvent`)**:
  ```rust
  pub enum SliderEvent {
      Change { thumb_id: ThumbId, value: f32 },
      Release { thumb_id: ThumbId, value: f32 },
      DragStart { thumb_id: ThumbId },
      DragEnd { thumb_id: ThumbId, value: f32 },
      ThumbAdded { thumb_id: ThumbId, value: f32 },
      ThumbRemoved { thumb_id: ThumbId },
      ThumbSelected { thumb_id: ThumbId },
      FocusChanged { focused: bool },
      HoverChanged { hovered: bool },
      EnabledChanged { enabled: bool },
  }
  ```

#### **ColorField (`color/color_field`)**
* **Implemented Events**:
  - `ColorFieldEvent::Change(Hsv)`
  - `ColorFieldEvent::Release(Hsv)`
  - `ColorFieldEvent::DragStart { hsv: Hsv }`
  - `ColorFieldEvent::DragEnd { hsv: Hsv }`
  - `ColorFieldEvent::HoverChanged { hovered: bool }`
  - `ColorFieldEvent::EnabledChanged { enabled: bool }`
* **Audit Result**: Color surface controls now emit value mutation, value commit, pointer drag lifecycle, hover, and enabled transitions from `ColorFieldState`. Color slider, arc, and ring controls intentionally use the unified `SliderEvent` surface instead of defining parallel color-specific slider events. Color composition consumers filter for value events before acquiring sync state, and `ColorCompositionSync::begin_guard()` is available for scoped sync cleanup when handlers may return early.
* **Standardized Event (`ColorFieldEvent`)**:
  ```rust
  pub enum ColorFieldEvent {
      Change(Hsv),
      Release(Hsv),
      DragStart { hsv: Hsv },
      DragEnd { hsv: Hsv },
      HoverChanged { hovered: bool },
      EnabledChanged { enabled: bool },
  }
  ```

#### **Scrollbar (`scrollbar`)**
* **Implemented Events**: `ScrollbarEvent::Change { value }`, `DragStart`, `DragEnd { value }`, `FocusChanged { focused }`, `HoverChanged { hovered }`, `EnabledChanged { enabled }`
* **Audit Result**: Scroll value mutation remains `Change { value }`; pointer drag lifecycle and standard focus/hover/enabled transitions are separate semantic events. SDK internals that use scrollbars for popup/list offsets filter only `Change`. `ScrollbarViewport` maps abstract content and visible ranges onto the scrollbar's internal range/value/thumb geometry.
* **Standardized Event (`ScrollbarEvent`)**:
  ```rust
  pub enum ScrollbarEvent {
      Change { value: f32 },
      DragStart,
      DragEnd { value: f32 },
      FocusChanged { focused: bool },
      HoverChanged { hovered: bool },
      EnabledChanged { enabled: bool },
  }
  ```

---

### 2.6 Navigation & Hierarchical Controls

#### **TabsNavigation (`tabs_navigation`)**
* **Implemented Events**: `TabsNavigationEvent::Change { tab_id, label }`, `Activate { tab_id, label }`, `FocusChanged { focused }`, `ItemFocused { tab_id, label }`
* **Audit Result**: Implemented as a semantic wrapper over `ControlGroup`; consumers receive tab-specific identifiers and labels instead of raw group payloads.
* **Standardized Event (`TabsNavigationEvent`)**:
  ```rust
  pub enum TabsNavigationEvent {
      Change { tab_id: SharedString, label: SharedString },
      Activate { tab_id: SharedString, label: SharedString },
      FocusChanged { focused: bool },
      ItemFocused { tab_id: SharedString, label: SharedString },
  }
  ```

#### **NavigationSidebar (`navigation_sidebar`)**
* **Implemented Events**: `NavigationSidebarEvent::Activate { node_id, label }`, `BranchExpandedChanged { node_id, expanded }`, `CollapsedChanged { collapsed }`, `FocusChanged { focused }`, `ItemFocused { node_id, label }`, `ItemHoverChanged { node_id, hovered }`, `RailSubmenuOpenChanged { node_id }`, `EnabledChanged { enabled }`
* **Audit Result**: Activation, branch expansion, sidebar collapse, focus, item-focus, hover, rail submenu visibility, and enabled transitions are now distinct semantic events. Internal scrollbar events remain filtered to `ScrollbarEvent::Change`.
* **Standardized Event (`NavigationSidebarEvent`)**:
  ```rust
  pub enum NavigationSidebarEvent {
      Activate { node_id: SharedString, label: SharedString },
      BranchExpandedChanged { node_id: SharedString, expanded: bool },
      CollapsedChanged { collapsed: bool },
      FocusChanged { focused: bool },
      ItemFocused { node_id: SharedString, label: SharedString },
      ItemHoverChanged { node_id: SharedString, hovered: bool },
      RailSubmenuOpenChanged { node_id: Option<SharedString> },
      EnabledChanged { enabled: bool },
  }
  ```

#### **Accordion (`accordion`)**
* **Implemented Events**: `AccordionEvent::ExpandedChanged { item_id, expanded }`, `FocusChanged { focused }`, `ItemFocused { item_id }`, `ItemHoverChanged { item_id, hovered }`, `EnabledChanged { enabled }`
* **Audit Result**: Section expansion remains the value event; focus, item focus, hover, and enabled transitions are now explicit lifecycle events.
* **Standardized Event (`AccordionEvent`)**:
  ```rust
  pub enum AccordionEvent {
      ExpandedChanged { item_id: SharedString, expanded: bool },
      FocusChanged { focused: bool },
      ItemFocused { item_id: SharedString },
      ItemHoverChanged { item_id: SharedString, hovered: bool },
      EnabledChanged { enabled: bool },
  }
  ```

#### **Pager (`pager`)**
* **Implemented Events**: `PagerEvent::PageChanged { page }`, `PageSizeChanged { page_size }`, `PageSizeOpenChanged { open }`, `EnabledChanged { enabled }`
* **Audit Result**: Page and page-size mutation events remain separate from page-size selector lifecycle and enabled transitions. `PagingListViewControl` filters only page mutation events.
* **Standardized Event (`PagerEvent`)**:
  ```rust
  pub enum PagerEvent {
      PageChanged { page: usize },
      PageSizeChanged { page_size: usize },
      PageSizeOpenChanged { open: bool },
      EnabledChanged { enabled: bool },
  }
  ```

#### **ListView (`list_view`)**
* **Implemented Events**: `ListViewEvent::SelectionChanged { selected_indices }`, `ActiveIndexChanged { active_index }`, `PageChanged { page }`, `PageSizeChanged { page_size }`, `ScrollChanged { top_index }`, `FocusChanged { focused }`, `RowHoverChanged { index, hovered }`, `EnabledChanged { enabled }`
* **Audit Result**: Selection, active row, paging, scrolling, focus, hover, and enabled transitions are explicit semantic events. Paged-list adapters filter only the page/selection events they consume.
* **Standardized Event (`ListViewEvent`)**:
  ```rust
  pub enum ListViewEvent {
      SelectionChanged { selected_indices: Vec<usize> },
      ActiveIndexChanged { active_index: Option<usize> },
      PageChanged { page: usize },
      PageSizeChanged { page_size: usize },
      ScrollChanged { top_index: usize },
      FocusChanged { focused: bool },
      RowHoverChanged { index: usize, hovered: bool },
      EnabledChanged { enabled: bool },
  }
  ```

#### **Listbox (`listbox`)**
* **Implemented Events**: `ControlGroupEvent::Change`, `Activate`, `FocusChanged`, `ItemFocused`
* **Audit Result**: `ListBox` is a typed alias over `ControlGroupControl<ListBoxItem>`, so listbox selection and focus semantics come from the standardized control-group event surface.

#### **TreeView (`tree_view`)**
* **Implemented Events**: `TreeViewEvent::NodeExpanded { node_id, data }`, `NodeCollapsed { node_id, data }`, `SelectionChanged { selected_ids }`, `ActiveNodeChanged { node_id }`, `ScrollChanged { top_index }`, `FocusChanged { focused }`, `RowHoverChanged { node_id, index, hovered }`, `EnabledChanged { enabled }`
* **Audit Result**: Expand/collapse and selection events remain stable. Active-node, scroll, focus, hover, and enabled transitions are now explicit lifecycle events.
* **Standardized Event (`TreeViewEvent<T>`)**:
  ```rust
  pub enum TreeViewEvent<T> {
      NodeExpanded { node_id: SharedString, data: T },
      NodeCollapsed { node_id: SharedString, data: T },
      SelectionChanged { selected_ids: HashSet<SharedString> },
      ActiveNodeChanged { node_id: Option<SharedString> },
      ScrollChanged { top_index: usize },
      FocusChanged { focused: bool },
      RowHoverChanged { node_id: SharedString, index: usize, hovered: bool },
      EnabledChanged { enabled: bool },
  }
  ```

---

### 2.7 Menus & Overlays

#### **PopupMenu / ContextMenu (`popup_menu`, `context_menu`)**
* **Implemented Events**:
  - `PopupMenuEvent::Select { item_id: SharedString, label: SharedString }`
  - `PopupMenuEvent::OpenChanged { open: bool }`
  - `PopupMenuEvent::Dismiss`
  - `PopupMenuEvent::FocusChanged { focused: bool }`
  - `PopupMenuEvent::HoverChanged { hovered: bool }`
  - `PopupMenuEvent::EnabledChanged { enabled: bool }`
  - `ContextMenuEvent` mirrors the same lifecycle shape.
* **Audit Result**: Implemented in `control.rs`. Selection closes emit `OpenChanged { open: false }` plus `Select`; cancelled closes from trigger toggle, outside click, focus loss, or `Escape` emit `Dismiss`. `FloatingMenu` is shared state/template infrastructure and intentionally has no standalone event emitter.
* **Standardized Event (`PopupMenuEvent`)**:
  ```rust
  pub enum PopupMenuEvent {
      Select { item_id: SharedString, label: SharedString },
      OpenChanged { open: bool },
      Dismiss,
      FocusChanged { focused: bool },
      HoverChanged { hovered: bool },
      EnabledChanged { enabled: bool },
  }
  ```

#### **OverlayWindow / Dialog (`overlay_window`)**
* **Implemented Events**: `DialogEvent::Opened`, `DialogEvent::Dismissed`
* **Audit Result**: Complete. `DialogEvent` is non-exhaustive and already emits reliable open/dismiss lifecycle events from `DialogControl::open_from` and `DialogControl::dismiss`. The earlier proposed confirm/cancel/open-state names were stale against the current overlay primitive.
* **Standardized Event (`DialogEvent`)**:
  ```rust
  pub enum DialogEvent {
      Opened,
      Dismissed,
  }
  ```

---

### 2.8 Layout & Structure Controls

#### **SplitView / ResizablePanels / DockSplitter (`split_view`, `resizable_panels`, `dock_splitter`)**
* **Implemented Events**:
  - `SplitViewEvent::ResizeStart`
  - `SplitViewEvent::SidebarWidthChanged { width: Pixels }`
  - `SplitViewEvent::ResizeEnd { width: Pixels }`
  - `SplitViewEvent::CollapsedChanged { collapsed: bool }`
  - `SplitViewEvent::SeparatorHoverChanged { hovered: bool }`
  - `SplitViewEvent::EnabledChanged { enabled: bool }`
  - `ResizablePanelsEvent::ResizeStart`
  - `ResizablePanelsEvent::SizesChanged { sizes_px: Vec<f32> }`
  - `ResizablePanelsEvent::ResizeEnd { sizes_px: Vec<f32> }`
  - `ResizablePanelsEvent::PanelHiddenChanged { panel_index, hidden }`
  - `ResizablePanelsEvent::HandleFocusChanged { handle_index, focused }`
  - `ResizablePanelsEvent::HandleHoverChanged { handle_index, hovered }`
  - `ResizablePanelsEvent::EnabledChanged { enabled }`
  - `DockSplitterEvent::ResizeStart`
  - `DockSplitterEvent::Resize { total_delta: f32 }`
  - `DockSplitterEvent::ResizeEnd`
  - `DockSplitterEvent::FocusChanged { focused }`
  - `DockSplitterEvent::HoverChanged { hovered }`
  - `DockSplitterEvent::EnabledChanged { enabled }`
* **Audit Result**: Complete. Pointer and keyboard resize paths emit explicit resize transactions. Programmatic layout visibility/collapse setters emit semantic state changes without requiring apps to infer them from visual state.
* **Standardized Event (`SplitViewEvent`)**:
  ```rust
  pub enum SplitViewEvent {
      ResizeStart,
      SidebarWidthChanged { width: Pixels },
      ResizeEnd { width: Pixels },
      CollapsedChanged { collapsed: bool },
      SeparatorHoverChanged { hovered: bool },
      EnabledChanged { enabled: bool },
  }
  ```

---

## 3. Summary & Phased Implementation Plan

1. **Phase 1: Universal State Trait & Common Variants - Complete**:
   - Audited SDK controls now expose typed focus, hover, enabled, open/dismiss, drag, resize, selection, and value events where those states apply.
   - Public SDK event enums are `#[non_exhaustive]`, and consumers must filter the variants they handle.
2. **Phase 2: Core Control Update - Complete**:
   - Command, boolean choice, selector-family, text input, slider, scrollbar, navigation, menu/overlay, and structural layout controls now emit semantic SDK events from `control.rs`.
   - Programmatic value setters update visual state without recursive mutation events; idempotent lifecycle setters emit explicit lifecycle transitions when state changes.
3. **Phase 3: Container Fan-In & Studio Event Log - SDK Complete / Showcase Open**:
   - Complete: `Toolbar`, `ControlGroup`, `RadioGroup`, `TabsNavigation`, list paging, scrollbar consumers, and selector adapters filter child events instead of assuming single-variant streams.
   - Open: Luma Studio's `EventLogView` has not yet been wired into comprehensive real-time event demos across all controls.
