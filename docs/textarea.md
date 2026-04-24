# TextArea Work Document

## Purpose

Build a core `TextArea` control after stabilizing `TextField`.

This document is intentionally scoped to the text editing control itself. It is
not a form-field composition plan. Labels, prompt text, validation messages,
feedback icons, tooltips, trailing icons, counters, and similar UI affordances
are add-ons that should live in a future wrapper/composition layer, not inside
the core `TextArea`.

## How We Got Here

The first attempt tried to move toward a shared, "awesome text field / text
area" direction before the simple single-line field was fully working. That was
the wrong order.

The problems showed up immediately:

- `TextField` had basic editing and layout bugs.
- `TextArea` was being built while the shared editing behavior was still
  unstable.
- Scrollbar work arrived before caret, selection, and scroll ownership were
  settled.
- TextArea scroll behavior fought cursor visibility and snapped back to the
  caret in some cases.
- Trackpad scrolling and thumb scrolling did not share a clear interaction
  contract.
- Preview/template layout broke because render models were hand-built without
  the same layout data as the live control.

The reset was useful: drop the active `TextArea`, make `TextField` work first,
then use that working baseline to build `TextArea` deliberately.

Current baseline:

- `TextField` works for single-line editing.
- Long text is clipped and horizontally scrolled.
- Selection, copy, cut, paste, backspace, delete, and keyboard movement work.
- The shared text editing helpers now live under
  `crates/sdk/src/controls/text/`.
- Gallery has working icon and no-icon `TextField` examples.
- TextArea is not currently part of the active SDK/gallery surface.

## Core Boundary

`TextArea` owns:

- multiline text value,
- placeholder rendering,
- focus and blur,
- cursor and selection,
- keyboard editing,
- clipboard editing,
- pointer placement and drag selection,
- vertical scrolling,
- caret visibility,
- enabled/disabled state,
- invalid state as a styling flag,
- theme/template integration,
- change/focus/blur events.

`TextArea` does not own:

- label text,
- prompt/helper text,
- validation message text,
- feedback icons,
- trailing icons/actions,
- tooltip behavior,
- character counters,
- form layout,
- field-level error presentation.

Those should be composed around `TextArea` later by a form-field wrapper.

## Design Principle

Do not touch the working `TextField` while building `TextArea`.

`TextArea` may use existing shared text editing helpers that already exist under
`crates/sdk/src/controls/text/`, but this pass must not refactor `TextField`,
rewrite the shared helpers, or move behavior around just because it looks
shareable.

The priority is to make the core `TextArea` work. Text area behavior is
complicated enough on its own: multiline layout, selection across lines, pointer
hit-testing, vertical scrolling, and caret visibility all interact. Broad
"make it common" refactors should wait until both controls work and the common
shape is proven by two stable implementations.

At the same time, do not block `TextArea` on a large upstream-editor port. The
next step is a pragmatic core control using the current SDK architecture. If we
later need IME-perfect byte-offset state or advanced wrapping, that can be a
targeted follow-up after the basic control is working.

Hard rule for this implementation:

- do not modify `crates/sdk/src/controls/textfield/*`,
- do not change existing `TextField` behavior,
- do not refactor unrelated controls,
- do not redesign `controls/text/` as part of bringing `TextArea` back,
- only touch existing shared helper code if a small bug fix is required for
  `TextArea` and it does not change `TextField` semantics.

## Files To Add

```text
crates/sdk/src/controls/textarea/
  mod.rs
  model.rs
  state.rs
  template.rs
  control.rs

crates/sdk/src/theme/textarea.rs

apps/gallery/src/gallery/panes/textarea/
  mod.rs
  pane.rs
```

Files to update:

```text
crates/sdk/src/controls/mod.rs
crates/sdk/src/theme/mod.rs
crates/sdk/src/theme/usage.rs
apps/gallery/src/gallery/panes/mod.rs
apps/gallery/src/gallery/panes/registry.rs
apps/gallery/src/gallery/theme.rs
```

Avoid changes outside this list unless they are strictly required to wire the
new control into the SDK or gallery. In particular, avoid edits to the
`textfield` module during this pass.

## Public API Shape

