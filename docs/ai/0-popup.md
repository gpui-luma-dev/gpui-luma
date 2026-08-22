# Popup and Overlay Lifecycle Consolidation

## Mission

Establish one reusable, trigger-aware overlay lifecycle for popup-like SDK controls and migrate the existing popup families onto it without collapsing their distinct interaction semantics.

The lifecycle must make the following behavior consistent across the SDK:

- Open, close, and toggle from a trigger or caller.
- Animate enter and exit while keeping closing content mounted.
- Dismiss on click-away according to an explicit policy.
- Dismiss on Escape through the shared focus/action path.
- Treat the trigger or anchor as part of the popup interaction surface.
- Prevent the opening gesture from immediately dismissing the popup.
- Track anchor/trigger and popup content bounds.
- Restore focus to the opener when the overlay closes when appropriate.
- Emit stable open/dismiss state transitions to the owning control.

The result should support `PopupMenu`, `ContextMenu`, `AnchoredPanel`, sidebar rail flyouts, and selector-family controls while leaving menu navigation, filtering, selection, and application-specific actions in their respective controls.

## Current state

The SDK already has several related layers:

- `OverlayPresence` in `crates/sdk/src/controls/overlay_presence.rs` provides animated mounted/unmounted overlay presence.
- `PopupLifecycle` in `crates/sdk/src/controls/popup_lifecycle.rs` currently centralizes popup open/close state, presence, trigger bounds, and trigger-aware outside-click exclusion.
- `PopupMenu` owns trigger rendering, menu navigation, nested submenu state, focus, and semantic menu events.
- `ContextMenu` owns target activation, menu navigation, nested submenu state, focus, and semantic menu events.
- `AnchoredPanel` owns caller-provided content, anchor/content bounds, configurable dismiss policies, focus-on-open, guarded opening, and focus restoration.
- `Selector`, `ComboBox`, `AutocompleteTextBox`, and `SearchSelector` each duplicate substantial open/close, `OverlayPresence`, click-away, Escape, and focus behavior around their own selection/query engines.
- `FloatingMenu` provides menu rendering/state support but is not itself a trigger or overlay lifecycle owner.
- Luma Studio’s Controls tab uses `TabsNavigation` to emit `DropdownRequested`, then owns an `AnchoredPanel` containing a custom control catalog picker.

## Desired architecture

Create a shared lifecycle boundary—likely evolving `PopupLifecycle` into `OverlayLifecycle` or `OverlaySession`—with responsibilities limited to overlay mechanics:

```text
Overlay lifecycle
    open / close / toggle
    presence animation
    anchor and content bounds
    click-away policy
    Escape dismissal
    guarded opening
    focus acquisition/restoration

Control-specific state
    PopupMenu: menu paths, submenus, action/select behavior
    ContextMenu: target activation and context positioning
    AnchoredPanel: caller-owned content and placement
    Selector: selected value and active option
    ComboBox: query and filtered options
    Autocomplete: editing/query behavior
    SearchSelector: search session and selection behavior
    Sidebar: rail node and submenu path
```

The lifecycle should support both visible triggers and indirect anchors:

- A `PopupMenu` trigger button.
- A selector text field/input surface.
- A right-click context target.
- A tab item that requests an anchored catalog popup.
- A collapsed sidebar rail item that opens a flyout.

The lifecycle must not impose one focus, Escape, submenu, or positioning policy on every control. Shared infrastructure should expose narrow capabilities and let each control select the appropriate policy.

## Specializations and interaction risks

### Focus ownership and input binding

Menus and inputs have different focus contracts:

- `PopupMenu` moves focus to the menu interaction entity so arrow navigation and menu actions can be handled there.
- `ComboBox` and `AutocompleteTextBox` must keep focus on the text-field handle while routing navigation actions to the popup options.
- `SearchSelector` and selector variants may combine input focus, active-option focus, and selection state differently.

The shared lifecycle may track focus ownership and restoration metadata, but it must not force every popup to transfer focus to its content. Focus routing remains a control-specific policy.

### Focus restoration versus focus loss

Closing because focus moved to another sibling control must not automatically restore focus to the opener. Doing so can steal focus from the user’s new target.

Focus restoration should be conditional and reason-aware:

- Explicit Escape dismissal may restore focus to the opener.
- Explicit trigger toggling may restore focus to the trigger.
- Item selection may restore focus according to the control contract.
- Click-away or ordinary focus-loss dismissal should generally preserve the newly focused target.

The lifecycle should represent a dismissal reason or restoration policy rather than unconditionally restoring focus after every close.

### Multi-stage Escape precedence

