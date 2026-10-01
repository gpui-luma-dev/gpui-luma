use std::sync::Arc;

use gpui_luma::controls::accordion::{AccordionTemplate, AccordionTheme, ThemedAccordionTemplate};
use gpui_luma::theme::{ControlSize, InteractionState};

use crate::controls::accordion::{accordion_content_palette, accordion_trigger_palette};
use crate::look::ShadcnLook;

struct ShadcnAccordionTheme {
    theme: ShadcnLook,
}

impl AccordionTheme for ShadcnAccordionTheme {
    fn resolve_trigger(
        &self,
        state: InteractionState,
        size: ControlSize,
    ) -> gpui_luma::controls::accordion::AccordionPalette {
        let tokens = self.theme.mode_tokens();
        accordion_trigger_palette(tokens.as_ref(), self.theme.mode(), state, size)
    }

    fn resolve_content(&self, expanded: bool) -> gpui_luma::controls::accordion::AccordionContentPalette {
        let tokens = self.theme.mode_tokens();
        accordion_content_palette(tokens.as_ref(), self.theme.mode(), expanded)
    }

    fn resolve_scale(&self, size: ControlSize, scale_factor: f32) -> gpui_luma::controls::accordion::AccordionScale {
        crate::tables::metrics::resolve_accordion_metrics(
            &self.theme.mode_tokens(),
            self.theme.mode(),
            size,
            scale_factor,
        )
        .scale()
    }

    fn metrics(&self) -> gpui_luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }
}

pub fn accordion_template(theme: ShadcnLook) -> Arc<dyn AccordionTemplate> {
    Arc::new(ThemedAccordionTemplate::new(accordion_theme(theme.clone())))
}

pub fn accordion_theme(theme: ShadcnLook) -> Arc<dyn AccordionTheme> {
    Arc::new(ShadcnAccordionTheme { theme: theme.clone() })
}
