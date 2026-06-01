# Design Spec: Luma Theme Studio (Native GPUI Customizer)

This document details the vision and layout architecture for a native GPUI-based **Luma Theme Studio** application. It focuses on a clean, unified initial screen displaying all components in functional groups, with interactive click-to-inspect and customize overlays.

---

## 1. Core Vision

The Theme Studio acts as a living, native styleguide inside the project. Rather than spreading controls across multiple gallery pages or tabs, it consolidates them onto a single high-density dashboard. 

* **Live Interactive Canvas**: Every component is rendered natively, allowing designers and developers to interact with hover states, focus rings, and sliders.
* **Instant Settings Overlay**: Clicking any component overlays its specific properties, tokens, and scale parameters inline or in a side-drawer.
* **No Code Changes Required**: The app uses the active runtime theme config directly, bypassing static JSON and dumper registries.

---

## 2. Main Screen Layout

The canvas is a free-form board of **shadcn-style demo cards** (composite product screens), not isolated control columns. Cards start in a 3×2 grid but can be **repositioned by dragging the ⋮⋮ handle** at the top of each card. Click the card body to open theme inspection. Current cards:

1. **Upgrade your subscription** — form fields, plan radio group, notes, checkboxes, actions  
2. **Create an account** — social buttons, email/password, primary CTA  
3. **Team Members** — avatar rows with role selectors  
4. **Chat** — message thread and composer  
5. **Cookie Settings** — switches and save action  
6. **Report an issue** — selectors, subject, description, actions  
7. **Payments** — paged `ListView` with checkboxes, status/email/amount columns, and Previous/Next footer  

Legacy spec sketch (superseded by demo cards):

The initial screen was originally specified as functional columns. Each group contains the standard variants of those controls under active Light/Dark and Size states:

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│  LUMA THEME STUDIO                                                [ Size: SM | MD | LG ]│
├────────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                        │
│  ┌─────────────────────────┐  ┌─────────────────────────┐  ┌─────────────────────────┐  │
│  │ 1. ACTIONS              │  │ 2. CHOICE CONTROLS      │  │ 3. INPUTS & FIELDS      │  │
│  ├─────────────────────────┤  ├─────────────────────────┤  ├─────────────────────────┤  │
│  │                         │  │                         │  │                         │  │
│  │  Button (Primary)       │  │  Switch  [o] On  ( ) Off│  │  TextField              │  │
│  │  [ Button ]             │  │                         │  │  [ Enter text...      ] │  │
│  │                         │  │  Checkbox               │  │                         │  │
│  │  Button (Ghost)         │  │  [x] Selected           │  │  TextArea               │  │
│  │  [ Cancel ]             │  │  [ ] Unselected         │  │  ┌───────────────────┐  │  │
│  │                         │  │                         │  │  │ Multi-line text   │  │  │
│  │  Icon Button            │  │  Radio Group            │  │  │ editing...        │  │  │
│  │  [ ⊕ ]                  │  │  (o) Option A           │  │  └───────────────────┘  │  │
│  │                         │  │  ( ) Option B           │  │                         │  │
│  │  Toggle                 │  │                         │  │  Slider                 │  │
│  │  [ On ]                 │  │  Control Group          │  │  ┌───────●───────────┐  │  │
│  │                         │  │  [ Item 1 ] [ Item 2 ]  │  │                         │  │
│  └─────────────────────────┘  └─────────────────────────┘  └─────────────────────────┘  │
│                                                                                        │
│  ┌─────────────────────────┐  ┌─────────────────────────┐                               │
│  │ 4. DROPDOWNS & SELECTION│  │ 5. NAV & FEEDBACK       │                               │
│  ├─────────────────────────┤  ├─────────────────────────┤                               │
│  │                         │  │                         │                               │
│  │  ComboBox               │  │  Tabs Navigation        │                               │
│  │  [ Select option    |▼] │  │  [ Home ] [ Settings ]  │                               │
│  │                         │  │                         │                               │
│  │  Selector               │  │  Circular Progress      │                               │
│  │  [ Choose item      |▼] │  │  ( 75% )                │                               │
│  │                         │  │                         │                               │
│  │  Popup Menu             │  │  Scrollbar              │                               │
│  │  [ Actions          |▾] │  │  ┌───[====]──────────┐  │                               │
│  └─────────────────────────┘  └─────────────────────────┘                               │
│                                                                                        │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

### Functional Columns Detail

1. **Actions**: 
   * `Button` (Primary, Secondary, Ghost, Destructive variants)
   * `IconButton`
   * `Toggle` button
2. **Choice Controls**: 
   * `Switch` (On / Off / Disabled states)
   * `Checkbox` (Checked / Unchecked / Indeterminate states)
   * `Radio Button` & `Radio Group`
   * `Control Group` / Toolbars
3. **Inputs & Fields**:
   * `TextField` (Standard & Ghost variants, placeholders)
   * `TextArea` (Multiline text box)
   * `Slider` (Ranged values)
4. **Dropdowns & Selection**:
   * `ComboBox` (Searchable drop-down)
   * `Selector` (Click-to-open selection row)
   * `Autocomplete` textbox
   * `Popup Menu` & `Context Menu` triggers
5. **Navigation & Feedback**:
   * `TabsNavigation`
   * `Progress` (Circular loading indicators)
   * `Scrollbar` (Horizontal and vertical indicators)

---

## 3. Interaction Design (Click-to-View Settings)

Rather than keeping inspector components visible at all times, clicking any control on the dashboard focuses that control and reveals its settings sheet overlay:

```
┌────────────────────────────────────────────────────────┐
│ Switch Settings                                    [X] │
├────────────────────────────────────────────────────────┤
│                                                        │
│  Scale Metrics (SwitchScale)                           │
│  • Width:  [ 34 ] px                                   │
│  • Height: [ 18 ] px                                   │
│  • Thumb:  [ 14 ] px                                   │
│                                                        │
│  Color Tokens                                          │
│  • Track background:  --input    [ hsla(0 0% 19% / 1) ]│
│  • Track active:      --primary  [ hsla(15 73% 54%)   ]│
│  • Thumb background:  --bg       [ hsla(0 0% 10% / 1) ]│
│                                                        │
│  [ Save Adjustments ]       [ Export Theme Stylesheet ]│
└────────────────────────────────────────────────────────┘
```

* **Interactive Overrides**: Adjusting the metric text fields or selecting a new color override immediately re-renders that specific component instance on the main board.
* **Theme Export**: A button to compile all current overrides into a standard CSS stylesheet file ready for the `tweakcn` directory.

## Implementation

App crate: `apps/theme-studio` (`gpui-luma-theme-studio`).

```bash
just theme-studio
# or with a tweakcn theme:
cargo run -p gpui-luma-theme-studio -- astrovista
```

Themes load from `apps/gallery/tweakcn/` (shared with the gallery). Exports write to `apps/gallery/tweakcn/exports/theme-studio-overrides.css`.

Embedded fonts live under `apps/theme-studio/src/assets/fonts/` (Rajdhani for `jarvis`, via `include_bytes!` in the binary). Layout and form macros (`declare_form`, `vstack`, `hstack`, `wrappanel`, `flow`, `form_field`) live in `crates/sdk/src/macros.rs` and are imported via `gpui_luma::{…}`.

Layout persists in `apps/theme-studio/panel-layout.toml`: **`[window]`** `width` / `height` (pixels) and each card’s **upper-right** corner (`right`, `top`) under panel keys. The file is created on first launch; resizing the window or dragging a card updates it.
