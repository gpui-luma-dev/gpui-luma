# Issue #0: Full Theme Mutability Fix

## Problem Statement

The repo now has a partial theme synchronization fix, but it is still split across two different behaviors:

1. **Mode flips on an existing look**
   - These are now handled reasonably well through the SDK-owned theme revision global.
   - Persistent controls such as `TextField`, `TextArea`, `Selector`, `ComboBox`, `Button`, `TabsNavigation`, `NavigationSidebar`, `Slider`, `Scrollbar`, `Progress`, `Accordion`, `TreeView`, and `ListView` observe a global revision and invalidate themselves when it changes.

2. **Full theme replacement**
   - These are still not truly mutable.
   - In Luma Studio, changing the active tweakcn theme or applying global color overrides still rebuilds demo panel internals instead of mutating the existing live theme state in place.

That means the current system is better than the original manual-notify / `refresh_demos` setup, but it is not yet a complete solution.

The remaining gap is:

- We can invalidate long-lived controls when the active theme **revision** changes.
- We cannot yet replace the underlying `ShadcnLook` theme data **in place** and have the entire tree naturally follow that update without rebuilding app-owned subtrees.

This is why the current behavior still feels inconsistent:

- **Gallery** mode toggles are close to correct because they mutate one live `ShadcnLook` and bump the SDK theme revision.
- **Luma Studio** mode toggles are improved for the same reason.
- **Luma Studio** full theme changes and override changes still reconstruct panel internals, because the app swaps to a new `Arc<ShadcnLook>` instead of mutating one shared live look.

## What Is Already Fixed

### 1. SDK-Owned Theme Global

The repo now has an SDK-owned invalidation mechanism in:

- `crates/sdk/src/theme/revision.rs`

This provides:

- `LumaThemeRevision`
- `LumaThemeSyncExt::bump_luma_theme_revision()`
- `observe_theme_revision(...)`

This design is correct in one important way: it keeps the SDK independent from `look-shadcn`. The SDK owns the invalidation channel, and concrete looks participate by triggering that channel.

### 2. Persistent Control Invalidation

Long-lived controls now observe the global revision and call `cx.notify()` when the theme revision changes.

The strongest version of this fix is in:

- `crates/sdk/src/controls/textfield/control.rs`
- `crates/sdk/src/controls/textarea/control.rs`

Those controls also clear cached layout state and bump a local render epoch so their cached GPUI element identities are not silently reused after a theme change.

Other persistent controls now at least observe the revision and invalidate:

- `command/button`
- `selector`
- `combobox`
- `search_selector`
- `navigation_sidebar`
- `tabs_navigation`
- `slider`
- `scrollbar`
- `progress`
- `accordion`
- `tree_view`
- `list_view`
- `control_group`

### 3. TextField / TextArea Border Mapping

The original textfield bug is now understood as a **token mapping** problem, not only a refresh problem.

The surface textfield/textarea outline was using the `input` token, which can disappear against light backgrounds in themes like `twitter.css`.

That has been corrected by switching surface text controls to use the `border` token for their outline in:

- `crates/look-shadcn/assets/style.toml`

The resolver tests were updated accordingly in:

- `crates/look-shadcn/src/controls/textfield.rs`
- `crates/look-shadcn/src/controls/textarea.rs`

### 4. Luma Studio Sidebar Refresh Bugs

Luma Studio also had its own stale-subtree problem in the token sidebar. That was improved by reapplying the sidebar snapshot and rebuilding the token accordion when theme snapshots change, instead of trying to incrementally retheme a stale accordion content subtree.

## Why The Current State Is Still Incomplete

The current architecture still treats `ShadcnLook` as mostly immutable shared data:

- `catalog`
- `light` tokens
- `dark` tokens
- `stylesheet`

Inside `ShadcnLook`, only the `mode` is currently mutable.

Because of that, a mode change can work by mutating one live look and bumping the revision global, but a full theme or override change still tends to create a **new** look object instead of mutating the existing one.

That creates two failure modes:

1. **App-owned persistent subtrees can capture the old look**
   - closures
   - toolbar chrome providers
   - scroll shells
   - panel-local template wiring

2. **App code still needs rebuild paths**
   - even if the outer panel entity remains alive, the panel often needs to reconstruct its internal controls because those controls were built from the old look object

This is why removing `refresh_demos` at the app level is not the same thing as achieving full mutability. Rebuilding panel internals inside each panel entity is narrower than rebuilding the entire board, but it is still a rebuild strategy.

## Desired End State

The desired end state is:

- one live `Arc<ShadcnLook>` per app
- one live control tree
- one SDK theme revision invalidation path
- mode changes, theme choice changes, and override changes all mutate the same live look state
- persistent controls invalidate and rerender without app-level subtree rebuilds

In that model:

- Luma Studio does not recreate demo boards
- Luma Studio does not recreate panel internals
- Gallery does not need manual propagation helpers
- controls behave the same regardless of whether the change was:
  - mode
  - base CSS theme
  - stylesheet change
  - override change

## Proposed Solution

### Phase 1: Make `ShadcnLook` Fully Mutable

Refactor `ShadcnLook` so the live shared state can be replaced in place.

Today, `ShadcnLookState` is effectively immutable except for `mode`. The full fix requires the following data to become replaceable at runtime:

- parsed CSS catalog
- light-mode tokens
- dark-mode tokens
- stylesheet config

This should be exposed through explicit mutation APIs, for example:

```rust
impl ShadcnLook {
    pub fn replace_theme_snapshot(&self, snapshot: ShadcnLookSnapshot);
    pub fn replace_css_catalog(&self, catalog: CssTokenCatalog);
    pub fn replace_with_overrides(&self, overrides: &HashMap<String, Hsla>);
    pub fn replace_stylesheet(&self, stylesheet: StylesheetConfig);
}
```

