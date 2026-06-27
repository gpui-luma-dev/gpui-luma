# Issue #0: Theme Reconcile (tweakcn HTML vs theme-studio)

## Summary

Matching tweakcn by eye plus a single theme CSS file (e.g. `elegent-luxury.css`) will stay **approximate**, not pixel-perfect. The CSS file sets **tokens**; tweakcn Cards UI applies **per-component Tailwind utility stacks** with modifiers (`dark:`, `hover:`, `data-[state=…]`, `[&_svg]:`, layout utilities, etc.). Our stack uses **semantic rules** (`style.toml`, interaction layers, palettes) — a different model.

**Practical goal:** match **trigger families** consistently across themes, not every utility class in saved HTML.

**Reference:** saved tweakcn editor HTML (`view-source_https___tweakcn.com_editor_theme.html`) — Cards tab, dark mode, default theme in that snapshot (not necessarily elegant-luxury loaded in the static file).

---

## Why 1:1 reverse-engineering is incomplete

| tweakcn | gpui-luma |
|---------|-----------|
| Token values in theme CSS | `crates/look-shadcn/assets/tweakcn/*.css` |
| Utility stacks on each element | `style.toml` + control palettes |
| Modifier cascade (dark overrides light hover text, etc.) | Explicit layer resolution (`InteractionLayer`) |
| Composited fills (`input/30` over `--card`) | Resolved alpha tokens |

Example modifiers on **select-trigger** (Report Area):

- `dark:bg-input/30`, `dark:hover:bg-input/50`
- `[&_svg:not([class*='text-'])]:text-muted-foreground`
- No `hover:text-accent-foreground`

Example modifiers on **popover-trigger outline** (Team role pickers):

- `dark:bg-input/30`, `dark:hover:bg-input/50`
- `hover:bg-accent`, `hover:text-accent-foreground` (unprefixed — can still apply in dark for text)
- `shadow-none`, `h-8`, `ml-auto`

Same tokens, different stacks → different painted result. Chasing one CSS file per hover mismatch is mostly tail-chasing.

**Local template modifiers** (`ControlTemplate::with_modifier`, `ext.rs`) help for child-part styling and small deltas, but do not replace **trigger family** taxonomy.

---

## HTML trigger inventory (Cards-relevant)

Counts in saved HTML: **3** `select-trigger`, **5** `popover-trigger`, **2** `dropdown-menu-trigger`.

### Three styling families

| Family | HTML slots | Dark bg hover | Hover text / caret |
|--------|------------|---------------|-------------------|
| **A — Form select** | `select-trigger` | `input/50` | Foreground unchanged; caret `muted-foreground` |
| **B — Outline popover** | `popover-trigger` + `outline` | `input/50` | `accent-foreground` (from unprefixed `hover:text-accent-foreground`) |
| **C — Ghost menu** | `dropdown-menu-trigger`, ghost `popover-trigger` | `accent/50` | `accent-foreground` |

### Per-card triggers (tweakcn)

| Card | Control | HTML slot | Variant | Notes |
|------|---------|-----------|---------|-------|
| **Team Members** | Role: Owner / Developer / Billing | `popover-trigger` | outline sm | `aria-haspopup=dialog` — **not** Select |
| **Report an issue** | Area, Security Level | `select-trigger` | — | `role=combobox` |
| **Share this document** | Link field | `select-trigger` | — | |
| **Date picker with range** | Date field | `popover-trigger` | outline | |
| **Payments** | Row ⋯ menu | `dropdown-menu-trigger` | ghost | |
| **(chrome)** | Theme / menus | `popover-trigger` ghost, `dropdown-menu-trigger` ghost | | |

### Developer vs Billing (label collision)

Same **label text** can mean different controls on different cards:

| Label seen | Card | tweakcn | Family |
|------------|------|---------|--------|
| Developer | Team Members (Jackson Lee row) | `popover-trigger` outline | B |
| Billing | Team Members (Isabella row) | `popover-trigger` outline | B |
| Billing | Report an issue (Area default value) | `select-trigger` | A |

Team Developer and Team Billing are **identical** in HTML (byte-identical classes except label). Report “Billing” is a **different family**.

### SDK mapping (tweakcn → gpui-luma)

| tweakcn | Intended SDK control |
|---------|---------------------|
| `select-trigger` | `Selector` (input fill / `ShadcnTextFieldStyle::Input`) |
| `popover-trigger` outline | `PopupMenu` with `TriggerStyle::Outline` (or outline trigger variant) |
| `dropdown-menu-trigger` ghost | `PopupMenu` with `TriggerStyle::Ghost` |
| `data-slot=button` (no haspopup) | `Button` (variant from `data-variant`) |

---

## theme-studio Cards tab vs tweakcn HTML

### Coverage

