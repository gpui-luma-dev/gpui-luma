# Issue #0: Make Template Modifiers the Default SDK Customization Path

## Description
The SDK has a shared template modifier seam in `crates/sdk/src/controls/template.rs`, but adoption is inconsistent across control families. In practice this means users often hear "just use a modifier" even when the relevant control template does not support modifiers, or supports them only through an internal helper that is not exposed on the builder.

That breaks one of the intended SDK ergonomics:

1. First reach for a modifier when the user wants to tweak a control.
2. If the change is bigger than a tweak, derive or replace the template.
3. If the change is still broader, derive or replace the appearance/theme/look.

Today that hierarchy is not reliable. For many controls the first step is unavailable, so callers get pushed directly into full template replacement or look/theme overrides.

## Intended Customization Model
We should make the customization ladder explicit and consistent:

1. **Modifier**: Small structural or chrome tweaks to the existing template root.
2. **Derived template**: Larger rendering changes that still preserve the control contract.
3. **Derived theme/look/appearance**: Changes to visual token resolution, chrome policy, or family-wide defaults.

The important point is not "modifiers replace all other customization." The point is that modifiers should be the first and easiest place to try when the desired change is a tweak rather than a rewrite.

## Problem
The current SDK mixes several incompatible patterns:

- Some control families use the shared `ControlTemplate<T, M>` / `TemplateWithModifiers<M>` seam.
- Some control families define their own modifier wrapper type.
- Some control families have modifier support internally but do not expose it through the builder.
- Many control families expose only `.template(...)`, forcing callers to replace the whole template even for minor tweaks.
- Compound controls are especially inconsistent: trigger shell, popup panel, item row, and list shell seams vary from family to family.

This creates repeated friction in design and implementation reviews:

- "Why is this customization so hard?"
- "Why didn't you use a modifier?"
- "Why did this need a custom theme wrapper?"

The answer changes per control family, which means the SDK currently lacks a coherent customization contract.

## Concrete Example: Theme Studio Selector
The recent Theme Studio theme chooser change is a good example of the current gap.

### Change Requested
In `apps/theme-studio/src/studio/theme_sidebar/mod.rs`, the theme selector needed two tweaks:

1. Show a custom selected-item/dropdown item layout with four fixed-size swatches.
2. Make the closed selector trigger background transparent.

### What Worked Cleanly
The swatch row was easy to do with `Selector::with_item_template(...)` because selector item content already has a dedicated seam.

### What Did Not Work Cleanly
The transparent trigger background could not be done with an item template modifier because the trigger chrome is rendered by `SelectorTemplate`, not by the item template. The actual trigger shell lives in `crates/sdk/src/controls/selector/template.rs`, where the outer trigger `div` applies:

- `bg(look.trigger_background)`
- `border_color(look.trigger_border)`
- padding, radius, layout, icon placement

Since `SelectorTemplate` currently has no modifier seam, the only narrow app-local option was to derive the selector theme and override `trigger_background` while preserving the shared template.

That solution was valid for a more significant tweak, but this is exactly the kind of request that should have been possible through a modifier if the selector template had adopted the intended seam.

## Audit Summary
### Top-Level Template Seams
Reviewed top-level SDK template traits:

- `AccordionTemplate`
- `AutocompleteTextBoxTemplate`
- `ButtonTemplate`
- `CardTemplate`
- `ComboBoxTemplate`
- `ContextMenuTemplate`
- `DialogTemplate`
- `DockSplitterTemplate`
- `FloatingMenuTemplate`
- `ListViewTemplate`
- `NavigationSidebarTemplate`
- `PagerTemplate`
- `PopupMenuTemplate`
- `ProgressTemplate`
- `ResizablePanelsTemplate`
- `ScrollbarTemplate`
- `SearchSelectorTemplate`
- `SelectionPanelTemplate`
- `SelectorTemplate`
- `SliderTemplate`
- `SplitViewTemplate`
- `TabsNavigationTemplate`
- `TextAreaTemplate`
- `TextFieldTemplate`
- `TreeViewTemplate`

Count:

- `25` top-level template seams reviewed
- `4` with built-in modifier support on the template itself
- `21` currently missing top-level modifier support

### Template Families With Built-In Modifier Support
These already support modifiers at the template layer:

- `ButtonTemplate`
- `AccordionTemplate`
- `ListViewTemplate`
- `SelectionPanelTemplate`

