# Fix Note: Button Size / Radius Preview Ownership

**Area:** `apps/theme-studio/src/studio/style/*`, `crates/sdk/src/controls/command/button/*`, `crates/sdk/src/controls/button_family/*`, `crates/look-shadcn/src/controls/button.rs`  
**Status:** Experimental change set. Visual direction improved, but ownership drifted into SDK and should be reviewed before keeping.

---

## Summary

The Theme Studio Style Guide work started as a **local preview experiment**:

1. remove the duplicate Style Guide heading,
2. add a local buttons tab switcher (`Template Preview` / `All Sizes`),
3. show button size and radius combinations,
4. make larger button sizes also scale the text and icon.

The visuals moved in the right direction, but the implementation crossed the wrong seams:

- preview-local concepts became SDK concepts,
- theme-owned meaning leaked into Theme Studio preview code,
- a local inspection surface started shaping shared button rendering behavior.

The core lesson is that **button size and radius must stay semantic and button-owned**, with the **theme crate resolving them**, and Theme Studio should only **display the resolved result**.

---

## What We Changed In The Experiment

### Theme Studio

- Removed the in-tab `Style Guide` heading and description.
- Added a local tab switcher to the Buttons section:
  - `Template Preview`
  - `All Sizes`
- Reworked `All Sizes` into per-variant matrices:
  - rows = button size
  - columns = semantic radius presets

### Button Preview Behavior

- Added a radius matrix for:
  - `No radius`
  - `Small`
  - `Medium`
  - `Large`
  - `Full`
- Patched button preview content so larger button sizes also rendered larger text and icons.

### SDK / Look Layer Changes

- Patched `DefaultButtonTemplate` so `radius_override` affects the visible control corner radius, not just the focus adorner.
- Added `icon_size` to `ButtonFamilyLook`.
- Added a semantic `ButtonRadiusPreset` enum.
- Added a shadcn button-side resolver for radius presets.

---

## Problems We Ran Into

## 1. Size semantics were initially implemented in the wrong place

The first pass made Theme Studio preview presenters responsible for translating:

- `ButtonSize::Sm/Md/Lg`
- into font size,
- line height,
- icon size,
- icon gap.

That produced the right picture, but it was structurally wrong.

Why this is wrong:

- `size` is not just a larger box.
- For buttons, `size` means a coordinated bundle of:
  - height,
  - padding,
  - gap,
  - typography,
  - icon scale.
- That meaning belongs to the **button look/theme seam**, not to Theme Studio preview closures.

---

## 2. Radius semantics were initially driven from raw metrics in Theme Studio

The first matrix used direct token values:

- `metrics.radius.none`
- `sm`
- `md`
- `lg`
- `pill`

That is also the wrong seam.

Why this is wrong:

- Theme Studio should not decide how a button interprets `Small` vs `Medium` radius.
- Radius should be **semantic and button-local**, just like size.
- `None / Small / Medium / Large / Full` should be button-facing abstract values.
- The look crate should translate those into concrete px for buttons.

In the experiment we partially corrected this by adding a button-side resolver, but that still introduced new shared API that needs broader review before it is kept.

---

## 3. The template became the place where we “fixed” too much

One bug was real: `radius_override` only changed focus-ring/adorner shaping, not the visible button corners.

That fix was mechanically correct:

- the template now uses the override for actual visible rounding.

But it exposed a larger issue:

- the preview needed custom override plumbing because Theme Studio was still inventing local button semantics instead of asking the button/theme seam for resolved values.

So even when the template patch was correct, it was still attached to a workflow whose ownership was not yet right.

---

## 4. Theme Studio preview began shaping SDK contracts

To support the experiment, we added shared concepts such as:

- `ButtonRadiusPreset`
- `ButtonFamilyLook.icon_size`

These are plausible additions, but they were introduced from a **Theme Studio experiment outward**, not from a deliberate SDK review inward.

That matters because:

- if these concepts are real SDK concepts, they need review across:
  - Gallery,
  - Pager,
  - icon buttons,
  - toggles,
  - radio/checkbox/switch templates that share button-family behavior,
  - any future look crates.
- if they are only preview conveniences, they do **not** belong in shared SDK/look contracts.

This is the main reason the change set should be treated as experimental and reviewed carefully before landing.

---

## 5. The preview surface had too much local rendering logic

By the end of the experiment, Theme Studio had accumulated:

- local button tabs,
- local size/radius matrix logic,
- local sample content logic,
- local button label/icon presentation logic,
- local semantic preset display logic.

