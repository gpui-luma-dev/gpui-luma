//! Arithmetic ingress shared by the build script and runtime stylesheet loader.

/// Evaluate explicit `math(...)` strings once, before typed schema validation.
/// Ordinary strings/numbers survive untouched; expressions cannot reference
/// tokens, variables or functions, so evaluation does not depend on layer order.
pub(super) fn normalize(value: &mut toml::Value) -> Result<(), String> {
    match value {
        toml::Value::String(source) if source.trim_start().starts_with("math(") => {
            let expression = source
                .trim()
                .strip_prefix("math(")
                .and_then(|s| s.strip_suffix(')'))
                .ok_or_else(|| format!("invalid math expression {source:?}"))?;
            // Leave grammar and operator precedence to fasteval. This numeric
            // alphabet excludes variable/function calls (including print).
            if !expression
                .bytes()
                .all(|c| c.is_ascii_digit() || c.is_ascii_whitespace() || b".+-*/%^()eE".contains(&c))
            {
                return Err(format!("math accepts only numeric arithmetic: {source:?}"));
            }
            let number = fasteval::ez_eval(expression, &mut fasteval::EmptyNamespace)
                .map_err(|error| format!("invalid math expression {source:?}: {error}"))?;
            if !number.is_finite() || !(number as f32).is_finite() {
                return Err(format!("math result must be finite and fit in f32: {source:?}"));
            }
            // Preserve integer schema fields such as motion durations; decimal
            // results still flow through the field's existing numeric checks.
            *value = if number.fract() == 0.0 && number >= i64::MIN as f64 && number < i64::MAX as f64 {
                toml::Value::Integer(number as i64)
            } else {
                toml::Value::Float(number)
            };
        }
        toml::Value::Table(table) => {
            for (key, value) in table {
                normalize(value).map_err(|error| format!("{key}: {error}"))?;
            }
        }
        toml::Value::Array(values) => {
            for (index, value) in values.iter_mut().enumerate() {
                normalize(value).map_err(|error| format!("[{index}]: {error}"))?;
            }
        }
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arithmetic_uses_floating_division_and_standard_precedence() {
        let mut value: toml::Value = toml::from_str(
            "ratio = 'math(32 / 28)'\ninteger = 'math((2 + 3) * 4)'\nscientific = 'math(1e-2 * 5)'\nplain = 0.5\ntext = 'hsla(0, 0%, 0%, 1)'",
        ).unwrap();
        normalize(&mut value).unwrap();
        assert_eq!(value["ratio"].as_float(), Some(32.0 / 28.0));
        assert_eq!(value["integer"].as_integer(), Some(20));
        assert_eq!(value["scientific"].as_float(), Some(0.05));
        assert_eq!(value["plain"].as_float(), Some(0.5));
        assert_eq!(value["text"].as_str(), Some("hsla(0, 0%, 0%, 1)"));
    }

    #[test]
    fn invalid_arithmetic_reports_the_field_path() {
        for expression in [
            "math(1 / 0)",
            "math(0 / 0)",
            "math(1e100)",
            "math(1 +)",
            "math((1 + 2)",
            "math()",
            "math(width / 28)",
            "math(print(1))",
        ] {
            let mut value: toml::Value =
                toml::from_str(&format!("[switch]\ntrack_width_scale = {expression:?}")).unwrap();
            let error = normalize(&mut value).unwrap_err();
            assert!(error.contains("switch: track_width_scale:"), "{error}");
        }
        let mut value = toml::Value::String(format!("math({})", "1+".repeat(4096)));
        assert!(normalize(&mut value).is_err());
    }
}
