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

## Current Direction
The pilot work clarified the naming direction we should standardize going forward:

1. Builder-level affordances should use `with_template_modifier(...)` and, for compound controls, `with_item_template_modifier(...)` / `with_panel_template_modifier(...)` where those seams exist.
2. Concrete template values should use `.with_modifier(...)`.
3. Wrapper-style helper functions such as `*_template_with_modifier(...)` should not be the preferred public story. Where possible, they should stay internal implementation details behind the builder or concrete template APIs.
4. Public wrapper types like `Modified*Template` should generally stay private unless callers truly need to name them.

This is not just style trivia. If modifiers are the default tweak path, the API has to make that path feel native rather than exceptional.

## Interface Policy
This project is still new enough that interface quality matters more than preserving old seams:

1. We do **not** need to preserve backward compatibility with awkward or transitional modifier APIs just because they existed earlier in the rollout.
2. If a wrapper-style or adapter-style API conflicts with a cleaner, more consistent builder-first interface, prefer the cleaner interface and update call sites.
3. The goal is a release-quality public surface with consistent naming and ownership, not a permanent accumulation of rollout-era compatibility shims.

## Completed So Far
The recent pilot work established a clearer baseline that is good enough to carry forward into the next families:

- `ButtonBuilder`, `SelectorBuilder`, `AccordionBuilder`, `ListViewBuilder`, and `ControlGroupBuilder` expose `with_template_modifier(...)`.
- `SelectionPanelBuilder`, `ComboBoxBuilder`, and `SearchSelectorBuilder` now also expose first-class builder modifier affordances on their main template seams.
- Concrete template values in the active pilot families expose `.with_modifier(...)`.
- `SelectorTemplate` now has a real top-level trigger-shell modifier seam rather than forcing the Theme Studio case into a theme override.
- The selector-family pressed-preview/list-shell bug was repaired in the shared selector-family list templates rather than patched only in the preview host. `Selector`, `ComboBox`, and `SearchSelector` now share the corrected padded-width behavior.
- The autocomplete popup theming bug was also fixed in the shared control path by routing popup styling through the builder/model `popup_look_provider` seam instead of a hardcoded default look.
- Public wrapper-shaped helper names have been reduced in the newer pilot families, and public `Modified*Template` wrapper types have been pushed out of the preferred API story where possible.
- `SelectionPanel` is no longer a builder-surface gap in the selector family, and the Gallery modifier examples now route through the builder-first seams rather than public helper wrapper adapters.

This is not the end-state for the whole SDK, but it is enough to establish the naming standard and move on from the initial pilot.

## Current Code State
As the code stands now:

- `SelectorBuilder` exposes the top-level `with_template_modifier(...)` affordance.
- `SelectionPanelBuilder` exposes `with_template_modifier(...)` and `with_item_template_modifier(...)`.
- `ComboBoxBuilder` exposes `with_template_modifier(...)`, `with_panel_template_modifier(...)`, and `with_item_template_modifier(...)`.
- `SearchSelectorBuilder` exposes `with_template_modifier(...)`, `with_panel_template_modifier(...)`, and `with_item_template_modifier(...)`.
- `AutocompleteTextBoxBuilder` now exposes `with_template_modifier(...)`; popup-look theming remains a separate seam rather than the only narrow customization path.

So the naming direction is effectively settled, and the selector family is now largely normalized at the builder/API level.

## Problem
The current SDK mixes several incompatible patterns:

- Some control families use the shared `ControlTemplate<T, M>` / `TemplateWithModifiers<M>` seam.
- Some control families define their own modifier wrapper type.
- Some control families have modifier support internally but do not expose it through the builder.
- Many control families expose only `.template(...)`, forcing callers to replace the whole template even for minor tweaks.
- Compound controls are especially inconsistent: trigger shell, popup panel, item row, and list shell seams vary from family to family.

This created repeated friction in design and implementation reviews:

