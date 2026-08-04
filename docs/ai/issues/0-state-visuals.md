# State Visuals: TextField Validation and Required States

## Goal

Replace the adorner-based validation approach with explicit state visuals that follow the
existing TextField/theme boundaries and remain composable by higher-level form fields.

The implementation should cover invalid, required, and focus presentation without adding a
generic adorner engine or hidden control behavior.

## Design Principles

- TextField theme resolution owns TextField chrome: background, foreground, border, caret,
  selection, typography, and focus styling.
- A required marker belongs to the label or field wrapper, not the low-level TextField.
- Validation messages belong to the field/form wrapper, not the low-level TextField.
- Validation behavior belongs to form/domain state. The theme only renders the state it is
  given.
- The SDK TextField must not silently inject icons, labels, helper text, or layout rows.
- Focus visuals must be explicit theme behavior. If a theme does not provide focus styling,
  focus must not create a visual effect.

## Target State Model

The existing TextField state remains the input to theme resolution:

- `focused`: the control owns and reports focus state.
- `focus_visible`: distinguishes keyboard-visible focus where the theme needs it.
- `invalid`: the form/validator owns the decision; the TextField renders it.
- `hovered` and `enabled`: continue resolving normal interaction chrome.

Required-ness and validation text should not be added to the low-level TextField solely for
visual display. A higher-level field model may own:

```text
FieldModel {
    label,
    required,
    value,
    validation_state,
    validation_message,
    text_field,
}
```

## Visual Contract

### Invalid TextField

When `invalid == true`, the TextField theme resolves its border to the destructive color.
The focus treatment, if the theme provides one, must use the same destructive color or be
suppressed according to the theme’s explicit precedence rule.

There must be one coherent border treatment—not a normal border plus an additional invalid
border layered accidentally.

The SDK TextField does not automatically render an alert icon. If an icon is needed, expose
an explicit suffix/content slot or compose it in a field wrapper.

### Required Field

The field wrapper renders the required marker next to the label, for example:

```text
Email *
```

The marker is not part of TextField’s input chrome and must not affect TextField layout unless
the wrapper intentionally reserves space for it.

### Validation Message

The field wrapper renders validation text below the TextField, for example:

```text
Please enter a valid email address.
```

The wrapper owns message layout, color, accessibility association, and whether the message is
shown after blur, submit, or an explicit validation event.

## Implementation Plan

### Phase 1: Remove Adorner Dependency

- Remove the TextField-specific adorner model, builder override, renderer, collection, and
  reserved-extent logic.
- Remove all TextField exposition examples that depend on focus pointers or adorner geometry.
- Keep focus and invalid state plumbing intact.
- Confirm no Button, ButtonFamily, or unrelated control code is changed.

### Phase 2: Resolve TextField Chrome Directly

- Resolve `TextFieldLook.border` from `TextFieldState.invalid` in each look implementation.
- Use the destructive theme token for invalid borders.
- Define focus styling as an explicit TextField theme contract, independent of validation.
- Ensure a theme that does not define focus styling produces no focus visual.
- Keep look resolution pure: construct the complete `TextFieldLook` once without mutating it
  afterward.

### Phase 3: Add Explicit Suffix Composition

Only if the product requires an invalid icon:

- Add a named TextField suffix/content slot to the SDK, or compose the icon in a wrapper.
- Do not make the base TextField infer or inject an alert icon from `invalid`.
- Ensure suffix content participates in normal measurement and does not shift on focus or
  validation transitions unexpectedly.

### Phase 4: Add a Field Wrapper Contract

- Introduce or identify the higher-level field wrapper responsible for label, required marker,
  TextField, and validation message.
- Keep required state and validation message ownership in that wrapper.
- Pass only the derived `invalid` state into TextField.
- Define blur/submit validation policy outside the theme and SDK TextField renderer.

### Phase 5: Update Luma Studio Exposition

Add focused examples for:

1. Normal TextField.
2. Focused TextField using the configured theme focus treatment.
3. Required field with an empty invalid value, destructive border, required marker, and
   validation message.

The exposition should make the ownership boundary visible: label and error text belong to the
field composition, while border and input chrome belong to TextField.

## Theme Inspector Plan

The inspector must describe actual theme outputs, not imply unsupported states.

- Remove Focus from the TextField/TextArea inspector state matrix when the active theme has no
  focus visual.
- Reintroduce Focus only when the TextField theme resolves a real focus visual.
- Inspect invalid border color from the same resolved TextField palette used by rendering.
- Keep required marker and validation message out of the low-level TextField theme inspector;
  inspect them through the field-wrapper inspector when that wrapper exists.
- Do not expose adorner geometry, reserved extents, or generic adorner categories.

## Acceptance Criteria

- Empty required fields do not show a focus-colored border before receiving focus.
- Invalid focused fields show one deliberate destructive border treatment.
- A theme without focus styling produces no focus visual.
- Required markers and validation messages are wrapper-owned and composable.
- TextField layout remains stable across focus, blur, and invalid transitions.
- No generic adorner types or adorner-specific template branches remain in the TextField path.
- Existing Button/ButtonFamily behavior and style-guide output are unchanged.
- SDK, look, inspector, and Luma Studio checks pass.
