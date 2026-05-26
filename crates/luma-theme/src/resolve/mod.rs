use std::collections::BTreeMap;

use anyhow::{Context as _, Result, bail};
use toml::Value;

use crate::base::{get_leaf_string, set_leaf};
use crate::catalog::TokenCatalog;
use crate::color::{apply_alpha, apply_lightness_delta, first_font, normalize_color, rem_to_px};
use crate::lexicon::{BindingCandidate, BindingLeaf, Lexicon};
use crate::mode::ThemeMode;

#[derive(Debug, Clone)]
pub struct ResolvedMode {
    pub values: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct ResolvedTheme {
    pub light: ResolvedMode,
    pub dark: ResolvedMode,
    pub reports: Vec<BindingReport>,
}

#[derive(Debug, Clone)]
pub struct BindingReport {
    pub mode: ThemeMode,
    pub path: String,
    pub source: String,
    pub value: String,
}

pub fn resolve_theme(
    catalog: &TokenCatalog,
    lexicon: &Lexicon,
    base_light: &Value,
    base_dark: &Value,
) -> Result<ResolvedTheme> {
    let light = resolve_mode(ThemeMode::Light, catalog, lexicon, &lexicon.light, base_light)?;
    let dark = resolve_mode(ThemeMode::Dark, catalog, lexicon, &lexicon.dark, base_dark)?;
    Ok(ResolvedTheme {
        light: light.resolved,
        dark: dark.resolved,
        reports: light.reports.into_iter().chain(dark.reports).collect(),
    })
}

struct ModeResolution {
    resolved: ResolvedMode,
    reports: Vec<BindingReport>,
}

fn resolve_mode(
    mode: ThemeMode,
    catalog: &TokenCatalog,
    lexicon: &Lexicon,
    bindings: &BTreeMap<String, BindingLeaf>,
    base_mode: &Value,
) -> Result<ModeResolution> {
    let palette = resolve_palette(mode, catalog, lexicon)?;
    let mut values = BTreeMap::new();
    let mut reports = Vec::new();

    for (path, leaf) in bindings {
        if matches!(leaf, BindingLeaf::InheritBase) {
            if let Some(value) = get_leaf_string(base_mode, path) {
                values.insert(path.clone(), value);
            }
            continue;
        }

        let resolved = match leaf {
            BindingLeaf::Cascade(candidates) => {
                resolve_cascade(mode, catalog, &palette, base_mode, &values, path, candidates)?
            }
            BindingLeaf::Resolve(candidates) => {
                resolve_cascade(mode, catalog, &palette, base_mode, &values, path, candidates)?
            }
            BindingLeaf::InheritBase => continue,
        };

        if let Some((value, source)) = resolved {
            reports.push(BindingReport { mode, path: path.clone(), source, value: value.clone() });
            values.insert(path.clone(), value);
        }
    }

    // Fixed-point pass for path back-references.
    for _ in 0..64 {
        let mut changed = false;
        for (path, leaf) in bindings {
            if values.contains_key(path) {
                continue;
            }
            let resolved = match leaf {
                BindingLeaf::Cascade(candidates) | BindingLeaf::Resolve(candidates) => {
                    resolve_cascade(mode, catalog, &palette, base_mode, &values, path, candidates)?
                }
                BindingLeaf::InheritBase => continue,
            };
            if let Some((value, source)) = resolved {
                reports.push(BindingReport { mode, path: path.clone(), source, value: value.clone() });
                values.insert(path.clone(), value);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    apply_from_palette(mode, &mut values, &mut reports)?;
    apply_color_overrides(mode, bindings, catalog, &palette, base_mode, &values, &mut reports)?;

    Ok(ModeResolution { resolved: ResolvedMode { values }, reports })
}

fn resolve_palette(mode: ThemeMode, catalog: &TokenCatalog, lexicon: &Lexicon) -> Result<BTreeMap<String, String>> {
    let mut palette = BTreeMap::new();
    for (name, candidates) in &lexicon.palette {
        if let Some((value, _)) = resolve_cascade(
            mode,
            catalog,
            &palette,
            &Value::Table(toml::map::Map::new()),
            &BTreeMap::new(),
            name,
            candidates,
        )? {
            palette.insert(name.clone(), value);
        }
    }
    Ok(palette)
}

fn resolve_cascade(
    mode: ThemeMode,
    catalog: &TokenCatalog,
    palette: &BTreeMap<String, String>,
    base_mode: &Value,
    resolved: &BTreeMap<String, String>,
    path: &str,
    candidates: &[BindingCandidate],
) -> Result<Option<(String, String)>> {
    for candidate in candidates {
        if candidate.from_palette {
            continue;
        }

        let source = describe_candidate(candidate);
        let raw = if let Some(token) = &candidate.token {
            catalog.get(mode, token).map(str::to_string)
        } else if let Some(name) = &candidate.palette {
            palette.get(name).cloned()
        } else if let Some(other_path) = &candidate.path {
            resolved.get(other_path).cloned().or_else(|| get_leaf_string(base_mode, other_path))
        } else if let Some(literal) = &candidate.literal {
            Some(literal.clone())
        } else if let Some(base) = &candidate.inherit {
            if base == "base" {
                get_leaf_string(base_mode, path)
            } else {
                None
            }
        } else {
            None
        };

        let Some(raw) = raw else {
            continue;
        };

        let normalize = !candidate.transforms.iter().any(|transform| transform == "no-normalize");
        let value = match apply_transforms(&raw, &candidate.transforms, normalize) {
            Ok(value) => value,
            Err(_) => continue,
        };
        return Ok(Some((value, source)));
    }

    Ok(None)
}

fn describe_candidate(candidate: &BindingCandidate) -> String {
    if let Some(token) = &candidate.token {
        format!("token.{token}")
    } else if let Some(name) = &candidate.palette {
        format!("palette.{name}")
    } else if let Some(path) = &candidate.path {
        format!("path.{path}")
    } else if candidate.literal.is_some() {
        "literal".into()
    } else if candidate.inherit.is_some() {
        "inherit.base".into()
    } else if candidate.from_palette {
        "from_palette".into()
    } else {
        "unknown".into()
    }
}

fn apply_transforms(raw: &str, transforms: &[String], mut normalize: bool) -> Result<String> {
    let mut value = raw.to_string();
    for transform in transforms {
        if transform == "no-normalize" {
            normalize = false;
            continue;
        }
        value = apply_single_transform(&value, transform)?;
    }

    if looks_like_color(&value) {
        value = normalize_color(&value, normalize)?;
    }
    Ok(value)
}

fn apply_single_transform(value: &str, transform: &str) -> Result<String> {
    if transform == "first_font" {
        return Ok(first_font(value));
    }
    if transform == "shadow_parse" {
        bail!("shadow_parse is not implemented yet");
    }

    if let Some(amount) = transform.strip_prefix("darken(").and_then(|v| v.strip_suffix(')')) {
        let delta = parse_percent_delta(amount)?;
        return apply_lightness_delta(value, -delta);
    }
    if let Some(amount) = transform.strip_prefix("lighten(").and_then(|v| v.strip_suffix(')')) {
        let delta = parse_percent_delta(amount)?;
        return apply_lightness_delta(value, delta);
    }
    if let Some(amount) = transform.strip_prefix("alpha(").and_then(|v| v.strip_suffix(')')) {
        let alpha = amount.parse::<f32>().context("invalid alpha transform")?;
        return apply_alpha(value, alpha);
    }
    if transform.starts_with("rem_to_px(") && transform.ends_with(')') {
        let base = transform
            .trim_start_matches("rem_to_px(")
            .trim_end_matches(')')
            .parse::<f32>()
            .context("invalid rem_to_px base")?;
        let px = rem_to_px(value, base)?;
        return Ok(trim_number(px));
    }
    if let Some(amount) = transform.strip_prefix("offset(").and_then(|v| v.strip_suffix(')')) {
        let offset = amount.parse::<f64>().context("invalid offset transform")?;
        let number = value.parse::<f64>().unwrap_or_else(|_| rem_to_px(value, 16.0).unwrap_or(0.0));
        return Ok(trim_number(number + offset));
    }

    bail!("unsupported transform `{transform}`")
}

fn parse_percent_delta(raw: &str) -> Result<f32> {
    raw.strip_suffix('%')
        .unwrap_or(raw)
        .parse::<f32>()
        .with_context(|| format!("invalid transform amount `{raw}`"))
}

fn trim_number(value: f64) -> String {
    if (value.fract()).abs() <= f64::EPSILON {
        format!("{}", value.round() as i64)
    } else {
        format!("{value}")
    }
}

fn looks_like_color(value: &str) -> bool {
    let value = value.trim();
    value.starts_with("hsl") || value.starts_with("oklch") || value.starts_with("rgb") || value.starts_with('#')
}

fn apply_from_palette(
    mode: ThemeMode,
    values: &mut BTreeMap<String, String>,
    reports: &mut Vec<BindingReport>,
) -> Result<()> {
    let prefix = "colors.";
    let mappings = [
        ("surface", "palette.action.ghost.background"),
        ("surface_hover", "palette.action.ghost.hover_background"),
        ("surface_pressed", "palette.action.ghost.pressed_background"),
        ("surface_disabled", "palette.state.disabled.background"),
        ("prominent", "palette.action.prominent.background"),
        ("prominent_hover", "palette.action.prominent.hover_background"),
        ("prominent_pressed", "palette.action.prominent.pressed_background"),
        ("selected", "palette.state.selected.background"),
        ("selected_hover", "palette.action.prominent.hover_background"),
        ("selected_pressed", "palette.action.prominent.pressed_background"),
        ("text", "palette.app.foreground"),
        ("text_inverse", "palette.state.selected.foreground"),
        ("text_disabled", "palette.state.disabled.foreground"),
        ("border", "palette.border.default"),
    ];

    for (leaf, source_path) in mappings {
        let path = format!("{prefix}{leaf}");
        if values.contains_key(&path) {
            continue;
        }
        if let Some(value) = values.get(source_path) {
            reports.push(BindingReport {
                mode,
                path: path.clone(),
                source: format!("from_palette.{source_path}"),
                value: value.clone(),
            });
            values.insert(path, value.clone());
        }
    }

    Ok(())
}

fn apply_color_overrides(
    mode: ThemeMode,
    bindings: &BTreeMap<String, BindingLeaf>,
    catalog: &TokenCatalog,
    palette: &BTreeMap<String, String>,
    base_mode: &Value,
    resolved: &BTreeMap<String, String>,
    reports: &mut Vec<BindingReport>,
) -> Result<()> {
    let prefix = "colors.overrides.";
    for (path, leaf) in bindings {
        if !path.starts_with(prefix) {
            continue;
        }
        let BindingLeaf::Cascade(candidates) = leaf else {
            continue;
        };
        if let Some((value, source)) = resolve_cascade(mode, catalog, palette, base_mode, resolved, path, candidates)? {
            let leaf_name = path.strip_prefix(prefix).expect("override prefix");
            let target = format!("colors.{leaf_name}");
            reports.push(BindingReport { mode, path: target.clone(), source, value: value.clone() });
            // Legacy destructive slots are ignored by the loader today; keep them out of output.
            if !leaf_name.starts_with("destructive") {
                // no-op: we don't emit destructive keys because RawColorTokens ignores them
            }
            let _ = value;
        }
    }
    Ok(())
}

pub fn apply_resolved_mode(mode: &mut Value, resolved: &ResolvedMode) -> Result<()> {
    for (path, value) in &resolved.values {
        if path.starts_with('_') || path.contains(".overrides.") {
            continue;
        }
        let emitted = if looks_like_color(value) || path.starts_with("palette.") {
            Value::String(value.clone())
        } else if let Ok(number) = value.parse::<i64>() {
            Value::Integer(number)
        } else if let Ok(number) = value.parse::<f64>() {
            Value::Float(number)
        } else {
            Value::String(value.clone())
        };
        set_leaf(mode, path, emitted)?;
    }
    Ok(())
}
