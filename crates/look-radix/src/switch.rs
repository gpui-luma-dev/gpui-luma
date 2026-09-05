//! Radix switch theme adapter.

use std::sync::Arc;

use luma::controls::switch::{SwitchPalette, SwitchTheme};
use luma::theme::{ControlSize, InteractionState, MetricTokens};

use crate::look::RadixLook;
use crate::semantic::SemanticRole;
use crate::typography::{font_family, label_typography};

struct RadixSwitchTheme {
    look: RadixLook,
}

impl SwitchTheme for RadixSwitchTheme {
    fn resolve(&self, on: bool, state: InteractionState, size: ControlSize) -> SwitchPalette {
        let track_background = if state.disabled {
            self.look.resolve_role(SemanticRole::Surface).hsla()
        } else if on {
            self.look.resolve_role(SemanticRole::Primary).hsla()
        } else {
            self.look.resolve_role(SemanticRole::Border).hsla()
        };

        let track_border = if state.focused && !state.disabled {
            self.look.resolve_role(SemanticRole::Focus).hsla()
        } else {
            track_background
        };

        let (thumb_background, thumb_border) = if state.disabled {
            (
                self.look.resolve_role(SemanticRole::MutedForeground).hsla(),
                self.look.resolve_role(SemanticRole::Surface).hsla(),
            )
        } else if on {
            let thumb = self.look.resolve_role(SemanticRole::PrimaryForeground).hsla();
            (thumb, thumb)
        } else {
            (
                self.look.resolve_role(SemanticRole::Background).hsla(),
                self.look.resolve_role(SemanticRole::Border).hsla(),
            )
        };

        SwitchPalette {
            track_background,
            track_border,
            thumb_background,
            thumb_border,
            thumb_shadow: Vec::new(),
            label_color: if state.disabled {
                self.look.resolve_role(SemanticRole::MutedForeground).hsla()
            } else {
                self.look.resolve_role(SemanticRole::Foreground).hsla()
            },
            label_typography: label_typography(size),
            label_font_family: font_family(&self.look),
        }
    }

    fn metrics(&self) -> MetricTokens {
        self.look.metrics()
    }
}

pub fn switch_theme(look: Arc<RadixLook>) -> Arc<dyn SwitchTheme> {
    Arc::new(RadixSwitchTheme { look: look.as_ref().clone() })
}