The exact API shape can vary, but the important behavior is:

- the `Arc<ShadcnLook>` identity remains stable
- the internal resolved theme data changes
- current mode is preserved unless explicitly changed

### Phase 2: Use the SDK Revision Global for All Theme Mutations

Once `ShadcnLook` is mutable in place, every theme change path should:

1. mutate the live look
2. call `cx.bump_luma_theme_revision()`

That includes:

- light/dark mode toggle
- theme choice change
- global color overrides
- stylesheet replacement

At that point there should be no semantic difference between "mode changed" and "theme changed" from the perspective of persistent controls.

### Phase 3: Eliminate App-Level Rebuild Workarounds

After the live look becomes mutable, Luma Studio should stop rebuilding panel internals on theme changes.

That means removing rebuild-based synchronization from:

- `apps/luma-studio/src/studio/demo_controls.rs`
- `apps/luma-studio/src/studio/app.rs`

and replacing it with narrow panel-level updates only where required.

Ideal final behavior:

- panels keep the same control entities alive
- control templates/themes resolve from the mutated shared look
- the revision global invalidates any caches

### Phase 4: Audit App-Owned Control Shells

Some app-owned composites will still need explicit retheme hooks even after `ShadcnLook` is mutable, because they have their own captured control/template state.

Likely audit targets:

- Luma Studio dashboard shell
  - `SplitView`
  - `NavigationSidebar`
  - paging toolbar chrome closures
- Tree view shell
  - `ScrollContainer`
  - scrollbar template capture
- any panel-local helper that stores a closure built from the previous look

The rule should be:

- if a control already resolves from the live look each render, the global revision is enough
- if a control or shell captured concrete template/theme objects from the old look, add explicit runtime setters or local refresh hooks

### Phase 5: Fill Setter Gaps in the SDK

Some controls already expose runtime mutators such as:

- `set_template(...)`
- `set_theme(...)`
- `set_panel_template(...)`
- `set_item_template(...)`
- `set_presenter(...)`

Others still have gaps.

Where app-owned persistent entities need to retheme in place, the SDK should expose enough runtime setters to avoid rebuilds.

This is the part most likely to widen scope, because the remaining problems are not all in one file. They are scattered across:

- `crates/sdk/src/controls/*`
- `crates/look-shadcn/src/look.rs`
- `apps/luma-studio/src/studio/panels/*`

## Concrete Work Items

### A. `look-shadcn`

- Refactor `ShadcnLookState` to support in-place replacement of:
  - catalog
  - token sets
  - stylesheet
- Add explicit mutation APIs
- Preserve current mode across replacement
- Keep resolver APIs stable for downstream callers

### B. SDK Theme Layer

- Keep `LumaThemeRevision` as the single invalidation path
- Ensure all theme mutation sites call `bump_luma_theme_revision()`
- Audit control-local caches and clear them where necessary

### C. SDK Controls

- Audit controls for runtime setter gaps
- Add missing setters where in-place retheming requires them
- Ensure controls that cache layout, render ids, or sub-entities invalidate correctly on theme revision

### D. Luma Studio

- Remove remaining panel-internal rebuild sync paths
- Convert panels to true retheme-in-place behavior
- Retheme app-owned shells and helper entities without respawn

### E. Gallery

- Confirm no remaining manual propagation or app-local theme refresh code is needed once the mutable-look path is complete

## Risks

### 1. Hidden Captures of Old Theme Objects

A stable `Arc<ShadcnLook>` only solves the problem if downstream controls and closures consult that live object at render time. If they captured a concrete template/theme object derived from an older snapshot, they can still remain stale.

### 2. Cache Invalidation Gaps

Controls with cached layout or render identity can appear visually stale even when the theme data changed correctly. `TextField` and `TextArea` already demonstrated this class of bug.

### 3. Scope Growth Through Composite Controls

Controls like:

- `NavigationSidebar`
- `SearchSelector`
- `ComboBox`
- `PagingListView`
- `TreeView` shells

can involve nested template, popup, scrollbar, and presenter wiring. Full mutability can spread from one top-level panel into multiple SDK runtime surfaces.

### 4. False Confidence from App-Level Rebuilds

Rebuilding panels can hide the underlying mutability gap. The goal of this issue is to remove rebuilds as a theme synchronization mechanism, not to improve them.

## Acceptance Criteria

- [ ] `ShadcnLook` theme choice changes can be applied in place without replacing the live `Arc<ShadcnLook>`
- [ ] global color overrides mutate the live look in place
- [ ] mode changes, theme changes, and override changes all use the same SDK revision invalidation path
- [ ] Luma Studio does not rebuild demo boards or panel internals to reflect theme changes
- [ ] Gallery does not require manual control notification plumbing
- [ ] `TextField` and `TextArea` update correctly across mode, theme, and override changes
- [ ] app-owned shells such as dashboard/tree side surfaces retheme correctly without respawn

## Recommended Implementation Order

1. Refactor `ShadcnLook` to support in-place theme replacement
2. route Luma Studio theme-choice and override changes through the live mutable look
3. remove panel-internal rebuild sync from Luma Studio
4. audit and patch remaining composite controls or app-local shells that still capture stale theme-derived objects
5. verify Gallery and Luma Studio end-to-end against:
   - mode change
   - theme choice change
   - token override change

## Summary

The repo has already solved the first half of the problem:

- there is now a correct SDK-owned theme invalidation global
- persistent controls can observe theme revisions
- textfield/textarea repaint and border-token behavior are much improved

The remaining work is to remove the last rebuild-based synchronization strategy by making `ShadcnLook` itself fully mutable in place and wiring the apps and controls to trust that model.

That is the real "full theme fix."
