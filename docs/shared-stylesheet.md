# Shared stylesheet ownership and migration

Issue #105 now has a shared SDK contract for configurable control geometry and
tab body motion. Shadcn and Radix supply separate `assets/style.toml` files using
`CommonStylesheet`; their palettes, numeric size semantics, token references and
visual recipes remain look-owned. Both paint and available inspectors resolve
against the selected configuration. SDK templates consume the resolved inputs.

## Ownership matrix

| Input or machinery | Current source | Ownership and migration decision |
| --- | --- | --- |
| Incoming tab panel duration | SDK builder, Radix builder, Radix Studio constant | SDK `CommonStylesheet` defines types, fallback, precedence and provenance; looks supply `fade_in_ms`. |
| Tab list gap, list padding and indicator height | SDK `TabsGeometryStylesheet`, per-look TOML | Shared fields with variant, general and supplied token/SDK fallback precedence. Paint and inspectors consume the same resolved metrics. |
| Shadcn tab labels, disabled list fill and selected indicator colors | `stylesheet/config/tabs.rs`, `controls/tabs.rs` | Look extensions: CSS token names and recipes remain Shadcn-owned. |
| Shadcn radii, trigger padding and typography | `controls/tabs.rs`, `tables/metrics/tabs.rs` | Catalog-derived trigger padding, typography and radii remain look recipes. Common list geometry retains its CSS spacing derivation when unset. |
| Radix baseline, inset and size metrics | `tabs.rs::TabsStyle`, `look.rs` | Remain look-owned. Existing setter is explicit tuning above shared geometry, preserving compatibility; gray-step interpretation stays in Radix. |
| Surface variant gap, padding, chip radius and ghost fill | Radix TOML and `tabs.rs` | Gap is authored in the surface variant; padding falls back to spacing.s1. Chip radius and color recipes remain look-owned. |
| Switch track width, height, padding, thumb size and gap | SDK `SwitchStylesheet`, per-look TOML | Per-size fields override general geometry. Shadcn retains legacy metric sections as a fallback; Radix radius recipes remain look-owned. |
| Slider width, height, track height and thumb size | SDK `SliderStylesheet`, per-look TOML | Per-size geometry overrides general values. Explicit thumb selections choose the corresponding look size. Legacy Shadcn metrics remain a fallback; radius stays look-owned. |
| Button, scrollbar and other family metrics | Shadcn `stylesheet/config/*`, `stylesheet/resolve/mod.rs`; Radix adapters | Typed family inputs now resolve through `CommonStylesheet`; token references and recipes remain look extensions. See family boundaries below. |
| CSS `first`, `first_layer`, `@action_layer`, `@outline_layer` and shadow references | Shadcn `stylesheet/resolve/mod.rs` | Look extensions tied to Shadcn catalogs and variants. |
| Optional boolean and interaction matching | Shadcn `stylesheet/config/mod.rs` | Look-owned matching for palette recipes; rule order and disabled-state semantics remain intact. Shared geometry matching uses look-owned size/variant keys. |
| `ButtonSelectorState` and normalized style/variant keys | Shadcn `stylesheet/selector.rs` | Mixed ownership: generic state matching is reusable, look enum mappings are extensions. |
| Tab selection, retained panels, transition implementation, indicator bounds and slot layout | SDK `controls/navigation/tabs/*` | Structural Rust behavior; stays in SDK code. |
| Color, metric and typography provenance | SDK `theme/provenance.rs`, Shadcn inspectors | Existing shared value types remain. Motion adds a shared source/value API. |

## Contract and lifetime

`CommonStylesheet::parse` reads a complete per-look TOML document, ignoring
look-owned root sections. Shadcn also deserializes the same common type as part of
its existing `StylesheetConfig`. Unknown fields inside `common` fail parsing, as
do negative, fractional, string, or out-of-range durations. Milliseconds use
`u32`. Geometry uses finite, nonnegative logical pixels; zero is valid. Variant
names belong to the look, with values under `[common.tabs.variants.<name>]`
overriding `[common.tabs.geometry]`. Missing geometry retains supplied token or
SDK defaults, with their original provenance. Size keys belong to each look, under
`[common.switch.sizes.<key>]` and `[common.slider.sizes.<key>]`, overriding
family geometry field by field. Radix switch and slider use numeric semantic keys `"1"`, `"2"`,
`"3"` (for example `[common.slider.sizes."2"]`); other families retain their own
ranges and exceptions. Shadcn uses `sm`, `md`, `lg`.
The shared schema accepts look-owned keys without imposing an SDK size enum.
Radix builders retain their semantic size; adapters map to SDK `ControlSize`
only at the control boundary. Shadcn snaps authored switch values to display
pixels; Radix preserves its existing logical-pixel behavior. Slider values
retain logical pixels; explicit thumb-size enums select the matching look key
independently of control size. Applications own file access and
input-size restrictions. Public parsers
return errors; the Radix embedded loader falls back safely, with tests validating
the bundled values.

