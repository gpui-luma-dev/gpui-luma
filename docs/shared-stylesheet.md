# Shared stylesheet ownership and migration


## Vanilla token baseline (#146)

The SDK stylesheet now owns shared metric, typography, light/dark palette and
light/dark elevation defaults under `[common.tokens]`. The existing public token
types remain the runtime representation. The build script compiles the embedded
data into their infallible default constructors; malformed packaged values fail
the build. There is no second authored token table in Rust. Runtime token tables
inherit missing fields from the same asset and reject unknown fields, invalid
lengths, invalid font weights and invalid colors/shadows.

Author palette and shadow colors consistently as `"hsla(h, s%, l%, a)"`:
hue is in degrees, saturation/lightness are percentages, and alpha is 0–1.
For example:

```toml
[common.tokens.palette.light.app]
foreground = "hsla(222.22, 47.37%, 11.18%, 1)"
```

The build and runtime loaders share the same HSLA conversion. Tagged
`ColorValue` persistence remains available for explicit source color spaces.

Numeric fields under `[common]` also accept quoted `"math(...)"` expressions,
evaluated once with `fasteval` before the usual field validation:

```toml
[common.switch.sizes.sm]
track_width_scale = "math(32 / 28)" # width / default control height
track_height_scale = "math(13 / 28)"
thumb_scale = "math(9 / 28)"
```

Ordinary numbers remain valid. Expressions support numeric literals (including
scientific notation), parentheses and `+ - * / % ^`; division uses floating-point
arithmetic. Variables, token references and function calls are not supported.
Malformed expressions, nonfinite results and results outside the f32 range are
rejected with the field path. Existing field constraints still apply (for example,
nonnegative dimensions and opacity in 0–1). Evaluation uses the same code for
compiled defaults and runtime loading, and respects `fasteval`'s parser limits.

`CommonStylesheet::resolve_layers(&[look_toml, application_toml])` validates each
layer and resolves SDK baseline → look → application. A later layer's general
geometry outranks earlier size/variant geometry; its own size/variant values apply
last. Explicit zero, false, and empty arrays replace prior values. Motion
transitions remain atomic: incomplete transitions are rejected rather than filled
from a previous transition. Existing motion policy and builder snapshots remain.

Call `sheet.theme_tokens(mode, mapping)` to capture shared tokens and the resolved
stylesheet for SDK theme factories. Public fields allow programmatic customization
after stylesheet resolution. Existing theme snapshots retain their original
configuration; rebuild tokens/themes to apply edits. Control-specific builder
callbacks and template modifiers still apply at the instance boundary.

Button/Icon/Toggle common geometry, Progress, Stepper, Scrollbar, Tabs list/indicator
geometry and Tooltip defaults now
consume the captured baseline. Shadcn and Radix load their common sections over
the SDK baseline, and matching migrated values have been removed from look files.
Shadcn's old `progress.metrics` fields no longer outrank inherited common geometry;
author overrides under `common.progress` instead. Radix explicitly authors its
surface-tab padding, which differs from vanilla.

```rust
use gpui_luma::{
    color::GamutMapping,
    controls::progress::{DefaultProgressTheme, ProgressTheme},
    theme::{ControlSize, ThemeMode, stylesheet::CommonStylesheet},
};
let sheet = CommonStylesheet::resolve_layers(&[
    "[common.progress.sizes.md]\ntrack_height = 9",
])?;
let tokens = sheet.theme_tokens(ThemeMode::Light, GamutMapping::CssLocalMinde)?;
let theme = DefaultProgressTheme::new(tokens);
assert_eq!(theme.resolve(true, ControlSize::Md).track_height, 9.0);
```

This is the token/factory stage of #146. The older family notes below describe
remaining recipe/fallback ownership, not a completed migration of every control.
See [the migration inventory](sdk-style-inventory.md) for outstanding work.

Issue #105 now has a shared SDK contract for configurable control geometry and
tab body motion. Shadcn and Radix supply separate `assets/style.toml` files using
`CommonStylesheet`; their palettes, numeric size semantics, token references and
visual recipes remain look-owned. Both paint and available inspectors resolve
against the selected configuration. SDK templates consume the resolved inputs.

## Table and selector factory coverage

The SDK Table factory reads `common.table` for surface padding, proportional
`padding_y_scale`, `row_hover_opacity` (0–1), and `row_radius`. Explicit
`padding_y` overrides the proportional result. Existing table state selection
and consumer look callbacks retain their precedence.

