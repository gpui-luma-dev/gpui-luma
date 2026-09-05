//! Shared look-runtime contracts for GPUI-Luma.
//!
//! # Authoring a look (without editing the SDK)
//!
//! 1. Depend on `luma` (SDK) and optionally `luma_look_core`.
//! 2. Implement the SDK `*Theme` traits your controls need (`ButtonFamilyTheme`,
//!    `TextFieldTheme`, …). Resolve into the SDK `*Look` / `*Palette` paint structs.
//! 3. Expose a **look-local** factory extension trait (e.g. `RadixLookControlExt`) that
//!    binds builders with `.template(...)` / `.theme(...)`. Do **not** add look variant
//!    enums (`primary`, `outline`, Radix `solid`/`soft`, …) to the SDK.
//! 4. Keep source interpretation (CSS catalogs, 12-step scales, Tailwind seeds) inside
//!    the look crate. Put only look-agnostic provenance / resolved-value shapes here.
//!
//! This crate deliberately excludes CSS parsing, `style.toml`, control factories, and
//! thread-local active-look helpers. Those stay look-specific until a second look proves
//! shared need.

mod provenance;

pub use provenance::{ColorSource, MetricSource, ResolvedColor, ResolvedMetric, ResolvedTypography, TypographySource};
