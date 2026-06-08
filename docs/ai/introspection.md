# Multi-Value Look Tables & Introspection Proc-Macro

> [!NOTE]
> **Status:** Design Proposal & Specification (Under Review)  
> **Workspace Edition Target:** Rust 2024  
> **Related:** [inspector.md](file:///Users/scg/Developer/GitHub/gpui-luma/docs/ai/inspector.md) (Live Theme Inspector UI design)

---

## 1. Specification of Provenance & Resolvers

Before looking at the implementation, we define where the provenance types live, what their inputs accept, and how they hook into the lookup lifecycle.

### A. Location: `crates/look-shadcn`
To adhere to the design policy in [look-cleanup.md](file:///Users/scg/Developer/GitHub/gpui-luma/docs/ai/look-cleanup.md), the core SDK (`crates/sdk`) must remain **styling-agnostic**. It knows nothing about CSS tokens, catalogs, selector blocks (`:root`/`.dark`), or line numbers.
* **`ResolvedColor` and `ColorSource`** live in `crates/look-shadcn/src/provenance.rs`.
* **Standard paint path**: Resolvers in `look-shadcn` evaluate the styling tables, extract the `.value` (`Hsla`), and hand it over to the SDK templates. No lookup overhead or custom metadata structures leak into normal paint frames.
* **Inspect path**: The Theme Studio inspector queries the look resolvers directly, which return the full `ResolvedColor` structures (with their nested `ColorSource` metadata) for visualization.

### B. `resolve_decl` Parsing Grammar
The `resolve_decl` helper accepts a string declaration slice and parses it according to the following rules:

1. **Double-Hyphen Tolerant**: Automatically strips leading `--` if present (e.g. `"--background"` and `"background"` resolve to the same catalog key).
2. **Transparent Keyword**: Matches `"transparent"` (case-insensitive) and resolves immediately to `hsla(0.0, 0.0, 0.0, 0.0)` with `ColorSource::Transparent`.
3. **Slash Alpha Suffix**: Matches `/` followed by an opacity percentage (e.g., `"input/50"` or `"accent/30%"`). It resolves the base token against the catalog, parses the base color, applies the alpha modifier, and returns `ColorSource::TokenAlpha` containing the base token and target alpha percent.

### C. Single Source of Truth (Resolving Duplication Paths)
To prevent drift between global palette definitions and control-level resolvers, we designate the look table as the **single source of truth** for style roles:

1. **Lightweight `ShadcnPalette`**: The global `ShadcnPalette` struct does not precompute or cache action roles (like `outline_role` or `ghost_role`) inside `palette.rs`. It only holds the base CSS tokens loaded directly from the catalog.
2. **On-Demand Resolution**: Controls evaluate their own `declare_look_table!` blocks dynamically at resolution time. This eliminates double-resolution logic and ensures that styling matrices are kept in a single location for each control.

---

## 2. Procedural Macro Implementation

This code belongs in the macro library crate (`crates/look-shadcn-macros`).

### `Cargo.toml`
```toml
[package]
name = "gpui-luma-look-shadcn-macros"
version = "0.1.0"
edition = "2024"

[lib]
proc-macro = true

[dependencies]
proc-macro2 = "1.0"
quote = "1.0"
syn = { version = "2.0", features = ["full", "parsing"] }
```

### `src/lib.rs`
```rust
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, braced, bracketed, Ident, Token, Type, Expr, Pat, LitStr
};

struct MultiValueLookTable {
    name: Ident,
    inputs: Vec<(Ident, Type)>,
    output_type: Type,
    output_fields: Vec<Ident>,
    matrix_rows: Vec<MatrixRow>,
}

struct MatrixRow {
    patterns: Vec<Pat>,
    outputs: Vec<TableValue>,
}

enum TableValue {
    /// A quoted CSS token like `"--background"`, `"--input/50"`, or `"transparent"`
    Css(LitStr),
    /// A raw Rust expression like `None` or `ShadcnToken::Primary`
    Expr(Expr),
}

impl Parse for TableValue {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(LitStr) {
            let lit: LitStr = input.parse()?;
            Ok(TableValue::Css(lit))
        } else {
            let expr: Expr = input.parse()?;
            Ok(TableValue::Expr(expr))
        }
    }
}

impl Parse for MultiValueLookTable {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // 1. Parse name: resolve_button_palette,
        let _: Ident = input.parse()?; // "name"
        let _: Token!(:) = input.parse()?;
        let name: Ident = input.parse()?;
        let _: Token!(,) = input.parse()?;

        // 2. Parse inputs: { ... },
        let _: Ident = input.parse()?; // "inputs"
        let _: Token!(:) = input.parse()?;
        let inputs_content;
        braced!(inputs_content in input);
        let mut inputs = Vec::new();
        while !inputs_content.is_empty() {
            let arg_name: Ident = inputs_content.parse()?;
            let _: Token!(:) = inputs_content.parse()?;
            let arg_type: Type = inputs_content.parse()?;
            inputs.push((arg_name, arg_type));
            if inputs_content.peek(Token!(,)) {
                let _: Token!(,) = inputs_content.parse()?;
            }
        }
        let _: Token!(,) = input.parse()?;

        // 3. Parse output: ButtonColorPalette { background, foreground, border },
        let _: Ident = input.parse()?; // "output"
        let _: Token!(:) = input.parse()?;
        let output_type: Type = input.parse()?;
        
        let output_fields_content;
        braced!(output_fields_content in input);
        let mut output_fields = Vec::new();
        while !output_fields_content.is_empty() {
            let field_name: Ident = output_fields_content.parse()?;
            output_fields.push(field_name);
            if output_fields_content.peek(Token!(,)) {
                let _: Token!(,) = output_fields_content.parse()?;
            }
        }
        let _: Token!(,) = input.parse()?;

        // 4. Parse matrix: [ ... ]
        let _: Ident = input.parse()?; // "matrix"
        let _: Token!(:) = input.parse()?;
        let rows_content;
        bracketed!(rows_content in input);
        let mut matrix_rows = Vec::new();

        while !rows_content.is_empty() {
            // Parse patterns (delimited by |)
            let mut patterns = Vec::new();
            loop {
                let pattern_content;
                bracketed!(pattern_content in rows_content);
                let pat: Pat = pattern_content.parse()?;
                patterns.push(pat);

                if rows_content.peek(Token!(|)) {
                    let _: Token!(|) = rows_content.parse()?;
                } else {
                    break;
                }
            }

            // Parse Arrow =>
            let _: Token!(=>) = rows_content.parse()?;

            // Parse output expressions/declarations (delimited by |)
            let mut outputs = Vec::new();
            loop {
                let val: TableValue = rows_content.parse()?;
                outputs.push(val);

                if rows_content.peek(Token!(|)) {
                    let _: Token!(|) = rows_content.parse()?;
                } else {
                    break;
                }
            }

            matrix_rows.push(MatrixRow { patterns, outputs });

            if rows_content.peek(Token!(,)) {
                let _: Token!(,) = rows_content.parse()?;
            }
        }

        Ok(Self {
            name,
            inputs,
            output_type,
            output_fields,
            matrix_rows,
        })
    }
}

/// Helper to strip verbose enum namespaces for clean static metadata
fn clean_pattern_string(pat: &Pat) -> String {
    let raw = quote! { #pat }.to_string();
    raw.replace("ShadcnButtonStyle :: ", "")
       .replace("Layer :: ", "")
       .replace("ThemeMode :: ", "")
       .replace(" ", "")
}

#[proc_macro]
pub fn declare_look_table(input: TokenStream) -> TokenStream {
    let MultiValueLookTable {
        name,
        inputs,
        output_type,
        output_fields,
        matrix_rows,
    } = parse_macro_input!(input as MultiValueLookTable);

    let input_names: Vec<_> = inputs.iter().map(|(n, _)| n).collect();
    let input_types: Vec<_> = inputs.iter().map(|(_, t)| t).collect();
    
    let metadata_name = Ident::new(&format!("{name}_metadata"), name.span());

    // Generate Match Arms for the evaluation function
    let match_arms = matrix_rows.iter().map(|row| {
        let patterns = &row.patterns;
        let outputs = &row.outputs;
        
        let field_resolvers = output_fields.iter().zip(outputs).map(|(field, val)| match val {
            TableValue::Css(lit) => {
                // Resolves CSS token string. `.into()` automatically wraps it in Option
                // if the target field is Option<ResolvedColor>.
                quote! { #field: resolver.resolve_decl(#lit)?.into() }
            }
            TableValue::Expr(expr) => {
                // Raw Rust expressions (e.g. `None` or custom tokens) map directly
                quote! { #field: #expr }
            }
        });

        quote! {
            (#(#patterns),*) => {
                Ok(#output_type {
                    #(#field_resolvers),*
                })
            }
        }
    });

    // Generate Metadata rows for introspection, preserving clean CSS names
    let metadata_rows = matrix_rows.iter().map(|row| {
        let input_strings = row.patterns.iter().map(|pat| {
            let cleaned = clean_pattern_string(pat);
            quote! { #cleaned.to_string() }
        });
        
        let output_strings = output_fields.iter().zip(&row.outputs).map(|(field, val)| match val {
            TableValue::Css(lit) => {
                let val_str = lit.value();
                // Clean formatting: avoids double prepends on keywords like "transparent"
                if val_str == "transparent" {
                    quote! { format!("{}: {}", stringify!(#field), #val_str) }
                } else {
                    quote! { format!("{}: --{}", stringify!(#field), #val_str) }
                }
            }
            TableValue::Expr(expr) => {
                quote! { format!("{}: {}", stringify!(#field), stringify!(#expr)) }
            }
        });

        quote! {
            crate::provenance::TableRuleMetadata {
                inputs: vec![#(#input_strings),*],
                outputs: vec![#(#output_strings),*],
            }
        }
    });

    let expanded = quote! {
        pub fn #name(
            resolver: &crate::provenance::LookResolver<'_>,
            #(#input_names: #input_types),*
        ) -> anyhow::Result<#output_type> {
            match (#(#input_names),*) {
                #(#match_arms)*
            }
        }

        pub fn #metadata_name() -> Vec<crate::provenance::TableRuleMetadata> {
            vec![#(#metadata_rows),*]
        }
    };

    expanded.into()
}
```

---

## 3. Sample Usage for `shadcn/controls/button.rs`

Rather than resolving arbitrary styles procedurally, we declare a single state table mapping styles and states to colors.

### A. Define the Structured Palette
First, define a simplified struct containing only the state-dependent colors (leaving typography/fonts to the layout engine):

```rust
#[derive(Clone, Debug)]
pub struct ButtonColorPalette {
    pub background: ResolvedColor,
    pub foreground: ResolvedColor,
    pub border: Option<ResolvedColor>, // Optional border field
}

impl ButtonColorPalette {
    /// Safe fallback implementation
    pub fn fallback() -> Self {
        Self {
            background: ResolvedColor::transparent(),
            foreground: ResolvedColor::fallback_foreground(),
            border: None,
        }
    }
}
```

### B. Declare the Style Table
Use the macro to map style, state layer, and mode to the colors. Note how the target field names (`background`, `foreground`, `border`) are declared in the output block:

```rust
declare_look_table! {
    name: resolve_button_colors,
    inputs: {
        style: ShadcnButtonStyle,
        layer: InteractionLayer,
        mode: ThemeMode,
    },
    output: ButtonColorPalette { background, foreground, border },
    matrix: [
        // Style      | Layer               | Mode   => background               | foreground              | border
        [Primary]     | [Layer::Default]    | [_]    => "primary"                | "primary-foreground"    | None,
        [Primary]     | [Layer::Hovered]    | [_]    => "primary-hover"          | "primary-foreground"    | None,
        [Primary]     | [Layer::Pressed]    | [_]    => "primary-active"         | "primary-foreground"    | None,
        [Primary]     | [Layer::Disabled]   | [_]    => "muted"                  | "muted-foreground"      | None,
        
        [Secondary]   | [Layer::Default]    | [_]    => "secondary"              | "secondary-foreground"  | None,
        [Secondary]   | [Layer::Hovered]    | [_]    => "secondary-hover"        | "secondary-foreground"  | None,
        [Secondary]   | [Layer::Pressed]    | [_]    => "secondary-active"       | "secondary-foreground"  | None,
        [Secondary]   | [Layer::Disabled]   | [_]    => "muted"                  | "muted-foreground"      | None,
        
        [Outline]     | [Layer::Default]    | [_]    => "transparent"            | "foreground"            | "border",
        [Outline]     | [Layer::Hovered]    | [Light]=> "accent"                 | "accent-foreground"     | "border",
        [Outline]     | [Layer::Hovered]    | [Dark] => "input/50"               | "accent-foreground"     | "input",
        [Outline]     | [Layer::Pressed]    | [_]    => "accent"                 | "accent-foreground"     | "border",
        [Outline]     | [Layer::Disabled]   | [_]    => "transparent"            | "muted-foreground"      | "border",
        
        [Ghost]       | [Layer::Default]    | [_]    => "transparent"            | "foreground"            | None,
        [Ghost]       | [Layer::Hovered]    | [Light]=> "accent"                 | "accent-foreground"     | None,
        [Ghost]       | [Layer::Hovered]    | [Dark] => "accent/50"              | "accent-foreground"     | None,
        [Ghost]       | [Layer::Pressed]     | [_]    => "accent"                 | "accent-foreground"     | None,
        [Ghost]       | [Layer::Disabled]   | [_]    => "transparent"            | "muted-foreground"      | None,
        
        [_]           | [_]                 | [_]    => "transparent"            | "foreground"            | None,
    ]
}
```

### C. Construct the final Appearance
In [button.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/look-shadcn/src/controls/button.rs), the resolver evaluates this table and merges the values with the fonts/typography configurations:

```rust
pub(crate) fn button_palette(
    ctx: &AppearanceContext,
    style: ShadcnButtonStyle,
    role: ButtonFamilyRole,
    _size: ControlSize,
) -> ButtonFamilyPalette {
    let state = ctx.state;
    let layer = state.layer();
    let theme_mode = ctx.theme_mode();
    let typography = ctx.typography();

    // 1. Resolve colors directly from the declarative state table
    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "button_resolver");
    let colors = resolve_button_colors(&resolver, style, layer, theme_mode)
        .unwrap_or_else(|_| ButtonColorPalette::fallback());

    // 2. Set conditional adorner (focus rings)
    let adorner = if state.focused {
        Some(AdornerSpec::FocusRing(FocusRingAdornerSpec {
            color: ctx.palette().focus_ring, // Reads basic ring token from the slim-down palette
            placement: match style {
                ShadcnButtonStyle::Ghost => AdornerPlacement::Inset,
                _ => AdornerPlacement::Oversize,
            },
            distance: ctx.metrics().border_width.default + ctx.metrics().focus.width,
            width: ctx.metrics().focus.width,
        }))
    } else {
        None
    };

    // 3. Assemble the final palette
    ButtonFamilyPalette {
        background: colors.background.into_hsla(),
        foreground: colors.foreground.into_hsla(),
        border: colors.border.map(|c| c.into_hsla()),
        adorner,
        typography: typography.text.label,
        font_family: typography.font.sans.family.clone().into(),
    }
}
```

---

## 4. Remaining Blockers, Risks, & Backlog

The following design gaps and implementation risks must be resolved before landing this architecture in the main workspace branches:

### A. Incomplete Button State Matrix
* **Risk**: The current sample matrix does not capture `ButtonFamilyRole::Toggle { selected: bool }`. Currently, unselected toggle buttons fall back to the `Outline` style.
* **Mitigation**: The input variables must include `role: ButtonFamilyRole` or `selected: bool` to ensure the matching arms can evaluate toggle states natively without manual check overrides.
* **Test Protection**: Existing visual tests in `button.rs` (such as `ghost_light_hover_pairs_accent_fill_with_accent_foreground`) will fail unless all state configurations are completely mapped in the table.

### B. ShadcnPalette Slim-down Refactoring Scope
* **Risk**: Modifying `ShadcnPalette` to be token-only is a wide-impact change. Controls other than buttons (like text inputs, lists, and menus) still rely on `palette.action(style)`.
* **Mitigation**: Perform a multi-phase migration. Maintain the legacy precomputed roles on `ShadcnPalette` temporarily as deprecated fields. Transition controls to their own declarative tables one-by-one. Once all controls are migrated, strip the deprecated fields and thin out `ShadcnPalette` to be token-only.
* **Focus Ring Token**: The reference to `ctx.palette().focus_ring` is valid because `focus_ring` resolves to the global CSS token `--ring`, which remains a first-class token in the slim-down palette.

### C. Underspecified Provenance APIs
* **Risk**: Structures like `LookResolver`, `ResolvedColor`, `TableRuleMetadata`, and custom `Into` implementations are referenced by the macro output but must be defined explicitly in `provenance.rs`.
* **Required Signatures**:
  ```rust
  pub struct TableRuleMetadata {
      pub inputs: Vec<String>,
      pub outputs: Vec<String>,
  }

  impl ResolvedColor {
      pub fn transparent() -> Self {
          Self { value: gpui::hsla(0.0, 0.0, 0.0, 0.0), source: ColorSource::Transparent }
      }
      pub fn into_hsla(self) -> gpui::Hsla {
          self.value
      }
  }
  ```

### D. Token Name Alignment
* **Risk**: CSS catalogs in look-shadcn do not define `"disabled-background"` or `"disabled-foreground"`.
* **Mitigation**: The table cells must use the standard variables `--muted` (for background) and `--muted-foreground` (for text foreground), aligning directly with the active CSS variables catalog.

### E. Static Metadata Formatting
* **Risk**: Formatting keywords statically (e.g. `format!("{}: --{}", field, val)`) will print `"background: --transparent"` which is syntactically invalid.
* **Mitigation**: The macro's metadata generator has been updated to check for raw CSS keywords like `"transparent"` and formats them directly without prepending the custom variable hyphens (`--`).
