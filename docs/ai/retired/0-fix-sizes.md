# Fix Note: Button Size / Radius Preview Ownership

**Area:** `apps/theme-studio/src/studio/style/*`, `crates/sdk/src/controls/command/button/*`, `crates/sdk/src/controls/button_family/*`, `crates/look-shadcn/src/controls/{button,toggle,templates,slider,scrollbar,progress}.rs`, `crates/look-shadcn/src/elements/badge.rs`, `crates/look-shadcn/assets/style.toml`  
**Status:** Retired / complete as of 2026-07-15. The main sizing rollout is done: Button-family, **Menus**, **Slider**, **Scrollbar / Progress**, **Badge**, **Pager**, and **Selectors** now consume the relevant look/size seams in Theme Studio or runtime surfaces. SDK fallback cleanup for slider, floating menu, popup menu panel sizing, and pager fallback geometry now derives from shared metric/typography tokens instead of local sm/md/lg or fixed px tables. Follow-up work moved to [`../issues/0-sizes-followup.md`](../issues/0-sizes-followup.md). Smarter padding curves moved to [`../issues/0-size-smarter.md`](../issues/0-size-smarter.md). Shadow round-trip remains tracked separately in [`../issues/0-shadow-enhance.md`](../issues/0-shadow-enhance.md).

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

## Progress (2026-07)

Early commits (`92706612`, `6d244407`, `9ecda374`) landed button/toggle/choice Style Guide matrices and look-layer size/radius resolution. Subsequent session work extended previews and fixed menu panel typography.

| Commit / session | Focus |
|------------------|--------|
| `92706612` | Buttons + Icon Buttons size×radius; `ButtonRadiusPreset`; `icon_size`; visible `radius_override` |
| `6d244407` | Toggle look + `style.toml` metrics; toggle matrices; `VariantStateTable` |
| `9ecda374` | Checkbox / Radio / Switch split; choice matrices + Sizes tabs |
| 2026-07-08 | **Menus** four-tab preview; floating-menu typography scales with size; style guide sections alphabetical; buttons text-only in Template/Sizes; menu-trigger `radius_override` in look |
| 2026-07-08 (later) | **Slider** dedicated section (Template Preview + Sizes); Input Controls split into Scrollbar / Slider / Text Field / Text Area; primary-only slider look; thumb + track radius presets; disabled inactive track = `border` (matches enabled) |
| 2026-07-15 | **Scrollbar / Progress** size contracts; `[scrollbar.metrics.sm|md|lg]`; `[progress.metrics.sm|md|lg]`; Theme Studio Scrollbar Sizes tab; Progress Sm/Md/Lg row; badge typography resolves from button font metrics |
| 2026-07-15 (later) | SDK default slider fallback geometry derives height, track height, and thumb size from `MetricTokens` instead of hardcoded Sm/Md/Lg match tables |
| 2026-07-15 (scrollbar follow-up) | Added `ScrollbarStyle::Ghost | Soft`; ghost keeps the transparent track, soft uses a muted track channel for Radix-like visibility; Theme Studio scrollbar Template Preview shows both variants in a state-column table; Sizes is horizontal-only; user-tested and accepted |
| 2026-07-15 (popup follow-up) | Popup menus gained independent `menu_size`; trigger size remains trigger-owned, while floating panel metrics resolve from menu size; user-tested and accepted |
| 2026-07-15 (catalog cleanup) | SDK fallback floating-menu typography/icon size now comes from typography + metric tokens; pager fallback geometry now comes from Sm control metrics + spacing tokens; pager template no longer clamps numeric buttons to a local fixed width |
| 2026-07-15 (pager guide) | Theme Studio Style Guide gained a Pager section copied from the Gallery pager pane examples and wired as an interactive preview |

### Theme Studio Style Guide (landed)

Shared preview pattern: **`VariantStateTable`** — variants as rows, interaction states as columns (`apps/theme-studio/src/studio/style/variant_state_table.rs`).

