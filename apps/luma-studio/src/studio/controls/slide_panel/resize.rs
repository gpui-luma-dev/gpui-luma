use gpui::{Context, IntoElement, Render, Window, div};

#[derive(Clone, Debug, Default)]
pub(crate) struct SlidePanelResizeDrag;

impl Render for SlidePanelResizeDrag {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}
