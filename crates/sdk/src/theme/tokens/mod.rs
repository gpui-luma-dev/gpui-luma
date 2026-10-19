pub mod elevation;
pub mod metrics;
pub mod palette;
pub mod typography;

pub use elevation::*;
pub use metrics::*;
pub use palette::*;
pub use typography::*;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ThemeMode {
    #[default]
    Light,
    Dark,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum ControlSize {
    Sm,
    #[default]
    Md,
    Lg,
}

/// Typed standard sizes, with named extensions for controls that actually use
/// additional sizes (for example Radix buttons and avatars).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ControlSizes<T> {
    pub sm: T,
    pub md: T,
    pub lg: T,
    pub custom: std::collections::HashMap<String, T>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SizeKey<'a> {
    Sm,
    Md,
    Lg,
    Custom(&'a str),
}

impl<'a> SizeKey<'a> {
    pub fn as_str(self) -> &'a str {
        match self {
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
            Self::Custom(key) => key,
        }
    }
}

impl From<ControlSize> for SizeKey<'_> {
    fn from(size: ControlSize) -> Self {
        match size {
            ControlSize::Sm => Self::Sm,
            ControlSize::Md => Self::Md,
            ControlSize::Lg => Self::Lg,
        }
    }
}

impl<'a> From<&'a str> for SizeKey<'a> {
    fn from(key: &'a str) -> Self {
        match key {
            "sm" => Self::Sm,
            "md" => Self::Md,
            "lg" => Self::Lg,
            _ => Self::Custom(key),
        }
    }
}

impl<'a> From<&'a String> for SizeKey<'a> {
    fn from(key: &'a String) -> Self {
        key.as_str().into()
    }
}

impl<T> ControlSizes<T> {
    pub fn get<'a>(&self, size: impl Into<SizeKey<'a>>) -> Option<&T> {
        match size.into() {
            SizeKey::Sm => Some(&self.sm),
            SizeKey::Md => Some(&self.md),
            SizeKey::Lg => Some(&self.lg),
            SizeKey::Custom(key) => self.custom.get(key),
        }
    }

    pub fn get_mut<'a>(&mut self, size: impl Into<SizeKey<'a>>) -> Option<&mut T> {
        match size.into() {
            SizeKey::Sm => Some(&mut self.sm),
            SizeKey::Md => Some(&mut self.md),
            SizeKey::Lg => Some(&mut self.lg),
            SizeKey::Custom(key) => self.custom.get_mut(key),
        }
    }

    pub fn insert(&mut self, key: String, value: T) {
        match SizeKey::from(key.as_str()) {
            SizeKey::Sm => self.sm = value,
            SizeKey::Md => self.md = value,
            SizeKey::Lg => self.lg = value,
            SizeKey::Custom(_) => {
                self.custom.insert(key, value);
            }
        }
    }

    pub fn keys(&self) -> impl Iterator<Item = &str> {
        ["sm", "md", "lg"].into_iter().chain(self.custom.keys().map(String::as_str))
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut T> {
        [&mut self.sm, &mut self.md, &mut self.lg].into_iter().chain(self.custom.values_mut())
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&str, &mut T)> {
        [("sm", &mut self.sm), ("md", &mut self.md), ("lg", &mut self.lg)]
            .into_iter()
            .chain(self.custom.iter_mut().map(|(key, value)| (key.as_str(), value)))
    }
}

impl<T: Default> ControlSizes<T> {
    pub fn get_or_default<'a>(&mut self, size: impl Into<SizeKey<'a>>) -> &mut T {
        match size.into() {
            SizeKey::Sm => &mut self.sm,
            SizeKey::Md => &mut self.md,
            SizeKey::Lg => &mut self.lg,
            SizeKey::Custom(key) => self.custom.entry(key.into()).or_default(),
        }
    }
}

impl<T: Default> FromIterator<(String, T)> for ControlSizes<T> {
    fn from_iter<I: IntoIterator<Item = (String, T)>>(iter: I) -> Self {
        let mut sizes = Self::default();
        for (key, value) in iter {
            sizes.insert(key, value);
        }
        sizes
    }
}

impl<T: Default, const N: usize> From<[(String, T); N]> for ControlSizes<T> {
    fn from(values: [(String, T); N]) -> Self {
        values.into_iter().collect()
    }
}

impl<T> IntoIterator for ControlSizes<T> {
    type Item = (String, T);
    type IntoIter =
        std::iter::Chain<std::array::IntoIter<(String, T), 3>, std::collections::hash_map::IntoIter<String, T>>;
    fn into_iter(self) -> Self::IntoIter {
        [("sm".into(), self.sm), ("md".into(), self.md), ("lg".into(), self.lg)]
            .into_iter()
            .chain(self.custom)
    }
}

impl ControlSize {
    /// Vanilla semantic key used by the shared stylesheet.
    pub fn stylesheet_key(self) -> &'static str {
        match self {
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
        }
    }
}

#[derive(Clone, Debug)]
pub struct LumaTheme {
    pub name: String,
    pub version: u32,
    pub modes: ThemeModes,
}

#[derive(Clone, Debug)]
pub struct LumaThemeMode {
    pub palette: LumaPalette,
    pub metrics: MetricTokens,
    pub typography: LumaTypography,
    pub elevation: LumaElevation,
}