- "Why is this customization so hard?"
- "Why didn't you use a modifier?"
- "Why did this need a custom theme wrapper?"

The answer changed per control family, which is why the pilot work focused first on locking the vocabulary and the preferred public path.

## Concrete Example: Theme Studio Selector
The recent Theme Studio theme chooser change is a good example of the current gap.

### Change Requested
In `apps/theme-studio/src/studio/theme_sidebar/mod.rs`, the theme selector needed two tweaks:

1. Show a custom selected-item/dropdown item layout with four fixed-size swatches.
2. Make the closed selector trigger background transparent.

### What Worked Cleanly
The swatch row was easy to do with `Selector::with_item_template(...)` because selector item content already has a dedicated seam.

### What Did Not Work Cleanly Before The Pilot
The transparent trigger background could not be done with an item template modifier because the trigger chrome is rendered by `SelectorTemplate`, not by the item template. The actual trigger shell lives in `crates/sdk/src/controls/selector/template.rs`, where the outer trigger `div` applies:

- `bg(look.trigger_background)`
- `border_color(look.trigger_border)`
- padding, radius, layout, icon placement

Before the pilot, `SelectorTemplate` had no useful top-level modifier seam, so the only narrow app-local option was to derive the selector theme and override `trigger_background` while preserving the shared template.

That solution was valid for a more significant tweak, but it was the wrong ergonomics for this size of change.

### What Works After The Pilot
The selector trigger shell now has a real template modifier seam, and the Theme Studio theme chooser can express the transparent trigger tweak through the template path itself:

- custom item content via `Selector::with_item_template(...)`
- trigger-shell tweak via `ThemedSelectorTemplate::with_modifier(...)`

That is much closer to the intended customization ladder: item content stays on the item seam, while trigger chrome stays on the selector template seam.

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
- `8` with built-in modifier support on the template itself
- `17` currently missing top-level modifier support

### Template Families With Built-In Modifier Support
These already support modifiers at the template layer:

- `AutocompleteTextBoxTemplate`
- `ButtonTemplate`
- `AccordionTemplate`
- `ListViewTemplate`
- `SelectionPanelTemplate`
- `SelectorTemplate`
- `ComboBoxTemplate`
- `SearchSelectorTemplate`

### Additional Family With Modifier Support Outside the Top-Level Trait Count
`ControlGroupTemplate` is closure-based rather than a named top-level trait, but it supports modifiers behind a builder-level `with_template_modifier(...)`.

### Builder-Level Exposure
This is a second problem separate from template support itself.

Direct builder-level `with_template_modifier(...)` entrypoints currently exist for:

- `AccordionBuilder`
- `ButtonBuilder`
- `AutocompleteTextBoxBuilder`
- `ControlGroupBuilder`
- `ListViewBuilder`
- `SelectionPanelBuilder`
- `SelectorBuilder`
- `ComboBoxBuilder`
- `SearchSelectorBuilder`

This is materially better than the starting point. The main remaining gaps are now outside the normalized selector/reference families and the autocomplete trigger shell.

## Findings
### 1. Modifier support is still not a universal SDK contract
The shared seam exists, and the pilot families are in better shape, but most control families still do not use it.

### 2. Builder ergonomics remain the main adoption risk
This is improving, but it is still the main rollout risk. The standard should remain "builder first, concrete template second, wrapper machinery hidden."

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

- [x] `ButtonTemplate` / `ButtonBuilder`
  Reason: simplest case, already demonstrated in Gallery prototype/custom button panes, validates modifier-first ergonomics for a single-root control.
- [x] `SelectionPanel`
  Reason: already has panel/item modifier precedent and is the best reference for compound modifier layering.
- [x] `Selector`
  Reason: highest-value missing seam and the clearest example of current friction between item customization and trigger-shell customization.

Current pilot status:

