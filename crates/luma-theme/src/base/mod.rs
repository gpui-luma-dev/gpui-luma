use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context as _, Result, bail};
use toml::Value;

#[derive(Debug, Clone)]
pub struct BaseTheme {
    pub name: String,
    pub version: u64,
    pub light: Value,
    pub dark: Value,
}

pub fn load_base_theme(path: &Path) -> Result<BaseTheme> {
    let source = std::fs::read_to_string(path).with_context(|| format!("read base theme {}", path.display()))?;
    load_base_theme_str(&source)
}

pub fn load_base_theme_str(source: &str) -> Result<BaseTheme> {
    let root: Value = toml::from_str(source).context("parse base theme TOML")?;
    let table = root.as_table().context("base theme root must be a table")?;

    let name = table.get("name").and_then(Value::as_str).unwrap_or("Imported Theme").to_string();
    let version = table.get("version").and_then(Value::as_integer).unwrap_or(1) as u64;
    let light = table.get("light").cloned().context("base theme missing [light]")?;
    let dark = table.get("dark").cloned().context("base theme missing [dark]")?;

    Ok(BaseTheme { name, version, light, dark })
}

pub fn get_leaf_value(mode: &Value, path: &str) -> Option<Value> {
    let mut current = mode;
    for segment in path.split('.') {
        current = current.as_table()?.get(segment)?;
    }
    Some(current.clone())
}

pub fn get_leaf_string(mode: &Value, path: &str) -> Option<String> {
    match get_leaf_value(mode, path)? {
        Value::String(value) => Some(value),
        Value::Integer(value) => Some(value.to_string()),
        Value::Float(value) => Some(value.to_string()),
        Value::Boolean(value) => Some(value.to_string()),
        _ => None,
    }
}

pub fn set_leaf(mode: &mut Value, path: &str, value: Value) -> Result<()> {
    let segments: Vec<_> = path.split('.').collect();
    if segments.is_empty() {
        bail!("empty path");
    }

    let mut current = mode;
    for segment in &segments[..segments.len() - 1] {
        let table = current.as_table_mut().with_context(|| format!("expected table at `{path}`"))?;
        current = table.entry(segment.to_string()).or_insert_with(|| Value::Table(toml::map::Map::new()));
    }

    let leaf = segments.last().expect("non-empty segments");
    current
        .as_table_mut()
        .with_context(|| format!("expected table for `{path}`"))?
        .insert(leaf.to_string(), value);
    Ok(())
}

pub fn clone_mode_with_inheritance(
    base_mode: &Value,
    bindings: &BTreeMap<String, crate::lexicon::BindingLeaf>,
) -> Result<Value> {
    let mut mode = base_mode.clone();

    for (path, leaf) in bindings {
        if matches!(leaf, crate::lexicon::BindingLeaf::InheritBase)
            && let Some(base_value) = get_leaf_value(base_mode, path)
        {
            set_leaf(&mut mode, path, base_value)?;
        }
    }

    Ok(mode)
}
