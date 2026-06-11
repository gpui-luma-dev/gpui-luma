# Guide: Adding New Controls to the Theming System

This document outlines the step-by-step playbook for developers and AI agents adding new interactive controls or sub-components to the workspace. By adhering to this pipeline, you ensure that behavior resides in the SDK while styling is fully decoupled into `style.toml`.

---

## Step 1: Define the Visual Palette & Templates in the SDK (`crates/sdk`)
Before wiring styling, define the structural rendering tree and event layout in the SDK:

1. **Create the rendering Template**: Define the rendering contract (`trait ControlTemplate<D>`) in `template.rs`.
2. **Define the visual Palette**: Declare the colors and layout measurements required by the template inside `theme.rs` (e.g. `ControlPalette` / `ControlAppearance`).
3. **Use Layout Scales**: Avoid hardcoded padding or size constants in the templates. Query geometry (heights, margins, radii) from `StandardBoxScale` or `ListRowScale` resolved by the standard layout cache.

---

## Step 2: Declare Stylesheet Config Models in `crates/look-shadcn`
Register the new control's configuration model in the stylesheet deserializer:

1. Open `crates/look-shadcn/src/stylesheet/config.rs`.
2. Add the control's mapping structs:
   ```rust
   #[derive(Debug, Deserialize, Clone, Default)]
   pub struct NewControlStylesheet {
       #[serde(default)]
       pub metrics: HashMap<String, NewControlMetricsRule>,
       #[serde(default)]
       pub color_rules: Vec<NewControlColorRule>,
   }

   #[derive(Debug, Deserialize, Clone)]
   pub struct NewControlColorRule {
       pub layer: Option<String>,
       pub active: Option<bool>,
       pub background: String,
       pub foreground: String,
       pub border: Option<String>,
   }
   ```
3. Implement the rule matcher:
   ```rust
   impl NewControlStylesheet {
       pub fn find_color_rule(&self, active: bool, layer: InteractionLayer) -> Option<&NewControlColorRule> {
           self.color_rules.iter().find(|rule| {
               // Implement state-filtering logic here
               matches_optional_bool(rule.active, active) && matches_optional_layer(rule.layer.as_deref(), layer)
           })
       }
   }
   ```
4. Register the sub-stylesheet in `StylesheetConfig`:
   ```rust
   #[derive(Debug, Deserialize, Clone, Default)]
   pub struct StylesheetConfig {
       // ...
       #[serde(default)]
       pub new_control: NewControlStylesheet,
   }
   ```

---

## Step 3: Implement Lookups in `stylesheet/mod.rs`
Expose helper lookup functions so the controls module can query the loaded stylesheet:

1. Open `crates/look-shadcn/src/stylesheet/mod.rs`.
2. Implement the query function:
   ```rust
   pub fn find_new_control_color_rule(
       stylesheet: &StylesheetConfig,
       active: bool,
       layer: InteractionLayer,
   ) -> Option<&NewControlColorRule> {
       stylesheet.new_control.find_color_rule(active, layer)
   }
   ```
3. (Optional) Implement `resolve_new_control_colors_metadata` to export rule lists to the Look Probe Inspector UI.

---

## Step 4: Write the Control Resolver in `look-shadcn`
Write the styling bridge in the control folder to link the SDK template to the stylesheet values:

1. Create or open the control file (e.g. `crates/look-shadcn/src/controls/new_control.rs`).
2. Write the palette lookup:
   ```rust
   pub fn resolve_new_control_colors(
       resolver: &LookResolver<'_>,
       stylesheet: &StylesheetConfig,
       active: bool,
       layer: InteractionLayer,
   ) -> anyhow::Result<NewControlColorPalette> {
       let rule = find_new_control_color_rule(stylesheet, active, layer)
           .ok_or_else(|| anyhow::anyhow!("no matching color rule"))?;
       
       // resolve variables or `@field` references against the CSS catalog
       let colors = resolve_new_control_color_rule(resolver, rule)?;
       Ok(NewControlColorPalette { ... })
   }
   ```
3. Hook up the main entrance function `new_control_appearance(...)` to retrieve the active stylesheet via `embedded_stylesheet()`.

---

## Step 5: Author the Mapping Rules in `style.toml`
Define the baseline styling rules for the control:

1. Open `crates/look-shadcn/assets/style.toml`.
2. Add the default styling rules under the component heading:
   ```toml
   # =====================================================================
   # NEW CONTROL
   # =====================================================================
   [[new_control.color_rules]]
   active = true
   layer = "default"
   background = "primary"
   foreground = "primary-foreground"

   [[new_control.color_rules]]
   active = false
   layer = "default"
   background = "background"
   foreground = "foreground"
   border = "border"
   ```
