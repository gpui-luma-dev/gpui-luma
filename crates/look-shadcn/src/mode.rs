use anyhow::Result;

use gpui_luma::theme::ThemeTokens;

use crate::catalog::CssTokenMap;
use crate::palette::ShadcnPalette;

#[derive(Clone, Debug)]
pub struct ShadcnModeTokens {
    pub catalog: CssTokenMap,
    pub palette: ShadcnPalette,
}

impl ShadcnModeTokens {
    pub fn from_catalog(catalog: CssTokenMap) -> Result<Self> {
        Ok(Self {
            palette: ShadcnPalette::from_catalog(&catalog)?,
            catalog,
        })
    }

    pub fn from_luma_tokens(tokens: &ThemeTokens) -> Self {
        Self {
            catalog: CssTokenMap::default(),
            palette: ShadcnPalette::from_luma_tokens(tokens),
        }
    }
}
