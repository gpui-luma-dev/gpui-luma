# SDK styling migration inventory (#146)

## Completed shared ownership

| Owner | Authoritative stylesheet fields | Status |
| --- | --- | --- |
| MetricTokens and nested metric types | `common.tokens.metrics.*` | Migrated; typed validation and compiled constructors |
| LumaTypography and nested font/text types | `common.tokens.typography.*` | Migrated; numeric weights validated in 1–1000 |
| LumaPalette and semantic palette groups | `common.tokens.palette.light/dark.*` | Migrated; existing tagged ColorValue storage |
| LumaElevation and shadow layers | `common.tokens.elevation.light/dark.*` | Migrated; finite geometry, nonnegative blur |
| Motion roles | `common.motion.*` | Existing #145 baseline and snapshot contract retained |
| Progress dimensions | `common.progress.sizes.sm/md/lg` | Migrated |
| Stepper dimensions | `common.stepper.sizes.sm/md/lg` | Migrated |
| Scrollbar dimensions and lengths | `common.scrollbar.sizes.sm/md/lg` | Migrated |
| Tooltip default and compact geometry | `common.tooltip.geometry`, `common.tooltip.sizes.compact` | Migrated |
| Tabs list padding and indicator | `common.tabs.geometry` | Migrated; token-derived list gap remains a fallback |
| Tree View and Accordion scales | `common.tree_view.geometry`, `common.accordion.geometry` | Proportions, indentation, content padding, glyph sizes and explicit layout overrides migrated; paint/inspection share resolution |
| Shadow painting mode | `common.tokens.elevation.*.*.layers[].inset` | Explicit false/true migrated; inset shadows do not enlarge outer projection |
| Choice scales | `common.checkbox`, `common.radio`, `common.switch` geometry and size recipes | Proportions, indicator radius, glyph inset and row padding migrated; pixel snapping remains Rust |
| Input themes | `common.textfield`, `common.textarea` | Default factories consume the common geometry over shared token fallbacks |
| Slider | `common.slider.geometry`, size-specific dimensions | Width and proportional floor/thumb recipes migrated; explicit thumb size uses its own size key |
| Floating menu | `common.floating_menu.geometry` | Width, proportional spacing/height recipes, disabled opacity and separator metrics migrated |
| Selection panel | `common.selection_panel.geometry` | Shared menu baseline with independent final overrides and label typography |
| Context menu target | `common.context_menu.geometry` | Target width migrated; remaining metrics come from shared tokens |
| Control group shell | `common.control_group.geometry` | Default shell padding and gap migrated |
| Dock splitter | `common.dock_splitter.geometry` | Hit width and visible line width migrated |
| Resizable panels | `common.resizable_panels.geometry.disabled_opacity` | Disabled opacity migrated and validated in 0–1 |
| Sidebar metric scale | `common.sidebar.geometry` | Widths, rail hit width, popover offset and default item dimensions migrated |
| Overlay window | `common.overlay_window`, `common.tokens.elevation.*.overlay_window` | Dimensions, backdrop opacity and nonmodal shadow migrated |

## Family coverage

These rows track the remaining migration; they do not claim all family literals
have been classified. Shared token migration benefits every token-backed theme.

