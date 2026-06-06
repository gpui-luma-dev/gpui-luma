First read docs/ai/*.md, Cargo.toml, README.md, and the modules relevant to my request. Then propose a plan before editing.

Reference docs:
- `docs/ai/project-overview.md`
- `docs/ai/architecture.md`
- `docs/ai/module-map.md`
- `docs/ai/reentrant-warning.md`
- `docs/ai/listbox.md`

Considerations:
  Tight responses are better than long verbose responses
  Apps in `apps/` are SDK consumers: use toolkit controls and themed builders, not hand-rolled interactive UI (see `project-overview.md`).
