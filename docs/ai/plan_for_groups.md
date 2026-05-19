# The "Anti-God-Group" Strategy: Introducing `ControlGroup`

To avoid creating a "God Group" while resolving the architectural overlap between `ChoiceGroup`, `RadioGroup`, and `ListBox`, we will use **Composition**. We will build one new, clean "Engine" and eventually wrap it in distinct, strongly-typed semantic controls.

## 1. The New Core Primitive: `ControlGroup`
We will build a brand new, standalone control called `ControlGroup` (`crates/sdk/src/controls/control_group`). 
- **Its only job:** Manage `CompositeFocus`, handle interaction state (hover/press), track `selected_indices` state, and enforce `ControlSelectionMode`.
- **What it DOES NOT do:** It knows absolutely nothing about Tabs, Toolbars, or Radios. It has no layout presets (no `style_preset(Radio)` or similar enums). It is 100% lookless and only knows how to render the `.template()` and `.item_template()` you pass it.

It will take the clean `CompositeFocus` architecture from the new `radio_group` and the depth of state management from the old `choice_group`.

## 2. Naming: Items, not Buttons
Even if the internal interaction engine uses GPUI's element state mechanics to get "free" hover/press/focus tracking, the API must not bleed "button-ness" into the consumer layer. A ListBox item might be a color swatch, a complex row, or an image. They are "Items" (pickable elements), not "Buttons".

The API will expose:
- `.template()` for rendering the group container.
- `.item_template()` for rendering the individual pickable items. 

We will create a specific `ControlGroupItemTemplate` trait (or type alias) so that consumers are providing templates for an "Item," completely abstracted away from the concept of a "Button."

## 3. The Features
- **`ControlSelectionMode`**: We will support the full depth of selection semantics to separate "required" from "optional":
  ```rust
  pub enum ControlSelectionMode {
      SingleRequired,   // Classic RadioGroup (always 1 selected)
      SingleAllowNone,  // Can be deselected (0 or 1 selected)
      Multiple,         // ToggleGroup/CheckboxGroup (0 to N selected)
  }
  ```
- **Managed State**: Port over the Managed vs. Unmanaged (`state_mode`) synchronization so the container can be controlled externally (e.g., source-of-truth React-style state).

## 4. The Semantic Controls (The Wrappers)
Once `ControlGroup` is proven, we will transition existing controls (`radio_group`, `listbox`, `tabs_navigation`) into entirely separate semantic modules that simply **wrap** the `ControlGroup` engine. The wrapper takes care of supplying the explicit themes.

For example, a future `listbox` would look like this:
```rust
pub fn listbox<T>(id: &str) -> ControlGroupBuilder<T> {
    control_group::new(id)
        .mode(ControlSelectionMode::SingleRequired) 
        .item_template(theme.listbox_item_template()) // Consumer explicitly maps the theme
        .template(theme.listbox_container_template())        
}
```

## The Execution Plan
1. **Create `ControlGroup`:** Scaffold `crates/sdk/src/controls/control_group` (mod, model, control, template). 
2. **Port Architecture:** Bring the `CompositeFocus` and interaction logic from the gallery's `radio_group`.
3. **Port Features:** Bring `ControlSelectionMode::Multiple` and Managed State from the old `choice_group`.

## Verification Plan
1. **Gallery Integration**: We will test the new `ControlGroup` by entirely replacing the `radio_group` usage in the Gallery app (`apps/gallery/src/gallery/panes/radio_group`) with this new `ControlGroup`. 
2. **Success Criteria**: The gallery must compile, render, and behave identically, proving that `ControlGroup` successfully replicates the new `RadioGroup`'s behavior while supporting future extensibility.
