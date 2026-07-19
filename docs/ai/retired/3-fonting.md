# Issue #3: Non-Uniform Support for Dynamic Typography Sizing

**Status:** Complete (Phases 1–3).  
**Area:** SDK `ControlSize` → look typography; Theme Studio Style Guide verification  

## Description

Throughout the GPUI-Luma component SDK, support for dynamic typography scaling based on `ControlSize` (`sm`, `md`, `lg`) is now uniform across text fields, selectors, choice labels, lists, tree views, and accordion triggers. Sized controls share the button metrics font curve (**12 / 14 / 16**) in look-shadcn.

---

## Progress

### Phase 1: TextField & TextArea — **Complete** (2026-07-19)

* SDK: `.size(ControlSize)` on TextField / TextArea (default `Md`); `.compact()` → `Sm`.
* look-shadcn: button metrics font curve **12 / 14 / 16**.
* Theme Studio Text Field Style Guide: Template Preview + Sizes.

### Phase 2: Selectors — **Complete** (2026-07-19)

* Autocomplete / Combobox / SearchSelector size plumbing + sized popup panels.
* Theme Studio Selectors Style Guide covers all four × Sm/Md/Lg.

### Phase 3: Remaining controls — **Complete** (2026-07-19)

Shipped:

* **Shared helper:** `apply_button_metrics_typography` in look-shadcn (`controls/typography.rs`); TextField helper delegates to it.
* **Checkbox / Radio / Switch:** `Theme::resolve(..., size)`; look factories scale `label_typography` with button metrics; templates pass `model.size`. Theme Studio Choice Sizes tabs already exist.
* **Listbox / List View:** `resolve_row` / row palettes use `size` for label typography (look + lookless).
* **TreeView / Accordion:** `.size(ControlSize)` / `set_size` on models; scale + `resolve_row` / `resolve_trigger` take size; look factories apply button metrics.

**Design pattern:** base `typography.text.label` (or body for text fields), then override `.size` / line-height from `button.metrics.{sm,md,lg}.font_size`. Lookless defaults use `apply_control_size_typography` (sm/body/lg roles).

---

## Code Audit (current)

### Controls that scale typography with `ControlSize`

* **Buttons / toggles / badges** — `button.metrics.*.font_size`
* **Floating menus** — item typography from button metrics / size roles
* **Selectors / Autocomplete / Combobox / SearchSelector** — sized trigger + panel
* **TextField / TextArea** — button metrics font curve
* **Checkbox / Radio / Switch** — label typography by size
* **Listbox / List View** — row label typography by size
* **TreeView / Accordion** — node / trigger typography by size
* **Tabs / cards** — role or title step by size

---

## Acceptance Criteria

- [x] **Text controls:** `sm` / `md` / `lg` TextField (and TextArea) typography follows button font metrics; `md` preserves prior look.
- [x] **Theme Studio verification (text):** Text Field Style Guide Sizes tab demonstrates Primary / Outline / Surface × Sm/Md/Lg.
- [x] **Selector family:** Autocomplete / Combobox / Selector / SearchSelector trigger + panel typography follow `ControlSize`.
- [x] **Choice labels:** Checkbox / Radio / Switch label typography follows `ControlSize` (Theme Studio Choice Sizes tabs).
- [x] **Lists / tree / accordion:** row and trigger typography follow `ControlSize`.
- [x] **Theme Studio verification (Phase 3):** Style Guide sections for Listbox, List View, Tree View, Accordion with Template Preview + Sizes tabs.
- [x] **Form baseline consistency:** sized text + choice + selector controls share font scale with buttons when using the same `ControlSize`.
- [x] **Look-shadcn conformance:** sizes come from stylesheet / typography tokens via `apply_button_metrics_typography`, not raw px literals in look code.