The SDK selector trigger now applies `common.selector` dimensions, typography,
`radius`, and `menu_offset_y`. `menu_offset_scale` multiplies the resolved trigger
gap; an explicit offset wins, including zero. Shadcn uses the same
geometry for paint and available inspector fields.

SDK selector-list panels inherit the shared floating-menu recipes. Author
independent panel overrides under `common.selector_list.geometry` or `.sizes`:
`padding`, `min_width`, `item_height`, `item_padding_x`, `item_gap`,
`item_icon_size`, `font_size`, `line_height`, `radius`, and `item_radius`.
`SelectorItemsPanelLook::with_stylesheet` exposes this final resolution step to
third-party looks. Shadcn uses semantic size keys; the current Radix default
selector panel uses its existing size Two composition. Icon/state recipes and the remaining template audit are still outstanding.

Raw SDK ListBox flow values describe host-supplied item geometry and scrolling,
not a second set of visual defaults. Its placement/virtualization arithmetic
stays in Rust; host/look-owned chrome needs a separate appearance audit.

## Pager and overlay defaults

SDK Pager now consumes `common.pager`, including `radius` and `disabled_opacity`.
The `minimal`, `minimal-edge`, and `numeric` size keys select presentations.
`padding_x_inset_scale` subtracts a multiple of `spacing.s1` from small-control
horizontal padding (clamped at zero); `padding_y_gap_scale` adds a multiple of the
small-control gap to vertical padding; `button_extra_width_scale` adds a multiple
of `spacing.s1` to the button's minimum width. Explicit dimensions win. Shadcn's
existing pager differences stay in its stylesheet rather than a second Rust table.

`common.popup_menu` now configures SDK trigger dimensions, typography, radius and
popup offset, and the bundled looks use `PopupMenuLook::with_stylesheet` for the
same resolution step. `menu_offset_scale` multiplies the resolved trigger gap;
`menu_offset_y` overrides it, including zero. Consumer radius overrides apply last.
SplitButton shares this PopupMenu theme/template path.

`PopoverButtonBuilder::stylesheet(&sheet)` and `PopoverPanelBuilder::stylesheet(&sheet)`
capture `common.popover` placement defaults. Use the `button` and `panel` size keys
for their distinct `offset_y`; `window_margin` is shared. Explicit builder offsets
and margins win regardless of whether `.stylesheet(...)` comes before or after them.
Dismissal, focus behavior and content stay owned by the caller.

`SlidePanelState::stylesheet(&sheet)` captures `common.slide_panel` geometry:
`side_width`, `edge_height`, and `edge_max_viewport_ratio` (0–1). Explicit size
configuration remains authoritative; fallback edge height is capped by the
available viewport. Both bundled-look state factories capture this configuration.
Existing states retain their visual snapshot when the original stylesheet changes.

Shared resize-handle presets now live in `common.resizable_panels.sizes.sm/md/lg`:
`lane_width`, `hit_target`, `grip_cross_axis`, and `grip_main_axis`, with shared
`grip_radius` under `.geometry`. `ResizeHandleSize::metrics_with_stylesheet` exposes
the same resolution to custom looks. SDK and Shadcn themes feed those resolved
metrics to the resize template; Shadcn's inspector reads the same stylesheet.
The one-pixel divider and hit-target minimums still need their final visual versus
interaction classification.

SDK Toolbar also applies its existing common padding and gap overrides. Template
borders, icons, state/variant token recipes, and the remaining family audits are
still outstanding; this does not complete #146.

## Group and selection template styling

`ControlGroupBuilder::stylesheet(&sheet)` and `RadioGroupBuilder::stylesheet(&sheet)`
capture shared styles for the group and its item render models. Missing values
inherit SDK defaults. Group border width, vertical gap, radius and disabled opacity
are authored in `common.control_group`. The resolved `ControlGroupListLook` carries
all shell dimensions so a supplied theme retains its choices unless the builder
explicitly applies a stylesheet. Consumer template modifiers run last.

Radio templates use `common.radio_group.geometry.item_gap` and `.disabled_opacity`,
with per-orientation `gap` under `.sizes.vertical` / `.sizes.horizontal`. Radix
keeps its existing compact horizontal gap as a look stylesheet difference.
`icon_group::icon_toolbar_template` shares the group shell, uses the shared pill
radius by default, and then applies `common.icon_group` dimensions/radius under
`.geometry` or `.sizes.toolbar`. Shadcn reuses this public SDK template.

