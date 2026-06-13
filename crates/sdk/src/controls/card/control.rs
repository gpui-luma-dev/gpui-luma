use gpui::{Context, IntoElement, Render, SharedString, Window, div, prelude::*, px};

use super::{CardBuilder, CardRenderModel};
use crate::theme::observe_theme_revision;

pub struct CardControl {
    model: super::CardModel,
}

impl CardControl {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> CardBuilder {
        CardBuilder::new(id)
    }

    pub(crate) fn from_builder(builder: CardBuilder, cx: &mut Context<Self>) -> Self {
        observe_theme_revision(cx, |_, cx| cx.notify()).detach();
        Self { model: builder.model }
    }

    fn render_model(&self) -> CardRenderModel<'_> {
        self.model.render_model()
    }
}

impl Render for CardControl {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let model = self.render_model();
        let mut host = div().w_full();

        if self.model.full_height {
            host = host.h_full().min_h(px(0.0));
        }

        host.child(self.model.template.render(&model, window, cx)).into_any_element()
    }
}
