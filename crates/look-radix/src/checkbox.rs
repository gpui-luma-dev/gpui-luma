//! Radix checkbox theme adapter.

use std::sync::Arc;

use luma::controls::checkbox::{CheckboxPalette, CheckboxTheme};
use luma::theme::{ControlSize, InteractionLayer, InteractionState, MetricTokens};

use crate::look::RadixLook;
use crate::semantic::SemanticRole;
use crate::typography::{font_family, label_typography};

struct RadixCheckboxTheme {
    look: RadixLook,
}

impl CheckboxTheme for RadixCheckboxTheme {
    fn resolve(&self, checked: bool, state: InteractionState, size: ControlSize) -> CheckboxPalette {
        let layer = state.layer();
        let (indicator_background, checkmark_color) = match (checked, layer) {
            (_, InteractionLayer::Disabled) => (
                self.look.resolve_role(SemanticRole::Surface).hsla(),
                self.look.resolve_role(SemanticRole::MutedForeground).hsla(),
            ),
            (true, InteractionLayer::Pressed) => (
                self.look.resolve_step(crate::scale::ScaleFamily::Color, 11).hsla(),
                self.look.resolve_role(SemanticRole::PrimaryForeground).hsla(),
            ),
            (true, InteractionLayer::Hovered) => (
                self.look.resolve_step(crate::scale::ScaleFamily::Color, 10).hsla(),
                self.look.resolve_role(SemanticRole::PrimaryForeground).hsla(),
            ),
            (true, InteractionLayer::Default) => (
                self.look.resolve_role(SemanticRole::Primary).hsla(),
                self.look.resolve_role(SemanticRole::PrimaryForeground).hsla(),
            ),
            (false, InteractionLayer::Pressed) => (
                self.look.resolve_role(SemanticRole::Soft).hsla(),
                self.look.resolve_role(SemanticRole::Foreground).hsla(),
            ),
            (false, InteractionLayer::Hovered) => (
                self.look.resolve_role(SemanticRole::Soft).hsla(),
                self.look.resolve_role(SemanticRole::Foreground).hsla(),
            ),
            (false, InteractionLayer::Default) => (
                self.look.resolve_role(SemanticRole::Background).hsla(),
                self.look.resolve_role(SemanticRole::Foreground).hsla(),
            ),
        };

        let indicator_border = if state.focused && !state.disabled {
            self.look.resolve_role(SemanticRole::Focus).hsla()
        } else if checked && !state.disabled {
            indicator_background
        } else {
            self.look.resolve_role(SemanticRole::Border).hsla()
        };

        let label_color = if state.disabled {
            self.look.resolve_role(SemanticRole::MutedForeground).hsla()
        } else {
            self.look.resolve_role(SemanticRole::Foreground).hsla()
        };

        CheckboxPalette {
            control_background: None,
            control_border: None,
            indicator_background,
            indicator_border,
            checkmark_color,
            label_color,
            label_typography: label_typography(size),
            label_font_family: font_family(&self.look),
            indicator_shadow: None,
        }
    }

    fn metrics(&self) -> MetricTokens {
        self.look.metrics()
    }
}

pub fn checkbox_theme(look: Arc<RadixLook>) -> Arc<dyn CheckboxTheme> {
    Arc::new(RadixCheckboxTheme { look: look.as_ref().clone() })
}