Resolution is explicit instance override, then look value, then supplied SDK/token
fallback. Tab body motion has a zero SDK fallback. Missing differs from authored
zero. `animated(false)` disables effective body
motion even when a duration is explicit; re-enabling restores the configured
value. Both SDK builders and live controls expose `body_motion()` with value and
source, including the look name and field path.

Look builders capture motion when converted to SDK builders during spawn. SDK
builders capture it when `.stylesheet(...)` is called. Existing controls keep
that motion snapshot. Geometry is resolved on subsequent renders; callers notify
affected views after Radix configuration changes. No setter schedules a redraw.
Radix clones share configuration; forks copy it independently. Shadcn supplies
configuration at construction through `from_css_str_with_stylesheet`; its
independent color copies preserve the stylesheet. No new Shadcn mutation API is
introduced. Palette and existing visual update behavior is unchanged.

## Baselines and duplicate sources

Shadcn supplies a 300 ms body fade, matching Radix, and retains a 3 px indicator. Its size typography remains
caption, label and body; trigger height remains line height plus vertical padding.
Radix retains 32/40 px line trigger heights, 8/16 px horizontal padding and a 2 px
indicator. Light/dark color and interaction recipes are untouched. Radix's look
now supplies a 300 ms body fade, intentionally extending the prior Radix Studio
setting to other content-bearing Radix tabs. The app constant and the builder's
forced zero are removed.

Shadcn tabs now use the selected stylesheet for both colors and geometry.
`ShadcnInspect` uses that same stylesheet, reporting shared authored paths as
`style.toml · common.tabs...` through its existing provenance API. Legacy helpers
without a look argument continue to use the embedded stylesheet. Radix exposes
`tabs_geometry` with shared typed provenance. Its stored `TabsStyle` is now an
optional explicit override rather than a second default source; default gap and
indicator height come from TOML. Surface tabs always hide the underline,
independently of line indicator tuning.

Switch geometry now uses the selected Shadcn stylesheet in both paint and
`ShadcnInspect`. Bundled `switch.metrics` values have moved into `common.switch`;
legacy sections remain readable below common values. Missing fields fall back to
SDK dimensions. Shadcn retains 32/40/48 px widths, 13/18/24 px heights, 9/14/20 px
thumbs and 2 px padding. Radix retains 28/35/42 px widths, 16/20/24 px heights,
14/18/22 px thumbs, 1 px padding and 8 px label gap. Radix
`SwitchSize::height` and `switch_scale_for` use the bundled shared configuration;
look-bound themes use live configuration. `switch_geometry` exposes matching
metric values and authored paths. No variant, radius, palette or shadow formulas
were extracted.

Slider geometry now uses the selected Shadcn stylesheet for painting and
inspection. `ShadcnInspect::inspect_slider_metrics_for` accepts control size and
explicit thumb selection; the existing method remains a medium-size wrapper.
The bundled legacy slider metrics have moved to `common.slider`. Legacy custom
sections, including radius token references, remain fallback inputs. Missing
dimensions use SDK metric-derived defaults rather than the old single fixed
fallback tuple.

Shadcn retains a 260 px width, 24/32/40 px heights, 4/6/8 px tracks and 12/16/20 px
thumbs. Radix retains a 120 px width, 16/20/24 px heights, 4/6/8 px tracks and
12/16/20 px thumbs. Radix radius remains half the resolved height; Shadcn retains
its pill token or legacy radius reference. Rust thumb-size constants have been
removed from both adapters. Explicit selections report the requested size and
resolved TOML path or SDK/legacy fallback. Radix `slider_geometry` exposes the
same values consumed by its live theme adapters.

All mode-bound Shadcn helpers now use the stylesheet captured with their token
snapshot, including switch/slider colors and elevation. Standalone catalog-only
resolvers retain embedded defaults. The legacy tabs list/item color rules remain
look-specific. No common color recipes were introduced.

