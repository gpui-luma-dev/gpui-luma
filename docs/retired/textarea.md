# TextArea Implementation Overview

This document describes the `TextArea` implementation as it exists in the codebase today, including architecture, behavior, theming, gallery integration, and current tradeoffs.

---

## 1) What was implemented

`TextArea` is a first-class SDK control with:

- multiline editing
- keyboard and pointer selection
- clipboard operations
- IME input handler integration
- caret blink + caret visibility management
- vertical scrolling (wheel + scrollbar + selection autoscroll)
- row-based height plus drag resize
- validation-driven invalid visual state
- theming support
- gallery demo pane with telemetry and state preview

The implementation is fully wired into both the SDK exports and the gallery navigation/registry.

---

## 2) Source layout

### SDK control

- `crates/sdk/src/controls/textarea/mod.rs`
- `crates/sdk/src/controls/textarea/model.rs`
- `crates/sdk/src/controls/textarea/state.rs`
- `crates/sdk/src/controls/textarea/template.rs`
- `crates/sdk/src/controls/textarea/control.rs`

### SDK theme

- `crates/sdk/src/theme/textarea.rs`

### Gallery

- `apps/gallery/src/gallery/panes/textarea/mod.rs`
- `apps/gallery/src/gallery/panes/textarea/pane.rs`

### Wiring/exports updated

- `crates/sdk/src/controls/mod.rs`
- `crates/sdk/src/theme/mod.rs`
- `crates/sdk/src/theme/usage.rs`
- `apps/gallery/src/gallery/panes/mod.rs`
- `apps/gallery/src/gallery/panes/registry.rs`
- `apps/gallery/src/gallery/theme.rs`

---

## 3) Public API shape

You create a control with builder-style configuration:

- `TextArea::new(id)`
- `.placeholder(...)`
- `.value(...)`
- `.enabled(...)`
- `.full_width(...)`
- `.rows(...)` (clamped to at least `1`)
- `.clean_on_escape(...)`
- `.select_all_on_tab_focus(...)`
- `.validator(...)`
- `.template(...)`
- `.theme(...)`
- `.spawn(cx)`

Events emitted:

- `TextAreaEvent::Change { value }`
- `TextAreaEvent::Focus`
- `TextAreaEvent::Blur`

Useful runtime setters on the entity:

- `set_enabled`
- `set_value`
- `set_placeholder`
- `set_clean_on_escape`
- `set_validator`

Also exposed for DnD plumbing:

- `TextAreaDrag`

---

## 4) Data model and state contract

## `TextAreaModel` (configuration + dependencies)

Holds all static/semistatic config:

- ids/strings (`id`, `placeholder`, `value`)
- behavior flags (`enabled`, `full_width`, `rows`, `clean_on_escape`, `select_all_on_tab_focus`)
- optional `validator`
- injected `template` and `theme`

## `TextAreaState` (interactive state)

Character-indexed selection state:

- `hovered`
- `focused`
- `focus_visible`
- `invalid`
- `cursor`
- `selection_anchor`
- `preferred_column`

It implements the shared text selection trait used by text editing helpers (`cursor`, anchor normalization, selection range, clamping).

---

## 5) Editing behavior

Keyboard handling is delegated through shared text editing logic with this policy:

- multiline enabled
- Enter inserts newline (`submit_on_enter: false`)
- pasted newlines preserved
- Tab remains focus navigation (`allow_tab_character: false`)
- Escape clear controlled by `clean_on_escape`

Supported behavior includes:

- character insertion/replacement
- backspace/delete across lines
- select all
- copy/cut/paste
- word and line-aware navigation (via shared helper behavior)
- focus navigation when appropriate

`Change` events are emitted only when text actually mutates.

---

## 6) Pointer interaction

Implemented pointer features include:

- click to place caret
- shift-click extension via selection semantics
- drag selection
- double click word selection (word-cluster helper)
- triple click select all
- drag-selection autoscroll when pointer leaves viewport
- selection hit-point clamping to viewport edges

Selection autoscroll runs on a timer tick and scales speed by distance outside the viewport, capped to prevent runaway scrolling.

---

## 7) Focus + caret lifecycle

The control tracks focus from a `FocusHandle` and maintains:

- focus/blur event emission counters
- select-all-on-keyboard-focus behavior (when enabled)
- caret blinking (epoch-based timer)
- caret pause + force-visible after edits/moves
- focus-visible flag behavior
- blur handling for disabled state

When disabled while focused, the control blurs and suppresses editing interactions.

---

## 8) Layout, shaping, and rendering pipeline

The live control renders as a custom element (`TextAreaElement`) with a `prepaint` + `paint` pass.

## Text shaping and line model

- Input is first split into logical hard lines by `\n`.
- Each hard line is then segmented into visual lines to fit viewport width.
- Segmentation is greedy and prefers whitespace boundaries when possible.
- Each visual segment is shaped (`ShapedLine`) and cached with global character start/end indices.