| Section | Preview tabs | Sizes / sizing preview | Notes |
|---------|--------------|------------------------|--------|
| Buttons | Template Preview, Sizes | size × radius | Text-only (no leading/trailing icon rows) |
| Icon Buttons | Template Preview, Sizes | size × radius | |
| Toggles | Template Preview, Sizes | size × radius; text + icon matrices on Template | |
| Checkbox | Template Preview, Sizes | size × Primary/Secondary | |
| Radio | Template Preview, Sizes | same | |
| Switch | Template Preview, Sizes | same | |
| **Menus** | Menu Trigger, Trigger Sizes, Floating Menu, Sizes | trigger: states + size×radius; panel: states + Sm/Md/Lg | Trigger = outline/ghost; panel uses `floating_menu_look(size)` |
| Pager | — (single layout) | Gallery pager examples | Minimal, Minimal + first/last, Numeric, Numeric no first/last, Numeric 5-slot; interactive item/page-size/enabled controls |
| Selectors | Template Preview, Sizes | Sm/Md/Lg trigger + open-panel rows | Template Preview = state × control-type grid at `Md`; Sizes rows reuse trigger/input `ControlSize` and `selector_items_panel_look(size)` |
| Scrollbar | Template Preview, Sizes | Template: horizontal Ghost/Soft × states plus vertical states; Sizes: horizontal Sm/Md/Lg | Own section (was grouped under Input Controls) |
| **Slider** | Template Preview, Sizes | Template: state columns (Standard/Hover/Focus/Active/Disabled); Sizes: Sm/Md/Lg × radius presets | Own section; 120px demo width; primary-only (no Secondary variant row) |
| Text Field | — | states | Own section (was grouped under Input Controls) |
| Text Area | — | states | Own section (was grouped under Input Controls) |
| Feedback | — (stacked blocks) | Badge Sm/Md/Lg inline samples; progress states + Sm/Md/Lg row | Progress remains in Feedback rather than a separate section |
| Sidebar, Tabs, Typography, … | legacy | — | Not on matrix + Sizes pattern yet |

Style guide section order: **alphabetical** by title (Buttons → … → Typography).

Other preview fixes (unchanged from earlier):

- Checkbox / Radio / Switch / Toggle / **Menus** in `section_allows_interaction()`.
- Toggle icon preview uses `toggle_icon_look_semantic` + `ButtonRadiusPreset::Full`.
- Choice previews use indicator-only role; Outline/Ghost omitted from choice variant rows.

### Look / SDK (landed)

- **`button_look_semantic` / `button_look`**: size → height, padding, gap, typography, icon size; radius preset → px.
- **`toggle_look` / `toggle_look_semantic`**: separate toggle metrics in `style.toml`.
- **`floating_menu_look`**: item typography + icon use button `font_size` / `icon_size` per `ControlSize` (was padding-only scaling).
- **`popup_menu`**: `trigger_radius_override` for menu-trigger radius preview.
- **`slider_look`**: primary-only color rules in `style.toml`; `[slider.metrics.sm|md|lg]`; separate thumb vs track radius resolution (`resolve_slider_thumb_radius_preset`, `resolve_slider_track_radius_preset`); disabled track rail uses `border` (not `muted`).
- **Slider template**: disabled opacity on Active/Domain fill only — Inactive rail stays full-opacity; thumb stays opaque when disabled.
- **`scrollbar_look`**: `ControlSize` resolves look-owned thickness, track thickness, thumb thickness, and minimum thumb length from `style.toml`.
- **`ScrollbarStyle`**: `Ghost` preserves the transparent track; `Soft` fills the track with muted color so the scrollbar channel is visible in previews or dense surfaces.
- **`progress_look`**: `ControlSize` resolves circular progress diameter and stroke width from `style.toml`.
- **`badge_look`**: `ControlSize` resolves typography from button font metrics so badge text/icon scale follows the same size policy as buttons and floating menu items.
- **Default slider theme**: fallback height, track height, and thumb size derive from `MetricTokens` (`control_height`, `spacing`, `gap`, `border_width`) rather than fixed Sm/Md/Lg literals.
- **Popup menu panel size**: `PopupMenuBuilder::menu_size(...)` decouples floating panel metrics from trigger size; `.size(...)` remains trigger-only.
- **Default floating menu theme**: fallback menu item typography follows `LumaTypography` size roles, and fallback icon size derives from item typography + border width.
- **Default pager theme**: fallback button height/min-width, padding, gap, group gap, and radius derive from Sm control metrics + spacing tokens; numeric/gap buttons use the resolved `button_min_width`.
- **Template**: `radius_override` paints visible control corners.
- **Gallery / paint**: `button_look` re-export restored.

