# AI coding instructions

Always answer concisely. Prefer short bullets. Do not give long explanations unless I ask.

Documentation:
- DO NOT EDIT `README.md` without explicit user approval. It is an independently maintained document and should generally not be touched by agents.

GUI applications:
- Do not launch or run GUI applications without explicit user approval to launch them.
- Approval to implement changes, build, or test does not authorize GUI launches, including startup or smoke tests.

Before making changes:
- Read the relevant source modules and repository instructions directly; do not rely only on summaries.
- Inspect relevant Rust modules directly; do not rely only on summaries.
- In apps under `apps/`, use SDK controls and theme factories (`ShadcnLook`, `*Builder`, `.spawn(cx)`). Do not home-brew buttons, checkboxes, inputs, or other interactive chrome with raw `div` styling.
- Prefer small, reviewable diffs.
- Run `cargo fmt`, `cargo clippy`, and relevant tests before finalizing when possible.

Rust conventions:
- Preserve existing error handling style.
- Do not introduce panics in library code unless existing conventions allow it.
- Prefer explicit types where they improve readability.
- Keep public API changes minimal and documented.