- `ButtonBuilder` now exposes `with_template_modifier(...)`.
- `SelectorBuilder` now exposes `with_template_modifier(...)`.
- `AccordionBuilder` now exposes `with_template_modifier(...)`.
- `ListViewBuilder` and `ControlGroupBuilder` already fit the preferred builder-level pattern and remain the reference for earlier adoption.
- `SelectionPanelBuilder` now exposes `with_template_modifier(...)` and `with_item_template_modifier(...)`.
- `ComboBoxBuilder` now exposes `with_template_modifier(...)`, `with_panel_template_modifier(...)`, and `with_item_template_modifier(...)`.
- `SearchSelectorBuilder` now exposes `with_template_modifier(...)`, `with_panel_template_modifier(...)`, and `with_item_template_modifier(...)`.
- Public wrapper-shaped helper names have been reduced in the newer pilot families so the preferred API reads more like a native contract and less like an adapter.
- Public `Modified*Template` wrapper types are no longer part of the preferred public story in the newer pilot families.
- Gallery selector-control examples now route through the builder-first seams for parameterized modifier demos instead of depending on public helper wrapper adapters.
- The selector-family preview/rendering bug that briefly distracted this work is fixed and should not block the API rollout anymore.
- The selector-family API normalization phase is effectively complete; remaining work is broader rollout beyond this family.

Pilot success criteria:

- [x] A small tweak to each completed pilot control can be expressed through a modifier without replacing the full template.
- [x] A more significant tweak can still be expressed by deriving/replacing the template.
- [x] A broader appearance change can still be expressed by deriving/replacing theme/look inputs.
- [x] Builder ergonomics make the modifier path obvious for the completed pilot families.
- [~] Tests cover modifier composition and default-path non-regression for the pilot families.
  Status: button/selector builder wrapping plus selector-family builder composition coverage now exist; deeper render-level non-regression coverage is still worth adding if these seams keep expanding.

Pilot conclusion:

- The naming direction is now clear enough to treat as the standard for new work.
- The selector-family normalization pass is complete enough to stop debating names and move on.
- Remaining selector-family work is now mostly render-level regression coverage, not builder-surface cleanup.
- Where cleanup choices conflict with backward compatibility, favor the cleaner release-quality interface and migrate local call sites.

### Next Step
The next concrete phase should move beyond the selector family:

1. Treat `Selector`, `SelectionPanel`, `ComboBox`, and `SearchSelector` as the reference compound-control families for modifier naming.
2. Use that selector-family baseline to drive the next rollout batch in non-selector controls that still only expose `.template(...)`.
3. Keep the public goal narrow: consistent builder ergonomics and seam naming across the remaining families, without reopening the broader naming debate.

### Phase 1: Add Top-Level Template Modifier Support Where Missing
These top-level control templates should support a root-level modifier seam.

- [x] `AutocompleteTextBoxTemplate`
  Path: `crates/sdk/src/controls/autocomplete/template.rs`
- [x] `CardTemplate`
  Path: `crates/sdk/src/controls/card/template.rs`
- [x] `ComboBoxTemplate`
  Path: `crates/sdk/src/controls/combobox/template.rs`
- [x] `ContextMenuTemplate`
  Path: `crates/sdk/src/controls/context_menu/template.rs`
- [x] `DialogTemplate`
  Path: `crates/sdk/src/controls/overlay_window/template.rs`
- [x] `DockSplitterTemplate`
  Path: `crates/sdk/src/controls/dock_splitter/template.rs`
- [x] `FloatingMenuTemplate`
  Path: `crates/sdk/src/controls/floating_menu/template.rs`
- [x] `NavigationSidebarTemplate`
  Path: `crates/sdk/src/controls/navigation_sidebar/template.rs`
- [x] `PagerTemplate`
  Path: `crates/sdk/src/controls/pager/template.rs`
- [x] `PopupMenuTemplate`
  Path: `crates/sdk/src/controls/popup_menu/template.rs`
- [x] `ProgressTemplate`
  Path: `crates/sdk/src/controls/progress/template.rs`
- [x] `ResizablePanelsTemplate`
  Path: `crates/sdk/src/controls/resizable_panels/template.rs`
