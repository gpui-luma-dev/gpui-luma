# Gallery → Luma Studio Controls Migration

## Status

**Active.** Long-term goal is to consolidate interactive control documentation into Luma Studio and retire Gallery as the primary consumer-facing showcase. Work is incremental; Gallery remains the validation harness until each control has a Studio exposition.

### Done (Controls tab)

- **`ControlExposition` module** — local template/control pattern under `apps/luma-studio/src/studio/controls/control_exposition/`
- **Button** — reference exposition (variants, live event stream, possible-events table, code sample)
- **Text Field** — lite exposition (preview + code sample)
- **Modal Overlay** — lite exposition (open/dismiss demo + code sample)
- **Controls panel shell** — scroll column, sticky headings, Control Index rail; panel no longer owns per-control wiring
- **Shared pieces** — `ControlEventStream`, `render_event_reference_section`, `render_control_exposition_card`, `EVENT_SECTION_INDENT`

### In progress (Gallery, parallel)

- Button pane **ThemeInspector** companion rail via slide-panel prototype (`apps/gallery/.../prototypes/slide_panel/`)
- Slide panel SDK promotion deferred until Gallery inset rail is stable

### Not started

- Remaining gallery panes → Studio control expositions (~40+ pages)
- Gallery retirement / feature flag
- ThemeInspector in Studio Controls (slide panel; see below)

---

## Problem

Today the workspace has **two overlapping surfaces**:

| Surface | Role today |
|--------|------------|
| **Gallery** (`apps/gallery`) | Full interactive validation: state matrices, inspectors, prototypes, edge-case demos |
| **Luma Studio — Controls tab** | API/usage documentation: how to spawn, subscribe, and compose SDK controls |

Gallery panes are rich but scattered. Luma Studio's Style Guide already covers **visual state matrices**; Controls should cover **integration** (builders, events, snippets). Duplicating full gallery panes inside Controls would bloat the doc column and fight the Control Index rail.

---

## Goal

For each SDK control family that Gallery already demonstrates:

1. **Derive** a focused Controls exposition from the gallery pane (preview + events + snippet — not the full inspector/matrix).
2. **Register** it in `CONTROL_CATALOG` and `ControlExposition` registry.
3. **Keep** deep validation (inspectors, resize demos, prototypes) in Gallery until promoted or replaced.
4. **Eventually** retire Gallery as the default entry point; Studio becomes the product shell.

---

## Controls tab architecture

### Page shell (`ControlsPanel`)

- Left: scrollable doc column (~980px), category headings, sticky section titles
- Right: **Control Index** (168px) — jump links synced to scroll anchors
- Theme sync via `sync_snapshot` on all exposition entities

### Per-control exposition (`ControlExposition`)

Each control is a **GPUI entity** that owns its demo state and renders a **full doc card**. Pattern mirrors SDK LMTP locally (Studio-only, not shipped in `crates/sdk`):

| Piece | File(s) | Role |
|-------|---------|------|
| **Catalog entry** | `catalog.rs` | Static metadata: id, title, description, category, snippet, `section_order` |
| **Exposition entity** | `control_exposition/{control}.rs` | Spawn SDK entities, subscriptions, `sync_look`, `Render` full card |
| **Card template** | `control_exposition/template.rs` | Shared card shell |
| **Registry** | `control_exposition/registry.rs` | `ControlExposition` enum, `spawn_all`, find/render/sync |
| **Shared event UI** | `event_stream.rs`, `event_reference.rs` | Reusable eventing blocks |

### Doc card sections (fixed order)

Every exposition renders the same skeleton:

1. **Title + description** — sticky-scroll anchor (`section_order` from catalog)
2. **Live preview** — SDK controls via look builders only
3. **Eventing** *(optional)* — intro + append-only `EventLogView` stream (`ControlEventStream`)
4. **Possible events** *(optional)* — static reference table (`EventReferenceSpec` rows); indented `EVENT_SECTION_INDENT` (100px)
5. **Code Sample** — mono snippet from catalog

Layout flag: `ControlExpositionLayout::BORDERLESS` (no nested bordered preview boxes).

### Button = reference template

Copy `control_exposition/button.rs` when adding a control:

```text
new()        → catalog_entry(id), spawn entities, cx.subscribe wiring
sync_look()  → refresh entities + event stream on theme change
render()     → render_control_exposition_card(preview, optional events ref, BORDERLESS)
```

Text Field and Modal Overlay are **lite** expositions (preview + code sample only). Add eventing when the control has parent-subscribable events worth demoing.

---

## What to take from Gallery vs what to leave

### Bring into Controls exposition