Result: no horizontal scrollbar; long lines are displayed through visual line segmentation within control width.

## Paint content

During prepaint, it computes:

- selection quads
- caret quad
- placeholder shaped line (empty + unfocused)

During paint, it draws:

- selection backgrounds
- placeholder or shaped text lines
- caret (only when focused/caret-visible)

It also updates a layout cache used for hit testing, scrolling, and range bounds.

---

## 9) Scrolling model

Vertical scrolling is owned by `TextArea`:

- wheel/trackpad scrolling updates `vertical_scroll`
- vertical scrollbar entity is synchronized with viewport/content extents
- cursor movement and edits call `ensure_cursor_visible`
- selection drag can autoscroll continuously
- scroll values are clamped to content bounds

Scrollbar integration includes:

- length/range/value sync
- thumb fraction from viewport/content ratio
- line/page steps
- enable/disable based on content overflow + control enabled state

---

## 10) IME / input-method integration

`TextArea` implements `EntityInputHandler` and supports:

- UTF-16 range conversion
- selected range reporting
- marked text range
- replace text in range
- replace-and-mark text in range
- bounds lookup for a text range
- character index lookup for a point

Internally, editing state is still character-indexed; conversion helpers bridge to UTF-16 for platform text input APIs.

---

## 11) Resizing behavior

A resize affordance is rendered in the bottom-right corner when enabled.

- drag adjusts `rows`
- row count is clamped (`1..=40`)
- line height is used as resize step
- layout cache invalidates on row change
- cursor visibility is re-evaluated after resize

This produces a fixed-row baseline with user-adjustable row count.

---

## 12) Theme and template architecture

## Theme

`TextAreaTheme` resolves `TextAreaAppearance` with fields for:

- background/foreground/border/placeholder
- selection background/caret/focus ring
- typography + font family
- sizing metrics (`padding`, `radius`, `border_width`, `min_height`)

Token usage metadata is registered via `TEXTAREA_THEME_USAGE` and included in global theme usage lists.

## Template

`TextAreaTemplate` + `TextAreaRenderModel` are implemented and exported.  
`ThemedTextAreaTemplate` is available and used in gallery previews.

Important current detail: the live `TextArea` control renders through its custom element path and does not currently route live painting through `model.template.render(...)`. The template contract is actively used by preview/demo flows.

---

## 13) Gallery implementation

The gallery `TextArea` pane includes:

- a live TextArea control
- `Set Sample` and `Clear` actions
- `Enabled`, `Escape clears`, `Strict validation` toggles
- telemetry output for focus/blur/change/value/line counts and last event
- template state preview cards (default/hover/focus/selection/disabled)

Strict validation demo validator allows:

- ASCII alphanumeric
- space
- newline

The pane uses `radix_theme.textarea_template()` from the gallery's active `RadixTheme` (passed via `GalleryPanes`), so theme helper usage is centralized and compile warnings are avoided.

---

## 14) Tests included

Unit tests exist for key behavior in `textarea` modules, including:

- cursor clamping and selection normalization (`state.rs`)
- coordinate/local-axis mapping
- line-index mapping with scroll
- max scroll and clamping
- cursor reveal scrolling
- autoscroll delta behavior and acceleration cap
- selection hit-position clamping (`control.rs`)

---

## 15) Current characteristics and tradeoffs

- Internal selection model is character-indexed.
- IME bridges through UTF-16 conversion helpers.
- Visual line segmentation is width-fit based (no horizontal scrollbar path).
- Vertical scroll ownership is explicit and stable.
- Template system exists and is used for preview; live control currently paints directly.
- Core editing + scrolling + selection behavior is complete and demoed in gallery.

---

## 16) Known follow-ups

- **Live template integration:** Route the live control render path through `TextAreaTemplate::render(...)` (or formally split responsibilities) so preview and runtime share one presentation contract.
- **Wrapping policy clarification:** Decide and document the intended long-line strategy (strict hard-line behavior vs viewport-fit segmentation vs optional soft wrap modes) and expose explicit configuration if needed.
- **Performance tuning:** Profile shaping/allocation behavior on large documents and consider caching/incremental updates for line metrics and paint quads.
- **Selection rendering consolidation:** Reduce duplication between custom element prepaint selection/caret drawing and template preview rendering data paths.
- **IME fidelity roadmap:** Keep current UTF-16 bridge behavior, but evaluate whether a future offset model change is warranted after broader editor/input requirements are validated.
- **Resize affordance ergonomics:** Validate row-resize UX (bounds, drag sensitivity, discoverability) and consider making resize behavior configurable.
- **Expanded test coverage:** Add integration-style tests for mixed IME + selection operations, resize + scroll interactions, and edge cases around multiline clipboard edits.