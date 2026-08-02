# VS Code Shell Layout Work Summary

**Status: Complete and verified.** The shell layout, Customize Layout dialog, panel behavior,
platform shortcuts, status bar, and sidebar visibility interactions were tested successfully.

## Scope

This work focused on `apps/shells/vscode`, especially the Customize Layout dialog and the workbench-style layout behavior behind it.

The goal was to make the shell behave more like a VS Code-style workbench while staying on SDK controls and theme resolution instead of hand-built interaction chrome.

## Completed Work

### Customize Layout Dialog

- Reworked the dialog to use SDK `Button` controls with local templates for VS Code-style menu rows.
- Removed the modal masking layer so layout changes are visible while the dialog is open.
- Added visibility rows for:
  - Activity Bar
  - Secondary Activity Bar
  - Primary Side Bar
  - Secondary Side Bar
  - Panel
  - Status Bar
- Added primary side bar position rows:
  - Left
  - Right
- Added panel alignment rows:
  - Left
  - Right
  - Center
  - Justify
- Updated custom row glyphs to use the button row's resolved foreground color.
- Kept dialog background and row colors theme-aware across light/dark changes.

### Activity Bars And Sidebars

- Split primary/secondary activity bars from sidebars.
- Treat activity bars as fixed icon rails, not resizable panels.
- Treat sidebars as resizable/hideable panels.
- Primary and secondary regions are logical pairs:
  - Moving the primary side bar position also moves the primary activity bar.
  - The secondary side bar/activity bar move to the opposite side.
- Physical resizable panel indexes remain left/right slots; logical primary/secondary mapping is handled above that.

### Local Workspace Layout Control

- Introduced an app-local control-style module under `apps/shells/vscode/src/workspace_layout/`.
- `WorkspaceLayout` now owns:
  - activity rail rendering
  - `WorkbenchLayout`
  - side panel visibility syncing
  - panel alignment geometry
  - bottom panel height state
  - bottom panel splitter
- `VscodeShellApp` now coordinates:
  - title bar
  - status bar
  - theme toggle
  - Customize Layout dialog
  - high-level `LayoutConfig`

### Resizable Side Panels

- `ResizablePanels` remains the lower-level SDK primitive for the sidebars/editor strip.
- Sidebars use SDK hide/show with `PanelHideMode::Completely`.
- Hidden side panels:
  - collapse to zero
  - keep restore state
  - suppress adjacent resize handles
  - cannot be manually dragged back open
  - must be shown through app/API state

### SDK Resizable Panel Fixes

- Fixed delayed/stale sizing by making `set_frame_size` refresh panel sizes immediately.
- Fixed hidden fixed side panel resize behavior:
  - hiding a fixed side panel next to a weighted editor now preserves the editor as weight-based
  - frame growth after hiding no longer leaves blank unclaimed space
- Added math regression coverage for fixed sidebars plus weighted editor behavior.
- Extended the render model with hidden-panel state.
- Default `ResizablePanels` template now skips handles adjacent to hidden panels.

### Bottom Panel

- Added bottom Panel visibility support.
- Added panel alignments:
  - `Center`: panel lives inside editor region; sidebars/activity rails extend full height.
  - `Left`: panel overlays the side/editor strip after the left activity rail, spanning left sidebar plus editor.
  - `Right`: panel overlays editor plus right sidebar, excluding activity rails.
  - `Justify`: panel overlays the full workbench strip, excluding activity rails.
- Activity rails remain fixed and full-height in all panel alignment modes.
- Added resizable panel height through SDK `DockSplitter`.
- Panel show/hide is separate from panel resizing.

## Current Behavior Notes

- Activity bars are fixed icon rails.
- Sidebars are resizable panels.
- Hidden sidebars are disabled from manual resizing.
- Bottom panel height is resizable when the panel is visible.
- Bottom panel alignments intentionally do not affect activity rail height.
- `Center` panel behavior is implemented inside `WorkbenchLayout`.
- `Left`, `Right`, and `Justify` panel behavior is implemented as overlays in `WorkspaceLayout`.

## Historical Design Notes

### Panel Height Is Not In LayoutConfig

Status: resolved. `LayoutConfig` now owns `panel_height_px`; reset restores the default,
and `WorkspaceLayoutEvent::PanelHeightChanged` synchronizes user resize changes back to the app model.

The panel height is now modeled in `LayoutConfig` and synchronized from the workspace control.

Previously:

- Reset does not have an explicit panel-height model value beyond current internal defaults.
- Future persistence cannot serialize panel height without extending `LayoutConfig`.
- Tests or callers cannot directly assert panel height through config.

Previous recommendation:

- Add `panel_height_px` or a typed panel sizing model to `LayoutConfig` when persistence/reset semantics matter.

### Panel Ownership Is Split

Status: accepted follow-up, not a blocker. The synchronization API is semantic, but `WorkbenchLayout` still owns
the Center-mode panel state and render slot.

`WorkspaceLayout` owns most bottom panel behavior, but `WorkbenchLayout` still owns enough panel state to render `Center` mode:

- `panel_visible`
- `panel_alignment`
- `panel_height_px`
- `panel_splitter`