Some of that is appropriate for a preview surface.
Some of it is not.

Good local preview work:

- choose which states/variants to show,
- organize matrices,
- label rows/columns,
- wire tabs and section layout.

Bad local preview work:

- deciding what button size means,
- deciding what button radius presets mean,
- deciding how button typography scales.

---

## What The Correct Ownership Should Be

## Button size

For buttons, `size` should be button-owned semantic input:

- either generic at SDK level if the whole SDK really shares the same meaning,
- or localized by control family if button size semantics differ from other controls.

For buttons specifically, size should resolve to:

- height,
- padding,
- gap,
- typography size/line height/weight,
- icon size,
- any other button-family-specific geometry.

That translation belongs in the **theme/look path**, not the template and not Theme Studio preview code.

## Button radius

Radius presets should also be semantic:

- `None`
- `Small`
- `Medium`
- `Large`
- `Full`

Those values should be translated **locally for buttons** by the look/theme layer.

Theme Studio should display:

- semantic radius presets as rows/columns/labels,
- and then render the **resolved button result**.

## Templates

Templates should not be the source of:

- size meaning,
- radius preset meaning,
- typography scaling policies.

Templates should only render a resolved render model / resolved look.

## Theme Studio

Theme Studio should remain a consumer:

- pick semantic knobs to preview,
- ask the real control/theme seam for resolved behavior,
- render that result.

It should not become a parallel implementation of button sizing logic.

---

## Recommended Follow-Up Fix

## Short version

Revert the experiment, then reintroduce the feature from the button/theme seam outward.

---

## Addendum: SDK Semantic Style Constant Catalog

This follow-up search was aimed at finding the semantic style vocabulary currently living in `crates/sdk` that is a likely candidate to move into `crates/look-shadcn`, or at least to be resolved there instead of being defined ad hoc in Theme Studio or individual templates.

### Core shared token vocabulary in SDK

These are the main shared semantic constants already defined in the SDK token layer:

- `crates/sdk/src/theme/tokens.rs`
  - `ControlSize { Sm, Md, Lg }`
  - `RadiusTokens { none, sm, md, lg, xl, pill }`
  - `LumaTextRole { H1, H2, H3, H4, P }`
  - `LumaTextScale { Xs, Sm, Md, Lg, Xl, TwoXl }`
  - text slots:
    - `body`
    - `label`
    - `caption`
    - `title`
    - `code`

- `crates/sdk/src/theme/layout.rs`
  - `StandardBoxScale`
  - `ListRowScale`

These are the clearest examples of shared semantic style policy already present in the SDK.

### Control-specific semantic enums in SDK

The following control modules define semantic style enums instead of only consuming resolved look values:

- `crates/sdk/src/controls/pager/model.rs`
  - `PagerStyle { Minimal, MinimalEdge, Numeric }`

- `crates/sdk/src/controls/popup_menu/model.rs`
  - `PopupMenuTriggerStyle { Outline, Ghost }`

- `crates/sdk/src/controls/textfield/theme.rs`
  - `TextFieldVariant { Standard }`

- `crates/sdk/src/controls/button_family/theme.rs`
  - `ButtonFamilyRole { Text, Icon, Toggle { selected } }`

- `crates/sdk/src/controls/color/composition.rs`
  - `CompositionSize { Sm, Md, Lg, Custom }`

- `crates/sdk/src/controls/color/style.rs`
  - `Size { XSmall, Small, Medium, Large, Size(Pixels) }`

Not all of these should necessarily move, but this is the active catalog of semantic style vocabulary embedded in SDK control code.

### Radius token use sites

The radius token family is already being interpreted directly in several SDK control themes:

- `crates/sdk/src/controls/selection_panel/theme.rs`
  - `metrics.radius.lg`
  - `metrics.radius.sm`

- `crates/sdk/src/controls/selector_panel/items_template.rs`
  - `metrics.radius.lg`
  - `metrics.radius.sm`

- `crates/sdk/src/controls/floating_menu/theme.rs`
  - `metrics.radius.lg`
  - `metrics.radius.sm`

- `crates/sdk/src/controls/card/theme.rs`
  - `metrics.radius.lg`

- `crates/sdk/src/controls/overlay_window/theme.rs`
  - `metrics.radius.xl`

- `crates/sdk/src/controls/scrollbar/theme.rs`
  - `metrics.radius.pill`

- `crates/sdk/src/controls/switch/theme.rs`
  - `metrics.radius.pill`

- `crates/sdk/src/controls/slider/theme.rs`
  - `metrics.radius.pill`

