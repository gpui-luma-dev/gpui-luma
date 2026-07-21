# Issue #2: Elevation and Box-Shadow Integration

**Status:** Done (2026-07-19)  
**Area:** SDK layout + look-shadcn `elevation_rules` → catalog shadows

## Summary

Elevation is first-class across SDK control looks/templates and look-shadcn stylesheet rules. Shadows resolve `style.toml` → catalog `--shadow-*` → look fields → template paint, with stable projection reserve across enabled/disabled.

**Shipped**
- `ShadowProjectionInsets` + elevation-slot / reserve-shadow geometry (`max(focus, shadow)`; enabled-state probe when disabled clears paint).
- `elevation_rules` for button/toggle, checkbox, radio, switch, textfield (Primary), floating_menu, card, slider; selection popups inherit floating_menu surface.
- Selection triggers: selector / search_selector Outline button elevation; combobox / autocomplete Primary textfield elevation.
- `ShadcnElementExt::shadow_cn` for app chrome; Luma Studio dashboard uses catalog `shadow` instead of a hardcoded panel shadow.
- Gallery prototype `shadow_button` kept as a tuning harness (not migrated into SDK).

**Explicitly out of scope**
- Outline / Surface textfield elevation — **won't do** (Primary-only).
- Luma Studio Other-tab ladder regen — separate open issue [`0-shadow-enhance`](../issues/0-shadow-enhance.md).
- Absolute shadow backing / full bleed-clip QA — unrelated leftovers; only if clipping appears after slots.

## Acceptance (met)
- Outline buttons and other rules-backed controls paint catalog shadows from `style.toml`.
- Primary textfield paints `shadow-xs`; Outline / Surface do not.
- Disabled elevation paint does not change measured layout (pager / primary fields).
- Theme swaps update resolved shadow tokens via catalog.

## Historical design notes

See git history for the full design (box-model / overflow / transparent-fill seams, selection-trigger addendum). Retained below as a short pointer only:

| Seam | Mitigation shipped |
|------|--------------------|
| Layout displacement | Elevation slot + reserve; paint gated, reserve stable |
| Parent `overflow_hidden` | Prefer overflow-visible on shadow chrome (e.g. textarea); avoid absolute backing by default |
| Transparent Outline fill | Blocked — use `.primary()` for drop shadow |
| Selection hybrid chrome | Outline / Primary textfield paths above |
