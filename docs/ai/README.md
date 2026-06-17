# AI Documentation Map

Welcome, AI Coding Assistant! 

To reduce cognitive load and avoid stale data, the documentation for this repository has been consolidated and moved to the parent `docs/` folder. Please use the following canonical guides instead of looking for separate files under `docs/ai/`.

## Canonical Documentation (Parent Directory)

- **[Architecture Guide](../architecture.md)**
  - *Read this first.* Documents crate maps, module structures, boundaries, and design splits (Logic, Model, Template, Theme).
- **[Development Guidelines](../development-guidelines.md)**
  - Documents Rust coding conventions, layout macro constraints (`h_flex!`, `v_flex!`, `stack!`), composability rules, and reentrancy snapshots.
- **[Theming & Styling](../theming.md)**
  - Documents Luma's Tier-1 (global design tokens) and Tier-2 (`style.toml`) mappings, typography guidelines, and the registration process for new controls/looks.
- **[Focus & Key Handling](../focus-and-key-handling.md)**
  - Documents GPUI's focus traversal model, custom focus scopes, action profiles, and keyboard input handlers.

## Active Tracking & Issues
- **[Active Issues & Proposals](./issues/)**
  - Contains design stubs and ongoing proposals (e.g., [`14-adorner.md`](./issues/14-adorner.md)) currently being implemented or evaluated.
