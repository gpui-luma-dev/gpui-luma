# Fix Adorners: Make Them a First-Class SDK Concept

## Problem

Adorners are intended to decorate an existing control without requiring control-specific
template changes. The current implementation does not yet provide a complete contract for
how an adorner participates in measurement, layout, painting, and clipping.

This creates several problems:

- Focus adorners can change the control's effective extent as focus comes and goes,
  shifting neighboring controls.
- The current transparent-border technique reserves space indirectly. It prevents some
  layout movement, but it is a workaround rather than an explicit layout policy.
- Oversized rings can be positioned with inconsistent offsets because the relationship
  between control bounds, adorner bounds, and metric tokens is not centrally defined.
- Some controls render adorners through the shared path, while custom or group templates
  bypass that path and implement validation borders independently.
- Invalid state has been added in places where the template does not expose a reliable
  generic adorner seam, producing inconsistent visuals and duplicated border logic.
- Adorners can be visually “on top” while still needing their paint overflow or reserved
  extent accounted for by the template and its parent.
- The `Option<AdornerSpec>` contract creates a single-item bottleneck: a focus ring cannot
  coexist with a validation badge, warning, status mark, or caller-defined gizmo adorner.
- Control entities currently synthesize alternate states, such as a focused probe, to
  discover reservation geometry. This makes every new adorner state a control-specific
  maintenance problem.
- App/prototype code currently matches `AdornerSpec` and performs ad hoc extent math,
  demonstrating that the SDK has not centralized collection, ordering, and extent
  resolution.

The result is that adding invalid or focus visuals is not yet as simple as attaching an
adorner to a control state. It can require changes to the control template, custom border
logic, or compensating layout behavior.

## Design Goal

Make adorners a first-class SDK concept with an explicit contract for:

1. State resolution by the control/theme boundary.
2. Geometry and placement relative to the measured control.
3. Whether the adorner contributes to measured extent.
4. Whether the adorner paints inside, over, or outside the control bounds.
5. Clipping and paint-overflow requirements.
6. Stable transitions between default, focus, invalid, disabled, and other states.

Adding an adorner should not require modifying an existing control template when the
control already exposes the generic adorner seam.

## Delivery Order

Focus adorners are the first priority. The focus geometry and extent contract must be
corrected and verified before implementing or migrating invalid adorners.

### Phase 1: Fix Focus Adorners

- Define whether the focus ring uses reserved extent, overlay, or overflow behavior.
- Make measurement and paint placement use the same metrics and pixel-snapping rules.
- Eliminate focus/blur layout shifts and inconsistent offsets.
- Verify clipping behavior for controls at container edges.
- Update existing focus-adorned controls without adding validation behavior.

### Phase 2: Add Invalid Adorners

Only after Phase 1 is stable:

- Replace the single-adorner contract with an ordered collection of generic adorners.
- Resolve validation color and geometry through the theme.
- Allow invalid, focus, badges, warnings, and future caller-defined adorners to coexist.
- Confirm all combinations preserve the shared extent contract.
- Migrate controls incrementally through the generic adorner seam.

Group-control validation remains deferred until the group chrome and extent behavior have
been separately designed and reviewed.

## Extent and Placement Model

“Adorner” describes ownership and layering. It must not implicitly determine layout extent.
The adorner contract should distinguish at least these policies:

### Overlay

Paints on top of the control within its existing bounds.

- Contributes no external extent.
- Must not move siblings.
- Appropriate for an inset validation mark or decoration fully contained by the control.

### Reserved

The template reserves the adorner's maximum geometry during measurement.

- The adorner contributes to the control's measured extent.
- The reserved extent exists in all states, even when the adorner is not painted.
- Focus transitions therefore paint into already-reserved space instead of changing layout.
- This is the correct model for an oversize focus ring when the ring must remain visible
  without being clipped.

### Overflow

Paints outside the control bounds, but does not become normal content extent.

- The template reports paint overflow/insets to its parent or clipping host.
- Appropriate when the visual should extend beyond the control but should not affect normal
  sibling layout.