### Still open / needs work

This section is retained for historical context. Active remaining work moved to [`0-sizes-followup.md`](../issues/0-sizes-followup.md).

#### SDK contract review (unchanged)

- **`ButtonRadiusPreset`**, **`ButtonFamilyLook.icon_size`**: confirm SDK vs look-only ownership.
- **Choice elevation / variant policy**: `shadow-sm` when enabled; Primary/Secondary = fill only; no surface/soft axis.
- **Choice variant model**: `ShadcnButtonStyle` as preview convenience vs look-owned types.

#### Control sizing — next targets

| Control | Look layer today | Theme Studio today | Likely work |
|---------|------------------|--------------------|-------------|
| **Slider** | `style.toml` primary color rules; `slider_look(..., size)`; thumb/track radius presets; SDK fallback derives geometry from `MetricTokens` | **Slider** section: Template Preview (states); Sizes (Sm/Md/Lg × radius) | **Done** (Theme Studio + look + SDK fallback cleanup) |
| **Scrollbar** | `[scrollbar.metrics.sm|md|lg]`; `scrollbar_look(..., size)` | Template Preview + Sizes | **Done / tested** (look metrics + SDK render-model size + Theme Studio Sizes tab) |
| **Progress** | `[progress.metrics.sm|md|lg]`; `progress_look(..., size)` | Feedback states + Sm/Md/Lg row | **Done** (look metrics + SDK render-model size + Feedback size row) |
| **Badge / tags** | `badge_look(variant, size)`; typography resolves from button `font_size` per size | Feedback shows Sm/Md/Lg **inline** | **Done** for typography scaling. Badge-specific metrics remain optional only if tags need a non-button sizing policy later |
| **Selectors** | Trigger = textfield/combobox/search/selector templates; panel = `selector_items_panel_look(size)` | Template Preview tab plus Sizes tab with Sm/Md/Lg trigger + open-panel rows | **Done** for Style Guide parity. No separate selector-size semantic was added; previews reuse `ControlSize` on the trigger/template side and `selector_items_panel_look(size)` for panels. |
| **Floating menu (runtime)** | Panel metrics resolve through popup `menu_size`; trigger metrics remain trigger-owned | Sizes tab previews panel via `floating_menu_look(size)` directly | **Done / tested** for popup menu runtime. Context/navigation standalone floating menus still default to Md unless their caller/theme exposes size |

#### Typography / size ownership elsewhere (catalog addendum)

Current cleanup status:

- **Done**: slider fallback geometry, default floating-menu fallback typography/icon sizing, popup menu panel sizing, default pager fallback geometry.
- **Still possible**: navigation sidebar fallback constants, list/listbox/control-group fallback spacing literals, and standalone context/navigation floating-menu sizing APIs if those surfaces need caller-controlled menu size.
- **Not urgent**: these are SDK fallback defaults. Shadcn runtime remains look-owned for product styling.

#### Related (not this note)

- Shadow token ladder + Theme Studio round-trip: [0-shadow-enhance.md](./0-shadow-enhance.md).

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

### Short version

Button-family, **Menus**, **Slider**, **Scrollbar**, **Progress**, **Badge**, **Pager**, and **Selectors** now consume the relevant look/size seams in Theme Studio. Selector sizing stays compositional: trigger/input `ControlSize` plus `selector_items_panel_look(size)`.

