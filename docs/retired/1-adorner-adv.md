# Adorner Advanced: Error and Validation Adorners

## Status

Planning issue. This document describes a future validation/error adorner enhancement;
it does not change the current focus-ring implementation.

## Goal

Add theme-driven error adorners for controls that own or present user-editable values,
without introducing control-specific border mutation or duplicating validation paint
logic in templates.

The enhancement should reuse the existing adorner architecture:

```text
Control state (invalid/error)
        ↓
Theme resolves typed AdornerSpec
        ↓
Template renders shared decorative overlay
```

## Typical Target Controls

### Initial targets

- **TextField** — already has `TextFieldState::invalid` and validator support.
- **TextArea** — already has `TextAreaState::invalid` and validator support.

These should be the reference implementations because they already own validation state,
caret/selection behavior, and form-control chrome.

### Composed input targets

- **ComboBox** — validation belongs to the text input/trigger host, not the popup panel.
- **Autocomplete** — validation belongs to the editable input host.
- **SearchSelector** — validation belongs to the trigger/input chrome.

These controls should expose or forward validation state rather than independently
inventing error adorners for their child controls.

### Secondary form targets

- **Checkbox**
- **Radio Button**
- **Switch**
- **Radio Group / Control Group**

These can be invalid in form workflows, but currently lack a first-class validation
state. Add validation state only when the control needs to communicate an actionable
form error; do not infer it from selection or disabled state.

### Conditional targets

- **Selector** — useful when a required selection is missing, but not intrinsically an
  input-validation control.
- **Slider and color controls** — only when a value-domain validator exists; otherwise
  external helper/error text is preferable.

Popup menus, navigation, layout controls, and display-only controls are not typical
error-adorner targets.

## Proposed Adorner Model

Add an error/validation variant to the typed adorner contract, for example:

- `AdornerSpec::Error(ErrorAdornerSpec)`
- color
- placement (`Inset` or `Oversize`)
- distance
- width

The exact name can follow the project’s eventual validation vocabulary (`Error`,
`Invalid`, or `Validation`). The spec must remain decorative-only: no handlers, focus
ownership, or hit-testing behavior.

The current one-adorner policy requires an explicit decision:

1. **Precedence model:** error adorner replaces the normal focus adorner while invalid.
2. **Composition model:** a look can return multiple ordered adorners, such as an error
   ring plus a focus ring.

Prefer precedence for the first implementation. It preserves the current
`Option<AdornerSpec>` contract and avoids ambiguous ring ordering. The error state should
win whenever the control is invalid and enabled; focus remains available through the
control’s normal accessibility behavior.

## Shadcn Color Policy

Use semantic Shadcn tokens rather than introducing a new fixed hue:

- Error adorner: `destructive`
- Error text/icon: `destructive`
- Error-filled surface: `destructive` with an appropriate theme-defined opacity
- Text on an error-filled surface: `destructive-foreground`

The adorner should not use `destructive-foreground`; that token is intended for content
painted on a destructive background. Existing invalid input borders should converge on
the same semantic source.

Light and dark themes may resolve `destructive` differently. The renderer should not
know those values or hardcode red tones.

## State and Precedence Rules

- Invalid state is distinct from disabled, hovered, pressed, and focused state.
- Disabled controls may retain invalid state for validation semantics, but their adorner
  should be muted or suppressed according to the theme policy.
- Invalid + focused: error adorner takes precedence over the standard focus ring in the
  initial implementation.
- Invalid + unfocused: error adorner remains visible if the theme considers persistent
  validation useful.
- Validation state changes must not alter measured bounds.
- Validator execution and error-message ownership remain in the control/application
  state boundary; the adorner only communicates the resolved visual state.

## Implementation Outline

1. Extend `AdornerSpec` with an error/validation variant and shared rendering support.
2. Add a theme-level helper for constructing error adorners from semantic color and
   metric tokens.
3. Update TextField and TextArea theme resolution to choose error versus focus adorner.
4. Ensure the shared host/overlay path preserves IDs, event handlers, caret/selection
   rendering, shadows, and stable footprint reservation.
5. Forward validation state through ComboBox, Autocomplete, and SearchSelector.
6. Add explicit invalid state APIs to Checkbox, Radio Button, Switch, and groups only if
   their form use cases require it.
7. Update Shadcn usage metadata and Gallery/Luma Studio inspectors to show the adorner
   source and destructive token provenance.
8. Add tests for invalid/focused, invalid/unfocused, disabled-invalid, theme switching,
   and focus-toggle geometry stability.

## Acceptance Criteria

- Error adorners are resolved entirely by themes and rendered by the shared adorner
  renderer.
- No migrated template mutates a focus/error border directly.
- Invalid state does not change measured control bounds or cause sibling jitter.
- Keyboard focus, hit testing, caret, selection, and validation events are unchanged.
- Shadcn light/dark themes use the semantic `destructive` token.
- Invalid + focused behavior is deterministic and documented.
- Inspector/provenance surfaces identify the `destructive` source.

## Out of Scope

- Error-message layout, helper text, tooltips, or announcement semantics.
- A general-purpose form-validation framework.
- New adorner kinds unrelated to validation.
- Hardcoded color literals in the SDK or look crates.