- Requires an explicit clipping policy; it must not depend on accidental `overflow_visible`
  behavior.

The SDK should use typed extent/placement data rather than relying on callers to infer the
policy from border widths or transparent colors.

## Adorner Object Model

An adorner is an attachable visual object, not a rendering callback or a control-specific
border branch. The SDK must not hardcode domain slots such as `focus`, `validation`, or
`status`; a caller or theme may provide any adorner, including a caller-defined
`GizmoAdorner`. Controls may install adorners through an API shaped like:

```rust
button.adorners(vec![Arc::new(GizmoAdorner::new(...))])
```

The configured adorner owns its visual behavior and geometry rules. Candidate adorners
include `BorderAdorner`, `UnderlineAdorner`, `CornerAdorner`, `FocusRingAdorner`, and
application-defined adorners. These should be replaceable or composable without changing
the host control template.

The reusable adorner configuration must not capture host-specific absolute coordinates.
Instead, rendering has two stages:

1. **Configured adorner:** stores semantic color/metric references, shape, layer ordering,
   and placement policy. Theme-derived values must remain resolvable at render time.
2. **Resolved adorner:** combines the configured object with the host bounds, radius,
   device scale, full control state, and clipping context to produce concrete paint bounds,
   reserved insets, overflow insets, and pixel-snapped geometry.
3. **Resolved plan:** aggregates an ordered collection of resolved adorners and combines
   their extent and host-chrome requirements.

The public shape should be generic rather than a fixed enum of product concepts:

```rust
type Adorners = Vec<Arc<dyn Adorner>>;

trait Adorner: Send + Sync {
    fn resolve(&self, context: AdornerContext) -> ResolvedAdorner;
}

struct ResolvedAdornerPlan {
    layers: Vec<ResolvedAdorner>,
    reserved_insets: Insets,
    overflow_insets: Insets,
    chrome_overrides: HostChromeOverrides,
}
```

The resolved plan is consumed by both measurement and painting. The geometry used to
reserve layout must be the same geometry used to paint each adorner. A template should
ask the shared plan for its resolved layout and then host the resulting layers; it should
not reimplement border widths, offsets, radius calculations, precedence, or
transparent-border behavior.

Ordering, replacement, and coexistence must be represented by generic layer metadata and
plan resolution, not by fields named after particular adorner domains. The control passes
its actual state once; it must not synthesize `focus_state`, `invalid_state`, or other
probe states for each possible adorner kind.

## Host Chrome Overrides

Some adorners need to replace or suppress part of the host's normal chrome rather than
only paint an additional layer. For example, a focus adorner may hide the normal border,
replace it with its own border treatment, or adjust the TextField background.

The resolved adorner plan should therefore be able to return bounded semantic host-chrome
overrides alongside its paint layers:

```rust
struct HostChromeOverrides {
    background: ChromeOverride<BackgroundSpec>,
    border: ChromeOverride<BorderSpec>,
    shadow: ChromeOverride<ShadowSpec>,
}

enum ChromeOverride<T> {
    Keep,
    Suppress,
    Replace(T),
    Overlay(T),
}
```

Templates should expose named chrome slots such as surface, border, shadow, and focus
decoration. The adorner may control those slots through the resolved override plan, but it
must not mutate arbitrary GPUI elements or change content composition, input behavior, or
event handling. If a required slot is not exposed by a template, a derived/custom template
remains the correct escape hatch.

The design must distinguish ordinary control state from independently attachable chrome:

- A focused or invalid TextField background that is intrinsic to the control belongs in the
  theme-resolved `TextFieldLook`.
- A background supplied by an attachable decoration belongs in the adorner host override.

Host overrides must use the same measurement contract as the adorner paint layer. Replacing
or suppressing a border must not change the control's external bounds during state
transitions; the resolved plan must reserve the maximum geometry required by the relevant
chrome states.

## Focus Adorner Requirements

Focus adorners should use a stable maximum geometry contract:

