mod interpolation;
pub(crate) mod motion;
mod mode;
pub(crate) use interpolation::spawn as spawn_interpolation_selector;
pub(crate) use motion::spawn_replay;
mod plot;
mod scrollbar;
pub(crate) use scrollbar::spawn_time_scrollbar;
pub(crate) use mode::{id as mode_id, mode_for_id, spawn as spawn_mode_selector};
pub(crate) use plot::RadixPlotStyle;

use std::sync::Arc;

use gpui::{Edges, Styled, px};
use gpui_luma::controls::frame::{FrameBuilder, FrameLook};
use gpui_luma::theme::{LumaChrome, LumaTextStyle, LumaTypography, ThemeMode};
use gpui_luma_look_radix::{Accent, Gray, Look, SemanticRole};

pub(crate) fn default_look() -> Arc<Look> {
    radix_look(ThemeMode::Dark)
}

fn radix_look(mode: ThemeMode) -> Arc<Look> {
    let look = Look::built_in();
    look.set_palettes(Accent::Indigo, Gray::Sand);
    look.set_mode(mode);
    Arc::new(look)
}

pub(crate) fn chrome(look: &Look) -> LumaChrome {
    let color = |role| look.resolve_role(role).hsla();
    LumaChrome {
        app_background: color(SemanticRole::Background),
        content_background: color(SemanticRole::Background),
        panel_background: color(SemanticRole::Surface),
        border: color(SemanticRole::Border),
        title_text: color(SemanticRole::Foreground),
        body_text: color(SemanticRole::Foreground),
        muted_text: color(SemanticRole::MutedForeground),
    }
}

/// Fixed host surface for the app's scroll viewport.
pub(crate) fn scroll_surface(look: &Look) -> FrameBuilder {
    let look = look.clone();
    FrameBuilder::new("plot-viz-scroll-surface").look_provider(move |_| FrameLook {
        background: look.resolve_role(SemanticRole::Background).hsla(),
        ..FrameLook::default()
    })
}

/// General decorated frame; content owns its own rendering and presentation.
pub(crate) fn panel_frame(look: &Look, id: &'static str) -> FrameBuilder {
    let look = look.clone();
    FrameBuilder::new(id).look_provider(move |_| FrameLook {
        background: look.resolve_role(SemanticRole::Surface).hsla(),
        border_color: look.resolve_role(SemanticRole::Border).hsla(),
        border_widths: Edges::all(px(1.0)),
        radius: px(8.0),
        ..FrameLook::default()
    })
}

pub(crate) fn typography() -> LumaTypography {
    LumaTypography::default()
}

pub(crate) trait TypographyExt: Styled + Sized {
    fn typography_style(self, style: LumaTextStyle) -> Self {
        self.text_size(px(style.size)).line_height(px(style.line_height)).font_weight(style.weight)
    }
}

impl<T: Styled> TypographyExt for T {}

pub(crate) mod visitors;

pub(crate) mod points;

pub(crate) mod pie;

pub(crate) mod radar;

pub(crate) mod radial;
