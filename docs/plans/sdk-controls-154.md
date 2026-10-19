# SDK controls app: inventory and first slice (#154)

## Scope and relationship to #149

Issue #154 refines #149 into **one** application, `apps/sdk-controls`
(`luma-sdk-controls`). The website navigation is the authority for group names,
labels, and ordering. The SDK app uses native SDK defaults, including its chrome;
Studio's Shadcn factories and CSS theme catalog do not belong in its dependency
graph. #149's shared samples/inspection/events, resizing and state retention
remain relevant. No issue is closed by this first slice.

## First slice

- Adapt shell-2026's titlebar, inset workspace canvas and resizable index/work pane.
  Extract the canvas geometry into `apps/control-support/src/shell.rs` and use it
  from both applications. Omit the unrelated destination rail.
- Index the documentation's Controls, Layout, Color controls and concrete
  Composition & helpers controls. Overview/prose entries are omitted. Switch is
  functional; every other listed control explicitly says **deferred**.
- Show one labeled, default-size/default-variant native Switch. Include keyboard
  activation, focus/hover/disabled events, an Enabled switch, programmatic reset,
  resizable inspector and scrollable event history.
- Bind inspection to the actual SDK `ButtonRenderModel<SwitchData>` and the same
  `SwitchTheme` instance passed to `ThemedSwitchTemplate`. Report native geometry,
  palette, typography and shadow values, with stable property names and source
  descriptions. Colors are explicitly settled endpoints, not interpolated frame
  colors; the SDK draws focus separately. No simulated Shadcn variants/states.
- Live light/dark changes replace templates/themes in place. Preserve sample,
  navigation, inspector, log and split entities, focus handles, selection, scroll
  handles and captured motion profile. Never rebuild controls to replay them.

## Shared support and remaining coupling

| Existing module (under `apps/luma-studio/src/studio/controls/`) | First-slice treatment | Next extraction boundary |
| --- | --- | --- |
| `event_log_view/{control,model,template,theme}.rs` and control tests | Move to `apps/control-support/src/event_log`; remove Shadcn typography dependency; retain Studio look adapter | Additional look-neutral tracking helpers as more controls migrate |
| `event_log_view/look.rs` | Keep in Studio; imports shared builder/theme interfaces | Shadcn-specific appearance stays here |
| `control_exposition/event_stream.rs` | Keep Studio wrapper, numbering and description | Wrapper UI still uses Shadcn typography/chrome |
| `control_exposition/switch.rs` | Share semantic event formatter; retain both Studio variants and all existing coverage | Extract additional sample behavior only when another consumer needs it |
| `control_exposition/inspector/schema.rs` | Move existing `InspectPropertyRow` unchanged to shared support and re-export; extract property-row UI behind look-neutral style inputs | `ControlInspectorResolver` still accepts `ShadcnLook`; generalize incrementally rather than copying the full inspector |
| `control_exposition/theme_inspector.rs`, `inspector/{adapters,metrics,render,box_model}*` | Keep Studio's richer inspector; native Switch adapter uses the shared property-row renderer | Extract look-neutral presentation and resolver inputs a control family at a time; retain specialized diagrams |
| `control_exposition/inspector_split.rs` | Keep Studio shell/global handle dependencies here; SDK app composes SDK resizable panels directly | Share split behavior once visibility/viewport contracts can be separated from Studio |
| `catalog.rs` and `control_catalog_picker.rs` | Keep Studio taxonomy/snippets | SDK catalog follows `docs/navigation.js` and has a drift test |
| `apps/common` and `apps/shells/common` | Do not import: both bring Shadcn-specific support | Fonts/assets can be isolated later if needed |

The native inspector is a narrow render-model adapter using the existing shared
row type, not a second general inspector metadata schema. It does not invent a
Motion section. #147 remains the owner of shared motion presentation; #152 may
replace handwritten property annotations later. #150 is not a prerequisite.

## Candidate scenario ownership and migration inventory