- Representative **live preview** (e.g. button variants row, not full template matrix)
- **Event subscription** demo + possible-events table (see `0-eventing.md`)
- **Usage snippet** (spawn + subscribe pattern)
- Short **feature description** (expanded prose in catalog, not gallery pane title alone)

### Leave in Gallery (for now)

- **ThemeInspector** / color inspectors / resolved token drill-down
- **State/size matrices** (Style Guide tab owns visual matrices in Studio)
- **Prototypes** (slide panel, shadow button, etc.) until promoted to SDK
- **Dialog variants** as separate pages (modal, modeless, draggable, positioning) — may collapse into one Studio exposition per family
- **Color picker** family (Studio has Palette / Theme Usage instead)

### Studio surface split (do not merge)

| Tab | Purpose |
|-----|---------|
| **Controls** | Integration docs — exposition cards |
| **Style Guide** | Variant/state/size visual matrices |
| **Theme Usage** | Token catalog — who consumes what |
| **ThemeInspector** *(future)* | Live resolved values — slide-panel rail in Controls or Style, not inline in every card |

---

## Gallery pane → Studio exposition mapping (target)

Grouped by `ControlCategory` in `catalog.rs`. Each row is one future `ControlDocEntry` + `{control}.rs` module.

### Command

- Button ← `gallery/panes/button`
- Toolbar ← `gallery/panes/toolbar`
- Badge ← `gallery/panes/badge`

### Choice

- Toggle, Toggle Group, Switch, Checkbox
- Radio Button, Radio Group

### Inputs

- Text Field ← done (lite)
- Text Area
- Slider, Scrollbar

### Selection

- ListBox, Scrolling List View, Paging List View
- ComboBox, Autocomplete TextField, Search Selector, Selector, Selection Panel

### Navigation & Panels

- Tabs Navigation, Navigation Sidebar, Accordion, Tree View
- Resizable Panels, Split View (unified/inset)
- Progress, Pager

### Overlays & Dialogs

- Modal Overlay ← done (lite)
- Popup Menu, Context Menu, Floating Menu
- Dialog variants (modal/modeless/draggable/positioning) — TBD: one or four expositions

### Gallery-only (no Controls exposition planned)

- Color-* panes, choice/selector templates, decorated/custom/shadow button prototypes, dock panel, settings stub, slide-panel prototype demo page

---

## Implementation workflow (per control)

1. Read gallery pane: preview entities, subscriptions, inspector (ignore inspector for Controls v1).
2. Add `ControlDocEntry` to `catalog.rs` (description, snippet, category, order).
3. Create `control_exposition/{id}.rs` from **button.rs** template.
4. Register variant in `registry.rs` (`ControlExposition` enum + `spawn_all`).
5. Declare module in `control_exposition/mod.rs`.
6. Verify: index link, sticky scroll, theme hot-reload, event stream (if applicable).
7. Optionally trim gallery pane later — **not** until Studio exposition is validated.

No changes to `ControlsPanel` beyond registry unless adding new shared section types.

---

## Related work

- **`0-eventing.md`** — event taxonomy; drives Possible events tables and EventLogView demos
- **`10-slide-panels.md`** (retired) — slide panel prototype; companion rail for ThemeInspector in Gallery Button pane first, then SDK, then Studio
- **`docs/architecture.md`** — apps compose SDK + look; no home-brew chrome in `apps/`

---

## Acceptance criteria (migration complete)

- [ ] Every planned SDK control family has a Studio `ControlExposition` registered in catalog
- [ ] Button exposition remains the documented reference for new controls
- [ ] Controls tab needs no per-control match arms in `panel.rs`
- [ ] Gallery panes for migrated controls marked deprecated or removed
- [ ] ThemeInspector accessible from Studio (slide-panel rail) for at least Button
- [ ] Gallery app optional / dev-only or removed from default workspace builds

---

## File map

```text
apps/luma-studio/src/studio/controls/
├── catalog.rs                          # CONTROL_CATALOG source of truth
├── panel.rs                            # scroll shell + index only
├── control_exposition/
│   ├── mod.rs
│   ├── model.rs                        # layout flags, EventReferenceSpec, indent
│   ├── template.rs                     # card shell, Code Sample header
│   ├── event_stream.rs                 # shared Eventing block
│   ├── event_reference.rs              # shared Possible events table
│   ├── button.rs                       # ★ reference template
│   ├── textfield.rs
│   ├── modal_overlay.rs
│   └── registry.rs
└── event_log_view/                     # readonly log control used by event stream

apps/gallery/src/gallery/panes/         # source material + validation until retired
```
