use std::sync::{Arc, OnceLock};

use gpui::{AnyElement, App, Div, FontWeight, Stateful, Window, div, px, prelude::*};
use lucide_icons::Icon as LucideIcon;

use super::ListBoxRenderModel;
use crate::controls::button_family::{ButtonFamilyRole, ButtonFamilyTheme, ButtonVariant, default_button_family_theme};
use crate::controls::command::button::ButtonTemplate;
use crate::controls::icon::LUCIDE_FONT_FAMILY;
use crate::controls::listbox::{ListBoxItemButtonRenderModel, ListBoxItemContentModel};

struct PlainListBoxItemButtonTemplate {
    theme: Arc<dyn ButtonFamilyTheme>,
}

pub fn default_listbox_item_button_template() -> Arc<dyn ButtonTemplate<bool>> {
    static TEMPLATE: OnceLock<Arc<dyn ButtonTemplate<bool>>> = OnceLock::new();

    TEMPLATE
        .get_or_init(|| Arc::new(PlainListBoxItemButtonTemplate { theme: default_button_family_theme() }))
        .clone()
}

pub(crate) fn render_listbox_row_content(
    model: &ListBoxRenderModel<'_>,
    content_model: ListBoxItemContentModel,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    if let Some(item_button_template) = model.item_button_template.as_ref() {
        let presenter = model.content.clone();
        let content_model_for_presenter = content_model.clone();

        let button_model = ListBoxItemButtonRenderModel {
            id: format!("{}-{}", model.id, content_model.item_id).into(),
            data: content_model.selected,
            content: Arc::new(move |_, cx| presenter(&content_model_for_presenter, cx)),
            kind: model.kind,
            role: ButtonFamilyRole::Toggle { selected: content_model.selected },
            size: model.size,
            state: content_model_state(content_model),
            round: false,
            radius_override: std::cell::Cell::new(None),
        };

        item_button_template.render(&button_model, window, cx).into_any_element()
    } else {
        (model.content)(&content_model, cx)
    }
}

impl ButtonTemplate<bool> for PlainListBoxItemButtonTemplate {
    fn render(&self, model: &ListBoxItemButtonRenderModel, _window: &mut Window, cx: &mut App) -> Stateful<Div> {
        let appearance = self.theme.resolve(
            button_variant(model.kind),
            ButtonFamilyRole::Toggle { selected: model.data },
            model.size,
            model.state,
        );
        let icon_size = (appearance.height * 0.50).max(12.0);

        div()
            .id(model.id.clone())
            .w_full()
            .flex()
            .items_center()
            .gap(px(appearance.gap))
            .text_size(px(appearance.typography.size))
            .line_height(px(appearance.typography.line_height))
            .font_weight(appearance.typography.weight)
            .child(render_selection_checkmark(model.data, icon_size))
            .child(div().flex_1().child((model.content)(model, cx)))
    }
}

fn render_selection_checkmark(selected: bool, size: f32) -> AnyElement {
    if selected {
        div()
            .size(px(size))
            .flex()
            .items_center()
            .justify_center()
            .font_family(LUCIDE_FONT_FAMILY)
            .font_weight(FontWeight::NORMAL)
            .text_size(px(size))
            .line_height(px(size))
            .child(char::from(LucideIcon::Check).to_string())
            .into_any_element()
    } else {
        div().size(px(size)).into_any_element()
    }
}

fn button_variant(kind: crate::controls::button_family::ButtonKind) -> ButtonVariant {
    match kind {
        crate::controls::button_family::ButtonKind::Standard => ButtonVariant::Standard,
        crate::controls::button_family::ButtonKind::Ghost => ButtonVariant::Ghost,
        crate::controls::button_family::ButtonKind::Prominent => ButtonVariant::Prominent,
    }
}

fn content_model_state(content_model: ListBoxItemContentModel) -> crate::theme::InteractionState {
    crate::theme::InteractionState {
        hovered: content_model.hovered,
        pressed: content_model.pressed,
        focused: content_model.focused,
        disabled: !content_model.enabled,
    }
}
