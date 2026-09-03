use std::sync::Arc;

use gpui::{Context, Entity, Render, Window, div, prelude::*};
use luma::controls::command::button::Button;
use luma::controls::presenter::HasPresenter;
use luma::controls::selector::{Selector, SelectorItem};
use luma::theme::ControlSize;
use luma_look_shadcn::prelude::*;
use luma_look_shadcn::ShadcnLook;
use luma::{declare_form, form_field, hstack, vstack};

use super::common::titled_card;

declare_form! {
    pub struct SharePanel {
        controls: {
            link_selector: Entity<Selector> = look
                .selector("share-link")
                .label("Anyone with the link")
                .items(link_items()),
            copy_button: Entity<Button> = look.outline_button("share-copy").label("Copy Link").size(size),
        },
        args: {
            look: Arc<ShadcnLook>,
            size: ControlSize,
        },
        fields: {}
    }
}

impl Render for SharePanel {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let link_selector = self.link_selector.clone();
        let copy_button = self.copy_button.clone();

        titled_card(
            "luma-studio-share-card",
            &self.look,
            380.0,
            "Share this document",
            "Anyone with the link can view this document.",
            move |_, _| {
                vstack! {
                    gap=12;
                    form_field!("Link", chrome; hstack! {
                        gap=8;
                        div().flex_1().min_w_0().child(link_selector.clone()),
                        copy_button.clone(),
                    }),
                }
                .into_any_element()
            },
            window,
            _cx,
        )
    }
}

fn link_items() -> Vec<SelectorItem> {
    vec![
        SelectorItem::new("anyone").label("Anyone with the link"),
        SelectorItem::new("restricted").label("Restricted"),
    ]
}
