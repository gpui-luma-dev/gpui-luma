use anyhow::Result;

use crate::theme::{LumaTheme, LumaTypography, MetricTokens, ThemeMode, ThemeTokens};

use super::catalog::{CssTokenMap, metrics_from_catalog, typography_from_catalog};
use super::palette::RadixPalette;

#[derive(Clone, Debug)]
pub struct RadixModeTokens {
    pub catalog: CssTokenMap,
    pub palette: RadixPalette,
    pub metrics: MetricTokens,
    pub typography: LumaTypography,
}

impl RadixModeTokens {
    pub fn from_catalog(catalog: CssTokenMap) -> Result<Self> {
        let native = LumaTheme::native();
        let scaffold = native.mode(ThemeMode::Light);
        Ok(Self {
            palette: RadixPalette::from_catalog(&catalog)?,
            metrics: metrics_from_catalog(&catalog, scaffold.metrics),
            typography: typography_from_catalog(&catalog, scaffold.typography.clone()),
            catalog,
        })
    }

    pub fn from_luma_tokens(tokens: &ThemeTokens) -> Self {
        Self {
            catalog: CssTokenMap::default(),
            palette: RadixPalette::from_luma_tokens(tokens),
            metrics: tokens.metrics,
            typography: tokens.typography.clone(),
        }
    }
}