Replaced bundled literal metric tables have moved into `common`; legacy custom
metric sections remain readable below common values. Token references such as
Button radius, Sidebar width and Table radius remain live look extensions.
Derived SDK geometry is kept as fallback rather than duplicated into literal
TOML tables. This preserves changes to theme metrics and CSS spacing tokens.

## Validation

Tests cover parsing, missing fields, explicit zero, overrides in either builder
order, animation disable, look-to-SDK wiring, custom Shadcn stylesheets and Radix
snapshot/fork isolation. Geometry tests pin mode/state/size/variant baselines,
variant precedence, explicit zero, invalid lengths, runtime tuning and selected
stylesheet paint/inspector parity. Switch tests additionally cover all sizes,
radius settings and variants, display snapping, missing fields and legacy metric
precedence. Slider tests cover mode/state/size/style matrices, explicit thumb
selections, selected stylesheet inspection, runtime edits and legacy radius
compatibility. Existing visual resolver tests remain the baseline.
No GUI launch is needed for these checks.

The migration does not introduce arbitrary layout rules or a CSS engine. SDK
control algorithms, structural row layout and look-specific matching recipes
remain Rust behavior.

## Shadcn Button family

Button and Toggle literal dimensions now live separately in `common.button`
and `common.toggle`, with the existing Shadcn `sm/md/lg` keys. Shared fields cover
height, horizontal/vertical padding, gap, icon size, font size and line height.
Button height remains a reference to `metrics.control.*`; radius remains a
Shadcn metric reference in `button.tokens.<size>` / `toggle.tokens.<size>`.
Common authored dimensions override those references and legacy custom
`button.metrics` / `toggle.metrics` sections. Unset fields retain SDK defaults;
zero remains explicit. Icon role padding, pill radius and local radius presets
remain Rust behavior. Font-family and weight still come from the theme.

The selected look's stylesheet now supplies Button/Toggle paint, typography,
colors and elevation, including the SDK theme adapter and explicit radius
callback. `ShadcnInspect` consumes that same selected configuration and reports
common paths or look-owned token references. Existing theme adapters observe
`replace_theme`; independently loaded looks retain their configuration.

Shared Button typography consumers (including menu items, badges, selectors and
input labels) now read the migrated fields, preserving bundled defaults. Their
family-specific common geometry is resolved separately from inherited typography.
Legacy standalone helpers continue using the bundled stylesheet. Custom Button
settings are scoped to the selected Button family; they do not rewrite general
SDK metric tokens or promise custom settings for all other families.

Regression coverage compares the migrated Button/Toggle output to legacy
fixtures across two CSS themes, both modes, all sizes/styles/roles and every
interaction-state combination. Further checks cover selected configuration
paint/inspection parity, zero, sparse fallbacks, legacy precedence, local radius
presets, live replacement and independent looks.

## Shadcn Checkbox and Radio

The selected Shadcn stylesheet now supplies Checkbox/Radio indicator geometry,
label typography, colors and elevation to their SDK theme adapters and
`ShadcnInspect`. Common `checkbox` fields are `indicator_size`, `glyph_size` and
`gap`; common `radio` fields are `indicator_size`, `dot_size` and `gap`. Shadcn
uses its own `sm/md/lg` keys, with per-size values above general geometry.

Unset dimensions still use `CheckboxScale` / `RadioScale` over the look's SDK
metrics. The bundled Shadcn stylesheet deliberately does not freeze these
SDK-derived defaults into a new literal table. Control height, padding, baseline
shift, Checkbox indicator radius and control radius retain SDK behavior; Radio
remains circular. Configuring an indicator dimension does not implicitly change
other unset dimensions. Authored dimensions snap at the display scale, and
explicit zero overrides fallbacks. Radio rendering suppresses its minimum-size
animated dot when the configured dot size is zero.

Labels retain the shared Button typography curve using the selected stylesheet.
Radio indicator defaults (`filled`, `ring`, `dot`) and color/elevation rules remain
Shadcn-owned. Hover/pressed color policy, focus border precedence, ContentOnly
shadow suppression and disabled render suppression remain intact. Standalone
compatibility helpers continue using the bundled stylesheet.

Tests compare defaults against SDK scales across sizes, both modes, two CSS
themes and multiple display scales. Palette baselines and custom stylesheet
paint/inspection checks cover all styles, selection values and interaction
states. Further checks cover sparse fallbacks, zero, authored paths, Radio
indicator defaults, live replacement and independent looks.

## Shadcn input, surface and utility families

