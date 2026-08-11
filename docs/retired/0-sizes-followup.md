# Sizes Follow-Up

**Area:** SDK/look sizing contract cleanup after `0-fix-sizes.md`  
**Status:** Follow-up backlog  

## Summary

The main sizing rollout is complete and retired in `docs/ai/retired/0-fix-sizes.md`.

Luma Studio now has resolved-size previews for the major control families, and shadcn runtime sizing is look-owned for the surfaces covered by that work:

- buttons / icon buttons
- toggles
- checkbox / radio / switch
- menus and popup menu panel sizing
- slider
- scrollbar
- progress
- badge typography
- pager guide/fallback cleanup
- selectors, including `Template Preview` and `Sizes`

This note tracks the remaining follow-ups that were intentionally left out of the main rollout.

## Open Follow-Ups

### SDK / API Contract Review

- Review whether `ButtonRadiusPreset` should remain public SDK vocabulary, stay button-local to look-shadcn, or move behind a narrower look-owned seam.
- Review whether `ButtonFamilyLook.icon_size` is the right SDK-level contract or an implementation detail of the shadcn button look.
- Review choice-control style policy:
  - `ShadcnButtonStyle` is currently used as preview convenience.
  - checkbox/radio/switch active fill is style-driven.
  - enabled elevation remains `shadow-sm`.
  - Primary/Secondary are fill choices, not shadow variants.

### Optional Catalog Cleanup

These are lower priority because shadcn runtime styling is already look-owned where product behavior matters.

- Navigation sidebar fallback constants.
- List/listbox/control-group fallback spacing literals.
- Standalone context/navigation floating-menu sizing APIs if those surfaces need caller-controlled menu size.
- Badge-specific metrics only if tags need a non-button sizing policy later.

### Smarter Size Curves

Done — see [`0-size-smarter.md`](./0-size-smarter.md). Padding/`Lg` inset curve and floating-menu density factors landed 2026-07-19.

## Not In Scope

- Shadow token ladder and Luma Studio round-trip: [`0-shadow-enhance.md`](./0-shadow-enhance.md)
- Choice disabled/elevation layout regression: [`2-elevation.md`](./2-elevation.md) (**done**)

## Retired Source Note

The original rollout note was moved to:

- `docs/ai/retired/0-fix-sizes.md`