### Additional Family With Modifier Support Outside the Top-Level Trait Count
`ControlGroupTemplate` is closure-based rather than a named top-level trait, but it does support modifiers via `template_with_modifier(...)` and a builder-level `with_template_modifier(...)`.

### Builder-Level Exposure
This is a second problem separate from template support itself.

Direct builder-level `with_template_modifier(...)` entrypoints currently exist only for:

- `ControlGroupBuilder`
- `ListViewBuilder`

That means even where a template family does support modifiers, the "easy path" is often still not surfaced to normal callers.

## Findings
### 1. Modifier support is not a universal SDK contract
The shared seam exists, but most control families do not use it.

### 2. Builder ergonomics are more inconsistent than template internals
Even some families that support modifiers internally still force callers into manual template wrapping or full replacement.

### 3. Compound controls are the highest-friction area
Selector, search selector, combobox, popup/context menus, tree/list/navigation surfaces all have multiple render layers. These are exactly the places where a small chrome tweak is common, and exactly the places where modifier seams are least consistent.

### 4. `look-shadcn` mostly inherits the inconsistency
The look crate usually just supplies the SDK template type already chosen by the SDK. It does not impose a consistent modifier policy on top of the SDK. There are isolated cases, like toggle templates, where `look-shadcn` itself uses a modifier internally, but that is not the dominant pattern.

## Work List
The work should not start as a single widescale refactor. It should begin with a small pilot batch that validates the seam design, builder ergonomics, and testing strategy before the rest of the SDK is touched.

### Preconditions Before Broad Rollout
Before treating this doc as an implementation-ready broad refactor spec, we should lock down a few missing contracts:

- [ ] Define the canonical modifier contract for a top-level template seam.
  Question: does the modifier always receive the outermost meaningful `Stateful<Div>` root?
- [ ] Define the canonical modifier contract for compound subtemplate seams.
  Question: when should callers get a trigger/container modifier vs an item-content modifier vs a panel/list-shell modifier?
- [ ] Define the minimum model richness required for modifier-friendly seams.
  Question: which state must always be available for conditional styling?
- [ ] Define the builder API policy.
  Question: when should a family expose `with_template_modifier(...)`, `with_item_template_modifier(...)`, `with_panel_template_modifier(...)`, or equivalent?
- [ ] Define the composition order contract.
  Question: how do modifiers compose with custom templates, look-owned templates, and theme/look overrides?

### Pilot Rollout
Recommended first batch:

- [ ] `ButtonTemplate` / `ButtonBuilder`
  Reason: simplest case, already demonstrated in Gallery prototype/custom button panes, validates modifier-first ergonomics for a single-root control.
- [ ] `SelectionPanel`
  Reason: already has panel/item modifier precedent and is the best reference for compound modifier layering.
- [ ] `Selector`
  Reason: highest-value missing seam and the clearest example of current friction between item customization and trigger-shell customization.

Pilot success criteria:

- [ ] A small tweak to each pilot control can be expressed through a modifier without replacing the full template.
- [ ] A more significant tweak can still be expressed by deriving/replacing the template.
- [ ] A broader appearance change can still be expressed by deriving/replacing theme/look inputs.
- [ ] Builder ergonomics make the modifier path obvious.
- [ ] Tests cover modifier composition and default-path non-regression for the pilot families.

After the pilot batch lands cleanly, continue to the larger template inventory below.

### Phase 1: Add Top-Level Template Modifier Support Where Missing
These top-level control templates should support a root-level modifier seam.

- [ ] `AutocompleteTextBoxTemplate`
  Path: `crates/sdk/src/controls/autocomplete/template.rs`
- [ ] `CardTemplate`
  Path: `crates/sdk/src/controls/card/template.rs`
- [ ] `ComboBoxTemplate`
  Path: `crates/sdk/src/controls/combobox/template.rs`
- [ ] `ContextMenuTemplate`
  Path: `crates/sdk/src/controls/context_menu/template.rs`
- [ ] `DialogTemplate`
  Path: `crates/sdk/src/controls/overlay_window/template.rs`
- [ ] `DockSplitterTemplate`
  Path: `crates/sdk/src/controls/dock_splitter/template.rs`
- [ ] `FloatingMenuTemplate`
  Path: `crates/sdk/src/controls/floating_menu/template.rs`