| tweakcn HTML card | theme-studio panel | In Cards tab? |
|-------------------|-------------------|---------------|
| Upgrade your subscription | `UpgradePanel` | ✓ |
| Create an account | `AccountPanel` | ✓ |
| Team Members | `TeamPanel` | ✓ |
| Report an issue | `ReportPanel` | ✓ |
| Payments | `PaymentsPanel` | ✓ |
| Cookie Settings | `CookiesPanel` | ✓ |
| Date picker with range | — | missing |
| Share this document | — | missing |
| Move Goal / Exercise Minutes / charts | — | missing |
| Chat | `ChatPanel` | theme-studio only |
| Tree view | `TreeViewPanel` | theme-studio only |
| Accordion | `AccordionPanel` | theme-studio only |
| System preferences | `SystemPreferencesPanel` | theme-studio only |

Cards tab panels: `InspectableId::CARDS` in `apps/theme-studio/src/studio/inspectable.rs`.

### Per-panel control comparison

| Panel | tweakcn HTML | theme-studio (`apps/theme-studio/src/studio/panels/`) | Match? |
|-------|--------------|--------------------------------------------------------|--------|
| **Team Members** | 3× `popover-trigger` outline sm | 3× `PopupMenu` outline (`team.rs`) | ✓ |
| **Report issue** | 2× `select-trigger` | 2× `Selector` (`report.rs`) | ✓ |
| **Report issue** | Cancel `ghost/sm`, Submit `default/sm` | `ghost_button`, `primary_button` | ≈ |
| **Payments** | Row ⋯ `dropdown-menu-trigger` ghost | `PopupMenu` ghost per row (`payments.rs`) | ✓ |
| **Payments** | Table + footer prev/next outline/sm | `PagingListView` + `outline_button` | ≈ |
| **Upgrade** | `radio-group`, 2× `checkbox` | `RadioGroup`, 2× `Checkbox` (`upgrade.rs`) | ✓ |
| **Upgrade** | Cancel **outline/sm** | `outline_button` | ✓ |
| **Upgrade** | Upgrade Plan **default/sm** | `primary_button` | ≈ |
| **Create account** | GitHub/Google **outline** | `outline_button` (`account.rs`) | ✓ |
| **Create account** | Create account **default** | `primary_button` | ≈ |
| **Cookie Settings** | 2× `switch` | 2× `Switch` (`cookies.rs`) | ✓ |
| **Cookie Settings** | Save **outline** | `outline_button` | ✓ |
| **Chat** | (no tweakcn card) | TextField + icon buttons (`chat.rs`) | theme-studio only |

### Label vs component (theme-studio specifics)

| theme-studio control | File | tweakcn equivalent |
|---------------------|------|------------------|
| `sofia_menu` label "Owner" | `team.rs` | `popover-trigger` outline |
| `jackson_menu` label "Developer" | `team.rs` | `popover-trigger` outline |
| `isabella_menu` label "Billing" | `team.rs` | `popover-trigger` outline |
| `area_selector` label "Billing" | `report.rs` | `select-trigger` (Area field) |
| `security_selector` label "Severity 2" | `report.rs` | `select-trigger` |

---

## Recommended approach

1. **Bucket by trigger family** (A / B / C), not by displayed label.
2. **Eyeball + HTML** for painted behavior per family: bg on hover, text change, caret change.
3. **One rule set per family** in `style.toml` / look crates; ignore layout-only utilities unless visually material.
4. **Spot-check 2–3 themes** — tune the family rule, not individual cards.

### Priority fixes (align theme-studio to tweakcn)

1. ~~**Team** (`team.rs`) — `PopupMenu` outline triggers instead of `Selector`.~~ **Done**
2. ~~**Payments** (`payments.rs`) — `PopupMenu` ghost for row ⋯ actions.~~ **Done**
3. ~~**Upgrade / Cookies** — `outline_button` for Cancel / Save (not `secondary_button`).~~ **Done**
4. ~~**Payments** prev/next — `outline_button` (tweakcn outline/sm).~~ **Done**

### What we already aligned (this effort)

- Selector trigger: `input/30` → `input/50` dark hover; caret via `trigger_icon` (`muted-foreground`).
- Outline button dark hover: `input/50` bg; `accent-foreground` text on hover (family B in `style.toml`).

---

## Workflow for future theme checks

1. Parse saved HTML for `data-slot` on triggers (`select-trigger`, `popover-trigger`, `dropdown-menu-trigger`).
2. Classify into family A / B / C (hover modifier summary).
3. Compare theme-studio panel Rust to that slot + variant, not to label text.
4. Accept approximate parity unless a specific family looks wrong across themes.

---

## Related files

| Area | Path |
|------|------|
| Theme CSS (tokens only) | `crates/look-shadcn/assets/tweakcn/elegent-luxury.css` |
| Style rules | `crates/look-shadcn/assets/style.toml` |
| Selector look | `crates/look-shadcn/src/controls/selector.rs`, `crates/sdk/src/controls/selector/` |
| Popup menu look | `crates/look-shadcn/src/controls/popup_menu.rs`, `crates/sdk/src/controls/popup_menu/` |
| theme-studio panels | `apps/theme-studio/src/studio/panels/` |
| Cards tab filter | `apps/theme-studio/src/studio/inspectable.rs` (`CARDS`) |
