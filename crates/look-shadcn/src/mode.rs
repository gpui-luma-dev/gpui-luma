use anyhow::Result;

use gpui_luma::theme::{LumaTypography, MetricTokens, ThemeTokens};

use crate::catalog::{CssTokenMap, metrics_from_catalog, typography_from_catalog};
use crate::palette::ShadcnPalette;

#[derive(Clone, Debug)]
pub struct ShadcnModeTokens {
    pub catalog: CssTokenMap,
    pub palette: ShadcnPalette,
    pub metrics: MetricTokens,
    pub typography: LumaTypography,
}

impl ShadcnModeTokens {
    pub fn from_catalog(catalog: CssTokenMap) -> Result<Self> {
        Ok(Self {
            palette: ShadcnPalette::from_catalog(&catalog)?,
            metrics: metrics_from_catalog(&catalog, MetricTokens::default()),
            typography: typography_from_catalog(&catalog, LumaTypography::default()),
            catalog,
        })
    }

    pub fn from_luma_tokens(tokens: &ThemeTokens) -> Self {
        Self {
            catalog: CssTokenMap::default(),
            palette: ShadcnPalette::from_luma_tokens(tokens),
            metrics: tokens.metrics,
            typography: tokens.typography.clone(),
        }
    }
}
