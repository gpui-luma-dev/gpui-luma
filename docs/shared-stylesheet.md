# Shared stylesheet contract

The SDK embeds `crates/sdk/assets/style.toml` as the authoritative vanilla
appearance and motion baseline. SDK-only applications need no look crate.
Shadcn and Radix inherit this data and author their own differences.

## Ownership and public APIs

`CommonStylesheet::sdk_default()` exposes the embedded baseline.
`CommonStylesheet::resolve_layers(&[look, application])` overlays sparse documents.
`sheet.theme_tokens(mode, mapping)` creates the existing public `ThemeTokens`
with retained source colors, projected paint colors and a captured stylesheet.
Pass those tokens to SDK `Default*Theme::new` factories and existing themed
templates; third-party looks use the same APIs.

| Data | Typed owner |
| --- | --- |
| Metrics, focus gaps, disabled opacity | `common.tokens.metrics` |
| Font families, weights, text styles | `common.tokens.typography` |
| Semantic light/dark colors, including shell chrome | `common.tokens.palette.light/dark` |
| Shadow layers, including drag previews | `common.tokens.elevation.light/dark` |
| Per-control geometry, proportions and decoration | `common.<family>.geometry/sizes` |
| Tab presentation geometry | `common.tabs.geometry/variants` |
| Semantic state recipe mappings | `common.recipes.<family>` |
| Named OpenType features (four-character tags, integer values) | `common.font_features` |
| Default glyph identities | `common.icons` |
| Picker viewport row budgets | `common.row_limits.<family>` |
| Shared helper/compact-input/drag/layout chrome | `common.shared.geometry`, `CommonStylesheet::shared_geometry()` |
| Title-bar dimensions and platform insets | `common.shell.geometry` |
| Typed animation roles | `common.motion` |

Token default constructors and public title-bar constants are compiled from the
asset at build time. They do not repeat authored numbers or colors in Rust.
The parsed common schema rejects unknown typed fields. Root sections belonging
to a look are left for that look's parser.

## Inheritance, validation and lifetime

Resolution order is SDK baseline → look stylesheet/recipes → programmatic look
customization → explicit consumer overrides. Missing fields inherit independently.
Valid zero/false values and empty arrays remain overrides. Later general geometry
replaces earlier size/variant values for the same field; specifics authored in the
same layer apply last.

Lengths and coefficients must be finite and nonnegative. Opacity is in 0–1.
Font weights are in 1–1000. Shadow blur is nonnegative; offsets/spread may be
negative. `inset` is required for each shadow layer. Row budgets are unsigned
integers; zero is retained, while viewport algorithms ensure at least one usable
row for nonempty content. Invalid configuration fails before mutation.

SDK themes capture immutable token/stylesheet snapshots. Builder-owned geometry,
icons and row limits are captured by `.stylesheet(&sheet)`; subsequent control
size selection resolves the captured size-specific geometry. Rebuild themes or
controls to adopt new snapshots. Bundled look theme adapters resolve their current
look on render; explicit look setters require the host to notify affected views.
Radix clones share edits and forks copy independently.

Instance icon pairs, explicit geometry/row-limit setters, look callbacks and final
template modifiers remain authoritative. Supplying Sidebar content later, including
runtime content replacement, preserves the host's captured disclosure defaults;
panel-local icon pairs or a panel-local stylesheet take precedence.

Motion keeps its existing separate snapshot and reduced-motion policy. An authored
transition replaces the complete role; partial transitions are rejected.

## Colors and arithmetic

Author palette and shadow colors as `"hsla(h, s%, l%, a)"`: hue is degrees,
saturation/lightness are percentages and alpha is 0–1.

```toml
[common.tokens.palette.light.app]
foreground = "hsla(222.22, 47.37%, 11.18%, 1)"
```

Build-time and runtime loading use the same conversion. Explicit tagged
`ColorValue` persistence remains available for other source color spaces; rendering
uses the chosen gamut-mapping policy without discarding source colors.

Numeric fields under `common` accept quoted `"math(...)"` expressions, evaluated
once with `fasteval` before normal field validation:

```toml
[common.switch.sizes.sm]
track_width_scale = "math(32 / 28)" # track width / control height
track_height_scale = "math(13 / 28)"
thumb_scale = "math(9 / 28)"
```