This is functional but awkward.

Recommendation:

- Reduce `WorkbenchLayout` back to sidebars + editor strip only.
- Move all bottom panel layout/rendering decisions into `WorkspaceLayout`.
- Give `WorkspaceLayout` a clearer internal region model for editor/sidebar/panel placement.

### Overlay Panel Modes Cover Content

`Left`, `Right`, and `Justify` use overlay panels rather than real layout partitioning of the affected content.

This matches the current visual target, but:

- editor/sidebar content still exists behind the panel
- future scrollable editor or sidebar content may need bottom insets
- hit-testing around overlapped content should be watched as real content is added

Recommendation:

- If the shell becomes content-heavy, move from overlay geometry to an explicit workbench grid/dock layout that partitions affected regions.

### Sync API Is Split

Status: resolved. `WorkspaceLayout` now exposes semantic setters for visibility, alignment,
side position, and panel height; the app no longer uses separate full/non-sidebar config sync paths.

The app currently uses two sync paths:

- `sync_from_config`: full sync, including sidebar hide/show.
- `sync_non_sidebar_from_config`: updates panel/activity/status/alignment without touching sidebars.

This fixed accidental sidebar changes when toggling Panel visibility, but the split is a smell.

Recommendation:

- Replace config-blob syncing with semantic setters:
  - `set_activity_bar_visible`
  - `set_secondary_activity_bar_visible`
  - `set_primary_side_bar_visible`
  - `set_secondary_side_bar_visible`
  - `set_panel_visible`
  - `set_panel_alignment`
  - `set_primary_side_bar_position`

### Local Control API Needs Hardening

`WorkspaceLayout` is intentionally app-local for evaluation, but it is not yet ready as an SDK control.

Gaps:

- no explicit model/template split
- limited typed public API
- panel sizing was not represented as a stable model
- sidebars are passed in, but editor/panel content is still demo content
- limited event surface for layout changes

Recommendation:

- Keep it local until the API shape stabilizes.
- If promoted, follow LMTP:
  - model: region visibility, alignment, sizing, logical side
  - control: hide/show, resize, event emission
  - template: workbench region rendering
  - theme: chrome colors and splitter/rail styling

### Customize Dialog Must Stay Visible During Configuration

The Customize Layout dialog is intended to be a modeless configuration surface.

Requirements:

- Opening the dialog must not mask or dim the shell.
- The dialog must remain visible while the user toggles layout regions.
- Layout changes should apply live behind the dialog.
- Dialog refreshes must not dismiss, reposition unexpectedly, or steal focus in a way that interrupts repeated configuration.

Recommendation:

- Treat Customize Layout as a persistent modeless overlay, not a modal dialog.
- Keep state synchronization granular so row updates do not recreate or dismiss the overlay.

### Customize Dialog Should Be Movable

Status: resolved. The dialog enables the SDK draggable modeless overlay, and the SDK clamps
dragged positions to the viewport bounds.

The dialog can cover the region being configured, especially when live layout changes happen behind it.

Requirements:

- User should be able to drag/move the dialog on screen.
- Position should remain stable while toggling layout options.
- Position should remain within the visible window bounds.

Recommendation:

- Add movable overlay support either to `OverlayWindow` or as a local dialog behavior.
- Prefer an SDK-level modeless floating panel primitive if this pattern appears elsewhere.

### Keyboard Mapping Must Be Multi-Platform

Status: resolved for the current shortcuts. Display labels and bindings now select Command on
macOS and Control elsewhere, with `B` and `J` actions wired to sidebar/panel toggles.

The shell currently shows shortcut hints such as command-key based combinations. Those are not portable across platforms.

Requirements:

- Keyboard shortcuts shown in the dialog must match the active platform.
- macOS should show Command-style hints.
- Windows/Linux should show Control-style hints where appropriate.
- The actual shell/app key bindings should match the displayed hints.

Recommendation:

- Add a small platform-aware shortcut presentation helper.
- Keep shortcut display data and registered key bindings sourced from the same mapping table.
- Avoid hardcoding glyphs like `⌘` directly in layout metadata unless wrapped behind platform translation.

### SDK Hidden vs Collapsed Semantics Need Documentation

Status: resolved. `PanelHideMode` and the hide/show methods now document the distinction between
minimum-size collapse and explicit hide/show recovery.

The SDK now distinguishes:

- collapse/expand: panel can remain part of interactive resizing
- hide/show: panel is disabled from manual resize recovery

This distinction is important and should be documented near `ResizablePanels`.

Recommendation:

- Add SDK docs/examples that show when to use collapse versus hide.
- Confirm other consumers do not rely on hidden panels being draggable back into visibility.

## Recommended Next Steps

- Add a small local test/demo checklist for the VS shell:
  - startup layout
  - hide/show primary sidebar
  - hide/show secondary sidebar
  - hide/show panel
  - resize after hidden sidebar
  - panel alignments with both activity rails visible
  - panel resize in all alignments
- Keep the Customize Layout dialog visible while live-configuring.
- Move bottom panel ownership fully into `WorkspaceLayout`.
- Decide whether `WorkspaceLayout` is a candidate SDK control or should remain a shell-specific composition.