Escape is not universally equivalent to `close()`.

`AutocompleteTextBox`, for example, requires this precedence:

1. Close the open popover.
2. If the popover is closed and the query is non-empty, clear the query when `clean_on_escape` is enabled.
3. If the query is empty, bubble Escape to the parent form or window.

The lifecycle may provide an Escape-dismissal primitive, but controls must be able to consume or bubble Escape based on their own editing and selection state.

### Nested overlays and cascading submenus

`PopupMenu` and `ContextMenu` have submenu hierarchies. A single trigger bounds value and open flag are insufficient for outside-click decisions.

The design must support a stack or tree of interactive surfaces:

- A click inside an open child submenu may remain within the overlay tree.
- A click inside the parent menu but outside the child submenu should close only the child submenu when that is the established menu rule.
- A click outside the complete parent/child tree should close the whole popup.
- Escape should close the deepest active submenu before dismissing the parent popup when keyboard navigation requires staged closure.

Menu navigation state remains menu-specific, but the shared geometry/dismissal layer must be able to register multiple surfaces or accept a control-owned containment predicate.

### Event timing and stale bounds

GPUI can dispatch `on_mouse_down` and `on_mouse_down_out` during the same frame. Newly opened content may not have completed layout, so trigger or content bounds can be absent or stale.

The lifecycle must guard against false outside dismissal during opening by using one or more of:

- an explicit guarded-open phase;
- the opening pointer event or event target identity;
- the anchor bounds captured before opening;
- deferred activation of outside-click handling until the next layout/frame;
- control-owned containment for trigger and overlay surfaces.

Bounds must not be treated as valid merely because an overlay is logically open.

### Exit animation interruption

Rapid trigger re-toggle while an overlay is exiting must retarget the existing presence transition rather than reset or recreate the overlay.

The implementation must preserve:

- mounted popup content during exit;
- scroll position in `PopupScrollSurface`;
- active menu/query state when reopening is semantically expected;
- correct logical open state and event ordering.

### Dynamic re-anchoring and content resizing

Selector popovers can resize while the user types or filters options. `ComboBox`, `AutocompleteTextBox`, and `SearchSelector` may need to recalculate:

- content bounds;
- above/below placement;
- viewport clamping;
- scroll geometry;
- anchor alignment.

The shared lifecycle should provide bounds/invalidation hooks, but placement calculation and resize policy may remain control-specific. Static anchor capture is insufficient for dynamic selector content.

### Complete popup inventory

Before finalizing the abstraction, inventory all popup-like controls in the workspace, including:

- `PopupMenu` and `ContextMenu`;
- `Selector`, `ComboBox`, `AutocompleteTextBox`, and `SearchSelector`;
- `AnchoredPanel` and Luma Studio’s Controls catalog picker;
- `PagerControl` page-size dropdown;
- `TabsNavigationControl` `DropdownRequested` flow;
- `SplitButtonControl` primary-action versus popup-trigger split;
- Toolbar overflow menus;
- Color Field and related color popovers;
- Sidebar icon-rail flyouts;
- Any app-local anchored panels or custom floating menu compositions.

Each inventory item must document its focus owner, Escape precedence, dismissal policy, surface hierarchy, anchor source, dynamic sizing behavior, and event contract before migration.

Do not make every popup a `PopupMenu`. The reusable boundary is the lifecycle, not the rendered content or semantic event type.

## Scope

### In scope

- Define the shared lifecycle contract and state transitions.
- Preserve explicit dismiss policies such as click-away, focus-loss, both, or keep-open.
- Centralize trigger/anchor-aware outside-click decisions.
- Centralize guarded opening and Escape handling.
- Centralize optional focus-on-open and focus restoration.
- Migrate `PopupMenu` and the sidebar rail flyout from duplicated lifecycle state.
- Migrate `AnchoredPanel` to the shared lifecycle.
- Evaluate and migrate selector-family controls incrementally.
- Preserve each control’s existing semantic events and domain behavior.
- Add focused unit and interaction tests.
- Update `docs/architecture.md` and relevant SDK API documentation.

### Out of scope

- Replacing `FloatingMenu` rendering or menu navigation with a generic popup renderer.
- Merging `PopupMenuEvent`, `ContextMenuEvent`, selector events, or tab events into one event enum.
- Moving Luma Studio’s Controls catalog content into the SDK.
- Rewriting all overlay controls in one change.
- Changing visual placement, theme resolution, icon contracts, or menu row styling unless required by lifecycle correctness.
- Introducing a generic adorner system.

## Work plan

### Phase 1 — Inventory and contract