- [ ] `NavigationSidebarTemplate`
  Path: `crates/sdk/src/controls/navigation_sidebar/template.rs`
- [ ] `PagerTemplate`
  Path: `crates/sdk/src/controls/pager/template.rs`
- [ ] `PopupMenuTemplate`
  Path: `crates/sdk/src/controls/popup_menu/template.rs`
- [ ] `ProgressTemplate`
  Path: `crates/sdk/src/controls/progress/template.rs`
- [ ] `ResizablePanelsTemplate`
  Path: `crates/sdk/src/controls/resizable_panels/template.rs`
- [ ] `ScrollbarTemplate`
  Path: `crates/sdk/src/controls/scrollbar/template.rs`
- [ ] `SearchSelectorTemplate`
  Path: `crates/sdk/src/controls/search_selector/template.rs`
- [ ] `SelectorTemplate`
  Path: `crates/sdk/src/controls/selector/template.rs`
- [ ] `SliderTemplate`
  Path: `crates/sdk/src/controls/slider/template/mod.rs`
- [ ] `SplitViewTemplate`
  Path: `crates/sdk/src/controls/split_view/template.rs`
- [ ] `TabsNavigationTemplate`
  Path: `crates/sdk/src/controls/tabs_navigation/template.rs`
- [ ] `TextAreaTemplate`
  Path: `crates/sdk/src/controls/textarea/template.rs`
- [ ] `TextFieldTemplate`
  Path: `crates/sdk/src/controls/textfield/template.rs`
- [ ] `TreeViewTemplate`
  Path: `crates/sdk/src/controls/tree_view/template.rs`

### Phase 2: Add Builder-Level Modifier Entry Points Where the Template Already Supports or Will Support Modifiers
The template seam is not enough if callers still have to hand-roll wrappers.

- [ ] `ButtonBuilder`
  Path: `crates/sdk/src/controls/command/button/model.rs`
  Reason: `DefaultButtonTemplate` already supports modifiers, but callers only get `.template(...)`.
- [ ] `AccordionBuilder`
  Path: `crates/sdk/src/controls/accordion/model.rs`
  Reason: `AccordionTemplate` already has `accordion_template_with_modifier(...)`, but builder ergonomics do not expose it.
- [ ] `SelectionPanel` builder/model surface
  Path: `crates/sdk/src/controls/selection_panel/model.rs`
  Reason: panel and item template modifier helpers exist, but there is no direct builder affordance mirroring `ControlGroupBuilder` / `ListViewBuilder`.
- [ ] Every Phase 1 family should expose a direct builder-level modifier helper once template support lands.

### Phase 3: Add Consistent Modifier Support to Important Subtemplate Layers
These are not always the first priority, but they are frequent tweak points and should follow the same philosophy.

- [ ] `AutocompleteItemsTemplate`
  Path: `crates/sdk/src/controls/autocomplete/template.rs`
- [ ] `ComboBoxItemsTemplate`
  Path: `crates/sdk/src/controls/combobox/items_template.rs`
- [ ] `ComboBoxPanelTemplate`
  Path: `crates/sdk/src/controls/combobox/panel_template.rs`
- [ ] `SearchSelectorItemsTemplate`
  Path: `crates/sdk/src/controls/search_selector/template.rs`
- [ ] `SearchSelectorPanelTemplate`
  Path: `crates/sdk/src/controls/search_selector/panel_template.rs`
- [ ] `SelectorItemsTemplate`
  Path: `crates/sdk/src/controls/selector_panel/items_template.rs`

For compound controls, modifier support is only useful if the render model is rich enough to drive conditional styling without forcing full template replacement. As these seams are added, verify that the models passed to modifiers expose the relevant interaction and composition state for that layer.

Examples:

- `SelectorRenderModel` should remain sufficient for trigger-shell decisions such as open/closed, selected item, active path, enabled, focused, and interaction state.
- `ComboBoxRenderModel` and `SearchSelectorRenderModel` should expose enough trigger and popup state to support conditional shell tweaks.
- Panel/item-layer modifier models should expose hover, pressed, selected, active, and focus-visible state where those conditions affect realistic styling tweaks.

These already have some modifier support and should be used as reference patterns:

- [ ] `SelectionPanelTemplate`
  Path: `crates/sdk/src/controls/selection_panel/template.rs`
  Note: keep this as the model for panel-root modifier wrapping.