Paths below are relative to Studio's `control_exposition/`. No scenario is removed
in this PR. Each migration requires a functional native equivalent and tests first.

| Area / current source | Core scenarios destined for SDK app | Coverage retained in Studio | Status |
| --- | --- | --- | --- |
| `switch.rs`, `checkbox.rs`, `radio_button.rs`, `toggle.rs`, `radio_group.rs` | Semantic value updates vs programmatic assignment; focus/hover/enabled; keyboard activation; rapid reversal | Primary/secondary variants and look-specific layouts | Native Switch slice only; other controls deferred |
| `textfield.rs`, `textarea.rs`, autocomplete/selector/combobox pages | Editing, required/invalid state, selection, keyboard/focus, programmatic sample updates, event ordering | CSS recipes, variant presentation, richer forms | Text Field is next representative input |
| `listbox/{vertical,horizontal,filtering,variable_height,spectrum,transfer,selection_controls}.rs` | Selection policy, range/select-all/clear, activation, focus-owned scrolling, variable heights, transfer/drop | Card/markup presentations and richer compositions | Next representative collection; retain all examples until split by behavior |
| `tree_view/`, `tree_view/workspaces/tests.rs`, `table*`, `scrolling_table.rs` | Keyboard traversal, expansion, selection, drag/drop, cancellation, virtualization, state retention | Rich workspace and table compositions | Deferred; owning crate tests remain authoritative |
| `tabs.rs`, `accordion.rs`, `sidebar.rs` | Selection during transition, reordering/cancel, post-drop layout, retained tab-page state | Look-specific indicator/disclosure presentation | Deferred |
| `modal_overlay.rs`, `overlay_draggable.rs`, `overlay_positioning.rs`, `popup_menu.rs`, `context_menu.rs`, `floating_menu.rs` | Presence interruption, Escape/outside dismissal, focus restoration/trapping, drag/position constraints | Rich dialogs/menu compositions and Shadcn chrome | Overlay Window is next representative overlay |
| `resizable_panels.rs`, `dock_panel.rs`, `split_view.rs`, `scrollbar.rs` | Resize/collapse/restore, keyboard resizing, focus-owned wheel routing, scroll retention | Look recipes and richer panel compositions | SDK shell already exercises resizing; dedicated pages deferred |
| Range, progress, toolbar, color and remaining indexed controls | Baseline appearance, semantic events, explicit overrides, stylesheet inheritance | Look-specific variants/CSS and compositions | Explicitly deferred in catalog |

Radix Studio continues to own Radix appearance/behavior. SDK regression tests stay
in `crates/sdk`; the app's headless tests cover integration rather than replacing
those tests.

## Motion follow-up (after first slice)

1. Reuse #147's shared formatter and section with explicit example profile binding.
2. Switch: interruption/reversal, repeated activation, disabled/reduced/instant
   policies, instance precedence, theme changes while an existing profile runs.
3. Tabs/collections: selection mid-transition, reorder/cancel and drop-layout motion.
4. Overlays: interrupted enter/exit and focus restoration.

Only show parameters relevant to the selected example. Reset/replay should mutate
retained entities. This PR's Switch supports manual rapid toggling; it does not
claim the full motion scenario matrix or expose unfinished Motion controls.

## Verification and use

Build with `cargo build -p luma-sdk-controls`. Launching with
`cargo run -p luma-sdk-controls` requires explicit user permission in agent work.
No GUI launch is part of this PR's verification.

Headless checks:

- `cargo test -p luma-sdk-controls -p luma-control-support`
- `cargo test -p luma-shell-2026`
- `cargo test -p luma-studio --features test-support`
- `cargo test -p gpui-luma --features test-support switch`
- `cargo clippy -p luma-sdk-controls -p luma-control-support -p luma-shell-2026 -p luma-studio --all-targets -- -D warnings`
- `cargo fmt --all --check`

The first slice deliberately leaves the full Studio inspector UI/diagrams,
additional control pages and Motion UI for subsequent
PRs. README is independently maintained and is not changed.
