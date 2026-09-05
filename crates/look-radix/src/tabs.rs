//! Radix tabs theme adapter.

use std::sync::Arc;

use gpui::{FontWeight, SharedString};
use luma::controls::tabs::{TabsItemLook, TabsListLook, TabsTheme};
use luma::theme::{ControlSize, InteractionState, LumaTextStyle};

use crate::look::RadixLook;
use crate::semantic::SemanticRole;
use crate::typography::font_family;

struct RadixTabsTheme {
    look: RadixLook,
}

impl TabsTheme for RadixTabsTheme {
    fn resolve_list(&self, enabled: bool, size: ControlSize) -> TabsListLook {
        let metrics = self.look.metrics();
        TabsListLook {
            background: (!enabled).then(|| self.look.resolve_role(SemanticRole::Surface).hsla()),
            border: Some(self.look.resolve_role(SemanticRole::Border).hsla()),
            radius: 0.0,
            padding: 0.0,
            gap: metrics.gap(size),
        }
    }

    fn resolve_item(&self, active: bool, state: InteractionState, size: ControlSize) -> TabsItemLook {
        let metrics = self.look.metrics();
        let control = metrics.for_size(size);
        let muted = self.look.resolve_role(SemanticRole::MutedForeground).hsla();
        let fg = self.look.resolve_role(SemanticRole::Foreground).hsla();
        let accent = self.look.resolve_role(SemanticRole::Primary).hsla();

        let label_color = if state.disabled {
            muted
        } else if active {
            fg
        } else if state.hovered {
            fg
        } else {
            muted
        };

        TabsItemLook {
            label_color,
            indicator: active.then_some(accent),
            label_typography: LumaTextStyle {
                size: match size {
                    ControlSize::Sm => 12.5,
                    ControlSize::Md => 14.0,
                    ControlSize::Lg => 16.0,
                },
                line_height: match size {
                    ControlSize::Sm => 18.0,
                    ControlSize::Md => 20.0,
                    ControlSize::Lg => 22.0,
                },
                weight: if active { FontWeight::MEDIUM } else { FontWeight::NORMAL },
            },
            radius: 0.0,
            padding_x: control.padding_x,
            height: control.height * 0.85,
            indicator_height: 2.0,
        }
    }

    fn font_family(&self) -> SharedString {
        font_family(&self.look)
    }
}

pub fn tabs_theme(look: Arc<RadixLook>) -> Arc<dyn TabsTheme> {
    Arc::new(RadixTabsTheme { look: look.as_ref().clone() })
}