This is useful because it shows that semantic radius policy is already spread across multiple control themes, not isolated to button work.

### Typography semantic use sites

The shared typography slots are also applied directly in many SDK control themes:

- `crates/sdk/src/controls/button_family/theme.rs`
  - `typography.text.label`

- `crates/sdk/src/controls/textfield/theme.rs`
  - `typography.text.body`

- `crates/sdk/src/controls/textarea/theme.rs`
  - `typography.text.body`

- `crates/sdk/src/controls/pager/theme.rs`
  - `typography.text.caption`

- `crates/sdk/src/controls/tabs_navigation/theme.rs`
  - `Sm -> caption`
  - `Md -> label`
  - `Lg -> body`

- `crates/sdk/src/controls/card/theme.rs`
  - `title`
  - `caption`
  - `body`

- `crates/sdk/src/controls/list_view/theme.rs`
  - `header_typography = caption`
  - `row label = label`

- `crates/sdk/src/controls/popup_menu/theme.rs`
  - `trigger_typography = label`

- `crates/sdk/src/controls/selector/theme.rs`
  - `trigger_typography = label`

- `crates/sdk/src/controls/navigation_sidebar/theme.rs`
  - item typography = `label`
  - caption use for secondary/sidebar text

This reinforces the same ownership issue from the button experiment: size is not just geometry; it often carries typography policy with it.

### Hardcoded semantic translation still embedded in SDK

Some modules are doing real style-policy translation themselves rather than only consuming a resolved look:

- `crates/sdk/src/controls/pager/theme.rs`
  - `PagerStyle` drives compact vs non-compact sizing
  - pager overrides button typography and geometry directly

- `crates/sdk/src/controls/popup_menu/theme.rs`
  - `Outline` vs `Ghost` is resolved in SDK theme code

- `crates/sdk/src/controls/slider/theme.rs`
  - slider height, track height, and thumb sizes are hardcoded by semantic size

- `crates/sdk/src/controls/color/swatch.rs`
  - swatch height and default radius are hardcoded from `ControlSize`

- `crates/sdk/src/controls/color/composition.rs`
  - `CompositionSize` resolves through embedded numeric translation logic

These are the places most likely to matter if style semantics are pushed more cleanly into the look crate.

### Migration-oriented takeaway

If this work is revisited, the likely split is:

- shared semantic tokens may stay typed in SDK, but their meaning should be resolved by look/theme code,
- control-specific style variants such as pager style, popup trigger style, and control-local size/radius translations should be reviewed as look-owned policy,
- app preview surfaces such as Theme Studio should only select semantic knobs and render the resolved result.

Behavioral enums that are not really style policy should likely remain in SDK, for example:

- popup/selector placement enums,
- width/selection/state modes,
- other navigation or interaction behavior contracts.

## Better target shape

1. Define whether button size is:
   - fully generic SDK size, or
   - button-local resolved semantics layered on top of `ControlSize`.

2. Define whether button radius presets are:
   - button-local semantics only, or
   - a wider SDK concept.

3. Put the translation in the look crate:
   - size -> height/padding/gap/typography/icon size
   - radius preset -> px radius

4. Expose those resolved values through the button look/render path.

5. Make Theme Studio preview consume those values rather than compute them.

---

## Concrete Revert Guidance

The changes most likely to need review or revert together are:

- `apps/theme-studio/src/studio/style/cards/buttons.rs`
- `apps/theme-studio/src/studio/style/style_guide.rs`
- `crates/sdk/src/controls/command/button/template.rs`
- `crates/sdk/src/controls/button_family/theme.rs`
- `crates/look-shadcn/src/controls/button.rs`
- `crates/look-shadcn/src/look.rs`
- `crates/sdk/src/controls/pager/theme.rs`

Two categories inside that set:

### Purely local preview experiment

- Buttons section tabs
- matrix layout
- row/column labels
- local section content organization

### Shared ownership changes needing SDK review

- `ButtonFamilyLook.icon_size`
- `ButtonRadiusPreset`
- shadcn radius preset resolver
- button template `radius_override` rendering behavior

---

## Final Takeaway

The experiment successfully exposed the real issue:

- **Theme Studio was trying to preview semantic button sizing and radius behavior before those semantics were properly owned by the button/theme seam.**

So the visual result was useful, but the implementation confirms the missing architecture:

- button size and radius should be semantic,
- button-local or carefully generalized,
- resolved by the look/theme layer,
- and merely displayed by Theme Studio.
