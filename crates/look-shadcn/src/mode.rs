use anyhow::Result;

use luma::theme::{InteractionLayer, LumaTypography, MetricTokens, ThemeMode};

use crate::catalog::{CssTokenMap, metrics_from_catalog, typography_from_catalog};
use crate::palette::ShadcnPalette;
use crate::state_color::StateColorTable;
use crate::tokens::ShadcnToken;

#[derive(Clone, Debug)]
pub struct ShadcnModeTokens {
    pub(crate) theme_mode: ThemeMode,
    pub catalog: CssTokenMap,
    pub palette: ShadcnPalette,
    pub metrics: MetricTokens,
    pub typography: LumaTypography,
    state_colors: StateColorTable,
}

impl ShadcnModeTokens {
    pub fn from_catalog(catalog: CssTokenMap, theme_mode: ThemeMode) -> Result<Self> {
        let palette = ShadcnPalette::from_catalog(&catalog, theme_mode)?;
        let state_colors = StateColorTable::from_catalog(&catalog, &palette, theme_mode);
        Ok(Self {
            theme_mode,
            metrics: metrics_from_catalog(&catalog, MetricTokens::default()),
            typography: typography_from_catalog(&catalog, LumaTypography::default()),
            state_colors,
            palette,
            catalog,
        })
    }

    /// Resolves a semantic token color for an interaction layer (O(1) lookup).
    pub fn resolve_color_state(&self, token: ShadcnToken, layer: InteractionLayer) -> gpui::Hsla {
        self.state_colors.get(token, layer)
    }
}
