# SDK styling migration inventory (#146)

## Completed shared ownership

| Owner | Authoritative stylesheet fields | Status |
| --- | --- | --- |
| MetricTokens and nested metric types | `common.tokens.metrics.*` | Migrated; typed validation and compiled constructors |
| LumaTypography and nested font/text types | `common.tokens.typography.*` | Migrated; numeric weights validated in 1–1000 |
| LumaPalette and semantic palette groups | `common.tokens.palette.light/dark.*` | Migrated; readable HSLA authoring, tagged ColorValue runtime storage |
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
| Context menu target | `common.context_menu.geometry` | Target width, border, disabled opacity and popup window margin migrated across SDK/Shadcn/Radix; remaining metrics come from shared tokens |
| Control group shell | `common.control_group.geometry` | Default shell padding and gap migrated |
| Dock splitter | `common.dock_splitter.geometry` | Hit width and visible line width migrated |
| Resizable panels | `common.resizable_panels.geometry.disabled_opacity` | Disabled opacity migrated and validated in 0–1 |
| Sidebar metric scale | `common.sidebar.geometry` | Widths, rail hit width, popover offset and default item dimensions migrated |
| Overlay window | `common.overlay_window`, `common.tokens.elevation.*.overlay_window` | Dimensions, backdrop opacity and nonmodal shadow migrated |
| Table factory | `common.table.geometry` | SDK surface padding, proportional vertical padding, row hover opacity, row radius, frame/header strokes and header padding ratio migrated; state/column/template recipes remain |
| Selector trigger | `common.selector.geometry` | SDK dimensions, typography, radius and popup offset migrated; Shadcn paint/inspection share the common geometry |
| Selector list panels | `common.selector_list.geometry/sizes` | SDK panels reuse floating-menu recipes; independent dimensions, typography and radii override afterward; bundled look adapters use the same seam |
| Pager | `common.pager.geometry/sizes` | SDK factory dimensions/typography/radius/opacity and proportional padding/width recipes migrated; Shadcn fixed defaults removed from Rust; page-size popup chrome and paging-table label gap migrated; remaining template/icon audit remains |
| Popup Menu / Split Button trigger | `common.popup_menu.geometry/sizes` | SDK factory now applies common dimensions/typography/radius/offset; bundled looks share final resolution; consumer radius wins |
| Popover placement | `common.popover.geometry/sizes.button/panel` | Raw builders capture defaults; explicit consumer offset/margin wins independently of method order |
| Slide Panel | `common.slide_panel.geometry`, shared resize presets | Captured side/edge sizing and viewport ratio; bundled look state factories inherit; resize divider thickness migrated; remaining shell border audit pending |
| Resize-handle presets | `common.resizable_panels.geometry/sizes` | Lane, hit target, grip extents and radius migrated; SDK/Shadcn paint and available inspection wired |
| Toolbar factory | `common.toolbar.geometry/sizes` | SDK applies existing padding/gap seam; other styling stays token-backed pending template/state audit |
| Group template chrome | `common.control_group.geometry` | Vertical gap, border width, radius and disabled-item opacity migrated; themed looks carry resolved chrome; explicit builder styles and final modifiers retain precedence |
| RadioGroup templates | `common.radio_group.geometry/sizes.vertical/horizontal` | Group/item gaps and disabled opacity migrated; wrappers capture styles; Radix authors its compact horizontal difference |
| IconGroup capsule | `common.icon_group.geometry/sizes.toolbar` | Shared public template inherits group shell dimensions and pill-radius token; independent geometry replaces fixed template modifiers |
| Selector/selection panel chrome | `common.selector_list`, `common.selection_panel`, metric border and Popover tokens | Border width and disabled opacity migrated across Selector, ComboBox, Autocomplete, SearchSelector and SelectionPanel templates; popup margins/offsets inherit shared placement and allow final independent overrides |
| Composite-input affordances | `common.autocomplete`, `common.combobox`, `common.search_selector` | Captured clear/search slot geometry, status typography dimensions, width reservations and search panel chrome migrated; selected look builders and Studio previews use the same resolved fields |

## Family coverage

These rows track the remaining migration; they do not claim all family literals
have been classified. Shared token migration benefits every token-backed theme.

