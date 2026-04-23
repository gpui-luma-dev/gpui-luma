# Text Input VNext

## 1. Purpose

This document records the recommended reset for SDK text inputs after the
initial `TextField` and `TextArea` port.

The main conclusion is now clear:

- we should keep the SDK control shell we already built,
- we should not keep iterating on the current per-control editing internals,
- we should rebuild the behavior layer from the shared input engine in
  `gpui-component`.

The goal is not to discard all current work. The goal is to replace the weak
part of the port with the architecture that originally made these controls
work.

## 2. What We Keep

The current SDK work is still useful and should remain:

- public `TextField` and `TextArea` builders,
- SDK control/module organization,
- templates and theme integration,
- gallery panes and routing,
- focus ring and appearance fixes,
- current event surface where it already matches expected SDK behavior.

The current code to preserve as shell and presentation:

- `crates/sdk/src/controls/textfield/*`
- `crates/sdk/src/controls/textarea/*`
- `crates/sdk/src/theme/textfield.rs`
- `crates/sdk/src/theme/textarea.rs`
- `apps/gallery/src/gallery/panes/textfield/*`
- `apps/gallery/src/gallery/panes/textarea/*`

## 3. What We Should Replace

The current editing engines should not be evolved much further in place:

- `crates/sdk/src/controls/textfield/control.rs`
- `crates/sdk/src/controls/textarea/control.rs`

The problem is not cosmetic. The problem is architectural.

Today:

- `TextField` contains a partial editing engine,
- `TextArea` contains a much smaller, different editing engine,
- both controls look similar,
- neither control is built on a shared text model,
- every parity bug becomes a one-off fix.

That is why the work felt disproportionately hard. We ported the visible
control surface, but not the shared editor core that supported it originally.

## 4. Reference Implementation

The correct source of truth is not `gpui-component/src/text`.
It is:

- [`/Users/scg/Developer/GitHub/gpui-component/crates/ui/src/input`](</Users/scg/Developer/GitHub/gpui-component/crates/ui/src/input>)

That directory already has the architecture we are missing.

Most important files:

- [`input.rs`](</Users/scg/Developer/GitHub/gpui-component/crates/ui/src/input/input.rs>)
  wires one shared state object into GPUI actions, focus tracking, mouse
  handling, and rendering shells.
- [`state.rs`](</Users/scg/Developer/GitHub/gpui-component/crates/ui/src/input/state.rs>)
  is the actual editor core: text storage, selection, clipboard behavior,
  paste normalization, mouse hit-testing, caret blink, focus lifecycle, and
  mode-aware mutation.
- [`mode.rs`](</Users/scg/Developer/GitHub/gpui-component/crates/ui/src/input/mode.rs>)
  defines single-line and multi-line as modes on one engine.
- [`movement.rs`](</Users/scg/Developer/GitHub/gpui-component/crates/ui/src/input/movement.rs>)
  owns horizontal and vertical movement, including preferred-column behavior.
- [`selection.rs`](</Users/scg/Developer/GitHub/gpui-component/crates/ui/src/input/selection.rs>)
  owns word and line selection semantics.
- [`element.rs`](</Users/scg/Developer/GitHub/gpui-component/crates/ui/src/input/element.rs>)
  owns text layout, caret geometry, selection painting, and mouse-to-offset
  mapping.

## 5. Why That Source Matters

`gpui-component` does not model text field and text area as separate editing
implementations. It models them as one editor engine with different mode and
policy.

That engine already covers the exact gaps that caused trouble in the SDK port:

- shared selection model,
- `Cmd/Ctrl+A/C/X/V`,
- word-wise movement and deletion,
- mode-aware paste rules,
- pointer hit-testing,
- vertical movement for multiline editing,
- caret blink lifecycle,
- focus and blur integration,
- line-aware geometry for multiline content.

This means the SDK does not need to invent a new architecture from scratch.
It should extract and simplify the one that already exists upstream.

## 6. Recommended Architecture In SDK

Introduce a shared text-input domain under:

```text
crates/sdk/src/controls/text/
```

Suggested split:

```text
controls/text/
  state.rs        shared editable state
  mode.rs         single-line vs multiline policy
  movement.rs     cursor movement and preferred-column logic
  selection.rs    selection utilities and double/triple-click behavior
  clipboard.rs    copy/cut/paste helpers
  caret.rs        blink and focus-visible behavior
  layout.rs       caret/selection geometry contracts
  keymap.rs       key event / action translation
```

`TextField` and `TextArea` should become thin wrappers over this shared layer.

Wrappers should own:

- public builder API,
- SDK event types,
- template selection,
- theme selection,
- control-specific policy such as submit-on-enter or rows.

Shared text core should own:

- text storage,
- cursor and selection state,
- pointer-based caret placement,
- movement and deletion semantics,
- clipboard behavior,
- paste normalization,
- focus sync,
- caret blink,
- render-model data for caret and selection.

## 7. Scope Of The Merge

We should not wholesale copy `gpui-component/src/input`.

### Port directly or adapt heavily

- mode concept from `mode.rs`
- selection semantics from `selection.rs`
- movement logic from `movement.rs`
- clipboard and mutation behavior from `state.rs`
- caret/focus behavior from `state.rs`
- geometry concepts from `element.rs`

### Do not port as part of this text-control merge

- LSP integration
- search panel
- code editor features
- diagnostics
- popovers and completion UI
- inline completion
- masking and number-input specific behavior unless required later
- app-specific root focus bookkeeping

The SDK needs the editor core, not the entire application editor platform.

## 8. Data Model Recommendations

The shared state should be richer than the current SDK state structs.