- [x] `ScrollbarTemplate`
  Path: `crates/sdk/src/controls/scrollbar/template.rs`
- [x] `SearchSelectorTemplate`
  Path: `crates/sdk/src/controls/search_selector/template.rs`
- [x] `SelectorTemplate`
  Path: `crates/sdk/src/controls/selector/template.rs`
- [x] `SliderTemplate`
  Path: `crates/sdk/src/controls/slider/template/mod.rs`
- [x] `SplitViewTemplate`
  Path: `crates/sdk/src/controls/split_view/template.rs`
- [x] `TabsNavigationTemplate`
  Path: `crates/sdk/src/controls/tabs_navigation/template.rs`
- [x] `TextAreaTemplate`
  Path: `crates/sdk/src/controls/textarea/template.rs`
- [x] `TextFieldTemplate`
  Path: `crates/sdk/src/controls/textfield/template.rs`
- [x] `TreeViewTemplate`
  Path: `crates/sdk/src/controls/tree_view/template.rs`

### Phase 2: Add Builder-Level Modifier Entry Points Where the Template Already Supports or Will Support Modifiers
The template seam is not enough if callers still have to hand-roll wrappers.

- [x] `ButtonBuilder`
  Path: `crates/sdk/src/controls/command/button/model.rs`
  Reason: `DefaultButtonTemplate` already supports modifiers, but callers only get `.template(...)`.
- [x] `AccordionBuilder`
  Path: `crates/sdk/src/controls/accordion/model.rs`
  Reason: `AccordionTemplate` already supported modifiers internally, and the builder should expose the standard path directly.
- [x] `SelectionPanel` builder/model surface
  Path: `crates/sdk/src/controls/selection_panel/model.rs`
  Reason: builder-level top-level and item modifier affordances now exist and establish the reference pattern for a compound panel control.
- [x] `SelectorBuilder`
  Path: `crates/sdk/src/controls/selector/model.rs`
  Reason: selector trigger-shell tweaks should be expressible from the normal builder path as well as from concrete template construction.
- [x] Selector-family follow-up refactor
  Paths:
  - `crates/sdk/src/controls/selection_panel/model.rs`
  - `crates/sdk/src/controls/combobox/model.rs`
  - `crates/sdk/src/controls/search_selector/model.rs`
  Reason: the family now shares the normalized builder naming pattern: top-level `with_template_modifier(...)` plus item/panel modifier affordances where those seams exist.
- [~] Every Phase 1 family should expose a direct builder-level modifier helper once template support lands.
  Status: top-level template support is now broadly in place across the SDK control set; the main remaining follow-up is whether helper-only surfaces without builders need any additional public affordances beyond their template helpers.

### Phase 3: Add Consistent Modifier Support to Important Subtemplate Layers
These are not always the first priority, but they are frequent tweak points and should follow the same philosophy.

- [x] `AutocompleteItemsTemplate`
  Path: `crates/sdk/src/controls/autocomplete/template.rs`
- [x] `ComboBoxItemsTemplate`
  Path: `crates/sdk/src/controls/combobox/items_template.rs`
- [x] `ComboBoxPanelTemplate`
  Path: `crates/sdk/src/controls/combobox/panel_template.rs`
- [x] `SearchSelectorItemsTemplate`
  Path: `crates/sdk/src/controls/search_selector/template.rs`
- [x] `SearchSelectorPanelTemplate`
  Path: `crates/sdk/src/controls/search_selector/panel_template.rs`
- [x] `SelectorItemsTemplate`
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
  Note: this is still useful reference functionality, but the public API should eventually favor builder or concrete-template affordances over wrapper-oriented helper names.
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

Current status:

- The public naming direction is now stable enough to follow.
- Internal implementation style is still mixed, but that is acceptable for now as long as the public API stays coherent.
- Future cleanup should prioritize public consistency over forcing all families onto one internal helper immediately.

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
