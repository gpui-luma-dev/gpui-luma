use proc_macro::TokenStream;
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
            TableValue::Css(lit) => quote! { #field: resolver.resolve_decl(#lit)?.into() },
            TableValue::Expr(expr) => quote! { #field: #expr },
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

        let output_strings = output_fields.iter().zip(&row.outputs).map(|(field, val)| match val {
            TableValue::Css(lit) => {
                let val_str = lit.value();
                if val_str == "transparent" {
                    quote! { format!("{}: {}", stringify!(#field), #val_str) }
                } else {
                    quote! { format!("{}: --{}", stringify!(#field), #val_str) }
                }
            }
            TableValue::Expr(expr) => quote! { format!("{}: {}", stringify!(#field), stringify!(#expr)) },
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