### Suggested implementation order

1. ~~**Slider** — Theme Studio Sizes tab (Sm/Md/Lg).~~ **Done**: dedicated **Slider** section with Template Preview + Sizes tabs; primary-only look; thumb/track radius presets; disabled inactive track matches enabled (`border`); SDK template dims fill only when disabled.
2. ~~**Badge** — use button `font_size` / `icon_size` in `badge_typography`; Feedback Sizes matrix.~~ **Done**: badge typography resolves from button size metrics; Feedback shows Sm/Md/Lg inline.
3. ~~**Progress** — `[progress.metrics.sm|md|lg]` + `progress_look(size)`; Feedback Sizes samples.~~ **Done**.
4. ~~**Scrollbar** — product decision: global vs sized; then metrics + preview.~~ **Done / tested**: Scrollbar has Template Preview + Sizes and `Ghost` / `Soft` variants.
5. ~~**Selectors** — parameterize preview `ControlSize`; document trigger = textfield size, panel = floating menu size.~~ **Done**: Style Guide has Sm/Md/Lg trigger + open-panel rows without adding a selector-specific size axis.
6. **SDK review** — moved to [`0-sizes-followup.md`](../issues/0-sizes-followup.md).
7. ~~**Popup menu** — decouple floating panel size from trigger size at runtime.~~ **Done / tested**.

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
  - **look-shadcn** has `[slider.metrics.sm|md|lg]`, primary color rules, and `slider_look(..., size)` — Theme Studio Slider previews use this seam; SDK default theme should defer there too

- `crates/look-shadcn/src/controls/scrollbar.rs` / `progress.rs`
  - single global metrics blocks; no `ControlSize` parameter yet (slider now has sm/md/lg in look + Theme Studio)

- `crates/look-shadcn/src/elements/badge.rs`
  - `badge_typography` maps size → text roles (caption/label/body), not button `font_size` / `icon_size`

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

## Concrete Revert / Review Guidance

### Landed — keep unless deliberately rolling back preview work

- `apps/theme-studio/src/studio/style/cards/buttons.rs` — matrix previews for buttons, icon buttons, toggles, checkbox, radio, switch
- `apps/theme-studio/src/studio/style/cards/menus.rs` — menu trigger + floating menu tabbed previews
- `apps/theme-studio/src/studio/style/style_guide.rs` — Input Controls split (Scrollbar / Slider / Text Field / Text Area); slider preview tabs; interaction allowlist; alphabetical order
- `apps/theme-studio/src/studio/style/variant_state_table.rs` — shared table layout
- `crates/look-shadcn/src/controls/button.rs` — semantic size/radius resolution
- `crates/look-shadcn/src/controls/toggle.rs` — toggle look + metrics
- `crates/look-shadcn/src/controls/floating_menu.rs` — item typography/icon scale with size
- `crates/look-shadcn/src/controls/popup_menu.rs` — trigger radius override
- `crates/look-shadcn/src/controls/templates.rs` — toggle template (modifier removed)
- `crates/look-shadcn/assets/style.toml` — toggle + slider metrics/color blocks
- `crates/look-shadcn/src/controls/slider.rs` — primary-only look; thumb/track radius; disabled track = `border`
- `crates/sdk/src/controls/slider/template/linear.rs` — disabled opacity on fill segments only
- `crates/sdk/src/controls/command/button/template.rs` — visible `radius_override`
- `apps/theme-studio/src/studio/style/cards/inputs.rs` — Slider Template Preview + Sizes matrix; Scrollbar / Text Field / Text Area sections

### Still needs SDK / API review or new look work

Historical list; current follow-ups moved to [`0-sizes-followup.md`](../issues/0-sizes-followup.md).

- `ButtonFamilyLook.icon_size`, `ButtonRadiusPreset`
- Choice controls: `ShadcnButtonStyle` variant model + elevation policy
- Optional fallback cleanup for navigation sidebar, list/listbox/control-group, and standalone context/navigation floating-menu sizing APIs.

