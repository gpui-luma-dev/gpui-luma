# Bug Report: `deferred()` overlay with `.occlude()` blocks hit-testing and invalidation for trailing DOM column siblings

**Component:** `gpui` (Hit-Testing / Element Tree / Invalidation)

---

## Summary

When a popup-capable control rendering a `deferred(anchored()...)` overlay with `.occlude()` is placed inside a column container (`flex_col`), any entity sibling rendered **after** the popup control in the same column fails to receive pointer events and misses prepaint invalidation updates. 

Moving the sibling entity outside of the popup control's layout column (e.g., to a separate container or dock panel) immediately resolves the issue.

---

## Root Cause Analysis

1. **`deferred()` Hit-Test Tree Leak**:
   While `deferred()` positions element painting into a high-priority overlay render pass, the element's layout/hit-test node and `.occlude()` boundary remain anchored to its declaration site in the DOM column tree. Mouse hit-testing in reverse paint order evaluates the column container and incorrectly allows the occluded popup bounds to block pointer events targeting trailing siblings.

2. **Prepaint Invalidation Scope**:
   Controls measuring trigger bounds in prepaint (`on_children_prepainted` / `on_prepaint`) issue `cx.notify()` updates during the prepaint pass. This dirty notification marks only the emitting entity for the subsequent frame, causing GPUI's invalidation cycle to skip re-evaluating adjacent sibling entities in the same container branch.

---

## Steps to Reproduce

1. Construct a vertical column container:
   ```rust
   div()
       .flex()
       .flex_col()
       .child(combobox_entity.clone()) // Popup control using deferred(anchored())
       .child(event_log_entity.clone()) // Sibling entity listening for events
   ```
2. Interact with the `ComboBox` (e.g., select items or type queries).
3. Observe that `event_log_entity` does not receive pointer events or render updated event logs.
4. Move `event_log_entity` outside the column (e.g., into a sibling dock pane or separate flex container).
5. Repeat interactions—`event_log_entity` now records and paints events normally.

---

## Expected Behavior

- `deferred()` overlays should isolate hit-testing and `.occlude()` regions to the overlay stack layer rather than blocking sibling hit-testing at the element's declaration site in the DOM.
- Prepaint `cx.notify()` calls should not preempt or suppress invalidation passes for sibling entities in the same layout branch.

---

## Actual Behavior

Trailing sibling entities within the same column container are occluded from pointer hit-testing and miss frame invalidation updates.

---

## Environment

- **OS:** macOS
- **Framework:** GPUI (`zed-industries/zed`)
