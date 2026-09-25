# Shadcn inspection boundary

`gpui-luma-look-shadcn` owns color selection, interaction-state overrides, fallbacks,
metric resolution, and typed typography. Its public `tables` module exposes the
results and source metadata. Metric tables reuse the SDK scales or the same paint
helpers used by rendering; they do not define a second layout algorithm.

`gpui-luma-look-shadcn-inspect` remains a companion crate because it owns inspection
DTOs, source formatting, elevation presentation, and color-control inspection
sections. Luma Studio consumes these through `ShadcnInspect`. Studio-specific row
and box-model adapters remain in the app. The SDK has no dependency on either
inspection or the Shadcn look.

Accordion is the reference for complete geometry and typography: its runtime theme
and inspector use the same metric table, including item gap. Studio passes the
window scale factor to `inspect_accordion_metrics_at_scale`; the older method is a
1x convenience wrapper. Its typography table contains the rendered text style and
source metadata. Display strings are assembled only by the companion crate.

The `format_*` helpers formerly exported by `luma_look_shadcn` are now exported by
`luma_look_shadcn_inspect`. Existing inspection DTO names and convenience functions
remain available. New size-aware Button typography and scale-aware Accordion
methods should be used when the inspected control differs from the default size
or pixel scale.

Cross-crate parity tests exercise runtime/inspection agreement, missing-token
fallbacks, interaction combinations, sizes, fractional Accordion scales, and
provenance. Keep this crate until its presentation responsibilities and consumers
are intentionally relocated; sharing resolution alone does not make it redundant.
