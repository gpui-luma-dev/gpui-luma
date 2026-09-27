# Desktop input policies (#87)

This implementation changes input behavior and configuration only. Theme colors,
borders, geometry, focus-indicator visibility, and the shared `ControlFocusState`
behavior are unchanged. The issue's visual-focus work is excluded by the user's
explicit no-styling instruction.

## Defaults and independent overrides

| Role/control | Wheel eligibility | Boundary behavior | Hover |
| --- | --- | --- | --- |
| TreeView, ListBox, scrolling Table | Pointer | Contain | Feedback; preserves active item |
| TextArea, ScrollContainer | Pointer | Chain | Feedback |
| Embedded SelectionPanel | Pointer | Contain | Feedback; preserves active item |
| Popup SelectionPanel | Pointer | Contain | Follows pointer while panel owns focus |
| ComboBox, SearchSelector, Autocomplete popup | Pointer while open and enabled | Contain | Existing candidate highlighting |
| Paged Table | Passes through | No wheel page changes | Feedback |

An embedded viewport does not infer policy from its ancestors. The host explicitly
chooses `RequireFocus` or `PassThrough` when desired. Pointer operations focus and
act in the same gesture by default; wheel input never acquires focus or selects.

Import the public policy types from `luma::interaction` or the SDK prelude:

- `WheelScrollPolicy::Pointer`: scroll under the pointer without requiring focus.
- `RequireFocus`: accept only when the configured focus scope owns real focus.
- `PassThrough`: leave both offset and propagation untouched. Programmatic
  positioning, deliberate scrollbar input, and drag auto-scroll remain available.
- `ScrollBoundaryPolicy::Contain`: consume applicable eligible input at limits,
  including empty/non-overflowing content.
- `Chain`: pass only wholly unhandled events. Partial movement consumes the entire
  event; no residual delta is forwarded or reapplied to the parent.
- `WheelFocusScope::Owner`: only the viewport's actual focus handle qualifies.
- `OwnerAndScrollbar` (default): also permits its designated scrollbar. Focus in
  an embedded row editor/button does not qualify.
- `Descendants`: explicit opt-in for descendants. This affects wheel eligibility
  only; ancestor navigation and editing still require the owner's focus.

Only the supported axis is used: Y for vertical viewports and X for horizontal
ListBox. Diagonal input uses that component. Unsupported axes and zero/non-finite
deltas pass through. Pixel deltas retain fractional precision; line deltas use
the control's line height. Eligibility is checked for each event, including
momentum. A zero gesture-end event may finish Table snapping but is not consumed.

## Construction, compatibility, and runtime changes

TreeView, ListBox, Table, TextArea, SelectionPanel, ScrollContainer, and the three
editable popup selectors expose independent wheel/boundary/focus-scope settings.
Shadcn builders forward these settings; Radix uses SDK builders for these families.

```rust,ignore
use luma::interaction::{ScrollBoundaryPolicy, WheelScrollPolicy};

let primary = shadcn::TreeView::new("primary")
    .look(look)
    .items(nodes)
    .wheel_scroll_policy(WheelScrollPolicy::Pointer)
    .scroll_boundary_policy(ScrollBoundaryPolicy::Chain)
    .spawn(cx);

let embedded = shadcn::TextArea::new("notes")
    .look(look)
    .wheel_scroll_policy(WheelScrollPolicy::RequireFocus)
    .scroll_boundary_policy(ScrollBoundaryPolicy::Contain)
    .spawn(cx);
```

Entity controls have `set_wheel_scroll_policy`, `set_scroll_boundary_policy`, and
`set_wheel_focus_scope` methods with their context argument. Retained scrolling
handles have equivalent setters. These changes preserve data, selection, and
position. Table clears pending gesture snapping when its wheel settings change.
There is no implicit application/subtree inheritance. Last explicit setter wins
for that dimension only.