Start small. The initial API should only expose core control behavior.

```rust
TextArea::new("id")
    .placeholder("Message")
    .value("...")
    .enabled(true)
    .full_width(true)
    .rows(4)
    .clean_on_escape(true)
    .select_all_on_tab_focus(true)
    .validator(...)
    .template(...);
```

Builder fields:

- `id: SharedString`
- `placeholder: SharedString`
- `value: SharedString`
- `enabled: bool`
- `full_width: bool`
- `rows: usize`
- `clean_on_escape: bool`
- `select_all_on_tab_focus: bool`
- `validator: Option<Validator>`
- `template: Arc<dyn TextAreaTemplate>`

Events:

```rust
pub enum TextAreaEvent {
    Change { value: String },
    Focus,
    Blur,
}
```

Do not add submit-on-enter. In a text area, Enter inserts a newline.

## State Contract

For the initial control, use a character-indexed selection model compatible with
the current `controls/text` helpers. This keeps the work bounded and prevents an
offset-model rewrite from being mixed into the TextArea reintroduction.

```rust
pub struct TextAreaState {
    pub hovered: bool,
    pub focused: bool,
    pub focus_visible: bool,
    pub invalid: bool,
    pub cursor: usize,
    pub selection_anchor: Option<usize>,
    pub preferred_column: Option<usize>,
}
```

Rules:

- `cursor` is a character offset into the full value.
- `selection_anchor` is a character offset into the full value.
- `selection_range()` normalizes anchor/cursor order.
- `preferred_column` is used by Up/Down movement.
- `invalid` is a state flag only; messages are not part of core `TextArea`.

This is intentionally compatible with the stabilized `TextField` baseline
without requiring `TextField` changes. A future byte-offset migration should
happen after both controls work, not as a private TextArea decision and not as a
prerequisite for this pass.

## Editing Policy

Use `EditableTextPolicy` from `crates/sdk/src/controls/text/state.rs`.

TextArea policy:

```rust
EditableTextPolicy {
    multiline: true,
    submit_on_enter: false,
    strip_newlines_on_paste: false,
    allow_tab_character: false,
    select_all_on_keyboard_focus: true,
    clear_on_escape: clean_on_escape,
}
```

Initial behavior:

- Enter inserts `\n`.
- Paste preserves newlines.
- Backspace/delete remove selection or adjacent text.
- Cmd/Ctrl+A selects all.
- Cmd/Ctrl+C copies selection.
- Cmd/Ctrl+X cuts selection.
- Cmd/Ctrl+V pastes clipboard text.
- Left/Right move by character, word, or line boundary according to platform
  modifiers already handled by shared helpers.
- Up/Down move between lines and preserve preferred column.
- Escape clears only when `clean_on_escape` is enabled.

Tab should initially remain focus navigation, not text insertion. We can add an
`allow_tab_character` builder later if there is a concrete need.

## Layout Model

The first TextArea should use explicit hard lines split by `\n`.

Initial layout:

- Split `value` into logical lines by newline.
- Shape each logical line independently.
- Use theme typography line height for row height.
- Compute caret x/y from line index and shaped x offset.
- Compute selection rectangles per affected line.
- Use a fixed viewport height based on `rows`.
- Use vertical scrolling when content exceeds the viewport.
- Clip content to the viewport.

Do not implement soft wrapping in the first pass. Soft wrapping is valuable, but
it changes hit-testing, selection rectangles, and vertical movement. It should be
a second milestone after hard-line multiline editing is stable.

Horizontal behavior for long lines:

- Initial implementation may clip long lines horizontally.
- Do not add horizontal scrollbar in the first pass.
- Do not auto-grow width.
- If horizontal movement becomes necessary, add horizontal offset as a follow-up
  after vertical scrolling is stable.

## Scroll Contract

TextArea owns vertical scroll state.

Required behavior:

- Trackpad/wheel scroll inside the text area moves the text viewport.
- Dragging the scrollbar thumb moves the text viewport.
- Releasing the scrollbar thumb must not snap back to the cursor.
- Cursor movement by keyboard should ensure the cursor is visible.
- Pointer scrolling should not automatically force the cursor back into view.
- Typing should ensure the cursor is visible after mutation.
- Mouse click placement should move the cursor and may scroll only through the
  normal pointer/drag behavior.

