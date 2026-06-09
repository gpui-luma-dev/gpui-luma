use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    braced, bracketed,
    parse::{Parse, ParseStream},
    parse_macro_input, Expr, Ident, LitStr, Pat, Token, Type,
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
    /// A quoted CSS token like `"background"`, `"input/50"`, or `"transparent"`.
    Css(LitStr),
    /// A raw Rust expression like `None`.
    Expr(Expr),
}

impl Parse for TableValue {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(LitStr) {
            Ok(TableValue::Css(input.parse()?))
        } else {
            Ok(TableValue::Expr(input.parse()?))
        }
    }
}

impl Parse for MultiValueLookTable {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let _: Ident = input.parse()?;
        let _: Token!(:) = input.parse()?;
        let name: Ident = input.parse()?;
        let _: Token!(,) = input.parse()?;

        let _: Ident = input.parse()?;
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

        let _: Ident = input.parse()?;
        let _: Token!(:) = input.parse()?;
        let output_type: Type = input.parse()?;

        let output_fields_content;
        braced!(output_fields_content in input);
        let mut output_fields = Vec::new();
        while !output_fields_content.is_empty() {
            output_fields.push(output_fields_content.parse()?);
            if output_fields_content.peek(Token!(,)) {
                let _: Token!(,) = output_fields_content.parse()?;
            }
        }
        let _: Token!(,) = input.parse()?;

        let _: Ident = input.parse()?;
        let _: Token!(:) = input.parse()?;
        let rows_content;
        bracketed!(rows_content in input);
        let mut matrix_rows = Vec::new();

        while !rows_content.is_empty() {
            let mut patterns = Vec::new();
            loop {
                let pattern_content;
                bracketed!(pattern_content in rows_content);
                patterns.push(pattern_content.call(Pat::parse_single)?);

                if rows_content.peek(Token!(|)) {
                    let _: Token!(|) = rows_content.parse()?;
                } else {
                    break;
                }
            }

            let _: Token!(=>) = rows_content.parse()?;

            let mut outputs = Vec::new();
            loop {
                outputs.push(rows_content.parse()?);
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

        Ok(Self { name, inputs, output_type, output_fields, matrix_rows })
    }
}

fn input_ident<'a>(inputs: &'a [(Ident, Type)], name: &str) -> &'a Ident {
    inputs
        .iter()
        .find(|(ident, _)| ident == name)
        .map(|(ident, _)| ident)
        .unwrap_or_else(|| panic!("declare_look_table! `{name}` special token requires an input named `{name}`"))
}

