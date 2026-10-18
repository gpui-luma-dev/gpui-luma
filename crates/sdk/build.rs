use std::{env, fs, path::PathBuf};

#[path = "src/theme/stylesheet/colors.rs"]
mod colors;
#[path = "src/theme/stylesheet/math.rs"]
mod math;

// Compile the embedded data into token constructors. Invalid packaged defaults
// fail the build rather than introducing a panic in infallible SDK factories.
fn metric(value: &toml::Value, name: &str) -> String {
    let fields = value.as_table().expect("metric token table");
    let fields = fields
        .iter()
        .map(|(field, value)| {
            let expression = if value.is_table() {
                let kind = match field.as_str() {
                    "spacing" => "SpacingTokens",
                    "radius" => "RadiusTokens",
                    "border_width" => "BorderWidthTokens",
                    "focus" => "FocusMetricTokens",
                    "control" => "ControlMetricScale",
                    "sm" | "md" | "lg" => "ControlMetricTokens",
                    _ => panic!("unknown metric token {field}"),
                };
                metric(value, kind)
            } else {
                let number =
                    value.as_float().or_else(|| value.as_integer().map(|n| n as f64)).expect("numeric metric token");
                assert!(
                    number.is_finite() && number >= 0.0 && number <= f32::MAX as f64,
                    "invalid metric token {field}"
                );
                format!("{:?}_f32", number as f32)
            };
            format!("{field}: {expression}")
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("{name} {{ {fields} }}")
}

fn main() {
    println!("cargo:rerun-if-changed=assets/style.toml");
    let source = fs::read_to_string("assets/style.toml").expect("SDK stylesheet");
    let mut sheet: toml::Value = toml::from_str(&source).expect("valid SDK stylesheet");
    math::normalize(&mut sheet["common"]).expect("valid stylesheet math");
    colors::normalize(&mut sheet["common"]["tokens"]["palette"]).expect("HSLA palette colors");
    colors::normalize(&mut sheet["common"]["tokens"]["elevation"]).expect("HSLA shadow colors");
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo output directory"));
    fs::write(output.join("metrics.rs"), metric(&sheet["common"]["tokens"]["metrics"], "MetricTokens"))
        .expect("write compiled metric tokens");
    for mode in ["light", "dark"] {
        fs::write(
            output.join(format!("palette_{mode}.rs")),
            palette(&sheet["common"]["tokens"]["palette"][mode], "Self"),
        )
        .expect("write compiled palette tokens");
        fs::write(
            output.join(format!("elevation_{mode}.rs")),
            elevation(&sheet["common"]["tokens"]["elevation"][mode]),
        )
        .expect("write compiled elevation tokens");
    }
    fs::write(output.join("typography.rs"), typography(&sheet["common"]["tokens"]["typography"], "Self"))
        .expect("write compiled typography tokens");
}

fn typography(value: &toml::Value, kind: &str) -> String {
    let fields = value
        .as_table()
        .expect("typography table")
        .iter()
        .map(|(field, value)| {
            let expression = if let Some(value) = value.as_str() {
                format!("{value:?}.to_string()")
            } else if value.is_table() {
                let nested = match field.as_str() {
                    "font" => "FontTokens",
                    "sans" | "mono" | "serif" => "FontFamilyToken",
                    "text" => "TextTokens",
                    "scale" => "TextScaleTokens",
                    "role" => "TextRoleTokens",
                    _ => "LumaTextStyle",
                };
                typography(value, nested)
            } else {
                let n = value.as_float().or_else(|| value.as_integer().map(|n| n as f64)).expect("typography number");
                assert!(n.is_finite() && n >= 0.0 && n <= f32::MAX as f64);
                if field == "weight" {
                    assert!((1.0..=1000.0).contains(&n));
                    format!("FontWeight({:?}_f32)", n as f32)
                } else {
                    format!("{:?}_f32", n as f32)
                }
            };
            format!("{field}: {expression}")
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("{kind} {{ {fields} }}")
}

fn elevation(value: &toml::Value) -> String {
    let fields = value
        .as_table()
        .expect("elevation table")
        .iter()
        .map(|(field, value)| {
            let layers = value["layers"]
                .as_array()
                .expect("shadow layers")
                .iter()
                .map(|layer| {
                    let alpha = layer["color"]["components"][3].as_float().expect("shadow alpha") as f32;
                    assert!(alpha.is_finite() && (0.0..=1.0).contains(&alpha));
                    assert_eq!(layer["color"]["space"].as_str(), Some("srgb"));
                    for component in &layer["color"]["components"].as_array().expect("shadow color")[..3] {
                        assert_eq!(component.as_float(), Some(0.0), "embedded shadows use neutral black");
                    }
                    let values = ["offset_x", "offset_y", "blur", "spread"].map(|key| {
                        let n = layer[key]
                            .as_float()
                            .or_else(|| layer[key].as_integer().map(|n| n as f64))
                            .expect("shadow metric");
                        assert!(n.is_finite() && n.abs() <= f32::MAX as f64);
                        if key == "blur" {
                            assert!(n >= 0.0);
                        }
                        format!("{:?}_f32", n as f32)
                    });
                    let inset = layer["inset"].as_bool().expect("shadow inset flag");
                    format!("LumaShadowLayer::new(color({alpha:?}_f32), {}, {inset})", values.join(","))
                })
                .collect::<Vec<_>>()
                .join(",");
            format!("{field}: LumaShadow::new(vec![{layers}])")
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("Self {{ {fields} }}")
}

fn palette(value: &toml::Value, kind: &str) -> String {
    let fields = value
        .as_table()
        .expect("palette table")
        .iter()
        .map(|(field, value)| {
            let expression = if let Some(components) = value.get("components") {
                assert_eq!(value["space"].as_str(), Some("srgb"), "embedded palette source space");
                let components = components.as_array().expect("color components");
                assert_eq!(components.len(), 4);
                let components = components
                    .iter()
                    .map(|c| {
                        let n = c.as_float().or_else(|| c.as_integer().map(|n| n as f64)).expect("color component");
                        assert!(n.is_finite() && (0.0..=1.0).contains(&n));
                        format!("{:?}_f32", n as f32)
                    })
                    .collect::<Vec<_>>()
                    .join(",");
                format!("color([{components}])")
            } else {
                let nested = match field.as_str() {
                    "app" => "AppPalette",
                    "surface" => "SurfacePalette",
                    "panel" | "floating" => "SurfaceWithBorderPalette",
                    "subtle" => "SurfaceTonePalette",
                    "state" => "StatePalette",
                    "hover" | "selected" | "disabled" => "StateTonePalette",
                    "pressed" => "StateBackgroundPalette",
                    "form" => "FormPalette",
                    "input" => "FormInputPalette",
                    "focus" => "FocusPalette",
                    "border" => "BorderPalette",
                    "navigation" => "NavigationPalette",
                    "data" => "DataPalette",
                    _ => panic!("unknown palette group {field}"),
                };
                palette(value, nested)
            };
            format!("{field}: {expression}")
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("{kind} {{ {fields} }}")
}