This is the main lesson from the failed TextArea pass: user scroll position and
caret visibility are related, but they are not the same state.

Recommended fields:

```rust
vertical_scroll: Pixels,
layout_cache: Option<TextAreaLayoutCache>,
scroll_origin: TextAreaScrollOrigin,
```

Where `TextAreaScrollOrigin` distinguishes programmatic cursor visibility from
direct user scrolling if needed.

## Template Contract

The template should render from a complete render model. It must not recompute
editing behavior.

Render model should include:

```rust
pub struct TextAreaRenderModel<'a> {
    pub id: &'a SharedString,
    pub placeholder: &'a SharedString,
    pub value: &'a SharedString,
    pub enabled: bool,
    pub full_width: bool,
    pub rows: usize,
    pub state: TextAreaState,
    pub caret_visible: bool,
    pub vertical_scroll: f32,
    pub line_metrics: Vec<TextAreaLineMetric>,
    pub selection_rects: Vec<TextAreaSelectionRect>,
}
```

The control computes `line_metrics` and `selection_rects` from GPUI text
shaping. The template only paints:

- shell,
- placeholder,
- text lines,
- selection backgrounds,
- caret,
- focus ring,
- invalid/disabled/hover states.

This avoids the preview bug we hit with `TextField`, where template previews did
not receive the same layout data as live controls.

## Theme

Add `TextAreaTheme` separately from `TextFieldTheme`, but keep the initial token
mapping parallel.

Appearance fields:

- `background`
- `foreground`
- `border`
- `placeholder`
- `selection_background`
- `caret`
- `focus_ring`
- `typography`
- `min_height`
- `padding_x`
- `padding_y`
- `radius`
- `border_width`

Use the same invalid border token as TextField:

```text
form.input.invalid_border
```

Do not add label, helper, prompt, or message colors to `TextAreaAppearance`.
Those belong to a future form-field wrapper.

## Gallery Scope

Add one TextArea page with:

- one normal TextArea,
- one disabled/readonly visual state only if the core supports it cleanly,
- `Set Sample` button,
- `Clear` button,
- `Enabled` checkbox,
- `Escape clears` checkbox,
- `Strict validation` checkbox,
- telemetry for value length, line count, focus count, blur count, change count,
  and last event,
- template state preview.

Do not add label/helper/error examples to this page. Those would teach the wrong
boundary.

Sample values should include:

- short single-line text,
- several hard lines,
- a long line that exceeds viewport width,
- enough lines to require vertical scrolling,
- multibyte text for selection and cursor smoke testing.

## Implementation Plan

### Phase 1: Restore The Module Skeleton

- Add `controls/textarea` module files.
- Export `textarea` from `controls/mod.rs`.
- Add `theme/textarea.rs`.
- Export `TextAreaTheme` from `theme/mod.rs`.
- Add TextArea usage metadata.
- Add gallery routing and page registration.

Exit criteria:

- Empty/minimal TextArea renders in gallery.
- Workspace compiles.

### Phase 2: State And Builder

- Add `TextAreaModel`.
- Add `TextAreaBuilder`.
- Add `TextAreaState`.
- Implement `TextSelectionState` for `TextAreaState`.
- Add `TextAreaEvent`.
- Add setters for `value`, `enabled`, `placeholder`, `clean_on_escape`, and
  validator.

Exit criteria:

- Programmatic set/clear works.
- Invalid state recomputes when value or validator changes.
- Unit tests cover cursor clamping and selection normalization.

### Phase 3: Keyboard Editing

- Wire `handle_key_down` from `controls/text/editing.rs`.
- Use multiline policy.
- Preserve newlines on paste.
- Emit `Change` only when value changes.
- Prevent default only for handled text commands.
- Keep Enter as newline insertion.
- Keep Tab as focus navigation for now.

Exit criteria:

- Typing works.
- Enter inserts newline.
- Backspace/delete work across lines.
- Copy/cut/paste work.
- Cmd/Ctrl+A works.
- Up/Down movement works by logical hard line.

