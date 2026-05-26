use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context as _, Result, bail};
use toml::Value;

#[derive(Debug, Clone)]
pub struct Lexicon {
    pub palette: BTreeMap<String, Vec<BindingCandidate>>,
    pub light: BTreeMap<String, BindingLeaf>,
    pub dark: BTreeMap<String, BindingLeaf>,
}

#[derive(Debug, Clone)]
pub enum BindingLeaf {
    Cascade(Vec<BindingCandidate>),
    InheritBase,
    Resolve(Vec<BindingCandidate>),
}

#[derive(Debug, Clone, Default)]
pub struct BindingCandidate {
    pub token: Option<String>,
    pub palette: Option<String>,
    pub path: Option<String>,
    pub literal: Option<String>,
    pub inherit: Option<String>,
    pub from_palette: bool,
    pub transforms: Vec<String>,
}

pub fn load_lexicon(path: &Path) -> Result<Lexicon> {
    let source = std::fs::read_to_string(path).with_context(|| format!("read lexicon {}", path.display()))?;
    load_lexicon_str(&source)
}

pub fn load_lexicon_str(source: &str) -> Result<Lexicon> {
    let root: Value = toml::from_str(source).context("parse lexicon TOML")?;
    let table = root.as_table().context("lexicon root must be a table")?;

    let palette = table
        .get("palette")
        .and_then(Value::as_table)
        .map(parse_palette_section)
        .transpose()?
        .unwrap_or_default();

    let light = parse_mode_section(table.get("light"), "light")?;
    let dark = parse_mode_section(table.get("dark"), "dark")?;

    Ok(Lexicon { palette, light, dark })
}

fn parse_palette_section(table: &toml::map::Map<String, Value>) -> Result<BTreeMap<String, Vec<BindingCandidate>>> {
    let mut out = BTreeMap::new();
    for (name, value) in table {
        out.insert(name.clone(), parse_candidate_array(value, &format!("palette.{name}"))?);
    }
    Ok(out)
}

fn parse_mode_section(value: Option<&Value>, mode: &str) -> Result<BTreeMap<String, BindingLeaf>> {
    let table = value.and_then(Value::as_table).context(format!("lexicon missing [{mode}] section"))?;
    let mut out = BTreeMap::new();
    collect_mode_bindings(table, mode, String::new(), &mut out)?;
    Ok(out)
}

fn collect_mode_bindings(
    table: &toml::map::Map<String, Value>,
    mode: &str,
    prefix: String,
    out: &mut BTreeMap<String, BindingLeaf>,
) -> Result<()> {
    for (key, value) in table {
        if key == "conflicts" {
            continue;
        }

        let path = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{prefix}.{key}")
        };

        if key.starts_with('_') {
            match (key.as_str(), value) {
                ("_inherit", Value::String(base)) if base == "base" => {
                    out.insert(path, BindingLeaf::InheritBase);
                }
                ("_resolve", value) => {
                    out.insert(path.clone(), BindingLeaf::Resolve(parse_candidate_array(value, &path)?));
                }
                _ => {}
            }
            continue;
        }

        match value {
            Value::Array(_) => {
                out.insert(path.clone(), BindingLeaf::Cascade(parse_candidate_array(value, &path)?));
            }
            Value::Table(nested) => collect_mode_bindings(nested, mode, path, out)?,
            _ => bail!("unsupported binding at `{path}` in [{mode}]"),
        }
    }

    Ok(())
}

fn parse_candidate_array(value: &Value, path: &str) -> Result<Vec<BindingCandidate>> {
    let items = value.as_array().with_context(|| format!("binding `{path}` must be an array"))?;
    items.iter().map(|item| parse_candidate(item, path)).collect()
}

fn parse_candidate(value: &Value, path: &str) -> Result<BindingCandidate> {
    let table = value.as_table().with_context(|| format!("candidate in `{path}` must be a table"))?;
    let mut candidate = BindingCandidate::default();

    for (key, value) in table {
        match key.as_str() {
            "token" => candidate.token = Some(expect_string(value, path, "token")?),
            "palette" => candidate.palette = Some(expect_string(value, path, "palette")?),
            "path" => candidate.path = Some(expect_string(value, path, "path")?),
            "literal" => candidate.literal = Some(expect_literal(value, path, "literal")?),
            "inherit" => candidate.inherit = Some(expect_string(value, path, "inherit")?),
            "from_palette" => candidate.from_palette = value.as_bool().unwrap_or(true),
            "transform" => candidate.transforms = parse_transform_list(value, path)?,
            other => bail!("unknown candidate key `{other}` in `{path}`"),
        }
    }

    Ok(candidate)
}

fn parse_transform_list(value: &Value, path: &str) -> Result<Vec<String>> {
    let array = value.as_array().with_context(|| format!("transform in `{path}` must be an array"))?;
    array.iter().map(|item| expect_string(item, path, "transform")).collect()
}

fn expect_literal(value: &Value, path: &str, field: &str) -> Result<String> {
    match value {
        Value::String(text) => Ok(text.clone()),
        Value::Integer(number) => Ok(number.to_string()),
        Value::Float(number) => Ok(number.to_string()),
        Value::Boolean(flag) => Ok(flag.to_string()),
        _ => bail!("`{field}` in `{path}` must be a string or number"),
    }
}

fn expect_string(value: &Value, path: &str, field: &str) -> Result<String> {
    value
        .as_str()
        .map(str::to_string)
        .with_context(|| format!("`{field}` in `{path}` must be a string"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_repo_lexicon() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../sdk/src/theme/lexicon.toml");
        let lexicon = load_lexicon(&path).expect("repo lexicon should load");
        assert!(lexicon.palette.contains_key("brand"));
        assert!(lexicon.light.contains_key("palette.app.background"));
    }
}