- Measure the control with the focus-adornment reservation or declared overflow policy.
- Paint the focus ring when focused.
- Paint the same geometry transparently or leave the reserved paint slot empty when not
  focused, according to the selected policy.
- Keep the control's external bounds stable across focus transitions.
- Use one shared metric calculation for both reservation and paint placement.

The transparent paint approach may remain an implementation detail, but it must be the
result of an explicit reservation policy—not the mechanism that defines layout stability.

## Validation Adorner Requirements (Phase 2)

Validation should be represented as control state and resolved by the theme, not as a new
control-template-specific border path.

- Invalid and focus must be able to coexist or be ordered/replaced according to generic
  plan metadata; the SDK must not hardcode validation-specific precedence.
- Invalid and invalid + focus must use the same declared geometry unless the theme explicitly
  chooses otherwise.
- Validation adorners must not alter measured bounds when replacing or accompanying another
  adorner.
- The semantic `destructive` token belongs to the theme/look resolution layer.
- Validation state must not introduce validation eventing, error-message ownership, or form
  lifecycle behavior into the SDK control.

## SDK Architecture Requirements

The enhancement should respect the LMTP split:

- **Model:** stores static adorner/extent configuration only when configuration is needed.
- **Control:** exposes runtime state such as focused, invalid, enabled, and hovered once to
  the template/adorner resolver; it does not construct synthetic state probes.
- **Template:** supplies host bounds and context, applies the resolved adorner measurement,
  and hosts its paint layer.
- **Theme:** resolves an arbitrary ordered adorner collection, colors, metrics, layer
  metadata, and semantic token provenance at render time.
- **Adorner object:** owns shape-specific layout and paint behavior without owning the
  host control's state or lifecycle, and may return bounded semantic host-chrome overrides.

The shared renderer should receive enough information to render the adorner without knowing
which control owns it. Templates should not mutate control state or duplicate adorner
geometry policy.

## Proposed Contract Questions

The implementation should settle these questions explicitly:

- Is the public API an arbitrary trait-object collection (`Vec<Arc<dyn Adorner>>`) or a
  concrete collection with shape-specific implementations behind it?
- Which data belongs to the configured adorner versus the render-time resolved adorner?
- How are layout reservation and paint overflow represented in the measured bounds API?
- Which layer owns clipping decisions when an overflow adorner crosses a parent boundary?
- How are arbitrary adorner layers ordered, composed, suppressed, or replaced without
  hardcoded fields for focus, validation, status, or other domain concepts?
- How are pixel snapping and device scale applied consistently to control and adorner bounds?
- How does a derived template preserve the base template's adorner extent contract?
- How can a caller add or replace an adorner such as `GizmoAdorner` without bypassing
  theme token resolution?
- Which semantic host-chrome slots are common enough to support (`background`, `border`,
  `shadow`, and focus decoration), and which changes require a custom template?
- How are host-chrome overrides included in the maximum geometry calculation so replacing
  or suppressing a border cannot shift content or neighboring controls?

The initial implementation should prefer a small typed generic collection and one resolved
plan over a fixed set of domain-specific slots. A `GizmoAdorner` must require no SDK enum
variant or control-template change.

## Acceptance Criteria

- Focus transitions do not shift neighboring controls.
- Invalid transitions do not shift neighboring controls.
- Adorner geometry is derived from the same metrics used for extent calculation.
- Overlay, reserved, and overflow behavior are distinguishable and testable.
- No transparent-border workaround is required to preserve layout stability, even if
  transparent paint remains an allowed implementation technique.
- Existing controls can adopt validation visuals through state/theme resolution without
  bespoke template border logic.
- Arbitrary adorners can be added, ordered, and composed without adding domain-specific
  fields to the SDK adorner model.
- No app or prototype code performs adorner matching, extent calculation, or paint geometry
  itself.
- Custom templates either inherit the shared adorner seam or explicitly document why they
  provide a different geometry contract.
- Tests cover default, focused, invalid, invalid + focused, disabled, theme switching,
  pixel snapping, clipping, and measured-extent stability.
- Inspector/provenance surfaces identify the semantic token and the resolved extent policy.