- Enumerate every SDK control that owns an overlay or popup.
- Record for each control:
  - open-state source of truth;
  - anchor/trigger bounds;
  - content bounds;
  - click-away behavior;
  - focus behavior;
  - Escape behavior;
  - guarded-open behavior;
  - presence animation;
  - emitted semantic events.
- Compare the current `PopupLifecycle`, `OverlayPresence`, and `AnchoredPanel` responsibilities.
- Define whether the public type remains `PopupLifecycle` or is renamed/generalized to `OverlayLifecycle`.
- Keep the public API minimal and document the migration intent.
- Record specialization constraints from focus ownership, Escape precedence, nested surfaces, stale bounds, interrupted presence, and dynamic placement before choosing shared methods.

### Phase 2 — Lifecycle implementation

- Add explicit dismiss-policy modeling if the existing policy cannot represent all current controls.
- Add trigger/anchor and content-bound registration, including multiple interactive surfaces or a control-owned containment predicate for nested overlays.
- Add `is_inside_anchor`, `is_inside_content`, and outside-click decision helpers without assuming a single surface hierarchy.
- Add guarded-open state so the opening pointer/focus transition cannot immediately dismiss the overlay, even when bounds are stale or unavailable.
- Add dismissal reasons and restoration policy rather than unconditional focus restoration.
- Add lifecycle methods for open, close, toggle, dismiss, presence sync, and frame scheduling while leaving Escape consumption/bubbling to controls that need multi-stage precedence.
- Define clear behavior for reopening while an exit animation is still running.
- Define focus ownership and restoration without assuming every overlay has a focusable child or that focus should leave an input field.
- Provide invalidation/re-anchoring hooks for content that changes size while open.
- Add tests for state transitions and pointer geometry.

### Phase 3 — PopupMenu migration

- Replace Popup Menu’s local open/presence/trigger lifecycle fields with the shared lifecycle.
- Keep `FloatingMenuState`, submenu transitions, highlight transitions, and `PopupMenuEvent` local to Popup Menu.
- Ensure the trigger is excluded from click-away dismissal.
- Preserve split-button semantics:
  - primary face emits `ActionClick`;
  - menu face toggles the popup;
  - menu item selection emits `Select`.
- Verify trigger click, click-away, Escape, focus loss, nested submenu close, and re-open during exit animation.

### Phase 4 — AnchoredPanel migration

- Replace local `open`, `OverlayPresence`, dismiss-guard, and bounds lifecycle state where appropriate.
- Preserve `AnchoredPanelDismissPolicy` as the caller-facing policy API.
- Preserve:
  - `open_from` and `toggle_guarded_from`;
  - focus-on-open;
  - opener focus restoration;
  - click-away versus focus-loss policy distinctions;
  - caller-owned content and placement.
- Ensure anchor bounds and content bounds are both recognized as inside the overlay interaction surface.
- Add regression tests for each dismiss policy.

### Phase 5 — ContextMenu and sidebar migration

- Migrate Context Menu’s overlay presence and dismissal checks while retaining target-driven activation and menu navigation.
- Migrate sidebar rail flyout lifecycle while retaining:
  - rail node identity;
  - rail submenu item state;
  - hover-driven nested submenu behavior;
  - custom sidebar placement and rendering.
- Verify the originating rail icon toggles closed instead of close-then-reopen.
- Verify outside click, Escape, item selection, focus, and animated exit behavior.

### Phase 6 — Selector family migration

Migrate incrementally, beginning with `Selector`, then `ComboBox`, `AutocompleteTextBox`, and `SearchSelector`.

- Keep query/editing behavior in each control’s behavior engine.
- Keep active-option navigation and selection logic local.
- Replace duplicated overlay mechanics with the shared lifecycle.
- Preserve selector-specific Escape semantics, especially clear-on-Escape versus dismiss-on-Escape precedence.
- Preserve focus/caret behavior when opening, filtering, selecting, and dismissing.
- Preserve dynamic popover placement and content resizing while filtering.
- Verify empty results, disabled controls, programmatic open/close, and rapid reopen/close.

### Phase 7 — Luma Studio integration review

- Confirm the Controls tab continues to use:
  - `TabsNavigationEvent::DropdownRequested` as the semantic trigger request;
  - `AnchoredPanel` for caller-owned catalog content;
  - `CloseOnClickAwayOrFocusLoss` policy;
  - guarded opening with the tab focus handle;
  - explicit close after catalog selection.
- Do not move the catalog picker into `PopupMenu` merely for visual similarity.
- Review other Luma Studio popup compositions for direct lifecycle duplication.
- Prefer SDK builders and semantic event subscriptions over app-local raw popup behavior.
- Add Pager page-size, Tabs dropdown, Split Button, Toolbar overflow, and color-popover paths to the integration inventory before declaring the migration complete.