/// Prepared theme tokens: source colors and an immutable current-backend preview.
/// Metrics/typography remain editable; color edits require rebuilding the palette.
#[derive(Clone, Debug)]
pub struct ThemeTokens {
    /// Resolved shared styling captured with this token snapshot.
    pub stylesheet: std::sync::Arc<crate::theme::stylesheet::CommonStylesheet>,
    pub palette: SrgbPalette,
    pub metrics: MetricTokens,
    pub typography: LumaTypography,
    pub elevation: SrgbElevation,
}

impl ThemeTokens {
    pub fn light() -> Self {
        Self {
            stylesheet: std::sync::Arc::new(crate::theme::stylesheet::CommonStylesheet::sdk_default().clone()),
            palette: SrgbPalette::light(),
            metrics: MetricTokens::default(),
            typography: LumaTypography::default(),
            elevation: SrgbElevation::light(),
        }
    }
    pub fn dark() -> Self {
        Self {
            stylesheet: std::sync::Arc::new(crate::theme::stylesheet::CommonStylesheet::sdk_default().clone()),
            palette: SrgbPalette::dark(),
            metrics: MetricTokens::default(),
            typography: LumaTypography::default(),
            elevation: SrgbElevation::dark(),
        }
    }

    /// Resolve once before passing tokens to control themes; invalid colors return a field-specific error.
    pub fn from_source(source: LumaThemeMode, mapping: crate::color::GamutMapping) -> anyhow::Result<Self> {
        Ok(Self {
            stylesheet: std::sync::Arc::new(crate::theme::stylesheet::CommonStylesheet::sdk_default().clone()),
            palette: source.palette.snapshot_srgb(mapping)?,
            metrics: source.metrics,
            typography: source.typography,
            elevation: source.elevation.snapshot_srgb(mapping)?,
        })
    }

    /// Retrieve a standalone source copy, including current non-color tokens.
    pub fn source(&self) -> LumaThemeMode {
        LumaThemeMode {
            palette: self.palette.source().clone(),
            metrics: self.metrics,
            typography: self.typography.clone(),
            elevation: self.elevation.source().clone(),
        }
    }
}

impl Default for ThemeTokens {
    fn default() -> Self {
        Self::light()
    }
}

#[derive(Clone, Debug)]
pub struct ThemeModes {
    pub light: LumaThemeMode,
    pub dark: LumaThemeMode,
}

impl LumaTheme {
    pub fn structural_default() -> Self {
        Self {
            name: "Structural".to_string(),
            version: 1,
            modes: ThemeModes::new(LumaThemeMode::light(), LumaThemeMode::dark()),
        }
    }

    pub fn mode(&self, mode: ThemeMode) -> &LumaThemeMode {
        self.modes.tokens(mode)
    }
}

impl Default for LumaTheme {
    fn default() -> Self {
        Self::structural_default()
    }
}

impl LumaThemeMode {
    pub fn light() -> Self {
        Self::new(LumaPalette::light(), MetricTokens::default(), LumaTypography::default(), LumaElevation::light())
    }

    pub fn dark() -> Self {
        Self::new(LumaPalette::dark(), MetricTokens::default(), LumaTypography::default(), LumaElevation::dark())
    }

    pub fn new(
        palette: LumaPalette,
        metrics: MetricTokens,
        typography: LumaTypography,
        elevation: LumaElevation,
    ) -> Self {
        Self { palette, metrics, typography, elevation }
    }
}

impl ThemeModes {
    pub fn new(light: LumaThemeMode, dark: LumaThemeMode) -> Self {
        Self { light, dark }
    }

    pub fn single(tokens: LumaThemeMode) -> Self {
        Self { light: tokens.clone(), dark: tokens }
    }

    pub fn tokens(&self, mode: ThemeMode) -> &LumaThemeMode {
        match mode {
            ThemeMode::Light => &self.light,
            ThemeMode::Dark => &self.dark,
        }
    }

    pub fn tokens_mut(&mut self, mode: ThemeMode) -> &mut LumaThemeMode {
        match mode {
            ThemeMode::Light => &mut self.light,
            ThemeMode::Dark => &mut self.dark,
        }
    }

    pub fn into_tokens(self, mode: ThemeMode) -> LumaThemeMode {
        match mode {
            ThemeMode::Light => self.light,
            ThemeMode::Dark => self.dark,
        }
    }
}

impl Default for ThemeModes {
    fn default() -> Self {
        Self::new(LumaThemeMode::light(), LumaThemeMode::dark())
    }
}

impl Default for LumaThemeMode {
    fn default() -> Self {
        Self::light()
    }
}

#[cfg(test)]
mod tests {
    use super::{LumaTheme, LumaThemeMode, ThemeMode, ThemeModes, ThemeTokens};

    #[test]
    fn default_tokens_use_light_structural_palette() {
        assert_eq!(
            LumaThemeMode::light().palette.app.background,
            ThemeTokens::default().palette.source().app.background
        );
    }

    #[test]
    fn theme_modes_select_light_and_dark_palettes() {
        let modes = ThemeModes::default();
        assert_ne!(
            modes.tokens(ThemeMode::Light).palette.app.background,
            modes.tokens(ThemeMode::Dark).palette.app.background
        );
    }

    #[test]
    fn structural_default_theme_exposes_semantic_layers() {
        let theme = LumaTheme::structural_default();
        let light = theme.mode(ThemeMode::Light);
        assert_eq!(theme.name, "Structural");
        assert_eq!(theme.version, 1);
        assert_eq!(light.metrics.radius.pill, 999.0);
        assert_eq!(light.typography.text.label.weight, gpui::FontWeight::MEDIUM);
        assert!(!light.elevation.menu.layers.is_empty());
    }
}
