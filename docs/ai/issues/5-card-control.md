# Issue #5: Add Card Control

## Description
A "Card" is a basic container component supported by Shadcn to visually group content blocks. We need a first-class SDK Card control that maps border, background, and header text sizing using custom CSS styling rules.

## Proposed Solution
Create a standard `Card` control in the SDK following the split module pattern (`model`, `control`, `template`, `theme`), and add color/metric resolvers in the look-shadcn crate.

## Tasks
- [ ] Create SDK Card files under `crates/sdk/src/controls/card/` (mod, model, template, theme).
- [ ] Define standard template rendering title, description/subtitle, children/body content, and footer.
- [ ] Register `CardStylesheet` and `CardColorRule` in [config.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/stylesheet/config.rs).
- [ ] Add rules to `style.toml` mapping Card colors to `card` / `card-foreground` / `border`.
- [ ] Wire Card resolving template inside [templates.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/controls/templates.rs).
- [ ] Add a demo card showcase page to the Gallery.

## Acceptance Criteria
- Cards display rounded corners, a subtle border, and card shadow based on active Shadcn variables.
- Sizing matches standard density tokens.
