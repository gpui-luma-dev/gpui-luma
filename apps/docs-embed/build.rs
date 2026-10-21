use std::{env, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    for name in [
        "button",
        "checkbox",
        "switch",
        "slider",
        "tabs",
        "radio-button",
        "radio-group",
        "toggle",
        "textfield",
        "textarea",
        "selector",
        "combobox",
        "autocomplete",
        "search-selector",
        "selection-panel",
        "pager",
        "icon-button",
        "split-button",
        "tooltip",
        "popover-button",
        "popup-menu",
        "context-menu",
        "toolbar",
        "accordion",
        "tree-view",
        "table",
        "control-group",
        "icon-group",
        "progress",
        "stepper",
        "color-swatch",
        "listbox",
        "color-slider",
        "color-field",
        "color-arc",
        "color-ring",
        "plot-cartesian",
        "plot-pie",
        "plot-radar",
        "plot-radial",
    ] {
        let page = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?).join(format!("../../docs/{name}.html"));
        println!("cargo:rerun-if-changed={}", page.display());
        let html = fs::read_to_string(page)?;
        let marker = format!("<code class=\"language-rust\" data-live-example=\"{name}-basic\">");
        let (_, sample) = html.split_once(marker.as_str()).ok_or("missing live control code sample")?;
        let (sample, _) = sample.split_once("</code>").ok_or("unclosed live control code sample")?;
        // Decode the HTML entities used by the documentation's Rust code blocks.
        // Decode ampersands last so escaped entity text stays literal Rust text.
        let source = sample
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#39;", "'")
            .replace("&amp;", "&");
        let module = name.replace('-', "_");
        fs::write(PathBuf::from(env::var("OUT_DIR")?).join(format!("{module}_sample.rs")), source)?;
    }
    Ok(())
}
