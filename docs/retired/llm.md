# GPUI-Luma Task Context

Use the existing SDK architecture. Controls own behavior, templates own GPUI
structure, and themes own look policy.

Before changing controls, read the relevant implementation under
`crates/sdk/src/controls`, the matching theme under `crates/sdk/src/theme`, and
the gallery usage under `apps/gallery/src/gallery`. Treat the gallery as the
first real consumer and keep SDK APIs consumer-owned rather than gallery-owned.

Prefer existing shared infrastructure:

- `controls/interaction.rs` for single-surface hover, pressed, disabled,
  focus, and tab-stop behavior.
- `controls/state.rs` for focus and composite item state.
- `controls/value.rs` for numeric ranges, clamping, snapping, and finite input.
- `controls/menu_navigation.rs` for menu traversal.
- `controls/button_family.rs` and `theme/button_family.rs` for button-like
  controls.
- `focus.rs` and `keyhandling.rs` for focus traversal and typed keyboard
  actions.

Follow the local control split:

- `model.rs`: public builder, stored model, render model, item models.
- `control.rs`: live entity, behavior, events, focus, input, mutation methods.
- `template.rs`: template trait, default themed template, handler bundles.
- `mod.rs`: public re-exports.

Keep APIs typed. Do not add string-to-icon or string-to-token normalization to
the SDK. App-owned icons must be passed explicitly as typed Lucide icons or
explicit SVG paths. SDK-owned affordance icons are acceptable for built-in
control state, such as checkmarks and menu chevrons.

Keyboard behavior should use typed actions and key contexts. Pointer and
keyboard activation should share the same semantic event path where applicable.
Focus traversal details are in `docs/focus-handling.md`.

When adding or changing a control, update or add gallery coverage and focused
tests for non-trivial behavior such as value coercion, navigation, selection
normalization, or event emission.

Reference docs:

- `docs/control-design.md`: architecture and rationale.
- `docs/control-guidelines.md`: practical implementation guidance.
- `docs/focus-handling.md`: implemented focus model.
