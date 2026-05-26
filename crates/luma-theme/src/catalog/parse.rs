use std::collections::BTreeMap;

use anyhow::{Context as _, Result, anyhow, bail};

use super::TokenCatalog;

pub fn parse_css_catalog(source: &str) -> Result<TokenCatalog> {
    let stripped = strip_css_comments(source);
    let light = parse_rule_block(&stripped, ":root")?;
    let dark = parse_rule_block(&stripped, ".dark")?;

    if light.is_empty() {
        bail!("CSS catalog is missing `:root` custom properties");
    }
    if dark.is_empty() {
        bail!("CSS catalog is missing `.dark` custom properties");
    }

    Ok(TokenCatalog { light, dark })
}

fn strip_css_comments(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut rest = source;
    while let Some(start) = rest.find("/*") {
        out.push_str(&rest[..start]);
        rest = rest[start + 2..].split_once("*/").map_or("", |(_, after)| after);
    }
    out.push_str(rest);
    out
}

fn parse_rule_block(source: &str, selector: &str) -> Result<BTreeMap<String, String>> {
    let mut tokens = BTreeMap::new();
    let mut search_from = 0;

    while let Some(selector_start) = find_selector(source, selector, search_from) {
        let Some(block_start) = source[selector_start..].find('{') else {
            break;
        };
        let block_start = selector_start + block_start + 1;
        let Some(block_end) = find_matching_brace(source, block_start - 1) else {
            break;
        };

        let block = &source[block_start..block_end];
        parse_declarations(block, &mut tokens)?;
        search_from = block_end + 1;
    }

    Ok(tokens)
}

fn find_selector(source: &str, selector: &str, from: usize) -> Option<usize> {
    let mut search_from = from;
    while let Some(found) = source[search_from..].find(selector) {
        let index = search_from + found;
        let before = source[..index].chars().last();
        let after = source[index + selector.len()..].chars().next();
        let valid_before = before.is_none_or(|ch| !ch.is_ascii_alphanumeric() && ch != '-');
        let valid_after = after.is_none_or(|ch| !ch.is_ascii_alphanumeric() && ch != '-');
        if valid_before && valid_after {
            return Some(index);
        }
        search_from = index + selector.len();
    }
    None
}

fn find_matching_brace(source: &str, open_index: usize) -> Option<usize> {
    let mut depth = 0;
    for (index, ch) in source[open_index..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(open_index + index);
                }
            }
            _ => {}
        }
    }
    None
}

fn parse_declarations(block: &str, tokens: &mut BTreeMap<String, String>) -> Result<()> {
    let mut cursor = 0;
    while cursor < block.len() {
        while cursor < block.len() && block.as_bytes()[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor >= block.len() {
            break;
        }

        if block[cursor..].starts_with("--") {
            let decl = next_declaration(&block[cursor..]).context("invalid custom property declaration")?;
            let name = decl
                .name
                .strip_prefix("--")
                .ok_or_else(|| anyhow!("expected custom property name, got `{}`", decl.name))?
                .to_string();
            tokens.insert(name, decl.value.trim().to_string());
            cursor += decl.consumed;
            continue;
        }

        if let Some(skip) = block[cursor..].find(';') {
            cursor += skip + 1;
            continue;
        }
        break;
    }

    Ok(())
}

struct CssDeclaration<'a> {
    name: &'a str,
    value: &'a str,
    consumed: usize,
}

fn next_declaration(source: &str) -> Result<CssDeclaration<'_>> {
    let Some(colon) = source.find(':') else {
        bail!("missing ':' in custom property declaration");
    };
    let name = source[..colon].trim();
    if !name.starts_with("--") {
        bail!("expected custom property, got `{name}`");
    }

    let mut value_start = colon + 1;
    while value_start < source.len() && source.as_bytes()[value_start].is_ascii_whitespace() {
        value_start += 1;
    }

    let mut index = value_start;
    let bytes = source.as_bytes();
    while index < source.len() {
        let ch = bytes[index];
        if ch == b';' {
            let value = source[value_start..index].trim();
            return Ok(CssDeclaration { name, value, consumed: index + 1 });
        }
        if ch == b'"' || ch == b'\'' {
            index = skip_quoted(source, index + 1, ch as char)? + 1;
            continue;
        }
        if ch == b'(' {
            index = skip_balanced(source, index + 1, '(', ')')? + 1;
            continue;
        }
        index += 1;
    }

    bail!("unterminated custom property `{name}`");
}

fn skip_quoted(source: &str, mut index: usize, quote: char) -> Result<usize> {
    while index < source.len() {
        let ch = source[index..].chars().next().unwrap();
        if ch == quote {
            return Ok(index);
        }
        if ch == '\\' {
            index += 2;
            continue;
        }
        index += ch.len_utf8();
    }
    Err(anyhow!("unterminated string literal"))
}

fn skip_balanced(source: &str, mut index: usize, open: char, close: char) -> Result<usize> {
    let mut depth = 1;
    while index < source.len() {
        let ch = source[index..].chars().next().unwrap();
        if ch == open {
            depth += 1;
        } else if ch == close {
            depth -= 1;
            if depth == 0 {
                return Ok(index);
            }
        } else if ch == '"' || ch == '\'' {
            index = skip_quoted(source, index + 1, ch)? + 1;
            continue;
        }
        index += ch.len_utf8();
    }
    Err(anyhow!("unterminated function in custom property value"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!("../../../../apps/gallery/tweakcn/enhance-material.css");

    #[test]
    fn parses_root_and_dark_blocks() {
        let catalog = parse_css_catalog(FIXTURE).expect("fixture should parse");
        assert_eq!(catalog.light.get("background").unwrap(), "oklch(1 0 0)");
        assert_eq!(catalog.dark.get("background").unwrap(), "oklch(0.298 0.033 276.877)");
        assert!(catalog.light.contains_key("primary"));
        assert!(catalog.light.contains_key("sidebar-ring"));
    }

    #[test]
    fn ignores_theme_inline_blocks() {
        let css = r#"
        @theme inline { --color-primary: red; }
        :root { --primary: blue; --background: white; }
        .dark { --primary: cyan; --background: black; }
        "#;
        let catalog = parse_css_catalog(css).expect("should parse");
        assert_eq!(catalog.light.get("primary").unwrap(), "blue");
        assert!(!catalog.light.contains_key("color-primary"));
    }

    #[test]
    fn parses_comma_hsl_and_quoted_font_stacks() {
        let css = r#"
        :root {
          --primary: hsl(15.2, 72.6%, 54.1%);
          --font-sans: 'Outfit', ui-sans-serif, system-ui;
          --background: hsl(204, 12.2%, 91.96%);
        }
        .dark {
          --primary: hsl(15.2, 72.6%, 54.1%);
          --background: hsl(0, 0%, 10.2%);
        }
        "#;
        let catalog = parse_css_catalog(css).expect("should parse");
        assert_eq!(catalog.light.get("primary").unwrap(), "hsl(15.2, 72.6%, 54.1%)");
        assert_eq!(catalog.light.get("font-sans").unwrap(), "'Outfit', ui-sans-serif, system-ui");
    }
}
