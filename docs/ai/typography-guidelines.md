# Typography Guidelines

This document describes how typography should be handled in `gpui-luma`, which APIs to use, and where each style system is appropriate.

## Goals

Typography in this workspace should be:

- consistent
- grepable
- theme-aware where it matters
- easy to apply without repeating `text_size + line_height + font_weight`

We use two related but different approaches:

1. **GPUI built-in text scale helpers** for pragmatic app-level sizing
2. **Luma typography tokens and look resolution** for SDK/look-layer semantics and full text style control

---

## Core Rule Set

## 1. App code (`apps/*`)

App code may use GPUI's built-in text helpers:

- `text_xs()`
- `text_sm()`
- `text_base()`
- `text_lg()`
- `text_xl()`
- `text_2xl()`

Use these when:

- you need a simple fixed text scale
- the text is local to one screen or demo
- grepability matters more than theme-driven text behavior
- the text does not need to be a reusable design-system semantic role

### Preferred app usage

Use GPUI built-ins for:

- compact helper copy
- inline metadata
- one-off section labels
- small demo labels
- simple app-local content where a fixed scale is acceptable

### App code should avoid

Avoid raw hardcoded `.text_size(px(...))` for normal semantic text when a built-in helper or resolved typography style would express it more clearly.

Examples to avoid in app code when unnecessary:

```rust
.text_size(px(12.0)).line_height(px(16.0))
.text_size(px(14.0)).line_height(px(20.0))
```

Prefer:

```rust
.text_sm()
.text_base()
.text_lg()
```

or, if the text should follow the active look more closely:

```rust
.typography_style(look.typography_scale(...))
```

---

## 2. SDK / look layer (`crates/sdk`, `crates/look-shadcn`)

SDK and look-layer code should **not** use GPUI's `text_sm()`-style helpers for normal semantic UI text.

Use the Luma typography system instead.

That means SDK/look code should rely on:

- `LumaTypography`
- `LumaTextStyle`
- `ShadcnLook::typography_role(...)`
- `ShadcnLook::typography_scale(...)`
- `LumaTypographyExt::text_h1()` ... `text_p()`
- `LumaTypographyExt::typography_style(...)`

### Why

GPUI built-ins only set font size.

They do **not** resolve:

- line height
- weight
- semantic role
- theme-specific typography policy

The SDK must own those concerns.

---

## Preferred APIs

## Semantic roles

Use semantic roles when the text represents document or layout structure.

Available helpers:

- `text_h1()`
- `text_h2()`
- `text_h3()`
- `text_h4()`
- `text_p()`

Use them for:

- page titles
- hero titles
- major section headings
- panel headings
- body paragraphs

### Example

```rust
div().text_h2().child("Account Settings")
div().text_p().child("Manage notification preferences and linked accounts.")
```

---

## Theme-resolved scale access

Use `ShadcnLook` when you want a full `LumaTextStyle` for a scale step.

Available resolvers:

```rust
look.typography_scale(ShadcnTextSize::Xs)
look.typography_scale(ShadcnTextSize::Sm)
look.typography_scale(ShadcnTextSize::Base)
look.typography_scale(ShadcnTextSize::Lg)
look.typography_scale(ShadcnTextSize::Xl)
look.typography_scale(ShadcnTextSize::TwoXl)
```

Use these when:

- you need a full style object
- you want to apply theme-resolved size + line height + weight together
- a semantic role is too specific, but GPUI fixed helpers are too limited

---

## `typography_style(...)`

Use `typography_style(style)` whenever you already have a `LumaTextStyle`.

This is the preferred way to apply a full text style without repeating the same 3 calls.

### Preferred

```rust
let style = look.typography_scale(ShadcnTextSize::Sm);
div().typography_style(style).child("Helper text")
```

### Avoid repeating manually

```rust
div()
    .text_size(px(style.size))
    .line_height(px(style.line_height))
    .font_weight(style.weight)
```

Use `typography_style(...)` instead.

---

