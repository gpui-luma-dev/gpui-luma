use std::rc::Rc;
use std::sync::Arc;

use gpui::{AnyElement, AppContext, Entity, Hsla, IntoElement, Pixels, SharedString, div, px};

use super::{SplitView, SplitViewTemplate, default_split_view_template};

pub type PaneRender = Rc<dyn Fn() -> AnyElement>;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SplitViewSeparatorVisibility {
    #[default]
    Always,
    Hover,
}

#[derive(Clone)]
pub struct SplitViewModel {
    pub(crate) id: SharedString,
    pub(crate) sidebar_width: Pixels,
    pub(crate) sidebar_min_width: Pixels,
    pub(crate) sidebar_max_width: Pixels,
    pub(crate) sidebar_collapsed_width: Pixels,
    pub(crate) collapsed: bool,
    pub(crate) resizable: bool,
    pub(crate) enabled: bool,
    pub(crate) separator_visibility: SplitViewSeparatorVisibility,
    pub(crate) separator_color: Option<Hsla>,
    pub(crate) separator_hover_color: Option<Hsla>,
    pub(crate) sidebar: PaneRender,
    pub(crate) content: PaneRender,
    pub(crate) template: Arc<dyn SplitViewTemplate>,
}

pub struct SplitViewRenderModel<'a> {
    pub id: &'a SharedString,
    pub sidebar_width: Pixels,
    pub sidebar_collapsed_width: Pixels,
    pub effective_sidebar_width: Pixels,
    pub collapsed: bool,
    pub resizable: bool,
    pub enabled: bool,
    pub separator_hovered: bool,
    pub separator_visibility: SplitViewSeparatorVisibility,
    pub separator_color: Option<Hsla>,
    pub separator_hover_color: Option<Hsla>,
}

pub struct SplitViewBuilder {
    pub(crate) model: SplitViewModel,
}

impl SplitViewBuilder {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            model: SplitViewModel {
                id: id.into(),
                sidebar_width: px(280.0),
                sidebar_min_width: px(220.0),
                sidebar_max_width: px(420.0),
                sidebar_collapsed_width: px(0.0),
                collapsed: false,
                resizable: true,
                enabled: true,
                separator_visibility: SplitViewSeparatorVisibility::Always,
                separator_color: None,
                separator_hover_color: None,
                sidebar: render_pane(|| div()),
                content: render_pane(|| div()),
                template: default_split_view_template(),
            },
        }
    }

    pub fn sidebar_width(mut self, width: Pixels) -> Self {
        self.model.sidebar_width =
            clamp_sidebar_width(width, self.model.sidebar_min_width, self.model.sidebar_max_width);
        self
    }

    pub fn sidebar_min_width(mut self, width: Pixels) -> Self {
        self.model.sidebar_min_width = width.max(px(0.0));
        self.model.sidebar_max_width = self.model.sidebar_max_width.max(self.model.sidebar_min_width);
        self.model.sidebar_width =
            clamp_sidebar_width(self.model.sidebar_width, self.model.sidebar_min_width, self.model.sidebar_max_width);
        self
    }

    pub fn sidebar_max_width(mut self, width: Pixels) -> Self {
        self.model.sidebar_max_width = width.max(self.model.sidebar_min_width);
        self.model.sidebar_width =
            clamp_sidebar_width(self.model.sidebar_width, self.model.sidebar_min_width, self.model.sidebar_max_width);
        self
    }

    pub fn sidebar_collapsed_width(mut self, width: Pixels) -> Self {
        self.model.sidebar_collapsed_width = width.max(px(0.0));
        self
    }

    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.model.collapsed = collapsed;
        self
    }

    pub fn resizable(mut self, resizable: bool) -> Self {
        self.model.resizable = resizable;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn separator_visibility(mut self, visibility: SplitViewSeparatorVisibility) -> Self {
        self.model.separator_visibility = visibility;
        self
    }

    pub fn separator_color(mut self, color: impl Into<Hsla>) -> Self {
        self.model.separator_color = Some(color.into());
        self
    }

    pub fn separator_hover_color(mut self, color: impl Into<Hsla>) -> Self {
        self.model.separator_hover_color = Some(color.into());
        self
    }

    pub fn sidebar<E, F>(mut self, render: F) -> Self
    where
        E: IntoElement,
        F: Fn() -> E + 'static,
    {
        self.model.sidebar = render_pane(render);
        self
    }

    pub fn content<E, F>(mut self, render: F) -> Self
    where
        E: IntoElement,
        F: Fn() -> E + 'static,
    {
        self.model.content = render_pane(render);
        self
    }

    pub fn template(mut self, template: Arc<dyn SplitViewTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<SplitView> {
        cx.new(|cx| SplitView::from_builder(self, cx))
    }
}

pub fn render_pane<E, F>(render: F) -> PaneRender
where
    E: IntoElement,
    F: Fn() -> E + 'static,
{
    Rc::new(move || render().into_any_element())
}

pub(crate) fn clamp_sidebar_width(width: Pixels, min_width: Pixels, max_width: Pixels) -> Pixels {
    width.max(min_width.max(px(0.0))).min(max_width.max(min_width))
}

pub(crate) fn effective_sidebar_width(collapsed: bool, sidebar_width: Pixels, collapsed_width: Pixels) -> Pixels {
    if collapsed {
        collapsed_width.max(px(0.0))
    } else {
        sidebar_width.max(px(0.0))
    }
}

#[cfg(test)]
mod tests {
    use gpui::px;

    use super::{clamp_sidebar_width, effective_sidebar_width};

    #[test]
    fn sidebar_width_clamps_to_configured_range() {
        assert_eq!(clamp_sidebar_width(px(120.0), px(220.0), px(420.0)), px(220.0));
        assert_eq!(clamp_sidebar_width(px(320.0), px(220.0), px(420.0)), px(320.0));
        assert_eq!(clamp_sidebar_width(px(520.0), px(220.0), px(420.0)), px(420.0));
    }

    #[test]
    fn effective_sidebar_width_uses_collapsed_width_without_destroying_expanded_width() {
        assert_eq!(effective_sidebar_width(false, px(320.0), px(0.0)), px(320.0));
        assert_eq!(effective_sidebar_width(true, px(320.0), px(0.0)), px(0.0));
        assert_eq!(effective_sidebar_width(true, px(320.0), px(64.0)), px(64.0));
    }
}
