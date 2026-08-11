# Group Controls Redesign: WPF-Like Decentralized Radio Groups

## The Problem

In Luma's current SDK and early prototypes, `RadioGroup` (and general selection grouping) is implemented as a monolithic parent control ([ControlGroupControl](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/control_group/control.rs#L22)) that forces a single layout container and owns the child elements.

This creates several issues:
1. **Rigid Layout & Visual Options:** The stock templates hardcode visual choices (like rendering a standard radio dot next to a text label) and layout constraints (like horizontal/vertical flex with specific gaps).
2. **Inconsistent & High-Friction Customization:** When callers want a custom layout (e.g., a grid of large card tiles), they have to bypass the stock template.
3. **Broken Keyboard Focus & Roving Tabindex:** Because custom templates bypass the stock row-rendering path, they must manually wire mouse and keyboard handlers. Callers frequently omit focus wiring, breaking arrow-key navigation and roving tabindex.

---

## Real-World Pain Points

### Example 1: Luma Studio Plan Upgrade Cards
In [upgrade.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/luma-studio/src/studio/content_tabs/cards/upgrade.rs#L194), to render plan options as side-by-side selectable cards instead of simple dot-indicators, the custom template had to unpack five separate event handler iterators and bind them manually:

```rust
fn plan_option_group_template(look: Arc<ShadcnLook>) -> RadioGroupTemplate<PlanOptionItem> {
    Arc::new(move |model, handlers, window, cx| {
        let RadioGroupTemplateHandlers {
            item_hovers,
            item_mouse_downs,
            item_mouse_ups,
            item_mouse_up_outs,
            item_clicks,
        } = handlers;

        let mut item_hovers = item_hovers.into_iter();
        let mut item_mouse_downs = item_mouse_downs.into_iter();
        // ... (repeats for all 5 iterators)

        let mut root = div().id(model.id.clone()).flex().w_full().gap(px(PLAN_CARD_GAP));

        for item in &model.items {
            let item_hover = item_hovers.next().unwrap();
            let item_click = item_clicks.next().unwrap();
            // ... (wires handlers manually to card div)
        }
        root
    })
}
```
* **Consequence:** This manual boilerplate is verbose, fragile, and led directly to the roving tabindex and arrow-key focus navigation breaking.

### Example 2: Gallery Delivery Window Tiles
In [gallery/panes/radio_group/pane.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/gallery/src/gallery/panes/radio_group/pane.rs#L432), a custom template is written to show selectable time-slot blocks. It duplicates the exact same five-iterator unpacking code to wire hovers and clicks onto the cards:

```rust
fn delivery_window_template(look: Arc<ShadcnLook>) -> RadioGroupTemplate<DeliveryWindowItem> {
    Arc::new(move |model, handlers, _window, _cx| {
        let RadioGroupTemplateHandlers {
            item_hovers,
            item_mouse_downs,
            // ...
        } = handlers;
        // ... (manual loops and handler attachment to delivery_window_card)
    })
}
```

---

## Proposed Solution: WPF-Style Decentralized Grouping

Instead of wrapping radio buttons in a monolithic parent control that dictates layout, we should separate the **logical coordination** from the **visual layout** entirely, mimicking WPF's `GroupName` mechanism.

In WPF, radio buttons are written as standalone elements and grouped logically by a name:
```xml
<StackPanel Margin="10">
    <TextBlock Text="Select your favorite flavor:" Margin="0,0,0,10" FontWeight="Bold"/>
    
    <RadioButton Content="Vanilla" GroupName="Flavors" IsChecked="True" Margin="0,5"/>
    <RadioButton Content="Chocolate" GroupName="Flavors" Margin="0,5"/>
    <RadioButton Content="Strawberry" GroupName="Flavors" Margin="0,5"/>
    <RadioButton Content="Mint Chip" GroupName="Flavors" Margin="0,5"/>
</StackPanel>
```

### The Rust/GPUI Equivalent
In GPUI, we can represent the group name using a shared logical group coordinator entity (`Entity<RadioGroup>`). Individual `RadioButton` elements are rendered wherever the caller wants, and they reference the shared group to coordinate selection and focus:

```rust
// 1. Create the logical group coordinator
let flavors = RadioGroup::new("Flavors", cx);

// 2. Lay out using standard layout containers, mirroring WPF properties
v_stack()
    .m(px(10.0)) // Margin="10"
    .child(
        div()
            .font_bold() // FontWeight="Bold"
            .mb(px(10.0)) // Margin="0,0,0,10"
            .child("Select your favorite flavor:")
    )
    .child(
        RadioButton::new("vanilla")
            .label("Vanilla") // Content="Vanilla"
            .group(&flavors) // GroupName="Flavors"
            .selected(true) // IsChecked="True"
            .my(px(5.0)) // Margin="0,5"
    )
    .child(
        RadioButton::new("chocolate")
            .label("Chocolate")
            .group(&flavors)
            .my(px(5.0))
    )
    .child(
        RadioButton::new("strawberry")
            .label("Strawberry")
            .group(&flavors)
            .my(px(5.0))
    )
    .child(
        RadioButton::new("mint-chip")
            .label("Mint Chip")
            .group(&flavors)
            .my(px(5.0))
    )
```

---

## Technical Architecture & Lifecycle

```
  +--------------------------+          +--------------------------+
  |    RadioButton ("vanilla")|          |  RadioButton ("chocolate")|
  |  - FocusHandle           |          |  - FocusHandle           |
  |  - on_action(ArrowDown)  |          |  - on_action(ArrowDown)  |
  +------------+-------------+          +------------+-------------+
               |                                     |
               | register id & FocusHandle           | register id & FocusHandle
               v                                     v
         +-------------------------------------------------+
         |                 RadioGroup Entity               |
         |  - Tracks active_id & selected_id               |
         |  - Roving Focus: Coordinates which FocusHandle  |
         |    is active and triggers AppContext::focus     |
         +-------------------------------------------------+
```

### 1. The Coordinator (`RadioGroup`)
The logical coordinator is a GPUI entity that stores selection, active index, and registered child focus handles:

```rust
pub struct RadioGroup {
    name: SharedString,
    selected_id: Option<SharedString>,
    active_id: Option<SharedString>,
    items: Vec<RegisteredRadioItem>,
}

struct RegisteredRadioItem {
    id: SharedString,
    focus_handle: FocusHandle,
    enabled: bool,
}

impl RadioGroup {
    pub fn new(name: impl Into<SharedString>, cx: &mut Context<Self>) -> Self {
        Self {
            name: name.into(),
            selected_id: None,
            active_id: None,
            items: Vec::new(),
        }
    }

    /// Registers/updates a radio button's metadata in the group.
    pub fn register(&mut self, id: SharedString, focus_handle: FocusHandle, enabled: bool) {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            item.focus_handle = focus_handle;
            item.enabled = enabled;
        } else {
            self.items.push(RegisteredRadioItem { id, focus_handle, enabled });
        }
    }
}
```

### 2. Roving Tabindex & Keyboard Navigation
To implement correct roving tabindex (WAI-ARIA compliance):
1. **Tab Stop Coordination:** Only the currently selected `RadioButton` in the group is marked as a tab stop (`tab_stop(true)`). All other buttons in the group have `tab_stop(false)`.
2. **Arrow Key Navigation:**
   * Each `RadioButton` listens for arrow key actions (`SelectNextItem`, `SelectPreviousItem`).
   * When an arrow key is pressed, the focused button calls the group coordinator to move selection.
   * The group coordinator updates its internal `selected_id`/`active_id` and programmatically focuses the newly active button's `FocusHandle` via `cx.focus(...)`.

```rust
impl RadioGroup {
    pub fn move_active_next(&mut self, cx: &mut Context<Self>) {
        let current_index = self.items.iter().position(|i| Some(&i.id) == self.active_id.as_ref());
        if let Some(idx) = current_index {
            let next_idx = (idx + 1) % self.items.len();
            let next_item = &self.items[next_idx];
            
            self.active_id = Some(next_item.id.clone());
            self.selected_id = Some(next_item.id.clone());
            
            // Programmatically transfer focus to the next button's FocusHandle
            cx.focus(&next_item.focus_handle);
            cx.notify();
        }
    }
}
```

### 3. Customized Items (Card Selection UI)
If a caller wants card tiles instead of standard radio buttons, they can style the individual button elements conditionally based on the group's state, without writing group templates:

```rust
let flavors = RadioGroup::new("UpgradePlans", cx);

h_stack()
    .child(
        RadioButton::new("starter")
            .group(&flavors)
            .custom_render(|state| {
                // state exposes: selected, active, hovered, pressed
                div()
                    .border_2()
                    .border_color(if state.selected { theme.ring } else { theme.border })
                    .bg(if state.selected { theme.selected_bg } else { theme.bg })
                    .child("Starter Plan")
            })
    )
```

---

## Generalizing to Other Group Controls (Toggles, Tabs, etc.)

We do **not** drop the concept of a generalized logical `ControlGroup`. Rather than being a monolithic layout container, `ControlGroup` acts as the pure logical/headless state engine and roving focus coordinator for **any** group control:

1. **Toggle/Toolbar Groups:** Toggle buttons (e.g., Bold/Italic/Underline) register their focus handles with a `ControlGroup` configured for `Multiple` selection. They are placed inside standard horizontal toolbars (`h_stack`) but share arrow-key and tab-focus coordination.
2. **Tab Strips:** Horizontal navigation tabs register with a `ControlGroup` configured for `SingleRequired` selection. They coordinate roving tabindex and keyboard focus as they are navigated.
3. **Card Groups:** Larger tile blocks (e.g., plan selection) register with a `ControlGroup` to inherit Arrow and Spacebar handlers.

This allows any custom visual components to be mixed, matched, and composed in arbitrary layouts, while sharing a single logical focus and selection engine.

---

## Initial Implementation Scope

To validate this design without risking regressions in stable SDK controls, the initial implementation of this decentralized architecture will be built entirely within the **Gallery App** under the experimental prototypes radio group folder:
* **Directory:** [apps/gallery/src/gallery/panes/prototypes/radio_control_group/](file:///Users/scg/Developer/GitHub/gpui-luma/apps/gallery/src/gallery/panes/prototypes/radio_control_group/)
* **Focus:** Replace the prototype's hardcoded layout/variants logic with the decentralized `RadioGroup` logical coordinator and independent `RadioButton` entities to prove roving focus and layout autonomy before promoting to the shared SDK.

---

## Benefits

* **Zero Custom Group Templates:** Callers layout options using standard container controls (like `v_stack` or `grid`). They never need to write layout wrappers that unpack handler iterators.
* **Guaranteed Focus Semantics:** Keyboard navigation and roving tabindex are enforced by the shared coordinator and the individual buttons. Bypassing standard layouts does not break focus.
* **Extremely Readable Code:** Relies on clear, declarative Rust builder patterns that mirror WPF’s group separation model.

---

## Summary of Work Done (Phase 1 Refactor)

The following changes were completed and verified to ensure that `RadioButton` can bind directly to a logical group coordinator without requiring a monolithic layout control:

### 1. Crate `crates/sdk` (Core Infrastructure)
* **[command/button/model.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/command/button/model.rs#L28):**
  * Defined the [ButtonGroupBinding](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/command/button/model.rs#L28) trait to abstract group actions (register, select, query data, query tab_stop, subscribe) from concrete group implementations.
  * Added the `group` field of type `Option<Arc<dyn ButtonGroupBinding<D>>>` to [ButtonModel](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/command/button/model.rs#L43).
* **[radio_button/mod.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/radio_button/mod.rs#L26):**
  * Exposed the `.group(impl ButtonGroupBinding<bool>)` builder method on [ButtonBuilder](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/radio_button/mod.rs#L26) to cleanly hook up radio groups.
* **[command/button/control.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/command/button/control.rs#L50):**
  * Added the `_group_subscription` field to [Button](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/command/button/control.rs#L17) to maintain subscription lifetimes.
  * In `Button::from_builder`, if a group is attached:
    1. Registers the button's ID and its `FocusHandle` with the group.
    2. Syncs the initial checked data state and roving `tab_stop` directly from the group's state.
    3. Subscribes to the group for dynamic selection changes (notifying and updating tab stops when another item is selected).
  * In `handle_click` and `handle_activate_control`, redirected button selection actions to execute `group.select(id, cx)` instead of local mutating operations.

### 2. Crate `apps/gallery` (Prototype Execution & Cleanup)
* **[radio_group.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/gallery/src/gallery/panes/prototypes/radio_control_group/control/radio_group.rs#L34):**
  * Created `PrototypeRadioGroupBinding` implementing `RadioButtonGroupBinding<bool>` to wrap the experimental `PrototypeRadioGroup` entity.
* **[radio_item.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/gallery/src/gallery/panes/prototypes/radio_control_group/control/radio_item.rs) [DELETED]:**
  * Completely deleted this redundant item wrapper file.
* **[demo.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/gallery/src/gallery/panes/prototypes/radio_control_group/control/demo.rs):**
  * Updated `RadioControlGroupDemo` struct and constructor to spawn standard SDK `RadioButton` controls directly in place of custom wrappers.
  * Implemented `group_layout` helper function to handle arrow key action delegation (bubbles key actions up to list/card layout containers and updates focus roving).
  * Cleared type signature warnings and simplified `notify_controls` to use standard entity notifications.


