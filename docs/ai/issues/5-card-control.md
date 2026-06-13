# Issue #5: Add Card Control

## Description
A "Card" is a basic container component supported by Shadcn to visually group content blocks. We need a first-class SDK Card control that maps border, background, and header text sizing using custom CSS styling rules.

## Proposed Solution
Create a standard `Card` control in the SDK following the split module pattern (`model`, `control`, `template`, `theme`), and add color/metric resolvers in the look-shadcn crate.

### Look Resolution & CSS Variable Mapping (e.g., Jarvis Theme)

The `Card` control resolves its visual properties dynamically at paint/layout time by querying design tokens from the active look stylesheet. When using a theme stylesheet like `jarvis.css`, the mappings behave as follows:

1. **Colors**:
   - **Background:** Mapped to `ShadcnToken::Card` (resolving CSS `--card`, e.g., HSL white in light mode, and dark navy `hsl(222.2 47.3% 11.1%)` in dark mode).
   - **Border:** Mapped to `ShadcnToken::Border` (resolving CSS `--border`, e.g., light grey `hsl(214.2 31.8% 91.3%)` in light mode, and dark slate `hsl(217.2 32.5% 17.4%)` in dark mode).
   - **Title Text:** Mapped to `ShadcnToken::CardForeground` (resolving CSS `--card-foreground`, e.g., dark text in light mode, off-white in dark mode).
   - **Description Text:** Mapped to `ShadcnToken::MutedForeground` (resolving CSS `--muted-foreground`, e.g., secondary grey).

2. **Metrics & Spacings**:
   - **Border Radius:** Mapped to `ShadcnRadius::Lg` (resolving CSS `--radius`, e.g., `0.25rem` / 4px in Jarvis light mode, `0.125rem` / 2px in Jarvis dark mode).
   - **Padding:** Uses standard density scaling derived from the active stylesheet's spacing scaling (CSS `--spacing`, e.g., `0.25rem`).

3. **Typography**:
   - **Font Family:** Inherits the active look's sans-serif font family `ShadcnFont::Sans` (resolving CSS `--font-sans`, which maps to `Rajdhani` under the Jarvis theme).
   - **Header Title:** Styled with `ShadcnTextSize::Base` or `Lg` and `FontWeight::SEMIBOLD`.

4. **Elevation (Shadow)**:
   - **Box Shadow:** Mapped to `ShadcnShadow::Default` (resolving CSS `--shadow`, e.g., a neutral shadow in light mode, or a vibrant cyan-hued glow `hsl(34 211 238 / 0.20)` under Jarvis dark mode).

## Tasks
- [ ] Create SDK Card files under `crates/sdk/src/controls/card/` (mod, model, template, theme).
- [ ] Subscribe to the SDK's theme revision global in the Card constructor to support safe, in-place invalidation and redraws upon theme changes.
- [ ] Define standard template rendering title, description/subtitle, children/body content, and footer.
- [ ] Register `CardStylesheet` and `CardColorRule` in [config.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/stylesheet/config.rs).
- [ ] Add rules to `style.toml` mapping Card colors to `card` / `card-foreground` / `border`.
- [ ] Wire Card resolving template inside [templates.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/controls/templates.rs).
- [ ] Add a demo card showcase page to the Gallery.
- [ ] Refactor the Gallery application's color inspector panels ([shell.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/gallery/src/gallery/panes/shared/inspector/shell.rs)) to use the new SDK `Card` control instead of manually styled `div` containers.
- [ ] Convert the local `card` and `card_header` helper functions in Theme Studio's panels ([common.rs](file:///Users/scg/Developer/GitHub/gpui-luma/apps/theme-studio/src/studio/panels/common.rs)) to use the new first-class SDK `Card` control.

## Acceptance Criteria
- Cards display rounded corners, a subtle border, and card shadow based on active Shadcn variables.
- Sizing matches standard density tokens.

## Usage Example

```rust
use gpui::{div, prelude::*};
use gpui_luma::controls::card::Card;
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma::{hstack, vstack};

// Spawning a Card control in a view render phase
let card_control = look
    .card("demo-card")
    .title("Report an issue")
    .description("What area are you having problems with?")
    .child(
        vstack! {
            gap=12;
            form_field!("Subject", chrome; self.subject_field.clone()),
            form_field!("Description", chrome; self.description_area.clone()),
        }
    )
    .footer(
        hstack! {
            gap=8 justify=end;
            self.cancel_button.clone(),
            self.submit_button.clone(),
        }
    )
    .spawn(cx);
```