| SDK family | Migration status |
| --- | --- |
| button/family | Default Button/Icon/Toggle themes consume common geometry; state and template recipes still need audit. |
| button/icon_button | Pending control-specific schema/factory/template audit. |
| button/split_button | Pending control-specific schema/factory/template audit. |
| choice/checkbox | Factory and common indicator/row recipes migrated; state and template chrome still need audit. |
| choice/control_group | Shell defaults migrated; item and template recipes still need audit. |
| choice/icon_group | Pending control-specific schema/factory/template audit. |
| choice/radio_button | Factory and common proportional indicator/dot recipes migrated; state and template chrome still need audit. |
| choice/radio_group | Pending control-specific schema/factory/template audit. |
| choice/switch | Factory and proportional track/thumb recipes migrated; explicit dimensions win; template chrome still needs audit. |
| choice/toggle | Pending control-specific schema/factory/template audit. |
| choice/toolbar | Pending control-specific schema/factory/template audit. |
| collection/listbox | Pending control-specific schema/factory/template audit. |
| collection/table | Pending control-specific schema/factory/template audit. |
| collection/tree_view | Row scale factory and both look adapters wired; state and template chrome still need audit. |
| layout/dock_splitter | Factory hit target and visible line migrated; template chrome still needs audit. |
| layout/resizable_panels | Factory disabled opacity migrated; template grip/layout styling still needs audit. |
| layout/scroll_container | Pending control-specific schema/factory/template audit. |
| layout/scrollbar | Fixed geometry migrated; template/scroll algorithms still need classification. |
| layout/split_view | Pending control-specific schema/factory/template audit. |
| navigation/accordion | Scale factory and Shadcn paint/inspection wired; state and template chrome still need audit. |
| navigation/pager | Pending control-specific schema/factory/template audit. |
| navigation/sidebar | Default metric scale migrated; engine/state and template recipes still need audit. |
| navigation/stepper | Fixed geometry migrated; template decoration still needs audit. |
| navigation/tabs | List padding/gap and indicator baseline wired; other recipes still need audit. |
| overlay/context_menu | Target geometry wired and width migrated; shared menu factory wired; template styling still needs audit. |
| overlay/floating_menu | Factory geometry, proportional recipes, opacity and separators migrated; template icon/state recipes still need audit. |
| overlay/overlay_window | Factory dimensions/backdrop opacity and nonmodal shadow migrated; template recipes still need audit. |
| overlay/popover_button | Pending control-specific schema/factory/template audit. |
| overlay/popup_menu | Pending control-specific schema/factory/template audit. |
| overlay/slide_panel | Pending control-specific schema/factory/template audit. |
| overlay/tooltip | Vanilla and compact geometry migrated; presentation coverage still needs audit. |
| range/progress | Fixed geometry migrated; template decoration still needs audit. |
| range/slider | Factory and proportional sizing migrated; pointer algorithms remain Rust; template chrome still needs audit. |
| selection/autocomplete | Pending control-specific schema/factory/template audit. |
| selection/combobox | Pending control-specific schema/factory/template audit. |
| selection/search_selector | Pending control-specific schema/factory/template audit. |
| selection/selection_panel | Shares resolved menu baseline with independent common geometry; template styling still needs audit. |
| selection/selector | Pending control-specific schema/factory/template audit. |
| selection/selector_list | Pending control-specific schema/factory/template audit. |
| text/textarea | Factory consumes common geometry; editing behavior unchanged; template and icon defaults still need audit. |
| text/textfield | Factory consumes common geometry; editing behavior unchanged; template and icon defaults still need audit. |

## Remaining literal triage

[sdk-style-candidates.csv](sdk-style-candidates.csv) records numeric candidates,
source locations, context and proposed ownership across controls, themes, layouts,
rendering and shared infrastructure. Its proposed fields are unreviewed hints.
Candidates include mathematical/structural values and must not be migrated blindly.
The file excludes test modules and comments. Integer-only styles, icon identifiers,
state/variant recipes and token references also require manual audit.

Values that stay in Rust require a specific classification: interaction behavior,
accessibility, structural layout algorithm, mathematical identity or derived
calculation. Transparent colors used as absence sentinels, clamps at zero, and
half/twice dimensions used to calculate centers/extents are generally mathematical;
opacity strengths, inset ratios, default icon choices and visual proportions are
configurable appearance even when written as arithmetic.

Before #146 is complete, classify every candidate, audit nonnumeric styling,
extend the existing common family schemas, wire remaining default factories and
helpers, remove duplicate authored defaults and verify bundled-look output.
