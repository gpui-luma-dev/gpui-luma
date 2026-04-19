use gpui::{AnyElement, Context, Entity, IntoElement, div, prelude::*};
use gpui_luma::controls::progress::Progress;

use crate::gallery::control::GalleryApp;

use super::super::shared::gallery_pane;

pub(in crate::gallery) struct ProgressPane {
    progress: Entity<Progress>,
    disabled_progress: Entity<Progress>,
}

impl ProgressPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>) -> Self {
        Self {
            progress: Progress::new("progress-example").range(1..100).value(41).spawn(cx),
            disabled_progress: Progress::new("disabled-progress-example")
                .range(1..100)
                .value(41)
                .enabled(false)
                .spawn(cx),
        }
    }

    pub(in crate::gallery) fn render(&self) -> AnyElement {
        gallery_pane(
            "Progress",
            div()
                .flex()
                .items_center()
                .gap_3()
                .child(self.progress.clone())
                .child(self.disabled_progress.clone())
                .into_any_element(),
        )
    }
}
