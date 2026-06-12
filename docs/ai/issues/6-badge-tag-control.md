# Issue #6: Add Badge/Tag Control

## Description
Badges or inline tags are small visual elements used to display status labels, counts, or categories. We need to add a Badge component with multiple color styles matching standard Shadcn variants: Default, Secondary, Destructive, and Outline.

## Proposed Solution
Create a standard `Badge` control in the SDK and define resolvers in the look-shadcn crate mapping variant colors to the active theme palette.

## Tasks
- [ ] Create SDK Badge files under `crates/sdk/src/controls/badge/` (mod, model, template, theme).
- [ ] Define Badge variants (`Default`, `Secondary`, `Destructive`, `Outline`) and builder methods.
- [ ] Add `BadgeColorRule` matcher inside look-shadcn.
- [ ] Add rules to `style.toml` defining text/background colors per variant.
- [ ] Add a Badge showcase section inside the Gallery.

## Acceptance Criteria
- Badge colors align with the corresponding Shadcn theme variables (e.g. `primary` background for Default, `destructive` for Destructive, transparent with borders for Outline).
- Text padding is scaled appropriately.