## Non-Goals

- A general-purpose form validation framework.
- Error-message layout, helper text, tooltips, or accessibility announcements.
- Validation eventing owned by SDK controls.
- Hardcoded destructive colors or control-specific visual constants.
- Adding separate “validated” control families when a shared control/group contract is
  sufficient.

## Migration Guidance

1. Preserve the working TextField/TextArea state and theme behavior while fixing focus
   geometry; do not broaden validation scope during Phase 1.
2. Move focus-adornment geometry onto the shared measurement path.
3. Verify focus measurement, paint placement, clipping, and transition stability.
4. Refactor the single-adorner API into the generic configured-adorners and resolved-plan
   model before adding validation badges or other secondary adorners.
5. Remove synthetic state probes from control entities and manual adorner logic from app
   prototypes.
6. Implement invalid adorners so they can coexist with or replace focus adorners without
   changing bounds.
7. Migrate other controls only after their templates expose the generic seam.
8. Defer group-control validation until the group chrome, item chrome, and extent policy are
   designed as one coherent system.

## Current Focus Inventory

This is the current source-level inventory before the generic adorner-plan redesign.
“Focus adorner” describes the legacy `AdornerSpec` path; “focusable” means the SDK control
owns or exposes a keyboard focus target. A control can be focusable without currently
painting a focus adorner. The inventory must be rechecked after arbitrary adorner
collections and resolved plans replace the single-item path.

| Control or family | Focus adorner today | Focusable today | Current notes |
| --- | --- | --- | --- |
| Button / icon button | Yes | Yes | Uses the shared Button Family focus adorner. |
| Toggle | Yes | Yes | Uses Button Family toggle chrome. |
| Checkbox | Yes | Yes | Choice-control theme resolves a focus ring. |
| Radio Button | Yes | Yes | Choice-control theme resolves a focus ring. |
| Switch | Yes | Yes | Choice-control theme resolves a focus ring. |
| TextField | Yes | Yes | Shared text input adorner path. |
| TextArea | Yes | Yes | Shared text input adorner path. |
| Selector | Yes | Yes | Adorner belongs to the visible trigger. |
| SearchSelector | Yes | Yes | Adorner belongs to the visible trigger/input chrome. |
| ComboBox / Autocomplete | Via editable host | Via editable host | Focus adorner is owned by the embedded TextField host. |
| RadioGroup / ControlGroup | Item-level | Yes | Group focus is available; the group shell does not currently have a reliable generic focus adorner contract. |
| Slider | Yes | Yes | Thumb/slider focus uses the slider adorner path. |
| Scrollbar | Yes | Yes | Scrollbar focus uses the scrollbar adorner path. |
| Popup Menu | Trigger-level | Yes | Focus adorner belongs to the menu trigger. |
| Context Menu | Target-level | Yes | Focus adorner belongs to the context-menu target. |
| Navigation Sidebar | Internal focus targets | Internal focus targets | Focusable rows/rail targets resolve navigation adorners. |
| Accordion | No | Yes | Has a focus handle, but its current theme palette has no adorner. |
| ListBox | No | Internal row navigation | Focus is represented through row state rather than a shared adorner today. |
| ListView | No | Internal row navigation | Row palette currently has no adorner. |
| TreeView | No | Internal row navigation | Row palette currently has no adorner. |
| Tabs Navigation | No generic adorner | Yes | Owns focus targets, but focus presentation is not currently a shared adorner. |
| Toolbar | Item-dependent | Yes | Individual toolbar items may use Button Family adorners. |
| Dialog / Overlay Window | No generic control adorner | Yes | Focus trap/ownership exists; container focus decoration is a separate concern. |
| Dock Splitter / Anchored Panel | No generic focus adorner | Yes | Focusable infrastructure controls; focus paint is not currently a shared adorner. |

This table should be rechecked after Phase 1. In particular, the redesign should decide
whether row-based controls and group containers need a reserved, overlay, or overflow focus
contract rather than adding ad hoc rings to each row or group template.
