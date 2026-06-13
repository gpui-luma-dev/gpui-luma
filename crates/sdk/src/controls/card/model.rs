use std::sync::Arc;

use gpui::{AnyElement, App, AppContext, Entity, IntoElement, SharedString, Window};

use super::{CardTemplate, default_card_template};
use super::control::CardControl;
use crate::theme::ControlSize;

pub type CardElementRenderer = Arc<dyn Fn(&mut Window, &mut App) -> AnyElement + Send + Sync>;

#[derive(Clone)]
pub struct CardModel {
    pub(crate) id: SharedString,
    pub(crate) size: ControlSize,
    pub(crate) title: Option<SharedString>,
    pub(crate) description: Option<SharedString>,
    pub(crate) header: Option<CardElementRenderer>,
    pub(crate) body: Vec<CardElementRenderer>,
    pub(crate) footer: Option<CardElementRenderer>,
    pub(crate) elevated: bool,
    pub(crate) full_height: bool,
    pub(crate) body_fill: bool,
    pub(crate) template: Arc<dyn CardTemplate>,
}

pub struct CardRenderModel<'a> {
    pub id: &'a SharedString,
    pub size: ControlSize,
    pub title: Option<&'a SharedString>,
    pub description: Option<&'a SharedString>,
    pub header: Option<&'a CardElementRenderer>,
    pub body: &'a [CardElementRenderer],
    pub footer: Option<&'a CardElementRenderer>,
    pub elevated: bool,
    pub full_height: bool,
    pub body_fill: bool,
}

pub struct CardBuilder {
    pub(crate) model: CardModel,
}

impl CardBuilder {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            model: CardModel {
                id: id.into(),
                size: ControlSize::Md,
                title: None,
                description: None,
                header: None,
                body: Vec::new(),
                footer: None,
                elevated: true,
                full_height: false,
                body_fill: false,
                template: default_card_template(),
            },
        }
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.model.size = size;
        self
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.model.title = Some(title.into());
        self
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.model.description = Some(description.into());
        self
    }

    pub fn header(mut self, header: impl Fn(&mut Window, &mut App) -> AnyElement + Send + Sync + 'static) -> Self {
        self.model.header = Some(Arc::new(header));
        self
    }

    pub fn header_element(mut self, header: impl IntoElement + Clone + Send + Sync + 'static) -> Self {
        self.model.header = Some(renderer_from_element(header));
        self
    }

    pub fn child(mut self, child: impl IntoElement + Clone + Send + Sync + 'static) -> Self {
        self.model.body.push(renderer_from_element(child));
        self
    }

    pub fn child_render(mut self, child: impl Fn(&mut Window, &mut App) -> AnyElement + Send + Sync + 'static) -> Self {
        self.model.body.push(Arc::new(child));
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = CardElementRenderer>) -> Self {
        self.model.body.extend(children);
        self
    }

    pub fn footer(mut self, footer: impl Fn(&mut Window, &mut App) -> AnyElement + Send + Sync + 'static) -> Self {
        self.model.footer = Some(Arc::new(footer));
        self
    }

    pub fn footer_element(mut self, footer: impl IntoElement + Clone + Send + Sync + 'static) -> Self {
        self.model.footer = Some(renderer_from_element(footer));
        self
    }

    pub fn elevated(mut self, elevated: bool) -> Self {
        self.model.elevated = elevated;
        self
    }

    pub fn full_height(mut self, full_height: bool) -> Self {
        self.model.full_height = full_height;
        self
    }

    pub fn body_fill(mut self, body_fill: bool) -> Self {
        self.model.body_fill = body_fill;
        self
    }

    pub fn template(mut self, template: Arc<dyn CardTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn render(self, window: &mut Window, cx: &mut App) -> gpui::Stateful<gpui::Div> {
        let model = self.model.render_model();
        self.model.template.render(&model, window, cx)
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<CardControl> {
        cx.new(|cx| CardControl::from_builder(self, cx))
    }
}

impl CardModel {
    pub(crate) fn render_model(&self) -> CardRenderModel<'_> {
        CardRenderModel {
            id: &self.id,
            size: self.size,
            title: self.title.as_ref(),
            description: self.description.as_ref(),
            header: self.header.as_ref(),
            body: &self.body,
            footer: self.footer.as_ref(),
            elevated: self.elevated,
            full_height: self.full_height,
            body_fill: self.body_fill,
        }
    }
}

fn renderer_from_element(element: impl IntoElement + Clone + Send + Sync + 'static) -> CardElementRenderer {
    Arc::new(move |_, _| element.clone().into_any_element())
}
