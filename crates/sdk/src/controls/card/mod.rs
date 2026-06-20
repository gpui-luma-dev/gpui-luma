mod control;
mod model;
mod template;
mod theme;

pub use model::{CardBuilder, CardElementRenderer, CardModel, CardRenderModel};
pub use template::{CardTemplate, ThemedCardTemplate, default_card_template};
pub use theme::{CardLook, CardTheme, DefaultCardTheme, default_card_theme};

use gpui::{Entity, SharedString};

use self::control::CardControl;

pub type Card = Entity<CardControl>;

pub fn new(id: impl Into<SharedString>) -> CardBuilder {
    CardBuilder::new(id)
}