The existing `require_focus_for_scroll(bool)` on TreeView/ListBox maps to
RequireFocus/Pointer, without changing boundary behavior. Studio's existing uses
remain valid. Studio → ListBox → Drag and drop reuses the existing pair of lists
to demonstrate Pointer/Chain on the left and RequireFocus/Contain on the right;
its layout and styling are unchanged.

A ScrollContainer has no implicit keyboard owner. Supply `wheel_focus_owner`
when using RequireFocus. Manually composed ListBoxes should call
`set_wheel_focus_owner(binding.focus_handle().clone())`; ListBoxControl and Shadcn
composition wire this automatically. Legacy state-only ListBox hosts remain
supported but cannot distinguish descendant focus without a real focus handle.

TreeView, ListBox, Table, TextArea, and SelectionPanel also support
`pointer_focus_policy(PointerFocusPolicy::Preserve)`: pointer operations leave
keyboard focus with its existing owner. This construction-time override does not
manufacture focus or let unfocused controls handle editing/navigation keys.
ListBoxBinding also exposes a setter. Existing visual focus rules are unchanged.

SelectionPanel defaults to `SelectionPanelRole::Embedded`. `Popup` permits
hover-to-active while the panel owns focus. An explicit
`HoverActivationPolicy::{PreserveActive, FollowPointer}` wins over either role,
regardless of setter order; explicit FollowPointer allows unfocused activation
candidate changes but never commits selection or acquires focus. Runtime
`set_hover_activation_policy` changes that override. Pressing a row emits an
active-item change only when the active item actually changes.

## Integration and remaining inventory

GPUI 1.21 ListState has no wheel-disable hook. Its shared adapter records the
position in capture and restores rejected native movement in bubble before
ancestors or painting. Do not install native ListState scroll callbacks that
expose that transient movement. ScrollHandle adapters use clipped overflow and
one explicit clamped delta; native and custom movement are never combined.
Positioning, keyboard reveal, virtualization, and drag auto-scroll use the same
retained geometry independently of wheel eligibility.

PopupScrollSurface keeps `render` for standalone scrolling. Composite owners use
`render_with_scroll_wheel` (or `render_without_wheel` plus their own callback).
Callbacks are attached inside the popup, where wheel events occur. Popup hitboxes
continue to block pointer interaction behind them but permit policy-selected
wheel propagation. Custom templates must preserve that distinction.

SearchSelector returns focus to its trigger after Escape/selection only when its
own popup search still owns focus. Outside dismissal does not steal focus from
another control. Disabling a focused collection releases focus. Disappearing
scrollbars return focus to the owning viewport (or blur for an ownerless container).

Source inspection, not native-platform verification:

- Plain Selector retains its separate native scrolling template and popup
  containment. Its wheel-policy integration remains a selector-template follow-up
  under #78; this implementation does not claim that API coverage.
- Popup/context menus retain legitimate hover/submenu behavior and existing menu
  focus contexts. Their current panels have no independent scrolling viewport.
- Commands, sliders, scrollbars, tabs, ControlGroup, and split handles retain
  existing interaction and styling. No wheel-to-value behavior is added.
- ColorField remains pointer-only. A separate follow-up needs a real focus handle,
  accessible value semantics, and keyboard movement in its color domain. No
  keyboard accessibility is claimed here. SplitButton follow-ups remain in #31.

## Verification

Headless GPUI dispatch covers the wheel-policy × boundary-policy × focused/unfocused
matrix for all six viewport families, checking child and parent offsets and focus.
Additional regressions cover popup single application/pass-through, Escape and
selection restoration, embedded/popup hover, descendant focus, legacy precedence,
runtime changes, builder forwarding, and existing positioning/virtualization/DnD.

Native GUI/device checks remain unperformed: no GUI launch was authorized. Pixel
and line events are simulated; physical trackpad momentum, window activation,
and platform focus traversal still require manual verification.