Text Field and Text Area resolve independent minimum height, padding and
typography fields. Text Field also resolves gap and icon size. Popup Menu and
Selector resolve trigger geometry while retaining their existing Button palette
recipes and independent floating-menu size. Text Area and Selector fallbacks
remain their SDK scales, rather than inheriting unrelated Button/Field geometry.
Explicit input look callbacks and popup trigger radius overrides apply last.

| Family | Common inputs | Look/SDK behavior retained |
| --- | --- | --- |
| Progress | Diameter, stroke width, track height, thumb size | State palette, animation and fill layout |
| Stepper | Badge size, track thickness | State colors and step layout |
| Scrollbar | Shell/track/thumb thickness, minimum thumb length, horizontal/vertical length | Radius, enabled/state colors, scroll algorithms |
| Floating / Context Menu | Surface/target padding and width, item geometry, typography, submenu offset | Token-derived radius, shadow/palette recipes, keyboard behavior |
| Badge | Minimum height, padding, gap, icon size, typography | Variant palette, border and radius |
| Card | Padding and section/header/body gaps | Radius, border, colors and elevation |
| Toolbar / Control Group / Table | Shell padding and gap where consumed | Hosted control/row layout and palette recipes |
| Sidebar | Section/item height, item padding, gap, icon size | Width/rail/mobile token references, navigation and state colors |
| Overlay Window | Padding, width bounds, estimated height, typography | Modal/floating recipes and positioning |
| Pager | Button/control dimensions, padding, gaps, typography | Look-owned `minimal`, `minimal-edge`, `numeric` keys and navigation |
| Tooltip | Padding, maximum width, typography | Token-derived radius, palette and interaction |

Accordion, Autocomplete, TreeView, ListBox, Split View and Resizable Panels now
use selected stylesheet rules for their existing palettes, elevation and inherited
metrics. Their SDK layout/algorithms remain structural; unused family geometry
fields were not invented.

Shadcn token snapshots retain an `Arc` to their selected stylesheet. Independent
color copies and token edits preserve it. `replace_theme` replaces the complete
snapshot; existing theme adapters observe the new configuration on subsequent
resolution. Retained token snapshots continue resolving their original config.
The caller remains responsible for notifying views after changes.

Existing inspection entry points preserve compatibility. Added size-aware
Progress, Stepper and Scrollbar inspection methods expose per-size sources;
older methods remain medium-size wrappers. Common authored values report
`style.toml · common.<family>...`; unset values retain token/SDK/legacy provenance.
Regression tests cover selected overrides across remaining families, independent
input geometry, popup trigger typography, explicit zero, sparse legacy fallback,
selected colors in both modes, snapshot lifetime and bundled size baselines.

## Radix families

Radix adapters now consume typed common geometry for Button (also Icon Button
and Toggle), Checkbox, Radio, Text Field, Text Area, Progress, floating menu
content, menu triggers/targets, Tooltip, Toolbar, segmented groups, Avatar,
Badge, Card, Callout and the project overlay-window extension. The SDK owns
optional length validation, per-size/general/fallback resolution and source
paths. The field lists are defined in `theme/stylesheet/geometry.rs`; each is an
actual resolved template input rather than an arbitrary style-property map.

| Family | Migrated inputs | Values that remain recipes or structural behavior |
| --- | --- | --- |
| Button / Icon Button / Toggle | Height, padding, gap, icon size and typography | Radius factors, role-dependent padding/icon rules, Classic shadows and tuning, selected-state colors and transitions |
| Checkbox / Radio | Indicator and glyph/dot dimensions, label gap | Checkbox radius factors, circular radio radius, shadows, state colors and SDK row behavior |
| Text Field / Text Area | Minimum height, padding and typography; field gap/icon size | Radius, variant border suppression, palette/shadow recipes, editing/caret algorithms |
| Progress | Track height | Radius calculated from resolved height, colors, SDK animation phase and fill layout |
| Floating menu | Padding, width, item geometry/typography and submenu offset | Radius tokens, elevation, disabled opacity, highlight colors, SDK keyboard/submenu behavior |
| Popup / Context Menu | Trigger/target geometry and typography | SDK trigger radius override, content colors and shared floating-menu presentation |
| Tooltip | Padding, maximum width and typography | Theme colors, radius tokens and SDK interaction behavior |
| Toolbar | Shell padding and gap | Explicit `ToolbarStyle` wins; separator ratio, hosted controls and navigation remain Rust behavior |
| Segmented group | Default segment height, horizontal padding and typography | Shared-track radius, selected/focus colors and SDK selection |
| Avatar | Diameter, optional icon size and one/two-letter typography | Radius factors, fallback text processing, image cropping and palette recipes |
| Badge / Card / Callout | Badge padding/type, Card padding, Callout spacing/icon/type | Borders, shadows, radii, palette recipes and Ghost negative margin |
| Overlay window | Padding, width bounds, estimated height and typography | Project-specific SDK size semantics, mode backdrop/shadow recipes and overlay positioning |
| TreeView | No new family fields | Its adapter supplies palette and shared label typography; SDK row scale/layout stays structural |

