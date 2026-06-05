use anyhow::Result;

use gpui_luma::theme::{LumaTypography, MetricTokens, ThemeTokens};

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
        Ok(Self {
            palette: RadixPalette::from_catalog(&catalog)?,
            metrics: metrics_from_catalog(&catalog, MetricTokens::default()),
            typography: typography_from_catalog(&catalog, LumaTypography::default()),
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
