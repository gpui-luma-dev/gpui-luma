# Issue #6: Add Badge/Tag Control

## Description
Badges or inline tags are small visual elements used to display status labels, counts, or categories. We need to add a Badge component in `look-shadcn` with Shadcn-style variants matching the desired dark-theme look reference from the Tangerine assets. The current target variants are `Default`, `Secondary`, `Outline`, and `Ghost`.

## Proposed Solution
Implement Badge as a look-specific visual element in `crates/look-shadcn` rather than as a core SDK control. Expose it through the Shadcn look extension surface, define stylesheet-backed variant resolvers mapping colors to the active theme palette, and support an optional icon rendered at the start or end of the badge. Its shape and spacing should scale from theme metric and radius tokens rather than hardcoded values.

## Tasks
- [ ] Create Badge files under `crates/look-shadcn/src/controls/` and expose the component through the Shadcn look extension API.
- [ ] Define typed Badge variants (`Default`, `Secondary`, `Outline`, `Ghost`) and badge configuration methods.
- [ ] Add optional badge icon support with placement at start or end of the content.
- [ ] Resolve badge padding, height, icon gap, and corner shape from shared size/metric/radius tokens.
- [ ] Add `BadgeColorRule` matcher inside look-shadcn.
- [ ] Add rules to `style.toml` defining text/background colors per variant.
- [ ] Add a Badge showcase section inside the Gallery.

## Acceptance Criteria
- Badge colors align with the corresponding Shadcn theme variables and the intended Tangerine dark-theme reference:
  - `Default` uses the prominent filled treatment.
  - `Secondary` uses the secondary surface/foreground pairing.
  - `Outline` uses a transparent or background-matched fill with visible border treatment.
  - `Ghost` uses a minimal low-chrome treatment appropriate for inline tags.
- Text padding, icon gap, and overall badge height scale appropriately from theme size metrics.
- Badge corner radius is derived from theme radius metrics instead of a fixed pill constant.
- Badge supports an optional icon at the start or end without breaking alignment.
- Badge remains a `look-shadcn` visual element and does not introduce an unnecessary SDK control surface or behavior layer.
