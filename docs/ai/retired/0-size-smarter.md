# Smarter Size Scaling

**Area:** Theme sizing tokens, selector/menu previews, `style.toml` metrics  
**Status:** In progress — button/toggle height + vert pad locked to Sm; horizontal pad scales (2026-07-19)

## Summary

The Sm/Md/Lg sizing plumbing is working, but the visual scaling curve was too aggressive.

In selector and menu previews, larger sizes increased font, height, and padding together. `Lg` felt overly padded: the font got larger, yet surrounding whitespace grew too much at the same time.

This was a shared size-token tuning issue (selectors, menus, and other controls that reuse button/control metrics), not a selector functionality bug.

## Desired Direction

Keep Sm/Md/Lg typography and geometry distinct, but make padding growth more gradual than font growth.

Policy:

- `Sm`: compact padding stays close to prior values.
- `Md`: baseline unchanged.
- `Lg`: increase font size and control height, but only slightly increase horizontal padding and row inset.
- Panel rows feel denser than trigger controls, especially for menu and selector lists.

Goal:

```text
font size growth: clear
height growth: moderate
padding/inset growth: gradual
panel density: tighter than triggers
```

## Landed Changes

1. **Button / toggle size curve** — Sm/Md/Lg share Sm **height** and **vertical padding**; horizontal pad scales `12/16/20` (buttons) and type/icon still scale.
2. **Catalog spacing** — `padding_y` multiplier locked to Sm (`1.25`) for all sizes; `padding_x` still grows `2.5/3.5/4.5`.
3. **Floating menu metrics wired** — `floating_menu_look` consumes `floating_menu.metrics.{sm,md,lg}`.
4. **Radius token aliases** — `resolve_stylesheet_metric` accepts `radius.sm|md|lg|xl|none`.

Rebuild Theme Studio after changes (`style.toml` is compile-time embedded).

## Validation

- Theme Studio: Selectors **Sizes** (trigger + open panel Sm/Md/Lg), Menus, Buttons, Toggles.
- Unit tests cover toggle Lg padding, catalog Lg multiplier, and floating-menu padding growth vs typography.

## Related

- Parent rollout: `docs/ai/retired/0-fix-sizes.md`
- Remaining API/catalog cleanup: [`0-sizes-followup.md`](./0-sizes-followup.md)
