//! Runtime policy controls for the two independent orientation examples.

use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use luma::controls::checkbox::{Checkbox, CheckboxEvent};
use luma::controls::listbox::{SelectionMode, SelectionPolicy};
use luma::controls::selector::{Selector, SelectorEvent, SelectorItem};
use luma::infra::presenter::HasPresenter;
use luma::vstack;
use luma_look_shadcn::{self as shadcn, ShadcnLook, LumaTypographyExt, ShadcnTextSize};
use luma_look_shadcn::prelude::*;

use super::{horizontal::HorizontalListExample, vertical::VerticalListExample};

const MODES: [(&str, &str, SelectionMode); 5] = [
    ("none", "No selection", SelectionMode::None),
    ("required", "Single · required", SelectionMode::SingleRequired),
    ("single", "Single · optional", SelectionMode::SingleAllowNone),
    ("multiple", "Multiple · toggle", SelectionMode::Multiple),
    ("extended", "Extended · range", SelectionMode::Extended),
];

pub(super) struct SelectionControls {
    look: Arc<ShadcnLook>,
    policy: SelectionPolicy,
    mode: Entity<Selector>,
    toggle_off: Checkbox,
    follows_active: Checkbox,
    vertical: Entity<VerticalListExample>,
    horizontal: Entity<HorizontalListExample>,
    _subscriptions: Vec<Subscription>,
}

impl SelectionControls {
    pub fn new(
        look: Arc<ShadcnLook>,
        vertical: Entity<VerticalListExample>,
        horizontal: Entity<HorizontalListExample>,
        cx: &mut Context<Self>,
    ) -> Self {
        let mode = shadcn::Selector::new("listbox-selection-mode")
            .look(&look)
            .label("Selection mode")
            .items(MODES.map(|(id, label, _)| SelectorItem::new(id).label(label)))
            .selected_id("extended")
            .spawn(cx);
        let toggle_off = shadcn::Checkbox::new("listbox-toggle-off")
            .look(&look)
            .enabled(false)
            .content(|_, _| div().child("Toggle off selected item").into_any_element())
            .spawn(cx);
        let follows_active = shadcn::Checkbox::new("listbox-follows-active")
            .look(&look)
            .enabled(false)
            .content(|_, _| div().child("Selection follows navigation").into_any_element())
            .spawn(cx);
        let subscriptions = vec![
            cx.subscribe(&mode, |this, _, event, cx| {
                if let SelectorEvent::Change { item_id, .. } = event
                    && let Some((_, _, mode)) = MODES.iter().find(|(id, _, _)| *id == item_id.as_ref())
                {
                    this.policy.mode = *mode;
                    this.apply_policy(cx);
                }
            }),
            cx.subscribe(&toggle_off, |this, _, event, cx| {
                if let CheckboxEvent::Change { checked } = event {
                    this.policy.toggle_off = *checked;
                    this.apply_policy(cx);
                }
            }),
            cx.subscribe(&follows_active, |this, _, event, cx| {
                if let CheckboxEvent::Change { checked } = event {
                    this.policy.selection_follows_active = *checked;
                    this.apply_policy(cx);
                }
            }),
        ];
        Self {
            look,
            policy: SelectionPolicy::new(SelectionMode::Extended),
            mode,
            toggle_off,
            follows_active,
            vertical,
            horizontal,
            _subscriptions: subscriptions,
        }
    }

    fn apply_policy(&mut self, cx: &mut Context<Self>) {
        self.vertical.update(cx, |list, cx| list.set_selection_policy(self.policy, cx));
        self.horizontal.update(cx, |list, cx| list.set_selection_policy(self.policy, cx));
        self.toggle_off
            .update(cx, |control, cx| control.set_enabled(self.policy.mode == SelectionMode::SingleAllowNone, cx));
        self.follows_active.update(cx, |control, cx| {
            control.set_enabled(
                matches!(self.policy.mode, SelectionMode::SingleAllowNone | SelectionMode::SingleRequired),
                cx,
            )
        });
        cx.notify();
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look;
        self.mode.update(cx, |_, cx| cx.notify());
        self.toggle_off.update(cx, |_, cx| cx.notify());
        self.follows_active.update(cx, |_, cx| cx.notify());
        cx.notify();
    }
}

impl Render for SelectionControls {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        with_look(&self.look, || {
            let hint = match self.policy.mode {
                SelectionMode::None => {
                    "Click or arrows move the active item. Enter or double-click activates; nothing is selected."
                }
                SelectionMode::SingleRequired => {
                    "One enabled item must remain selected. Applies to Vertical and Horizontal below."
                }
                SelectionMode::SingleAllowNone => {
                    "Select one item or clear it. Toggle-off and navigation selection are optional."
                }
                SelectionMode::Multiple => {
                    "Click or Space toggles items independently. Cmd/Ctrl+A selects all; Cmd/Ctrl+Shift+A clears."
                }
                SelectionMode::Extended => {
                    "Click selects one; Cmd/Ctrl-click toggles; Shift-click or Shift+arrows selects a range. Cmd/Ctrl+Shift adds a range. Cmd/Ctrl+A selects all; Cmd/Ctrl+Shift+A clears."
                }
            };
            vstack! { gap=8.0;
                div().child("Selection policies · Vertical and Horizontal")
                    .typography_style(self.look.typography_scale(ShadcnTextSize::Sm)).text_color(self.look.chrome().title_text),
                div().w(px(250.0)).child(self.mode.clone()),
                div().flex().flex_wrap().gap(px(12.0)).child(self.toggle_off.clone()).child(self.follows_active.clone()),
                div().typography_style(self.look.typography_scale(ShadcnTextSize::Xs)).text_color(self.look.chrome().muted_text).child(hint),
            }
        })
    }
}