## Luma scale helpers

Currently available:

- `typography_xs()`
- `typography_sm()`
- `typography_md()`
- `typography_lg()`
- `typography_xl()`
- `typography_2xl()`

These exist to provide fluent access to theme-aware scale application while avoiding name collisions with GPUI.

## Current recommendation

These helpers are supported, but they are **not the preferred long-term default** for all new code.

### Prefer instead

- in **apps**: GPUI built-in helpers like `text_sm()`
- in **SDK/look**: `typography_style(look.typography_scale(...))`

### Keep using when appropriate

It is still okay to use `typography_xs()`-style helpers when they make code clearer, especially in look-aware rendering code.

---

## Name collision rule

Do not create SDK typography helpers named:

- `text_xs()`
- `text_sm()`
- `text_base()`
- `text_lg()`
- `text_xl()`
- `text_2xl()`

These names already belong to GPUI `Styled`.

The collision is not just syntactic; the semantics differ:

- GPUI `text_sm()` = fixed font size only
- Luma typography = size + line height + weight, possibly theme-resolved

That is why SDK-owned scale helpers use `typography_*` names instead.

---

## Exceptions: when hardcoded text sizing is acceptable

Literal `.text_size(px(...))` remains acceptable for:

- icon glyph sizing
- debug views
- inspectors
- measurement overlays
- diagrams
- intentionally numeric visual mockups
- places where text sizing is part of the visual artifact rather than reusable UI typography

Examples:

- lucide icon font renderers
- inspector layout grids
- prototype geometry overlays

Do not spend effort forcing these into the typography system unless there is clear user-facing value.

---

## Decision guide

## Use GPUI built-ins when...

- you are in `apps/*`
- the text is local and pragmatic
- fixed scale is fine
- theme-driven line-height/weight does not matter much

Example:

```rust
div().text_sm().child("Last event: none")
```

## Use semantic roles when...

- the text represents structure
- the meaning is more important than the raw size

Example:

```rust
div().text_h3().child("Panel Heading")
```

## Use `typography_style(...)` when...

- you already have a `LumaTextStyle`
- you want a full style applied cleanly
- you want to avoid manual repetition

Example:

```rust
let caption = look.typography_scale(ShadcnTextSize::Xs);
div().typography_style(caption).text_color(muted).child("Secondary metadata")
```

---

## Theme Studio size selector note

The `SM / MD / LG` selector in theme studio currently behaves as a **control-size selector**, not a typography-size selector.

That means:

- controls that consume `ControlSize` may resize
- text resolved from `ShadcnLook::typography_role(...)` / `typography_scale(...)` does **not** automatically change with that selector

This is expected under the current architecture.

If broader UI scaling is desired later, it should be handled as a separate design-system feature.

---

## Recommended long-term usage

### Apps
Prefer:

- GPUI built-in text helpers for simple scale
- `typography_style(...)` only when a look-resolved style is clearly useful

### SDK / look layer
Prefer:

- semantic role helpers (`text_h1()` ... `text_p()`)
- explicit `typography_style(look.typography_scale(...))`
- explicit `typography_style(look.typography_role(...))`

### De-emphasize over time
The `typography_xs()`-style fluent helpers may eventually be deprecated in favor of:

- GPUI built-ins in apps
- explicit `typography_style(...)` in SDK/look code

They remain valid for now.

---

## Examples

### App-level pragmatic helper text

```rust
div().text_xs().line_height(px(15.0)).text_color(muted).child("Keyboard: Enter selects the active row.")
```

### SDK/look semantic heading

```rust
div().text_h4().text_color(title_text).child("Field Group")
```

### Full look-resolved style application

```rust
let style = look.typography_scale(ShadcnTextSize::Sm);
div().typography_style(style).text_color(body).child("Theme-aware helper text")
```

### Explicit role resolution

```rust
let hero = look.typography_role(ShadcnTextRole::H1);
div().typography_style(hero).text_color(title_text).child("Foundation Controls")
```
