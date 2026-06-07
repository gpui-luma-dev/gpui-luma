use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::theme::{InteractionLayer, InteractionState, ThemeTokens};

#[derive(Clone, Debug)]
pub struct ResizablePanelsAppearance {
    pub border: Hsla,
    pub divider: Hsla,
    pub grip: Hsla,
    pub grip_emphasis: Hsla,
    pub disabled_opacity: f32,
}

pub trait ResizablePanelsTheme: Send + Sync {
    fn resolve(&self, state: InteractionState) -> ResizablePanelsAppearance;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultResizablePanelsTheme {
    tokens: ThemeTokens,
}

pub fn default_resizable_panels_theme() -> Arc<dyn ResizablePanelsTheme> {
    static THEME: OnceLock<Arc<dyn ResizablePanelsTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultResizablePanelsTheme::default())).clone()
}

impl DefaultResizablePanelsTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl ResizablePanelsTheme for DefaultResizablePanelsTheme {
    fn resolve(&self, state: InteractionState) -> ResizablePanelsAppearance {
        let palette = &self.tokens.palette;
        let disabled = state.layer() == InteractionLayer::Disabled;

        ResizablePanelsAppearance {
            border: palette.border.default,
            divider: if disabled {
                palette.state.disabled.background
            } else {
                palette.border.strong
            },
            grip: if disabled {
                palette.state.disabled.foreground
            } else {
                palette.border.strong
            },
            grip_emphasis: if disabled {
                palette.state.disabled.foreground
            } else {
                palette.state.hover.background
            },
            disabled_opacity: if disabled { 0.45 } else { 1.0 },
        }
    }
}
