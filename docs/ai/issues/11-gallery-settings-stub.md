# Issue #11: Add Gallery Settings Pane Stub

## Description
The Gallery application sidebar contains a footer node for "Settings". The sidebar correctly switches the internal active route to `"settings"`, but the Settings page placeholder pane needs to be confirmed/hooked up cleanly. We need to verify that navigation routes successfully display an empty/stub settings pane.

## Proposed Solution
Verify that `SETTINGS_PAGE` is correctly mapped to `GalleryPageKind::Settings` in the registry and that clicking the footer button correctly switches the view to render the stub settings pane. If there are any subscription gaps in `GalleryApp::new` for footer nodes, resolve them.

## Tasks
- [ ] Verify routing for footer nodes in [control.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/gallery/src/gallery/control.rs).
- [ ] Connect the Settings footer button to the settings stub pane defined in [pane.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/gallery/src/gallery/panes/settings/pane.rs).
- [ ] Render a basic empty panel layout stating "Settings Pane (Stub - Functionality defined later)" inside the settings pane.

## Acceptance Criteria
- Clicking the "Settings" button in the gallery's bottom sidebar section navigates to and displays the empty settings pane.
