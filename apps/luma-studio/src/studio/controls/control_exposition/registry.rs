use std::sync::Arc;

use gpui::{AnyElement, App, AppContext, Context, Entity, IntoElement};

use super::button::ButtonControlExposition;
use super::modal_overlay::ModalOverlayControlExposition;
use super::textfield::TextFieldControlExposition;
use crate::studio::controls::catalog::ControlDocEntry;

pub enum ControlExposition {
    Button(Entity<ButtonControlExposition>),
    TextField(Entity<TextFieldControlExposition>),
    ModalOverlay(Entity<ModalOverlayControlExposition>),
}

impl ControlExposition {
    pub fn spawn_all<T: 'static>(look: Arc<gpui_luma_look_shadcn::ShadcnLook>, cx: &mut Context<T>) -> Vec<Self> {
        vec![
            Self::Button(cx.new(|cx| ButtonControlExposition::new(cx, look.clone()))),
            Self::TextField(cx.new(|cx| TextFieldControlExposition::new(cx, look.clone()))),
            Self::ModalOverlay(cx.new(|cx| ModalOverlayControlExposition::new(cx, look.clone()))),
        ]
    }

    pub fn id(&self, cx: &App) -> &'static str {
        self.entry(cx).id
    }

    pub fn entry(&self, cx: &App) -> ControlDocEntry {
        match self {
            Self::Button(entity) => entity.read(cx).entry(),
            Self::TextField(entity) => entity.read(cx).entry(),
            Self::ModalOverlay(entity) => entity.read(cx).entry(),
        }
    }

    pub fn sync_look(&self, look: Arc<gpui_luma_look_shadcn::ShadcnLook>, cx: &mut App) {
        match self {
            Self::Button(entity) => {
                entity.update(cx, |exposition, cx| exposition.sync_look(look, cx));
            }
            Self::TextField(entity) => {
                entity.update(cx, |exposition, cx| exposition.sync_look(look, cx));
            }
            Self::ModalOverlay(entity) => {
                entity.update(cx, |exposition, cx| exposition.sync_look(look, cx));
            }
        }
    }

    pub fn render(&self, cx: &App) -> AnyElement {
        match self {
            Self::Button(entity) => entity.clone().into_any_element(),
            Self::TextField(entity) => entity.clone().into_any_element(),
            Self::ModalOverlay(entity) => entity.clone().into_any_element(),
        }
    }

    pub fn find<'a>(expositions: &'a [Self], id: &str, cx: &App) -> Option<&'a Self> {
        expositions.iter().find(|exposition| exposition.id(cx) == id)
    }
}
