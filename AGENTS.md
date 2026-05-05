# AI coding instructions

Before making changes:
- Read docs/ai/project-overview.md and docs/ai/architecture.md.
- Inspect relevant Rust modules directly; do not rely only on summaries.
- Prefer small, reviewable diffs.
- Run `cargo fmt`, `cargo clippy`, and relevant tests before finalizing when possible.

After making architectural or API changes:
- Update docs/ai/module-map.md or docs/ai/architecture.md if they became stale.

Rust conventions:
- Preserve existing error handling style.
- Do not introduce panics in library code unless existing conventions allow it.
- Prefer explicit types where they improve readability.
- Keep public API changes minimal and documented.