## Acceptance criteria

- All migrated popup controls use one shared lifecycle for open/close/presence and dismissal mechanics.
- Clicking a popup trigger while its popup is open closes it exactly once.
- Clicking inside popup content does not dismiss it.
- Clicking outside both trigger/anchor and content dismisses it when policy allows.
- Escape dismisses the overlay through the shared focus/action path.
- Guarded opening prevents the opening click or focus transition from immediately closing the overlay.
- Focus restoration remains correct for controls that support it.
- Focus is not stolen from a sibling control after click-away or ordinary focus-loss dismissal.
- Selector-specific Escape precedence and parent action bubbling remain intact.
- Nested menu surfaces dismiss at the correct hierarchy level.
- Opening cannot self-dismiss because of stale or missing bounds.
- Re-toggle during exit preserves the expected mounted state, scroll state, and transition continuity.
- Dynamically resizing selector popovers remain correctly anchored and viewport-clamped.
- Existing menu, selector, and application semantic events remain unchanged.
- Popup and selector animations remain interruptible and do not leave stale mounted content or stale open state.
- Luma Studio Controls catalog behavior remains unchanged: tab reactivation toggles the catalog picker, selection changes the Controls exposition, and the picker closes after selection.
- Sidebar rail and popup trigger behavior remain consistent with standard popup expectations.

## Verification matrix

### Shared lifecycle

- Open, close, toggle, and reopen during exit animation.
- Trigger/anchor geometry inclusion.
- Content geometry inclusion.
- Outside click.
- Escape.
- Guarded open.
- Optional focus restoration.
- Dismissal reason and restoration policy.
- Guarded opening with stale/missing bounds.
- Multiple-surface containment.
- Re-anchor after content-size changes.

### PopupMenu and split button

- Labeled trigger.
- Icon-only trigger.
- Split primary action versus popup trigger.
- Click-away and Escape.
- Nested submenu navigation and close.
- Disabled state.

### AnchoredPanel and Controls tab picker

- Below/above placement.
- Click-away policy.
- Focus-loss policy.
- Combined policy.
- Keep-open policy.
- Tab reactivation toggle.
- Catalog selection closes picker and updates Controls panel.

### Selectors

- Selector open/close and selection.
- ComboBox query/filter and selection.
- Autocomplete query and clear-on-Escape behavior.
- SearchSelector filtering and selection.
- Empty results and disabled state.
- Focus/caret restoration.
- Text-field focus remains owned by the input controls.
- Multi-stage Escape clears, dismisses, or bubbles in the documented order.
- Filtering-driven resize and above/below placement remain stable.

### Additional popup controls

- Pager page-size dropdown.
- Tabs Controls catalog picker.
- Split Button action face versus popup face.
- Toolbar overflow.
- Color Field and related color popovers.

### Sidebar rail

- Rail icon opens flyout.
- Same rail icon closes flyout.
- Click-away closes flyout.
- Escape closes flyout.
- Nested submenu behavior.
- Animated close remains mounted until settled.

## Risks and design constraints

- `PopupMenu` has a visible trigger while `ContextMenu` and some selectors use indirect or focus-based anchors.
- `PopupMenu` focus ownership differs from input-backed selector focus ownership.
- Focus-loss dismissal and explicit dismissal have different restoration requirements.
- Escape may be an editing command, a popup dismissal, or a parent-level command depending on control state.
- Nested menu surfaces require hierarchical containment rather than one overlay bounds value.
- GPUI event/layout timing can make bounds temporarily stale during opening.
- Selector content can resize and require continuous re-anchoring.
- Selector Escape behavior may conflict with generic overlay dismissal and must retain its precedence rules.
- Focus restoration must not steal focus from a newly activated control or from an unrelated window.
- Click-away handling must account for deferred/anchored GPUI elements and event ordering.
- Presence animation and logical open state must remain distinct.
- A generic lifecycle must not force all controls to emit identical event enums.
- The existing SDK/app LMTP split must remain intact: lifecycle state belongs in controls, while Luma Studio composes and reacts to semantic events.

## Deliverables

- Shared lifecycle implementation and public documentation.
- Migrated Popup Menu, Anchored Panel, Context Menu, sidebar rail, and selector-family controls.
- Focused lifecycle and control regression tests.
- Updated architecture documentation.
- Manual verification notes for Luma Studio Controls tab, popup menus, selectors, split buttons, context menus, and sidebar rail flyouts.
