# Radix Studio UI: Top-Level Tabs

Update `apps/radix-studio` to organize the workbench into top-level tabs. The current screen is the **Custom Palette** tab.

## Requested tabs

Add a top tab bar with these items, in this order:

1. `Custom Palette`
2. `Colors`
3. `Icons`

The initial active tab is `Custom Palette`.

The tab bar should sit below the title bar and above the tab content, spanning the content area. Tab labels should be centered horizontally in the available width. The selected item should read as a rounded, button-like control with a filled dark surface and light/bold text, matching the supplied reference image. Unselected items should be transparent with muted foreground text. The treatment must work in both light and dark theme modes through the Radix look rather than hardcoded theme-specific colors.

## Current implementation to reorganize

`apps/radix-studio/src/app.rs` currently renders one long page containing:

- The `Create a custom palette` header.
- Theme mode toggles.
- Accent, gray, and background seed fields.
- The color and gray scale grid.
- The component preview canvas.

It also creates `preview_tabs` with `Themes`, `Primitives`, `Icons`, and `Colors`, but renders those tabs inside the right column of the preview canvas. That control is not the requested application-level navigation and should be removed from that column as the new top-level tab bar is introduced.

## Code organization

Keep `RadixStudioApp` responsible for shared application state, look/theme changes, tab selection, and routing the active content. Move tab-specific rendering into focused modules:

```text
apps/radix-studio/src/
├── app.rs                 # Window shell, shared state, top-level tab routing
├── tabs.rs                # Tab identifiers, tab model, and shared tab bar
├── custom_palette.rs      # Current palette editor and component preview content
├── colors.rs              # Colors tab content
├── icons.rs               # Icons tab content
├── components/            # Reusable content sections shared by tabs
│   ├── mod.rs
│   ├── palette.rs         # Seed fields, copy menu, scale grid
│   └── preview.rs         # Component preview canvas and preview controls
```

The exact filenames may be adjusted to fit the existing code, but tab-specific view code should not remain as one monolithic `app.rs` render function.

### Shared state and entities

- Add a typed tab identifier, for example `RadixStudioTab::{CustomPalette, Colors, Icons}`.
- Store the active tab in `RadixStudioApp`.
- Subscribe to the top-level tabs control and update the active tab from its semantic change event.
- Preserve shared entities and state—`Look`, theme mode toggles, seed fields, swatch overlay, signup mesh cache, and subscriptions—at the app or shared component boundary so switching tabs does not recreate them unnecessarily.
- Render only the active tab's content while keeping the top tab bar mounted.
- Keep the tab bar accessible as a tablist with tab items and a clear selected state.

## Content routing

### Custom Palette

Move the current page content here:

- `Create a custom palette` heading.
- Light/dark mode toggle.
- Accent, gray, and background seed inputs.
- Copy palette menu.
- Color/gray scale grid and swatch details overlay.
- Component preview canvas, including the signup, status, and action previews.

### Colors

Create the initial tab surface and leave room for the color documentation/content that will be added later. It must use the shared app shell and active look.

### Icons

Create the initial tab surface for the Radix icon inventory and future icon previews. Use [`docs/ai/0-react-icon.md`](0-react-icon.md) as the source inventory. This tab should be able to display the Radix SVG assets sourced from `~/Downloads/radix-icons` when those assets are added to the app.

## SDK and look constraints

- Use the SDK `Tabs` control and the Radix look-owned builder (`radix::Tabs::new(...)`) for tab behavior and focus handling.
- If the existing tabs presentation cannot provide the requested selected pill, add or adjust the Radix look-owned tabs presentation rather than building tab buttons from raw styled `div`s in the app.
- Preserve keyboard navigation, focus indication, hit testing, and theme invalidation.
- Keep application-specific layout and content in `apps/radix-studio`; do not add Radix-specific visual recipes to the SDK core.

## Acceptance criteria

- [ ] A top-level tab bar shows `Custom Palette`, `Colors`, and `Icons`.
- [ ] `Custom Palette` is selected on startup and displays the current screen.
- [ ] The selected tab has a rounded button-like surface matching the reference treatment.
- [ ] Unselected tabs remain visually quiet and readable in both theme modes.
- [ ] Selecting `Colors` or `Icons` swaps the content without recreating shared app state.
- [ ] The old `Themes / Primitives / Icons / Colors` preview tabs no longer appear in the preview column.
- [ ] Tab interaction remains accessible and uses SDK control semantics.
- [ ] Tab-specific rendering is separated into focused modules.

## Verification

- Manually verify startup selection, switching, focus traversal, and selected/unselected styling in light and dark modes.
- Confirm the palette editor, scale swatches, overlay, and preview controls retain their existing behavior after moving into `Custom Palette`.
- Run `cargo fmt`, relevant `cargo test` targets, and `cargo clippy` when implementation begins.