Selector panels share `SelectorItemsPanelLook::from_menu`: border width comes from
`MetricTokens::border_width.default`, disabled opacity from the resolved menu, and
popup offset/window margin from the shared Popover panel placement. Independent
`common.selector_list` overrides apply afterward, including `border_width`,
`disabled_opacity`, `popup_offset_y`, and `window_margin`. Selector/ComboBox/
Autocomplete/SearchSelector shells and disabled rows consume these resolved values.
Selector alignment/inset calculations account for the actual border thickness.

SelectionPanel independently supports `border_width`, `disabled_opacity`, `radius`
and `item_radius` through `common.selection_panel`, after the shared menu/panel
baseline. Both SDK and Shadcn factories resolve these fields. Explicit zero removes
a border, margin, gap or disabled-item opacity; invalid opacity is rejected.
Icons, search/clear affordance geometry and the remaining state/variant recipes are
still pending the broader audit.

## Composite input affordances

Autocomplete, ComboBox and SearchSelector builders expose `.stylesheet(&sheet)`
to capture their typed common geometry. Size-specific overrides resolve against
the builder's final size. Existing text-field/popup providers and consumer template
modifiers stay independent. Shadcn builders forward their selected common styles;
Studio previews pass the same resolved geometry through the public render models.

`common.autocomplete` owns `gap`, `status_gap`, clear-button `clear_inset`,
`clear_width`, `clear_icon_size`, fallback `trigger_width`, and `popup_extra_width`
(the extra width reserved beyond label and menu padding). Status typography comes
from the selected theme's `text.scale.sm`; optional `status_font_size` and
`status_line_height` override dimensions while preserving the theme's font weight.

`common.combobox` owns `affordance_inset`, `affordance_width`,
`affordance_icon_size`, `affordance_gap` and `trigger_extra_gap`. The clear button
moves past the arrow slot plus the configured gap. Minimum trigger measurement
uses the same dimensions and grows when the configured inter-affordance gap needs
more space, retaining stock width reservation for the built-in defaults.

`common.search_selector` owns the search slot/icon dimensions, extra trigger width
reservation (`trigger_icon_extra_width`, `trigger_extra_gap`), search-field padding
and separator thickness, the field's vertical padding reduction and height floor,
and `trailing_icon_size`. `search_chrome_height` is the estimated search/chrome
reservation used when filling the popup viewport; it defaults to the previous 57px
reservation. Row budgeting now also reads the panel's configured popup offset and
window margin, and guards zero-height rows before division.

These dimensions accept ordinary numbers or `"math(...)"`. Explicit zero remains
valid, missing fields inherit SDK defaults, and negative/nonfinite dimensions are
rejected. Icon identifiers, the SearchSelector trailing-icon color and remaining
viewport-count/state recipes still need the broader appearance audit.

## Additional default factories

Choice scales now expose `compute_with_stylesheet` using the same typed common
geometry as the look adapters. Their proportional coefficients are authored as
`indicator_scale`, `dot_scale`, `track_width_scale`, `track_height_scale` and
`thumb_scale`; explicit dimensions take precedence. Checkbox `glyph_inset` is
snapped before calculating the remaining glyph box, preserving display-scale
behavior. Shared metric edits still affect proportional dimensions.

Text Field and Text Area default themes now apply common geometry after resolving
their shared token inputs. Slider width, target-height floor and default thumb
proportion come from the baseline; explicit thumb selection resolves its own size
key. Floating Menu owns shared proportional row/padding recipes and validated
opacity/separator fields. Selection Panel reuses that baseline and applies its
own `common.selection_panel` geometry afterward, retaining label typography.

Context Menu, Control Group, Dock Splitter, Resizable Panels and Overlay Window
factories now consume their common geometry. The nonmodal overlay shadow is a
shared elevation token; both modes retain their original values. Bundled looks
author their backdrop-opacity and menu-inset differences. Sidebar hosts can call
`SidebarMetricScale::from_stylesheet` and then apply explicit host edits.

Calling `theme_tokens` on a sparse parsed sheet now fills the complete baseline,
including size/variant fields, while preserving explicit zero. Themes retain an
immutable snapshot. Shadcn Switch no longer reads legacy `switch.metrics`; author
its overrides in `common.switch`. Inherited SDK size presets remain separate from
Radix's numeric semantic sizes; the adapters continue selecting numeric keys.