Division is real floating-point division. Literals, scientific notation,
parentheses and `+ - * / % ^` are supported. Variables, token references and
function calls are rejected. Malformed/nonfinite/out-of-f32 results fail with the
field path. Plain numbers remain valid.

## Semantic state recipes

State dispatch remains control behavior: disabled > pressed > hovered > default,
with independent focus/invalid signals. The tokens selected by those recipes are
authored through validated `PaletteRole` references:

```toml
[common.recipes.button]
"state.hover.background" = "state.selected.background"

[common.recipes.checkbox]
"state.selected.background" = "data.positive"
```

A recipe maps a consumed semantic slot to a shared semantic source, rather than
copying a concrete color. All references read the original projected palette, so
swapping two slots cannot cascade. Missing mappings inherit the SDK recipe.
Programmatic token-color edits remain visible through the mapping.

`PaletteRecipes::palette(family, &tokens.palette)` exposes the same operation to
third-party themes. Default mappings preserve the vanilla palette in both modes.
Intentional control-specific behavior remains: selected TreeView rows keep their
selection during hover/press, while Sidebar permits transient hover/press paint.
Disabled active Tabs suppress focus paint.

Shadcn keeps its existing look-owned color/elevation rules and CSS token catalog.
Radix authors ordered button state rules and Classic tuning under
`button_recipes` in its look stylesheet. `Look::set_button_recipes(source)`
loads sparse overrides over that embedded data; a supplied rules array replaces
the ordered list. Matching rules run in order over variant, selected,
high-contrast, mode, interaction-layer and enabled-focus axes. Colors reference
tone/gray steps, contrast or semantic roles. Scale steps are validated in 1–12;
unknown colors/variants/layers/modes and invalid tuning are rejected.
`set_classic_params` remains a final programmatic override. Forks retain recipe
isolation. Regression tests compare every existing variant/state/mode/contrast
combination with the pre-migration output.

Sidebar title/subtitle text uses the selected theme typography. Section small caps
are authored as `sidebar_section = [["smcp", 1]]` under `common.font_features`.
A value of zero disables a feature; an empty array clears the inherited feature
set. Tags must have four ASCII letters/digits. Table emphasis and Stepper numbers
reuse the shared title and label weight tokens, with resolved look overrides final.

## Icons, placement and decoration

`common.icons` uses validated Lucide catalog names or `svg:path/to/icon.svg`.
Pager/Selector/Checkbox/SelectionPanel/Stepper icon objects and disclosure pairs
override captured defaults regardless of builder order. Tabs render custom SVG
accessories through the shared icon renderer. TreeView node icons remain content;
default folder/file icons and Accordion drag handles come from selected themes.
Title-bar window actions use the same icon-role contract.

Selector and PopupMenu placement share resolved popup margins, including centered
row alignment. Tooltip pointer geometry, target gaps and shortcut styling,
OverlayWindow placement/header/shell geometry, and SlidePanel resize dividers
resolve through their selected look. Table divider width participates in both
paint and public height helpers.

Slider hit testing resolves the same theme thumb dimensions as paint, and thumb borders use the selected metric stroke. Scrollbar paint retains explicit zero thickness instead of restoring a fixed visual minimum. Radial
dial size/radius, Progress sweep/shimmer fractions, Stepper label/glyph decoration,
input carets/resize chrome, Toolbar/SplitView/DockSplitter strokes and Sidebar
shell/rail spacing are typed stylesheet inputs. Opacity is never pixel-snapped;
logical geometry is snapped only where the existing control recipe requires it.

## SDK-only and custom-look example

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

Use the same resolved common sheet in a custom look, create its theme factories,
and attach existing SDK themed templates. Consumers keep SDK behavior, focus and
events. See [Create a Look](create-look.html#baseline) for compiled examples.

## Audit and retained Rust

[The family inventory](sdk-style-inventory.md) covers all SDK controls and shared
styling helpers. [The CSV](sdk-style-candidates.csv) preserves historical source
expressions and their classification/rationale; line zero denotes a family-level
summary. Superseded rows describe removed expressions, not current source.
`scripts/check_sdk_style_inventory.py` rejects unclassified entries.

Rust retains resolution/validation, semantic state dispatch, transparency/empty
sentinels, circular/centering/path mathematics, measured layout/virtualization,
accessibility/hit targets and pointer/keyboard/scroll behavior. Test fixtures
deliberately retain literal expected output. These are not a second authored
appearance table.

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
