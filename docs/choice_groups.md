# ChoiceGroup Implementation Work Plan

## Scope
- Introduce a new `ChoiceGroup` control to replace both `radio_group` and `toggle_group`.
- Follow the SDK architecture in `docs/control-design.md` and implementation guidance in `docs/control-guidelines.md`:
  - lookless logic core
  - model/presenter/template separation
  - content presenter (`ControlContent<M>`)
  - template modifier pipeline
- First milestone: support icon-button toolbar usage end-to-end.
- Do not refactor other controls in this phase unless required for `ChoiceGroup` integration.

## Milestones
1. **M1: Icon Toolbar ChoiceGroup (Priority)**
   - `ChoiceGroup` works for icon-button toolbar interaction.
   - Single-select behavior first (toolbar mode).
   - Themed template + presenter content support.
2. **M2: Full Selection Modes**
   - Add multi-select semantics.
   - Reach parity for current `toggle_group` and `radio_group` use patterns.
3. **M3: Migration + Removal**
   - Migrate call sites from legacy controls.
   - Deprecate and remove `radio_group` and `toggle_group`.

## State Model Contract (Swift-ier + Reactive)

`ChoiceGroup` supports both **Initial State** and **Managed State**:

- **Initial / Unmanaged**
  - `.selected("bold")` sets only the initial selected item at spawn time.
  - After spawn, internal group state owns selection unless externally updated through API.
- **Managed / Source of Truth**
  - If a parent model/binding is supplied, the parent owns selection.
  - Group emits `Change` events, and parent updates the model.
  - Group re-renders from model state (single source of truth).

### Required behavior
- `.selected(...)` is treated as an initial default when managed state is present.
- Managed state value always wins on render.
- `Change` event is always emitted from user interaction; parent decides whether/how to commit.

## Proposed API Sample (M1: Icon Toolbar First)

The first target is an icon-toolbar style group. This sample reflects the intended `ChoiceGroup` API shape and replaces old toolbar-style grouping.

```rust
ChoiceGroup::single("editor-toolbar")
    .items(vec![
        ChoiceItem::new("bold", EditAction::Bold),
        ChoiceItem::new("italic", EditAction::Italic),
        ChoiceItem::new("underline", EditAction::Underline),
        ChoiceItem::new("code", EditAction::Code),
    ])
    .selected("bold") // initial value if unmanaged; default fallback if managed
    .layout(ChoiceGroupLayout::Horizontal)
    .template(theme.choice_group_template())
    .content(|item, _group_model| {
        let icon = match item.value {
            EditAction::Bold => LucideIcon::Bold,
            EditAction::Italic => LucideIcon::Italic,
            EditAction::Underline => LucideIcon::Underline,
            EditAction::Code => LucideIcon::Code,
        };

        div()
            .font_family("lucide")
            .child(char::from(icon).to_string())
            .into_any_element()
    })
    .spawn(cx);
```

Notes:
- Use `.template(...)` (not `template_factory(...)`) to match `control_design2` style.
- Keep style overrides in the template modifier pipeline (`template.with_modifier(...)`) rather than group-level element mutation.
- Additional variants (`multiple`, vertical layout, checkbox/switch/tab visuals) are planned for M2.

## Visual Contract for Ghost/Icon Toolbar (Selected Must Be Obvious)

For toolbar/icon use, Ghost style can hide selected state unless explicitly mapped.

### Template requirement
`ChoiceGroupTemplate` must communicate selected state to child/item visuals with an explicit semantic signal:
- `item.state.selected == true` must map to **Active appearance** in toolbar mode.
- `item.state.active` remains focus/navigation state.
- Selected visual should persist even when not keyboard-focused.

### Implementation guidance
- Include both in render model:
  - `selected` (persistent selection)
  - `active/focus_visible` (interaction/focus)
- For Ghost-like variants, resolve appearance using role semantics, e.g.:
  - `role = ToolbarItem { selected: true }` => active background/border/fg
  - `role = ToolbarItem { selected: false }` => ghost resting appearance
- Ensure hover/pressed layers compose on top of selected baseline (not replacing it).

## Phase Plan

### Phase 1 — Control Skeleton
- Create `crates/sdk/src/controls/choice_group/`:
  - `mod.rs`
  - `model.rs`
  - `control.rs`
  - `template.rs`
- Export in `crates/sdk/src/controls/mod.rs`.
- Define core public types:
  - `ChoiceGroupControl`
  - `ChoiceGroupBuilder`
  - `ChoiceGroupEvent`
  - `ChoiceGroupSelectionMode` (initially `Single`; `Multiple` in M2)
  - `ChoiceGroupLayout` (`Horizontal`, `Vertical`)

### Phase 2 — Model + Logic Core
- Implement model state:
  - id
  - items (id/label/enabled + typed value as needed)
  - selected id(s)
  - active/focus item state
  - enabled/layout
  - template reference
  - state mode (`Unmanaged` or `Managed`)
- Implement interaction logic:
  - click/toggle selection
  - hover/pressed tracking
  - keyboard navigation (arrow keys, first/last, activation)
  - focus handling and notifications
- Emit `ChoiceGroupEvent::Change` with stable payload for toolbar usage and managed updates.

### Phase 3 — Template + Presenter Pipeline
- Implement `ChoiceGroupTemplate` in `control_design2` style.
- Add presenter API:
  - `.content(|item, model| -> IntoElement)`
- Add template API:
  - `.template(...)`
  - template-level `.with_modifier(...)` support via modifier pipeline
- Add layout-aware rendering in template:
  - horizontal for toolbar
  - vertical ready for later use cases
- Add selected->active appearance mapping contract for Ghost toolbar variant.

### Phase 4 — Icon Toolbar Milestone (M1)
- Build first integration using icon actions (bold/italic/underline/code).
- Use ghost/toolbar visual style through template/theme.
- Validate behavior:
  - single selection
  - pointer interaction
  - keyboard focus + activation
  - disabled item behavior
  - selected item remains visibly active in ghost style
  - managed vs unmanaged state behaves correctly
- Ensure presenter can render icon-only items cleanly.

### Phase 5 — Hardening
- Add tests:
  - selection transitions
  - disabled-state guards
  - keyboard navigation semantics
  - event payload correctness
  - managed-state precedence over `.selected(...)`
  - selected->active visual mapping hook in template model
- Run diagnostics/build and resolve errors.
- Document API usage snippet in docs/gallery notes.

## Acceptance Criteria (M1)
- `ChoiceGroup` exists and is exported from SDK controls.
- Icon-toolbar use case works with:
  - single selection
  - `content(...)` icon rendering
  - template styling/modifiers
  - keyboard + pointer interaction
  - clear selected visual in Ghost style (active appearance semantics)
- `.selected(...)` works as initial value in unmanaged mode and as fallback/default in managed mode.
- `Change` event supports single-source-of-truth parent updates.
- At least one gallery integration demonstrates replacement viability for toolbar scenario.
- No regression in unrelated controls.

## Risks / Open Questions
- Should M1 ship single-select only, or include multiple-select immediately?
- Final item identity/value strategy (string IDs vs generic typed values with stable IDs).
- Accessibility semantics for unified control (radio-like vs toolbar-toggle-like behavior).
- Migration order and temporary compatibility helpers (if any).

## Next Steps
1. Finalize M1 API surface (single-select toolbar-first + managed/unmanaged contract).
2. Implement skeleton + core logic.
3. Implement template/presenter integration with selected->active appearance mapping.
4. Integrate icon-toolbar example and validate.
5. Start parity planning for M2 (multi-select + broader migration).