| SDK family | Migration status |
| --- | --- |
| button/family | Default Button/Icon/Toggle themes consume common geometry; state and template recipes still need audit. |
| button/icon_button | Pending control-specific schema/factory/template audit. |
| button/split_button | Uses shared PopupMenu factory/template geometry; face-specific states and divider/icon/template recipes still need audit. |
| choice/checkbox | Factory and common indicator/row recipes migrated; state and template chrome still need audit. |
| choice/control_group | Shell spacing/radius/border and default-item opacity migrated; builder/render models carry captured styles. Item marker, state and remaining template recipes still need audit. |
| choice/icon_group | Capsule template now inherits group chrome and pill token, with independent overrides. Reused control-group item state/marker audit remains. |
| choice/radio_button | Factory and common proportional indicator/dot recipes migrated; state and template chrome still need audit. |
| choice/radio_group | Vertical/horizontal and item gaps plus disabled opacity migrated; both wrappers capture styles. Radio indicator size/role and remaining state/template audit pending. |
| choice/switch | Factory and proportional track/thumb recipes migrated; explicit dimensions win; template chrome still needs audit. |
| choice/toggle | Pending control-specific schema/factory/template audit. |
| choice/toolbar | SDK factory now honors existing common padding/gap. Radius/separator token bindings and template/state recipes still need audit. |
| collection/listbox | Raw SDK flow/control modules own item placement, viewport/scroll/selection algorithms; visual item chrome is host/look-owned. Keep flow arithmetic in Rust. Host visual defaults still need audit. |
| collection/table | SDK surface/row factory recipes, shell/header strokes, header padding ratio and drag preview/marker geometry and disabled-row opacity and column resize chrome, text-column padding and muted-cell opacity migrated; remaining column/template styling and full state recipes still need audit. |
| collection/tree_view | Row scale factory and both look adapters wired; state and template chrome still need audit. |
| layout/dock_splitter | Factory hit target and visible line migrated; template chrome still needs audit. |
| layout/resizable_panels | Opacity, lane/hit-target and grip extents/radius migrated. Split placement and collapse/keyboard thresholds are layout/interaction; Frame and split divider widths now use shared resizable-panel geometry. |
| layout/scroll_container | Pending control-specific schema/factory/template audit. |
| layout/scrollbar | Fixed geometry migrated; template/scroll algorithms still need classification. |
| layout/split_view | Pending control-specific schema/factory/template audit. |
| navigation/accordion | Scale factory and Shadcn paint/inspection wired; state and template chrome still need audit. |
| navigation/pager | Factory recipes/dimensions/typography/radius/opacity migrated; page-size trigger width, icons, button-state/template styling still need audit. |
| navigation/sidebar | Default metric scale migrated; engine/state and template recipes still need audit. |
| navigation/stepper | Fixed geometry migrated; template decoration still needs audit. |
| navigation/tabs | List padding/gap and indicator baseline wired; other recipes still need audit. |
| overlay/context_menu | Target geometry, border, disabled opacity and popup placement migrated; shared menu factory wired; remaining menu styling still needs audit. |
| overlay/floating_menu | Factory geometry, proportional recipes, opacity and separators migrated; template icon/state recipes still need audit. |
| overlay/overlay_window | Factory dimensions/backdrop opacity and nonmodal shadow migrated; template recipes still need audit. |
| overlay/popover_button | Placement defaults migrated for button/panel builders; focus/dismissal are behavior and supplied trigger/content chrome is host-owned. |
| overlay/popup_menu | Factory dimensions/typography/radius/offset migrated and shared with looks; state/icon/template recipes still need audit. |
| overlay/slide_panel | Side/edge defaults and viewport ratio captured; resize grip shares migrated presets. Resize divider thickness migrated; remaining shell border audit pending; movement/interpolation and resize clamps stay behavioral/layout code. |
| overlay/tooltip | Vanilla and compact geometry migrated; presentation coverage still needs audit. |
| range/progress | Fixed geometry migrated; template decoration still needs audit. |
| range/slider | Factory and proportional sizing migrated; pointer algorithms remain Rust; template chrome still needs audit. |
| selection/autocomplete | Popup chrome plus clear geometry, root/status gaps, width reservations and theme-owned status typography migrated. Icon identity, viewport row-count defaults and remaining state/variant recipes still need audit. |
| selection/combobox | Popup chrome, clear/arrow affordance geometry and matching trigger-width reservations migrated. Icon identity, clear-icon color semantics and remaining state/template recipes still need audit. |
| selection/search_selector | Popup chrome, search/trailing icon sizes, input padding/floor and search panel reserve migrated; viewport budget uses resolved placement. Icon identity, hard-coded trailing color and remaining state/viewport recipes still need audit. |
| selection/selection_panel | Menu baseline plus independent geometry, radii, border width and disabled opacity reach SDK/Shadcn templates. Selection icons, scrollbar reservation and remaining recipes still need audit. |
| selection/selector | SDK trigger geometry, typography and popup offset migrated; Shadcn paint/inspection wired; icon/state/template recipes remain. |
| selection/selector_list | Shared public menu-to-panel factory owns metric borders, menu opacity and Popover placement before independent overrides. Template border/opacity and border-dependent inset calculations migrated; icon audit remains. |
| text/textarea | Factory consumes common geometry; editing behavior unchanged; template and icon defaults still need audit. |
| text/textfield | Factory consumes common geometry; editing behavior unchanged; template and icon defaults still need audit. |

## Remaining literal triage

[sdk-style-candidates.csv](sdk-style-candidates.csv) records numeric candidates,
source locations, context and proposed ownership across controls, themes, layouts,
rendering and shared infrastructure. Rows still marked `needs classification` are unreviewed hints; reviewed rows
record their migrated field or the rationale for staying in Rust. Source locations
and context are historical audit snapshots, retained to identify removed literals
after migration. Later reviews also include shorthand border/gap calls.
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
