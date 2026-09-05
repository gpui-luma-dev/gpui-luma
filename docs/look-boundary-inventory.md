# Look boundary inventory (Phase 0)

Evidence map before `look-core` / `look-radix`. Classifications:

- **Source** — look-specific token interpretation (stay in the look crate)
- **Core candidate** — look-agnostic contracts (belong in `look-core` when shared)
- **Recipe** — control factories / resolvers / look-owned chrome (stay in the look)
- **SDK** — already in `crates/sdk` (do not move)

## `look-shadcn` modules

| Module | Class | Notes |
|---|---|---|
| `catalog/` | Source | CSS `:root` / `.dark` parsing |
| `tokens.rs`, `palette.rs`, `action.rs`, `mode.rs` | Source | `ShadcnToken`, palette fallbacks |
| `state_color.rs`, `color.rs`, `shadow.rs`, `resolve.rs` | Source | CSS/state math |
| `stylesheet/` | Source | `style.toml` Shadcn recipes |
| `look_context.rs` | Source | Bundles `ShadcnModeTokens` + CSS catalog |
| `context.rs` (`with_look`) | Source (for now) | TLS typed to `Arc<ShadcnLook>` |
| `provenance.rs` | Core candidate | Resolved values + sources (genericize off CSS) |
| `usage.rs`, `tables.rs` | Core candidate (shapes only) | Usage metadata shapes, not Shadcn token tables |
| `look.rs` | Recipe + Source | Snapshot/mode lifecycle + Shadcn API |
| `controls/*`, `controls/ext.rs`, `controls/templates.rs` | Recipe | Theme adapters + `ShadcnLookControlExt` |
| `elements/badge.rs`, `controls/card.rs` | Recipe | Look-owned composition |
| `ext.rs`, `focus.rs`, `paint.rs` | Recipe | Shadcn element helpers |
| `prelude.rs`, `lib.rs` | — | Re-exports |

## SDK (do not move)

`*Theme` traits, `*Look` / `*Palette` paint structs, `MetricTokens`, `LumaThemeRevision`, LMTP seams.

## Studio hard-deps

Almost all Studio surfaces take `Arc<ShadcnLook>` and call `ShadcnLookControlExt` / `with_look`. Few paths use bare `Arc<dyn *Theme>`. A second look does not need Studio wiring for the stub milestone.

## Implication

A second look implements SDK theme traits + a look-local factory ext. Shared extract is provenance/resolved-value shapes only until radix proves more duplication.