---

## Final Takeaway

The experiment exposed the real issue: **Theme Studio was previewing semantic sizing before the button/theme seam owned it.** That is largely fixed for buttons, toggles, choices, menu triggers/panels, and **slider** (Template Preview + Sizes on the look seam).

**Remaining follow-ups moved out:**

- SDK/API contract review and optional fallback cleanup: [`0-sizes-followup.md`](../issues/0-sizes-followup.md)
- Smarter size padding curves: [`0-size-smarter.md`](../issues/0-size-smarter.md)
- Shadow ladder / Theme Studio: [`0-shadow-enhance.md`](../issues/0-shadow-enhance.md)

Target architecture unchanged: size semantics resolved in look-shadcn; Theme Studio selects knobs and renders resolved results.

---

## Addendum: Naming — shadcn/ui semantics, not Radix Themes

**Decision (2026-07):** look-shadcn and Theme Studio previews target **shadcn/ui** vocabulary. Early notes and experiments sometimes referenced **Radix Themes** (numeric sizes `1–4`, choice variants classic/surface/soft). That framing was the wrong seam for this crate — use **shadcn** names in constants, TOML keys, and preview labels.

### Style / variant constants

- **Use:** `ShadcnButtonStyle` (`Primary`, `Secondary`, `Outline`, `Ghost`) — already the look-shadcn style axis for buttons and toggles.
- **Do not introduce:** parallel `Radix*` style enums or Radix Themes variant names (`classic`, `surface`, `soft`) unless we deliberately add a second look crate.
- **Choice controls today:** API accepts `ShadcnButtonStyle`, but checkbox/radio/switch color rules mostly use it only for **checked/on accent** (`@action_layer`). Theme Studio previews show **Primary + Secondary only**; Outline/Ghost are omitted because they map to the same foreground token for indicators.

### Size constants

- **Use:** SDK `ControlSize` (`Sm`, `Md`, `Lg`) and `style.toml` keys `button.metrics.sm|md|lg` (and parallel toggle/choice metric blocks).
- **Do not introduce:** `ShadcnButtonSize::One..Four`, Radix numeric tiers, or TOML keys `[button.metrics.1]` … `[button.metrics.4]`.
- **Why:** shadcn/ui button sizing is Tailwind-style `sm` / default (`md`) / `lg`, not Radix Themes `1 | 2 | 3 | 4`.

### Icon-only buttons

Size and role are separate axes:

- **Size:** `ControlSize::Sm | Md | Lg`
- **Role:** `ButtonFamilyRole::Icon` vs `Text`

`sm` / `md` / `lg` icon buttons = `role: Icon` + `size: Sm|Md|Lg`. No separate shadcn `size="icon"` metric unless we later want API parity for a fixed square default.

### Radius

`ButtonRadiusPreset` (`None`, `Small`, `Medium`, `Large`, `Full`) stays **button-local** in look-shadcn. Theme Studio size matrices:

- **Rows:** Small / Medium / Large (`ControlSize`)
- **Columns:** radius presets (buttons/toggles) or Primary/Secondary (choice sizes tab)
- **Resolution:** `button_look_semantic(..., size: ControlSize, radius: Option<ButtonRadiusPreset>, ...)`

### Elevation (choice controls — open)

Checkbox/radio/switch/toggle indicators share **`shadow-sm` when enabled** regardless of `ShadcnButtonStyle`. That follows toggle policy, not button elevation (Primary/Secondary/Ghost = no shadow). Primary/Secondary changes **fill when active**, not shadow tier. A Radix-style surface/soft split would be a **new look-owned axis**, not a rename of Outline/Ghost.

### Dropped from experiment

- `ShadcnButtonSize::One..Four`
- `[button.metrics.1]` … `[button.metrics.4]` in `style.toml`
- Radix Themes choice variant naming as a stand-in for shadcn button styles
