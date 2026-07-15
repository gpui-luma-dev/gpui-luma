# Smarter Size Scaling

**Area:** Theme sizing tokens, selector/menu previews, `style.toml` metrics  
**Status:** Proposed follow-up  

## Summary

The Sm/Md/Lg sizing plumbing is working, but the visual scaling curve is too aggressive.

In selector and menu previews, larger sizes increase:

- trigger font size
- trigger/control height
- panel row height
- horizontal padding / row inset
- vertical breathing room around text

The result is technically correct sizing, but `Lg` feels overly padded. The font gets larger, yet the surrounding whitespace grows too much at the same time. This makes large controls feel loose rather than simply larger and more legible.

This is not a selector functionality bug. It is a size-token tuning issue shared by selectors, menus, and likely other controls that reuse button/control metrics.

## Desired Direction

Keep Sm/Md/Lg typography and geometry distinct, but make padding growth more gradual than font growth.

Suggested policy:

- `Sm`: compact padding can remain close to current values.
- `Md`: keep as the baseline.
- `Lg`: increase font size and control height, but only slightly increase horizontal padding and row inset.
- Panel rows should generally feel denser than trigger controls, especially for menu and selector lists.

## Suggested Solution

Introduce a smarter size curve for spacing tokens instead of treating all dimensions as equally scalable.

Recommended approach:

1. Audit the current `button.metrics.sm|md|lg`, floating-menu metrics, selector-panel metrics, and textfield/control metrics.
2. Keep font-size deltas visible across Sm/Md/Lg.
3. Reduce `Lg` horizontal padding growth for trigger-like controls.
4. Reduce `Lg` item padding / row inset growth for menu and selector panel rows.
5. Preserve hit-target height where needed, but avoid making inner text padding grow linearly with the total control height.

The goal is:

```text
font size growth: clear
height growth: moderate
padding/inset growth: gradual
panel density: tighter than triggers
```

## Notes

Selectors exposed the issue clearly because the Style Guide now shows Sm/Md/Lg trigger + open-panel rows together. Menus show the same pattern, so this should be handled as shared sizing-token tuning rather than selector-specific adjustment.