fn css_field_resolver(lit: &LitStr, inputs: &[(Ident, Type)]) -> TokenStream2 {
    let value = lit.value();
    match value.as_str() {
        "@outline_layer" => {
            let layer = input_ident(inputs, "layer");
            quote! { resolver.resolve_outline_layer_decl(#layer)? }
        }
        "@action_layer" => {
            let style = input_ident(inputs, "style");
            let layer = input_ident(inputs, "layer");
            quote! { resolver.resolve_action_layer_decl(#style, #layer)? }
        }
        "@action_foreground" => {
            let style = input_ident(inputs, "style");
            quote! { resolver.resolve_action_foreground_decl(#style)? }
        }
        "@action_default" => {
            let style = input_ident(inputs, "style");
            quote! {
                resolver.resolve_action_layer_decl(#style, gpui_luma::theme::InteractionLayer::Default)?
            }
        }
        "@primary_layer" => {
            let layer = input_ident(inputs, "layer");
            quote! {
                resolver.resolve_action_layer_decl(
                    crate::controls::ShadcnButtonStyle::Primary,
                    #layer,
                )?
            }
        }
        "@primary_default" => quote! {
            resolver.resolve_action_layer_decl(
                crate::controls::ShadcnButtonStyle::Primary,
                gpui_luma::theme::InteractionLayer::Default,
            )?
        },
        "@darken_border" => quote! { resolver.resolve_darken_border_decl(0.08)? },
        "@accent_whisper_40" => quote! { resolver.resolve_accent_whisper_decl(40)? },
        "@accent_whisper_pressed_40" => quote! { resolver.resolve_accent_whisper_pressed_decl(40)? },
        "@label" => {
            let disabled = input_ident(inputs, "disabled");
            quote! { resolver.resolve_label_decl(#disabled)? }
        }
        other if other.starts_with("first(") && other.ends_with(')') => {
            let tokens: Vec<_> =
                other.trim_start_matches("first(").trim_end_matches(')').split(',').map(str::trim).collect();
            quote! { resolver.resolve_first_decl(&[#(#tokens),*])? }
        }
        other if other.starts_with("first_layer(") && other.ends_with(')') => {
            let layer = input_ident(inputs, "layer");
            let tokens: Vec<_> =
                other.trim_start_matches("first_layer(").trim_end_matches(')').split(',').map(str::trim).collect();
            quote! { resolver.resolve_first_layer_decl(&[#(#tokens),*], #layer)? }
        }
        _ => quote! { resolver.resolve_decl(#lit)? },
    }
}

fn metadata_output_string(field: &Ident, val: &TableValue) -> TokenStream2 {
    match val {
        TableValue::Css(lit) => {
            let val_str = lit.value();
            match val_str.as_str() {
                "transparent" => quote! { format!("{}: {}", stringify!(#field), #val_str) },
                "@outline_layer" => quote! { format!("{}: @outline_layer", stringify!(#field)) },
                "@action_layer" => quote! { format!("{}: @action_layer", stringify!(#field)) },
                "@action_foreground" => quote! { format!("{}: @action_foreground", stringify!(#field)) },
                "@label" => quote! { format!("{}: @label", stringify!(#field)) },
                other if other.starts_with("first(") => {
                    quote! { format!("{}: {}", stringify!(#field), #other) }
                }
                other if other.starts_with("first_layer(") => {
                    quote! { format!("{}: {}", stringify!(#field), #other) }
                }
                _ => quote! { format!("{}: --{}", stringify!(#field), #val_str) },
            }
        }
        TableValue::Expr(expr) => quote! { format!("{}: {}", stringify!(#field), stringify!(#expr)) },
    }
}

fn clean_pattern_string(pat: &Pat) -> String {
    let raw = quote! { #pat }.to_string();
    raw.replace("ShadcnButtonStyle :: ", "")
        .replace("InteractionLayer :: ", "")
        .replace("ThemeMode :: ", "")
        .replace(" ", "")
}

#[proc_macro]
pub fn declare_look_table(input: TokenStream) -> TokenStream {
    let MultiValueLookTable { name, inputs, output_type, output_fields, matrix_rows } =
        parse_macro_input!(input as MultiValueLookTable);

    let input_names: Vec<_> = inputs.iter().map(|(n, _)| n).collect();
    let input_types: Vec<_> = inputs.iter().map(|(_, t)| t).collect();

    let metadata_name = Ident::new(&format!("{name}_metadata"), name.span());

    let match_arms = matrix_rows.iter().map(|row| {
        let patterns = &row.patterns;
        let outputs = &row.outputs;

        let field_resolvers = output_fields.iter().zip(outputs).map(|(field, val)| match val {
            TableValue::Css(lit) => {
                let resolver_expr = css_field_resolver(lit, &inputs);
                quote! { #field: (#resolver_expr).into() }
            }
            TableValue::Expr(expr) => quote! { #field: (#expr).into() },
        });

        quote! {
            (#(#patterns),*) => {
                Ok(#output_type {
                    #(#field_resolvers),*
                })
            }
        }
    });

    let metadata_rows = matrix_rows.iter().map(|row| {
        let input_strings = row.patterns.iter().map(|pat| {
            let cleaned = clean_pattern_string(pat);
            quote! { #cleaned.to_string() }
        });

        let output_strings =
            output_fields.iter().zip(&row.outputs).map(|(field, val)| metadata_output_string(field, val));

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