At minimum:

```rust
pub struct EditableTextState {
    pub hovered: bool,
    pub focused: bool,
    pub focus_visible: bool,
    pub invalid: bool,
    pub caret_visible: bool,
    pub selection_reversed: bool,
    pub selected_range: Range<usize>,
    pub preferred_column: Option<(f32, usize)>,
}
```

Policy should be explicit:

```rust
pub struct EditableTextPolicy {
    pub multiline: bool,
    pub submit_on_enter: bool,
    pub strip_newlines_on_paste: bool,
    pub allow_tab_character: bool,
    pub select_all_on_keyboard_focus: bool,
    pub clear_on_escape: bool,
}
```

Recommended defaults:

`TextField`:

- `multiline = false`
- `submit_on_enter = true`
- `strip_newlines_on_paste = true`
- `allow_tab_character = false`

`TextArea`:

- `multiline = true`
- `submit_on_enter = false`
- `strip_newlines_on_paste = false`
- `allow_tab_character = true` or SDK-defined

## 9. Input Model Recommendations

The current SDK uses ad hoc `on_key_down` branching in each control.
The upstream engine is better because it splits:

1. key bindings and actions,
2. semantic movement and selection methods,
3. render-time geometry.

The SDK should adopt that shape.

Preferred command families:

- movement commands,
- selection commands,
- delete commands,
- clipboard commands,
- insert commands,
- submit and escape commands.

This should replace the current arrangement where `TextField` and `TextArea`
each interpret raw keys on their own.

## 10. Layout And Rendering Recommendations

The current SDK text field uses per-character cells and the current text area
still uses a temporary "`|` inside the string" caret trick. Neither should be
the long-term model.

The shared layer should expose layout-facing data so templates can render:

- the caret as its own visual element,
- selection as explicit highlighted ranges,
- placeholder independently from text content,
- multiline caret placement based on line geometry,
- focus ring independently from editor internals.

Template responsibilities should remain:

- structure,
- spacing,
- background and border,
- icons and affixes,
- focus ring,
- caret and selection drawing from render data.

Templates should not own:

- text mutation,
- selection rules,
- clipboard behavior,
- keyboard shortcut interpretation,
- cursor movement semantics.

## 11. Execution Plan

This should be handled as a deliberate merge, not as incremental bug-fixing.

### Phase 1: Freeze The Current Shell

- Keep the current SDK builders, modules, themes, and gallery pages.
- Treat the existing text control internals as temporary.
- Avoid adding new parity fixes unless they unblock the merge.

### Phase 2: Create Shared SDK Text Core

- Add `crates/sdk/src/controls/text/`.
- Port the minimal reusable subset from `gpui-component/src/input`:
  `mode`, `movement`, `selection`, clipboard helpers, caret/focus behavior.
- Remove app-specific dependencies while porting.
- Keep APIs small and SDK-oriented.

### Phase 3: Rebuild TextField On The Shared Core

- Move `TextField` to the shared state first.
- Preserve existing SDK public API where possible.
- Preserve current gallery page.
- Verify:
  selection, select-all, copy, cut, paste, submit, mouse caret placement,
  double-click word select, triple-click all/select-line equivalent for
  single-line input.

### Phase 4: Rebuild TextArea On The Shared Core

- Move `TextArea` to the same shared state with multiline mode.
- Add real multiline movement.
- Add shared shortcut handling.
- Add pointer-based caret placement and selection.
- Add proper paste behavior and selection replacement.

### Phase 5: Replace Temporary Rendering Paths

- Remove string-spliced caret rendering.
- Replace ad hoc hit targets with shared geometry-backed rendering data.
- Keep current SDK theming and focus-ring approach.

### Phase 6: Test And Gallery Hardening

- Add unit tests around shared text behavior.
- Add control tests for field vs area policy differences.
- Extend gallery examples to show selected text, multiline editing, and invalid
  states with real behavior.

## 12. Merge Strategy

The merge should be extraction-first, not copy-first.

Recommended method:

1. Identify the minimum reusable logic in `gpui-component/src/input`.
2. Port that logic into SDK-owned shared modules with simpler names and fewer
   dependencies.
3. Rebind `TextField` to the new shared core.
4. Rebind `TextArea` to the same shared core.
5. Remove obsolete duplicated logic from the current control files.

This is better than copying the entire upstream `Input` component because:

- the upstream component contains code-editor concerns the SDK does not want,
- the SDK already has its own control/template/theme boundaries,
- a direct copy would import too much policy and too many dependencies.

## 13. Risks

Main risks:

- over-porting code editor features and making the SDK input stack too large,
- under-porting geometry logic and ending up with another partial editor,
- keeping too much of the current per-control mutation logic and reintroducing
  divergence,
- breaking current gallery visuals while swapping the behavior layer.

Mitigations:

- port the shared engine in narrow slices,
- keep the public control shell stable,
- test selection and clipboard behavior before visual refinement,
- treat `TextArea` parity as a mandatory acceptance criterion, not a follow-up.

## 14. Immediate Next Step

The next implementation task should be:

- create `crates/sdk/src/controls/text/`,
- port the minimal shared state and mode model from
  `gpui-component/src/input`,
- wire `TextField` onto it first.

Do not continue polishing the current `TextArea` control in isolation.

## 15. Recommendation Summary

We should not abandon all current work.

We should abandon the current text-editing internals and replace them with a
shared SDK text core derived from `gpui-component/src/input`.

That gives the right reset boundary:

- keep the SDK shell,
- replace the behavior engine,
- merge over the minimal reusable upstream architecture,
- then rebind `TextField` and `TextArea` as policy variants of one editor.