### Phase 4: Layout And Painting

- Shape each hard line.
- Build line metrics.
- Paint text lines.
- Paint caret.
- Paint selection rectangles across multiple lines.
- Paint placeholder when empty and not focused, matching TextField behavior.
- Keep shell height fixed from `rows`.

Exit criteria:

- Multiline text paints correctly.
- Selection across lines is visible.
- Empty state is stable.
- Setting/clearing value does not resize unexpectedly except by configured
  `rows`.

### Phase 5: Pointer Selection

- Map pointer x/y to line and character offset.
- Click places cursor.
- Drag selects across lines.
- Double-click selects word cluster using shared helper.
- Triple-click can be deferred unless it is easy after line hit-testing exists.

Exit criteria:

- Mouse placement matches visible text.
- Drag selection works within and across lines.
- Selection does not break with multibyte text.

### Phase 6: Vertical Scrolling

- Add vertical scroll state.
- Clip content to viewport.
- Add wheel/trackpad scrolling.
- Add scrollbar integration if the SDK scrollbar is ready for this use.
- Ensure cursor visibility after keyboard movement and typing.
- Do not force cursor visibility after direct user scroll.

Exit criteria:

- Trackpad scroll works inside the text area.
- Scrollbar thumb works if present.
- Releasing the thumb does not snap back to the cursor.
- Keyboard movement keeps the caret visible.
- Typing at the bottom scrolls only enough to keep the caret visible.

### Phase 7: Gallery And Manual Pass

- Add gallery page.
- Add state preview.
- Add telemetry.
- Add samples that exercise scrolling and selection.
- Run a manual checklist before calling the control stable.

Exit criteria:

- Gallery demonstrates core behavior without form-field add-ons.
- Manual pass finds no blocking editing/layout/scroll issues.

## Manual Acceptance Checklist

- Empty placeholder displays.
- Typing first character does not resize the control.
- Clearing text does not resize the control.
- Enter inserts newline.
- Paste preserves multiline content.
- Backspace joins lines correctly.
- Delete joins lines correctly.
- Cmd/Ctrl+A selects all lines.
- Copy/cut/paste work with multiline selection.
- Shift+arrows select across lines.
- Up/Down preserve column.
- Click places caret on the intended line.
- Drag selects across line boundaries.
- Selection rendering handles partial first and last lines.
- Long content scrolls vertically by trackpad.
- Scrollbar drag, if present, does not snap back on release.
- Keyboard caret movement keeps caret visible.
- Pointer scrolling does not force caret visibility.
- Disabled state blocks editing.
- Invalid state changes border only, with no message rendering.
- Multibyte text does not break cursor placement or selection.

## Explicit Non-Goals For This Pass

- No label API.
- No helper/prompt text API.
- No validation message API.
- No tooltip API.
- No trailing icon API.
- No feedback icon API.
- No character counter.
- No soft wrapping.
- No auto-grow height.
- No rich text.
- No syntax highlighting.
- No undo/redo unless it falls out naturally from existing GPUI input handling.
- No IME architecture rewrite.

## Risks

The highest-risk parts are layout and scroll ownership.

Hard-line layout keeps the first pass small, but it means long lines are not
soft-wrapped yet. That is acceptable for the initial control because it lets us
prove multiline editing, selection, and vertical scrolling first.

The second risk is offset correctness. The current shared helpers use character
offsets. That matches the stabilized `TextField`, but it is not the final
highest-fidelity model for IME and byte/UTF-16 boundaries. Do not solve that in
private TextArea code. Also do not solve it by changing `TextField` during this
pass. If it becomes necessary, record it as follow-up work after `TextArea` is
working.

## Definition Of Done

TextArea is done for this pass when:

- it is part of the SDK exports,
- it has a gallery page,
- it supports core multiline editing,
- it supports selection and clipboard operations,
- it supports vertical scrolling without cursor snap-back,
- it uses theme/template architecture like other controls,
- it has unit coverage for shared editing/state behavior,
- `cargo fmt --check` passes,
- `cargo check --workspace` passes,
- `cargo test -p gpui-luma` passes,
- a manual gallery pass completes without blocking issues.
