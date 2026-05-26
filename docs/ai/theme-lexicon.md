# Theme import lexicon

**Machine-readable bindings:** [`crates/sdk/src/theme/lexicon.toml`](../crates/sdk/src/theme/lexicon.toml)

**Related:** [`next-step-theme-import.md`](next-step-theme-import.md), [`next-step-variants.md`](next-step-variants.md)

---

## History

A full `lexicon.toml` was drafted in an earlier design session ([chat 4542198e](agent-transcripts)) at `crates/sdk/src/theme/lexicon.toml` but **was never committed** to git. The file below is a **rebuilt** lexicon aligned with:

- Four-weight ladder (`action.subtle` + `ButtonKind::Subtle`)
- Validated Astrovista/Jarvis manual imports
- Post-review design notes (semantic palette layer, normalize-by-default)

---

## Three-layer pipeline

```text
CSS export  →  catalog (token values per mode)
                    ↓
              palette (semantic aliases: brand, outline_surface, …)
                    ↓
              lexicon.toml (aliases → Luma paths + transforms)
                    ↓
              base theme (default-theme.toml) via { inherit = "base" }
                    ↓
              theme.toml (resolved output)
```

| Layer | File / artifact | Answers |
|---|---|---|
| **Catalog** | Parsed from `tweakcn-*.css` (per theme) | What is `--primary` in light/dark? |
| **Palette** | `[palette]` in `lexicon.toml` | What do tokens *mean* (`brand`, `outline_border`, …)? |
| **Lexicon** | `[light.*]` / `[dark.*]` in `lexicon.toml` | Where do palette roles land in Luma schema? |
| **Base** | `default-theme.toml` | What is missing from the catalog? |

The lexicon is **source-agnostic** — no CSS paths, only logical token names.

---

## Binding grammar (resolver contract)

Each leaf is a **cascade** (first match wins):

```toml
background = [
  { palette = "brand" },
  { token = "primary" },
  { inherit = "base" },
]
```

| Form | Meaning |
|---|---|
| `{ token = "name" }` | Catalog lookup (colors normalized by default) |
| `{ palette = "name" }` | Semantic alias from `[palette]` |
| `{ path = "palette.app.background" }` | Another resolved Luma path (same mode) |
| `{ literal = "…" }` | Fixed value |
| `{ inherit = "base" }` | `default-theme.toml` leaf (terminal) |
| `{ from_palette = true }` | Whole `colors` section via `ColorTokens::from_palette` |

**Transforms** (optional, chained): `darken(n)`, `lighten(n)`, `alpha(n)`, `mix(…)`, `rem_to_px(base)`, `first_font`, `shadow_parse`, `offset(n)`.

**Defaults (planned resolver behavior):**

- `{ token = … }` and `{ palette = … }` → **normalize** catalog colors unless `transform = ["no-normalize"]`
- `{ path = … }`, `{ literal = … }`, `{ inherit = … }` → no normalize

**Missing** = token absent from catalog → try next candidate. Unparseable color or failed transform = resolver error.

Section `_inherit = "base"` means “copy entire subsection from `default-theme.toml` unless a leaf overrides.”

---

## Action roles (import mapping)

| Luma slot | shadcn recipe | Palette aliases |
|---|---|---|
| `action.prominent` | primary / default | `brand`, `brand_fg` |
| `action.standard` | secondary (filled) | `fill_secondary`, `fill_secondary_fg` |
| `action.subtle` | outline | `outline_surface`, `outline_fg`, `outline_border` |
| `action.ghost` | ghost | `canvas`, `accent` (hover) |

`destructive` → `form.input.invalid_border` only (no `action.danger` in current SDK schema).

---

## Importer usage

```bash
# One file
cargo run -p luma-theme -- import apps/gallery/tweakcn/astrovista.css \
  --out apps/gallery/src/assets/themes/tweakcn-astrovista.toml \
  --name "Astrovista"

# Every *.css in a directory → *.toml in dest-dir
cargo run -p luma-theme -- import \
  --source-dir apps/gallery/tweakcn \
  --dest-dir apps/gallery/src/assets/themes
```

Debug catalog only:

```bash
cargo run -p luma-theme -- catalog path/to/theme.css
```

Resolver should emit a **binding report** per slot (which palette/token won, fallbacks used).

---

## Tuning workflow

1. Parse CSS → catalog fixture for one theme.
2. Run resolver → `theme.toml` draft.
3. Gallery visual check (`-- astrovista` / `-- jarvis`).
4. Adjust **`[palette]`** for semantic conflicts (brand vs nav selected).
5. Adjust **lexicon overrides** only where palette is insufficient.
6. Re-run golden test: `LumaTheme::from_toml_str`.

See `[[conflicts]]` at the bottom of `lexicon.toml` for documented open questions.