Geometry coefficient fields are finite and nonnegative. Opacity fields are finite
and constrained to 0–1. Absolute dimensions override proportional derivation;
clamping, pixel snapping, center calculations and interaction behavior stay in
Rust. Tree View and Accordion use the same geometry seam for their proportional rows,
indentation, content padding and icon sizes. SDK themes capture their resolved
configuration; bundled look adapters resolve their current configuration on render.
Inspection preserves authored paths while reporting the final snapped paint values.
Accordion's logical icon/chevron dimensions keep their original snapping behavior.

Shadow layers now include a required boolean `inset`, authored explicitly in the
baseline. The public constructors accept it as their final argument. An explicit
false replaces true through stylesheet layering and reaches GPUI unchanged; inset
shadows do not reserve outer projection space. Control-specific template/icon/state
recipes still require the audit tracked in the inventory.

## Ownership matrix

| Input or machinery | Current source | Ownership and migration decision |
| --- | --- | --- |
| Post-drop item motion | `[common.motion.<control>.drop_layout]`, instance `.motion(...)` | SDK keyed geometry executes the configured transition; commits remain synchronous. |
| Incoming tab panel motion | `[common.motion.tabs.content]`, instance `.motion(...)` | SDK instant baseline; Shadcn and Radix supply 300 ms cubic out. |
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
| Color, metric and typography provenance | SDK `theme/provenance.rs`, Shadcn inspectors | Existing shared value types remain. Motion exposes typed profiles and transition descriptions. |

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

Motion resolves the SDK baseline, look stylesheet roles, programmatic look edits,
and instance `.motion(...)` edits, in that order. `motion_enabled(false)` makes
motion immediate without erasing the authored profile. Inspect the captured
roles with `motion_profile()` and transitions with `Transition::description()`.
Look builders capture defaults during spawn. SDK builders accept the corresponding
profile through `.motion_defaults(...)`. Existing controls keep their snapshot.
Geometry is resolved on subsequent renders; callers notify
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

## Drop-layout motion (#140)

`[common.motion.<control>.drop_layout]` configures post-drop motion. SDK collection
builders accept `.motion(|m| m.drop_layout = Transition::spring(300.0, 28.0))`.
Use `Transition::instant()` to disable one role, or `motion_enabled(false)` to
disable all roles. Reduced motion settles immediately. Existing controls retain
their captured look profile.

The SDK commits collection state and lifecycle events before animation. The
motion module owns bounded keyed geometry, while looks supply transitions and
row appearance. Only mounted local positions animate; there are no retained ghost
views or extra off-screen construction. Transfer destinations show new keys at
their final positions while displaced survivors animate. Native-dispatch and
paint-output tests cover timing, interruption and final layout. See
[Drag and drop](drag-drop.html#drop-motion) for the per-control coverage and limits.

## Typed motion roles

`[common.motion]` is SDK-owned and shared by vanilla SDK applications and looks.
See [Motion](motion.html) for the schema, role list, and usage examples.
`CommonStylesheet::motion_defaults()` overlays authored typed roles onto the vanilla SDK baseline.
Programmatic look edits and instance overrides then apply as before. An authored
role replaces a complete transition; missing roles inherit. Instant transitions
and zero durations disable a role. Timed transitions require duration and easing;
springs require positive finite stiffness and damping. Invalid fields fail parsing.

Look authors can start with public `MotionDefaults::default()` and overlay
`sheet.motion.apply_to(&mut defaults)`. SDK builders accept the same component
profiles through `motion_defaults(...)`, without any dependency on a look crate.
`Transition::description()` exposes built-in parameters or identifies custom
motion.

## Embedded SDK baseline

The SDK embeds `crates/sdk/assets/style.toml` as the authoritative vanilla motion
baseline. Both look stylesheets inherit it and author only their overrides.
`CommonStylesheet::sdk_default()` exposes the parsed asset and
`MotionDefaults::vanilla()` exposes the cached compiled profiles. Control profile
`Default` implementations use those profiles; motion values are not repeated in
Rust. Existing public duration constants are generated from the file at build time.
`CommonStylesheet::default()` is still an empty override sheet, and resolving its
motion inherits the SDK baseline. `common.motion.transition` configures the
standalone `Transition::default()`; controls use their explicit typed roles.
Shared token defaults now come from `common.tokens`; remaining control-specific
recipes are tracked in the migration inventory.
