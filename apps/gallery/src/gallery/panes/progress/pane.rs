use gpui::{AnyElement, Context, Entity, IntoElement, div, prelude::*};
use gpui_luma::controls::progress::Progress;

use crate::gallery::control::GalleryApp;
use crate::gallery::theme::GalleryThemePack;

use super::super::shared::{gallery_pane, notify_entity};

#[derive(Clone)]
pub(in crate::gallery) struct ProgressPane {
    progress: Entity<Progress>,
    disabled_progress: Entity<Progress>,
}

impl ProgressPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, theme: &GalleryThemePack) -> Self {
        Self {
            progress: Progress::new("progress-example")
                .range(1..100)
                .value(41)
                .template(theme.progress_template())
                .spawn(cx),
            disabled_progress: Progress::new("disabled-progress-example")
                .range(1..100)
                .value(41)
                .enabled(false)
                .template(theme.progress_template())
                .spawn(cx),
        }
    }

    pub(in crate::gallery) fn render(&self, theme: &GalleryThemePack) -> AnyElement {
        gallery_pane(
            "Progress",
            div()
                .flex()
                .items_center()
                .gap_3()
                .child(self.progress.clone())
                .child(self.disabled_progress.clone())
                .into_any_element(),
            theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.progress, cx);
        notify_entity(&self.disabled_progress, cx);
    }
}
