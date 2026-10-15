use gpui_luma::controls::tabs::TabsEventExt;
use gpui_luma::controls::scrollbar::ScrollbarEventExt;
use gpui_luma::controls::radio_group::RadioGroupEventExt;
use std::sync::Arc;

use gpui::{Entity, Context, FocusHandle, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::radio_group::{RadioGroup, RadioGroupEvent, RadioGroupItem};
use gpui_luma::controls::scroll_container::{ScrollContainer, ScrollbarAutoHideActivate, ScrollbarVisibility};
use gpui_luma::controls::scrollbar::{ScrollbarEvent, default_scrollbar_template};
use gpui_luma::focus::LumaFocusScopeExt;
use gpui_luma::shell::TitleBar;
use gpui_luma::theme::ThemeMode;
use gpui_luma_look_radix::Look;

use crate::theme::{self, TypographyExt};
use gpui_luma::controls::tabs::{Tabs, TabsEvent, TabsItem};

pub(crate) struct PlotVizApp {
    tabs: Entity<Tabs>,
    active_tab: gpui::SharedString,
    entrance: gpui_luma::motion::VisualTransition,
    radial: Result<Entity<crate::examples::radial::RadialGallery>, gpui_luma_plot::DatasetError>,
    radar: Result<Entity<crate::examples::radar::RadarGallery>, gpui_luma_plot::DatasetError>,
    pies: Result<crate::examples::pie::PieGallery, gpui_luma_plot::DatasetError>,
    area: Result<crate::examples::area::AreaGallery, gpui_luma_plot::DatasetError>,
    bars: Result<Entity<crate::examples::bar::BarExample>, gpui_luma_plot::DatasetError>,
    focus_scope: FocusHandle,
    look: Arc<Look>,
    mode_selector: RadioGroup<RadioGroupItem>,
    scroll: ScrollContainer,
    examples: Result<crate::examples::line::LineGallery, gpui_luma_plot::DatasetError>,
    _subscriptions: Vec<Subscription>,
}

impl PlotVizApp {
    pub(crate) fn new(cx: &mut Context<Self>) -> Self {
        let look = theme::default_look();
        let mode_selector = theme::spawn_mode_selector("theme-mode", &look, cx);
        let scroll = ScrollContainer::new("plot-viz-scroll", default_scrollbar_template(), cx)
            .visibility(ScrollbarVisibility::AutoHide)
            .auto_hide_activate(ScrollbarAutoHideActivate::HoverOrMove);
        mode_selector.clone().on_event(cx, |this, event: &RadioGroupEvent, cx| {
            if let RadioGroupEvent::Change { value: Some(value) } = event {
                this.set_mode(theme::mode_for_id(value.as_ref()), cx);
            }
        });
        let scrollbar = scroll.scrollbar();
        let scroll_for_event = scroll.clone();
        scrollbar.clone().on_event(cx, move |_, event: &ScrollbarEvent, cx| {
            if let ScrollbarEvent::Change { value } = event {
                scroll_for_event.set_vertical_offset(*value, cx);
            }
        });
        let tabs = gpui_luma_look_radix::Tabs::new("chart-tabs")
            .look(&look)
            .line()
            .items(["Line", "Area", "Bar", "Pie", "Radar", "Radial"].map(|name| TabsItem::new(name).label(name)))
            .active("Line")
            .spawn(cx);
        tabs.clone().on_event(cx, |this, event: &TabsEvent, cx| {
            if let TabsEvent::Change { tab_id, .. } = event {
                if this.active_tab != *tab_id {
                    this.entrance.snap_to(0.0);
                    this.entrance.set_target(1.0);
                }
                if tab_id.as_ref() == "Radar"
                    && this.active_tab.as_ref() != "Radar"
                    && let Ok(radar) = &this.radar
                {
                    radar.update(cx, |radar, cx| radar.enter(cx));
                }
                if tab_id.as_ref() == "Radial"
                    && this.active_tab.as_ref() != "Radial"
                    && let Ok(radial) = &this.radial
                {
                    radial.update(cx, |radial, cx| radial.enter(cx));
                }
                this.active_tab = tab_id.clone();
                this.scroll.set_vertical_offset(0.0, cx);
                cx.notify();
            }
        });
        Self {
            tabs,
            active_tab: "Line".into(),
            entrance: {
                let mut motion = gpui_luma::motion::VisualTransition::new(0.0, std::time::Duration::from_millis(900));
                motion.set_target(1.0);
                motion
            },
            radial: crate::examples::radial::RadialGallery::spawn(&look, cx),
            radar: crate::examples::radar::RadarGallery::spawn(&look, cx),
            pies: crate::examples::pie::PieGallery::spawn(&look, cx),
            area: crate::examples::area::AreaGallery::spawn(&look, cx),
            bars: crate::examples::bar::BarExample::spawn(&look, cx),
            focus_scope: cx.focus_handle(),
            examples: crate::examples::line::examples(&look, cx),
            look,
            mode_selector,
            scroll,
            _subscriptions: { Vec::new() },
        }
    }

    fn set_mode(&mut self, mode: ThemeMode, cx: &mut Context<Self>) {
        if self.look.mode() == mode {
            return;
        }
        self.look.set_mode(mode);
        self.mode_selector.update(cx, |selector, cx| selector.set_selected(theme::mode_id(mode), cx));
        cx.refresh_windows();
        cx.notify();
    }
}

impl Render for PlotVizApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.entrance.sync();
        self.entrance.schedule_frame(window, cx);
        let progress = self.entrance.progress();
        self.scroll.sync_scrollbar(cx);
        let chrome = theme::chrome(&self.look);
        let typography = theme::typography();
        let gallery = match self.active_tab.as_ref() {
            "Radial" => match &self.radial {
                Ok(radial) => radial.clone().into_any_element(),
                Err(error) => div().child(error.to_string()).into_any_element(),
            },
            "Radar" => match &self.radar {
                Ok(radar) => radar.clone().into_any_element(),
                Err(error) => div().child(error.to_string()).into_any_element(),
            },
            "Pie" => match &self.pies {
                Ok(pies) => pies.render(&self.look, progress, cx).into_any_element(),
                Err(error) => div().child(error.to_string()).into_any_element(),
            },
            "Area" => match &self.area {
                Ok(area) => area.render(progress, cx).into_any_element(),
                Err(error) => div().child(error.to_string()).into_any_element(),
            },
            "Bar" => match &self.bars {
                Ok(example) => {
                    crate::examples::motion::display(example, progress, cx);
                    example.clone().into_any_element()
                }
                Err(error) => div().child(error.to_string()).into_any_element(),
            },
            "Line" => match &self.examples {
                Ok(examples) => crate::examples::line::gallery(examples, &self.look, progress, cx).into_any_element(),
                Err(error) => div().text_color(chrome.muted_text).child(error.to_string()).into_any_element(),
            },
            _ => div()
                .text_color(chrome.muted_text)
                .child(format!("{} charts are coming next.", self.active_tab))
                .into_any_element(),
        };
        div()
            .luma_focus_scope(&self.focus_scope)
            .size_full()
            .flex()
            .flex_col()
            .font_family(typography.font.sans.family)
            .text_color(chrome.body_text)
            .bg(chrome.app_background)
            .child(
                TitleBar::new()
                    .height(crate::app_shell::SHELL_TITLEBAR_HEIGHT)
                    .background_color(chrome.app_background)
                    .border_color(chrome.border)
                    .text_color(chrome.title_text)
                    .child(div().px_4().typography_style(typography.text.label).child("Plot Viz")),
            )
            .child(
                theme::scroll_surface(&self.look).flex_1().min_h_0().render(cx).child(
                    self.scroll.render(
                        div()
                            .w_full()
                            .flex()
                            .flex_col()
                            .items_center()
                            .gap(px(12.0))
                            .p_6()
                            .child(div().typography_style(typography.text.scale.two_xl).child("Luma Plot"))
                            .child(
                                div()
                                    .typography_style(typography.text.body)
                                    .text_color(chrome.muted_text)
                                    .child("Data, specification, presentation, and view"),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(8.0))
                                    .child(
                                        div()
                                            .typography_style(typography.text.caption)
                                            .text_color(chrome.muted_text)
                                            .child("Theme"),
                                    )
                                    .child(self.mode_selector.clone()),
                            )
                            .child(self.tabs.read(cx).tab_list())
                            .child(gallery)
                            .child(
                                div()
                                    .typography_style(typography.text.caption)
                                    .text_color(chrome.muted_text)
                                    .child("Static preview · values · missing data · runtime theme switching"),
                            )
                            .into_any_element(),
                    ),
                ),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[gpui::test]
    fn startup_and_each_tab_entry_restart_reveal(cx: &mut gpui::TestAppContext) {
        let (host, cx) = cx.add_window_view(|_, cx| PlotVizApp::new(cx));
        host.read_with(cx, |app, _| {
            assert_eq!(app.active_tab, "Line");
            assert!(app.entrance.is_animating());
            assert!(app.entrance.progress() < 1.0);
        });
        for name in ["Area", "Bar", "Pie", "Radar", "Radial", "Line"] {
            host.update(cx, |app, cx| {
                app.entrance.snap_to(1.0);
                app.tabs.update(cx, |_, cx| cx.emit(TabsEvent::Change { tab_id: name.into(), label: name.into() }));
            });
            host.read_with(cx, |app, _| {
                assert_eq!(app.active_tab, name);
                assert!(app.entrance.progress() < 0.1);
                assert!(app.entrance.is_animating());
            });
            cx.update(|window, cx| {
                window.draw(cx).clear(cx);
            });
        }
    }
}