- [ ] `SelectionPanelItemTemplate`
  Path: `crates/sdk/src/controls/selection_panel/item_template.rs`
  Note: keep this as the model for item-content modifier wrapping.
- [ ] `ListViewColumnCellTemplate`
  Path: `crates/sdk/src/controls/list_view/column_template.rs`
  Note: keep this as the model for cell/item modifier wrapping.

### Phase 4: Normalize Internal Pattern Choice
The repo currently has at least three modifier implementation styles:

- shared `ControlTemplate<T, M>`
- local `Modified*Template` wrapper structs
- closure-based template wrappers

We do not necessarily need a single implementation immediately, but we should pick a preferred default for new work.

Recommended default:

1. Use `ControlTemplate<T, M>` for simple themed single-root controls.
2. Use explicit `Modified*Template` wrappers for compound templates that cannot easily fit the generic helper.
3. Always expose a builder-level `with_template_modifier(...)` when the family supports template modifiers.

### Phase 4A: Wrapper Flattening
Wrapping existing templates via `Modified*Template` is still the right default pattern for compound controls because it preserves the base template logic and keeps modifier composition explicit.

However, repeated wrapping can create avoidable layering when helpers blindly wrap a template that is already a `Modified*Template`.

Recommended follow-up:

- [ ] Where practical, make `*_template_with_modifier(...)` helpers flatten existing modified wrappers instead of always nesting another wrapper layer.
- [ ] Prefer internal flattening through explicit wrapper-owned APIs or representation changes rather than trait-object downcasting as the primary design.

Important constraint:

- The current template traits are plain trait objects and are not consistently designed for safe downcasting.
- Adding `Any`/`as_any()`-style plumbing across every template family just to append modifiers would widen the abstraction surface and add more trait boilerplate.
- So the useful idea here is **flatten nested modifier wrappers when the implementation makes that easy**, not **standardize on downcast-heavy wrapper mutation**.

## Suggested Upgrade Rules
When updating a control family, use these rules:

1. A modifier should receive the outermost meaningful root for the template seam.
2. A modifier should be able to tweak chrome without requiring full template replacement.
3. A larger structural change should still be able to derive or replace the full template.
4. Theme/look derivation should remain the right tool when the change is about token resolution or family-wide chrome policy, not a one-off tweak.
5. Compound controls should expose separate seams deliberately:
   - trigger/container modifier
   - item-content modifier when item rendering is customizable
   - panel/list-shell modifier when popup/list chrome is customizable
6. Modifier models for compound controls should expose enough state that callers can write conditional tweaks without forking the entire template.

## Testing Strategy
Widescale template refactors are structurally risky and are not well-covered by typechecking alone. This work should target a repeatable unit/regression test pattern rather than relying only on ad hoc manual verification.

Recommended coverage:

- [ ] Template seam tests
  Verify that a modifier can mutate a known root property for the upgraded template family.
- [ ] Builder wiring tests
  Verify that builder-level modifier APIs actually propagate through builder -> control -> render template.
- [ ] Composition order tests
  Verify the intended ordering between base template, modifier(s), custom template replacement, and look/theme overrides where applicable.
- [ ] State-sensitive modifier tests
  Verify that modifiers can respond correctly to states such as focused, hovered, pressed, open, selected, active, and disabled.
- [ ] Default behavior non-regression tests
  Verify that controls with no modifiers still render and behave the same way as before.

Suggested pilot-first test targets:

- [ ] `ButtonTemplate`
- [ ] `SelectionPanelTemplate`
- [ ] `SelectorTemplate`

## Acceptance Criteria
- For every user-facing control family, "tweak the control" should have an obvious modifier-first path.
- Small chrome/layout changes should not require full template replacement.
- Larger changes should still have a straightforward derive/replace-template path.
- Look/theme derivation should remain available for changes that are broader than a modifier.
- Builder ergonomics should be consistent enough that reviewers can safely say "use a modifier" without having to inspect the control family first.
- Pilot families should establish a reusable modifier-testing pattern before the broader rollout proceeds.

## Notes
This is not a request to eliminate custom templates or look/theme derivation. The goal is to make the customization ladder reliable:

- **modifier first**
- **derived template second**
- **derived appearance/theme/look third**

The selector case is a good example of why this matters. The swatch row customization was a normal template-content customization. The transparent trigger background was a small tweak that should have had a modifier path, but instead needed a derived selector theme because the trigger template seam skipped modifier support.
