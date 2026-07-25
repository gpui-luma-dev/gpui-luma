use std::sync::Arc;

use gpui::{AnyElement, App, AppContext, Context, Entity, IntoElement};

use super::button::ButtonControlExposition;
use super::checkbox::CheckboxControlExposition;
use super::color_slider::ColorSliderControlExposition;
use super::modal_overlay::ModalOverlayControlExposition;
use super::radio_button::RadioButtonControlExposition;
use super::switch::SwitchControlExposition;
use super::textfield::TextFieldControlExposition;
use crate::studio::controls::catalog::ControlDocEntry;

pub enum ControlExposition {
    Button(Entity<ButtonControlExposition>),
    Checkbox(Entity<CheckboxControlExposition>),
    RadioButton(Entity<RadioButtonControlExposition>),
    Switch(Entity<SwitchControlExposition>),
    ColorSlider(Entity<ColorSliderControlExposition>),
    TextField(Entity<TextFieldControlExposition>),
    ModalOverlay(Entity<ModalOverlayControlExposition>),
}

impl ControlExposition {
    pub fn spawn_all<T: 'static>(look: Arc<gpui_luma_look_shadcn::ShadcnLook>, cx: &mut Context<T>) -> Vec<Self> {
        vec![
            Self::Button(cx.new(|cx| ButtonControlExposition::new(cx, look.clone()))),
            Self::Checkbox(cx.new(|cx| CheckboxControlExposition::new(cx, look.clone()))),
            Self::RadioButton(cx.new(|cx| RadioButtonControlExposition::new(cx, look.clone()))),
            Self::Switch(cx.new(|cx| SwitchControlExposition::new(cx, look.clone()))),
            Self::ColorSlider(cx.new(|cx| ColorSliderControlExposition::new(cx, look.clone()))),
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
            Self::Checkbox(entity) => entity.read(cx).entry(),
            Self::RadioButton(entity) => entity.read(cx).entry(),
            Self::Switch(entity) => entity.read(cx).entry(),
            Self::ColorSlider(entity) => entity.read(cx).entry(),
            Self::TextField(entity) => entity.read(cx).entry(),
            Self::ModalOverlay(entity) => entity.read(cx).entry(),
        }
    }

    pub fn sync_look(&self, look: Arc<gpui_luma_look_shadcn::ShadcnLook>, cx: &mut App) {
        match self {
            Self::Button(entity) => {
                entity.update(cx, |exposition, cx| exposition.sync_look(look, cx));
            }
            Self::Checkbox(entity) => {
                entity.update(cx, |exposition, cx| exposition.sync_look(look, cx));
            }
            Self::RadioButton(entity) => {
                entity.update(cx, |exposition, cx| exposition.sync_look(look, cx));
            }
            Self::Switch(entity) => {
                entity.update(cx, |exposition, cx| exposition.sync_look(look, cx));
            }
            Self::ColorSlider(entity) => {
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

    pub fn render(&self, _cx: &App) -> AnyElement {
        match self {
            Self::Button(entity) => entity.clone().into_any_element(),
            Self::Checkbox(entity) => entity.clone().into_any_element(),
            Self::RadioButton(entity) => entity.clone().into_any_element(),
            Self::Switch(entity) => entity.clone().into_any_element(),
            Self::ColorSlider(entity) => entity.clone().into_any_element(),
            Self::TextField(entity) => entity.clone().into_any_element(),
            Self::ModalOverlay(entity) => entity.clone().into_any_element(),
        }
    }

    pub fn find<'a>(expositions: &'a [Self], id: &str, cx: &App) -> Option<&'a Self> {
        expositions.iter().find(|exposition| exposition.id(cx) == id)
    }
}