The default Radix button tokens now derive from the bundled button geometry,
removing a second table of heights/padding/gaps/icon sizes. Explicit Button size
`"4"` stays separate from size `"3"` throughout its look resolver. Toggle now also
retains its original `ButtonSize` at the theme boundary; selecting size Four
previously collapsed into SDK `Lg`/size Three. Its height, padding and typography
now use size Four's configuration. Progress radius follows the resolved track
height, and a Ghost Card's negative margin follows its resolved padding.
Avatar icon sizing retains its existing diameter-based floating-point recipe
unless explicitly authored; full radius follows the resolved diameter.

Existing public local overrides continue after common resolution. Toolbar
builders retain `None` for unset style, so ordinary construction inherits the
look; passing `.style(...)` or `toolbar_theme_with` is a deliberate instance
geometry override. Geometry resolves against the selected look on subsequent
renders. No setters launch apps or schedule notifications. Default SDK metric
construction remains a snapshot; family configuration edits do not rewrite the
look's general `MetricTokens` or another family's dimensions.

### Reference audit and pre-existing API gaps

The [Radix Themes component reference](https://www.radix-ui.com/themes/docs/components)
uses component-specific numeric semantic ranges, not a universal small/medium/large
axis. These upstream ranges were checked individually:

| Component | Reference size range | Current Rust adapter |
| --- | --- | --- |
| [Button](https://www.radix-ui.com/themes/docs/components/button) / [Icon Button](https://www.radix-ui.com/themes/docs/components/icon-button) | `"1"`–`"4"` | All four retained, including Toggle's ButtonSize extension |
| [Avatar](https://www.radix-ui.com/themes/docs/components/avatar) | `"1"`–`"9"` | All nine retained; default Three |
| [Badge](https://www.radix-ui.com/themes/docs/components/badge) | `"1"`–`"3"` | All three retained; default One |
| Checkbox, Radio, Text Field, Text Area, Progress | `"1"`–`"3"` | Numeric keys retained; default Two |
| [Card](https://www.radix-ui.com/themes/docs/components/card) | `"1"`–`"5"` | Existing Rust API has One/Two/Three; Four/Five remain a separate parity gap |
| [Dropdown Menu](https://www.radix-ui.com/themes/docs/components/dropdown-menu) / [Context Menu](https://www.radix-ui.com/themes/docs/components/context-menu) | `"1"`–`"2"` for content | Existing factories expose no menu-content size option; the migrated content retains its current size Two |
| [Segmented Control](https://www.radix-ui.com/themes/docs/components/segmented-control) | `"1"`–`"3"` | Existing helper exposes only its default size Two presentation |
| [Callout](https://www.radix-ui.com/themes/docs/components/callout) | `"1"`–`"3"` | Existing helper exposes one soft presentation; no size/variant axis was added |
| [Tooltip](https://www.radix-ui.com/themes/docs/components/tooltip) | No numeric size prop | General geometry only; existing 260 px width preserved, although upstream defaults to 360 px |

Toolbar, TreeView, Toggle and overlay window are SDK/project extensions, not
additional Radix Themes component APIs. Overlay sizes deliberately keep SDK
`sm/md/lg` keys. Reference `Responsive` props describe the upstream breakpoint
API; this migration preserves the existing Rust enums without adding responsive
layout machinery. Other pre-existing gaps, including Progress Classic and
segmented variants, remain outside this geometry migration. New shared fields
do not imply those public props have been implemented.

Validation covers every migrated length field, unknown fields, numeric keys,
explicit zero, per-size/general/fallback precedence and authored source paths.
Resolved-output baselines cover both modes, all existing checkbox/radio radii,
input sizes/variants/interaction states, progress sizes, menu/tooltip geometry,
live edits, fork isolation and local Toolbar/Card/Avatar override precedence.
