//! Radix Studio app shell — shared state, page routing, and window chrome.

use gpui_luma::prelude::TooltipEntityExt;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use gpui::{
    App, Context, Entity, FocusHandle, Hsla, Render, RenderImage, AnyElement, ScrollHandle, Subscription, Task, Window,
    div, prelude::*, px,
};
use gpui_luma::controls::button::ButtonEvent;
use gpui_luma::controls::overlay_window::{OverlayWindow, OverlayWindowMode, OverlayWindowPosition};
use gpui_luma::controls::popup_menu::PopupMenu;
use gpui_luma::controls::radio_group::{RadioGroup, RadioGroupItem, RadioGroupEvent};
use crate::controls::theme_mode;
use gpui_luma::focus::LumaFocusScopeExt;
use gpui_luma::infra::menu_item::MenuItem;
use gpui_luma::infra::presenter::HasPresenter;
use gpui_luma::shell::TitleBar;
use gpui_luma::controls::tabs::Tabs;
use gpui_luma::theme::ThemeMode;
use gpui_luma::{WideMiddle, dock_panel, spawn_wide_middle, vstack};
use gpui_luma_look_radix::{Accent, Gray, Look, LookControlExt, ScaleFamily, SemanticRole};
use gpui_luma_look_radix as radix;

use crate::color_hex::format_hex;
use crate::screens::custom_palette::{MeshCache, MeshRequest};
use crate::controls::{
    ClassicShadowEditor, ClassicShadowEditorEvent, ColorTextField, ColorTextFieldEvent, ScreenNav, ScreenNavEvent,
    SCREEN_TABS,
};
use crate::screens::{colors, custom_palette, developer, icons, style_guide};
use crate::tabs::RadixStudioTab;

const CONTENT_MAX_W: f32 = 1280.0;
const SCREEN_ENTER_DURATION: Duration = Duration::from_millis(300);
const CONTROL_BAR_FADE_DISTANCE: f32 = 96.0;
const STARTUP_MODE: ThemeMode = ThemeMode::Dark;

#[derive(Clone, Debug)]
struct SwatchSelection {
    family: &'static str,
    step: u8,
    hex: String,
}

/// Exact editor inputs, independent of the derived palette scale colors.
#[derive(Default)]
struct PaletteEdits {
    light: [Option<Hsla>; 3],
    dark: [Option<Hsla>; 3],
}

impl PaletteEdits {
    fn startup() -> Self {
        let mut edits = Self::default();
        edits.set(STARTUP_MODE, 2, gpui::black());
        edits
    }

    fn for_mode(&self, mode: ThemeMode) -> &[Option<Hsla>; 3] {
        match mode {
            ThemeMode::Light => &self.light,
            ThemeMode::Dark => &self.dark,
        }
    }

    fn colors(&self, theme: &Look) -> radix::CustomColors {
        // Defaults come from the named theme, never from the generated output.
        // Otherwise editing one input would silently change the other two seeds.
        let edits = self.for_mode(theme.mode());
        radix::CustomColors {
            accent: edits[0].unwrap_or_else(|| theme.resolve_role(SemanticRole::Primary).hsla()),
            gray: edits[1].unwrap_or_else(|| theme.resolve_step(ScaleFamily::Gray, 8).hsla()),
            background: edits[2].unwrap_or_else(|| theme.resolve_role(SemanticRole::Background).hsla()),
        }
    }

    fn apply(&self, theme: &Look, draft: &Look) {
        if self.for_mode(theme.mode()).iter().any(Option::is_some) {
            draft.set_custom_colors(self.colors(theme));
        }
    }

    fn set(&mut self, mode: ThemeMode, index: usize, color: Hsla) {
        let values = match mode {
            ThemeMode::Light => &mut self.light,
            ThemeMode::Dark => &mut self.dark,
        };
        values[index] = Some(color);
    }
}

pub struct RadixStudioApp {
    focus_scope: FocusHandle,
    /// Named palettes every screen but Custom Palette paints from.
    theme: Arc<Look>,
    /// Forked look the Custom Palette screen edits in isolation.
    draft: Arc<Look>,
    theme_accent: Accent,
    theme_gray: Gray,
    palette_edits: PaletteEdits,
    screen_nav: Entity<ScreenNav>,
    page_scroll: ScrollHandle,
    screens: Entity<Tabs>,
    mode_selector: RadioGroup<RadioGroupItem>,
    accent_field: Entity<ColorTextField>,
    gray_field: Entity<ColorTextField>,
    background_field: Entity<ColorTextField>,
    copy_menu: Entity<PopupMenu>,
    home: custom_palette::PreviewControls,
    style_guide: Option<style_guide::State>,
    shadow_editor: Entity<ClassicShadowEditor>,
    developer: Option<developer::State>,
    swatch_info: Arc<Mutex<Option<SwatchSelection>>>,
    swatch_overlay: OverlayWindow,
    preview_layout: Entity<WideMiddle>,
    signup_mesh_cache: MeshCache,
    signup_mesh_task: Option<Task<()>>,
    _subscriptions: Vec<Subscription>,
}

impl RadixStudioApp {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_scope = cx.focus_handle();
        let theme_accent = Accent::Indigo;
        let theme_gray = Gray::Auto;
        let theme = Arc::new(Look::built_in());
        theme.set_mode(STARTUP_MODE);
        cx.set_global(theme.as_ref().clone());
        // Custom Palette edits a fork, so seed changes never repaint the rest of the app.
        let draft = Arc::new(theme.fork());
        let palette_edits = PaletteEdits::startup();
        let seed_colors = palette_edits.colors(&theme);
        palette_edits.apply(&theme, &draft);

        let mode_selector = theme_mode::spawn("palette-mode", &draft, cx);
        let mut screen_builder = radix::Tabs::new("screen-nav-tabs")
            .look(&theme)
            .line()
            .fade_in(SCREEN_ENTER_DURATION)
            .active("page-custom-palette");
        for (screen, id, label) in SCREEN_TABS {
            let app = cx.entity().downgrade();
            screen_builder = screen_builder.tab_with(id, label, move |window, cx| {
                app.update(cx, |app, cx| app.render_screen(screen, window, cx))
                    .unwrap_or_else(|_| div().into_any_element())
            });
        }
        let screens = screen_builder.spawn(cx);
        let screen_nav = cx.new(|cx| ScreenNav::new(&theme, &draft, screens.clone(), cx));
        let home = custom_palette::PreviewControls::spawn(&draft, cx);
        let accent_color = seed_colors.accent;
        let gray_color = seed_colors.gray;
        let background_color = seed_colors.background;
        let accent_field = cx
            .new(|cx| ColorTextField::new(Arc::clone(&draft), "seed-accent", accent_color, cx))
            .help("Accent seed color. Click the swatch to pick a color or enter a hex value.", cx);
        let gray_field = cx
            .new(|cx| ColorTextField::new(Arc::clone(&draft), "seed-gray", gray_color, cx))
            .help("Gray seed color for neutral surfaces and text. Click the swatch or enter a hex value.", cx);
        let background_field = cx
            .new(|cx| ColorTextField::new(Arc::clone(&draft), "seed-background", background_color, cx))
            .help("Background seed color. Click the swatch to pick a color or enter a hex value.", cx);

        let mut subscriptions = Vec::new();
        // Screen presenters read app-owned state; refresh them when that state changes.
        subscriptions.push(cx.observe(&cx.entity(), {
            let screens = screens.downgrade();
            move |_, _, cx| {
                if let Some(screens) = screens.upgrade() {
                    screens.update(cx, |_, cx| cx.notify());
                }
            }
        }));
        for (index, field) in [(0, &accent_field), (1, &gray_field), (2, &background_field)] {
            subscriptions.push(cx.subscribe(field, move |this, _, event: &ColorTextFieldEvent, cx| {
                let ColorTextFieldEvent::Change { color } = event;
                this.palette_edits.set(this.draft.mode(), index, *color);
                this.palette_edits.apply(&this.theme, &this.draft);
                this.home.notify(cx);
                this.screen_nav.update(cx, |nav, cx| nav.theme_changed(cx));
                cx.notify();
            }));
        }

        let copy_menu = radix::PopupMenu::new("copy-palette")
            .look(&draft)
            .solid()
            .primary()
            .label("Copy")
            .items([
                MenuItem::new("copy-css").label("Copy as CSS"),
                MenuItem::new("copy-json").label("Copy as JSON"),
                MenuItem::new("copy-hex").label("Copy hex values"),
            ])
            .spawn(cx);

        let swatch_info = Arc::new(Mutex::new(None::<SwatchSelection>));
        let swatch_close = radix::Button::new("swatch-info-close").look(&draft).ghost().label("Close").spawn(cx);
        let swatch_overlay = draft
            .overlay_window("swatch-info")
            .mode(OverlayWindowMode::Modeless)
            .position(OverlayWindowPosition::Center)
            .content({
                let info = Arc::clone(&swatch_info);
                let close = swatch_close.clone();
                move |_, _, _| {
                    let summary = info
                        .lock()
                        .ok()
                        .and_then(|guard| guard.clone())
                        .map(|s| format!("{} · step {} · #{}", s.family, s.step, s.hex))
                        .unwrap_or_else(|| "No swatch selected".into());
                    vstack! {
                        gap=12;
                        div().text_lg().font_weight(gpui::FontWeight::SEMIBOLD).child("Scale step"),
                        div().text_sm().child(summary),
                        div().text_xs().child("Provenance: ScaleStep from look-radix stub"),
                        close.clone(),
                    }
                    .w_full()
                    .into_any_element()
                }
            })
            .theme_child(swatch_close.clone())
            .spawn(cx);

        subscriptions.push(cx.subscribe(&mode_selector, |this, _, event: &RadioGroupEvent, cx| {
            if let RadioGroupEvent::Change { value: Some(value) } = event {
                this.set_mode(
                    if value.as_ref() == "light" {
                        ThemeMode::Light
                    } else {
                        ThemeMode::Dark
                    },
                    cx,
                );
            }
        }));
        subscriptions.push(cx.subscribe(&screen_nav, |this, _, event: &ScreenNavEvent, cx| match event {
            ScreenNavEvent::Change { .. } => {
                this.page_scroll.set_offset(gpui::point(px(0.0), px(0.0)));
                cx.notify();
            }
            ScreenNavEvent::ModeChange { mode } => this.set_mode(*mode, cx),
            ScreenNavEvent::ResetTheme => this.reset_theme(cx),
        }));
        subscriptions.push(cx.subscribe(&swatch_close, |this, _, event: &ButtonEvent, cx| {
            if matches!(event, ButtonEvent::Click) {
                this.swatch_overlay.update(cx, |overlay, cx| overlay.dismiss(cx));
            }
        }));
        let shadow_editor = cx.new(|cx| ClassicShadowEditor::new(&theme, cx));
        subscriptions.push(cx.subscribe(&shadow_editor, |_this, _, event: &ClassicShadowEditorEvent, cx| {
            let ClassicShadowEditorEvent::Changed = event;
            cx.notify();
        }));

        let preview_layout = spawn_wide_middle(custom_palette::preview_layout_defaults(), cx);

        Self {
            focus_scope,
            theme,
            draft,
            theme_accent,
            theme_gray,
            palette_edits,
            screen_nav,
            page_scroll: ScrollHandle::new(),
            screens,
            mode_selector,
            accent_field,
            gray_field,
            background_field,
            copy_menu,
            home,
            style_guide: None,
            developer: None,
            shadow_editor,
            swatch_info,
            swatch_overlay,
            preview_layout,
            signup_mesh_cache: MeshCache::default(),
            signup_mesh_task: None,
            _subscriptions: subscriptions,
        }
    }

    fn active_tab(&self, cx: &App) -> RadixStudioTab {
        self.screen_nav.read(cx).active()
    }

    fn set_mode(&mut self, mode: ThemeMode, cx: &mut Context<Self>) {
        self.theme.set_mode(mode);
        self.draft.set_mode(mode);
        self.draft.set_palettes(self.theme_accent, self.theme_gray);
        self.palette_edits.apply(&self.theme, &self.draft);
        self.mode_selector.update(cx, |group, cx| group.set_selected(theme_mode::mode_id(mode), cx));
        self.screen_nav.update(cx, |nav, cx| nav.theme_changed(cx));
        self.sync_seed_fields(cx);
        cx.notify();
    }

    /// Restores both looks and the theme editors to their startup values.
    fn reset_theme(&mut self, cx: &mut Context<Self>) {
        self.palette_edits = PaletteEdits::startup();
        self.theme.set_palettes(self.theme_accent, self.theme_gray);
        self.draft.set_palettes(self.theme_accent, self.theme_gray);
        self.draft.set_classic_params(Default::default());
        self.shadow_editor.update(cx, |editor, cx| editor.reset(cx));
        self.set_mode(STARTUP_MODE, cx);
    }

    /// Prepaint supplies dimensions only; all raster work runs off the UI thread.
    pub fn request_signup_mesh(&mut self, width: u32, height: u32, cx: &mut Context<Self>) {
        let Some(mut request) = self.signup_mesh_cache.request(MeshRequest::for_look(&self.draft, width, height))
        else {
            return;
        };
        self.signup_mesh_task = Some(cx.spawn(async move |this, cx| {
            loop {
                let image = cx.background_executor().spawn(async move { request.render() }).await;
                let next = this.update(cx, |this, cx| {
                    let next = this.signup_mesh_cache.complete(request.key, image);
                    if next.is_none() {
                        this.signup_mesh_task = None;
                        cx.notify();
                    }
                    next
                });
                match next {
                    Ok(Some(latest)) => request = latest,
                    _ => break,
                }
            }
        }));
    }

    fn cached_signup_mesh(&self) -> Option<Arc<RenderImage>> {
        self.signup_mesh_cache.image()
    }

    fn sync_seed_fields(&self, cx: &mut Context<Self>) {
        self.home.notify(cx);
        let colors = self.palette_edits.colors(&self.theme);
        self.accent_field.update(cx, |field, cx| field.set_color(colors.accent, cx));
        self.gray_field.update(cx, |field, cx| field.set_color(colors.gray, cx));
        self.background_field.update(cx, |field, cx| field.set_color(colors.background, cx));
    }

    pub fn open_swatch_info(&mut self, family: ScaleFamily, step: u8, cx: &mut Context<Self>) {
        let color = self.draft.resolve_step(family, step).hsla();
        if let Ok(mut guard) = self.swatch_info.lock() {
            *guard = Some(SwatchSelection { family: self.draft.palette_label(family), step, hex: format_hex(color) });
        }
        let focus = self.focus_scope.clone();
        self.swatch_overlay.update(cx, |overlay, cx| {
            overlay.open_from(Some(focus), cx);
            cx.notify();
        });
        cx.notify();
    }

    fn surface(&self) -> Hsla {
        self.theme.resolve_role(SemanticRole::Surface).hsla()
    }

    fn border(&self) -> Hsla {
        self.theme.resolve_role(SemanticRole::Border).hsla()
    }

    fn fg(&self) -> Hsla {
        self.theme.resolve_role(SemanticRole::Foreground).hsla()
    }

    fn muted(&self) -> Hsla {
        self.theme.resolve_role(SemanticRole::MutedForeground).hsla()
    }

    /// Chrome colors for the Custom Palette screen, which follows the draft instead.
    fn draft_chrome(&self) -> DraftChrome {
        DraftChrome {
            surface: self.draft.resolve_role(SemanticRole::Surface).hsla(),
            border: self.draft.resolve_role(SemanticRole::Border).hsla(),
            fg: self.draft.resolve_role(SemanticRole::Foreground).hsla(),
            muted: self.draft.resolve_role(SemanticRole::MutedForeground).hsla(),
            accent: self.draft.resolve_role(SemanticRole::Primary).hsla(),
        }
    }
}

struct DraftChrome {
    surface: Hsla,
    border: Hsla,
    fg: Hsla,
    muted: Hsla,
    accent: Hsla,
}

impl RadixStudioApp {
    fn render_screen(&mut self, active_tab: RadixStudioTab, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let surface = self.surface();
        let fg = self.fg();
        let muted = self.muted();
        let draft_chrome = self.draft_chrome();
        let content = match active_tab {
            RadixStudioTab::CustomPalette => custom_palette::page(
                custom_palette::PageArgs {
                    look: &self.draft,
                    mode_selector: self.mode_selector.clone(),
                    accent_field: self.accent_field.clone(),
                    gray_field: self.gray_field.clone(),
                    background_field: self.background_field.clone(),
                    copy_menu: self.copy_menu.clone(),
                    preview_layout: self.preview_layout.clone(),
                    mesh_image: self.cached_signup_mesh(),
                    app: cx.entity(),
                    controls: self.home.clone(),
                    surface: draft_chrome.surface,
                    border: draft_chrome.border,
                    muted: draft_chrome.muted,
                    fg: draft_chrome.fg,
                    accent: draft_chrome.accent,
                },
                cx,
            ),
            RadixStudioTab::Colors => colors::page(&self.theme, fg, muted),
            RadixStudioTab::Icons => icons::page(fg, muted, surface),
            RadixStudioTab::StyleGuide => {
                let guide = self.style_guide.get_or_insert_with(|| style_guide::State::new(&self.theme, cx));
                guide.render(&self.theme, window, cx)
            }
            RadixStudioTab::Developer => self
                .developer
                .get_or_insert_with(|| developer::State::new(&self.theme, cx))
                .render(&self.theme, self.shadow_editor.clone(), cx),
        };
        let body = vstack! { gap=28; content };
        div()
            .id("radix-studio-scroll")
            .relative()
            .flex_1()
            .w_full()
            .min_h_0()
            .flex()
            .flex_col()
            .overflow_y_scroll()
            .track_scroll(&self.page_scroll)
            .on_scroll_wheel(cx.listener(|_, _, _, cx| cx.notify()))
            .when(active_tab == RadixStudioTab::Icons, |d| d.child(icons::hero_decoration(fg)))
            .child(body.relative().w_full().max_w(px(CONTENT_MAX_W)).mx_auto().px_8().pt(px(20.0)).pb_8())
            .into_any_element()
    }
}

impl Render for RadixStudioApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let surface = self.surface();
        let border = self.border();
        let fg = self.fg();
        let active_tab = self.active_tab(cx);
        // Fade with scroll distance so scrolling back to the top reverses smoothly.
        let progress = (f32::from(-self.page_scroll.offset().y) / CONTROL_BAR_FADE_DISTANCE).clamp(0.0, 1.0);
        let opacity = progress * progress * (3.0 - 2.0 * progress);
        let background = self.theme.resolve_role(SemanticRole::Background).hsla();
        let bar_background = Hsla { a: background.a * opacity, ..background };
        let bar_border = Hsla { a: border.a * opacity, ..border };
        let page_background = match active_tab {
            RadixStudioTab::Colors => colors::page_background(self.theme.mode()),
            // The draft owns this page, so its wash tracks the palette being edited.
            RadixStudioTab::CustomPalette => {
                let recipe = radix::PageBackground { settle_at: 0.5, ..Default::default() };
                let (start, _) = recipe.stops(&self.draft);
                let end = self.palette_edits.colors(&self.theme).background;
                gpui::linear_gradient(
                    180.0,
                    gpui::linear_color_stop(start, 0.0),
                    gpui::linear_color_stop(end, recipe.settle_at),
                )
            }
            RadixStudioTab::Icons | RadixStudioTab::StyleGuide | RadixStudioTab::Developer => {
                self.theme.page_background()
            }
        };

        let title_bar = TitleBar::new().background_color(surface).border_color(border).text_color(fg).child(
            div()
                .id("radix-studio-titlebar")
                .h_full()
                .w_full()
                .flex()
                .flex_row()
                .items_center()
                .px_3()
                .text_color(fg)
                .child("Radix Studio"),
        );

        dock_panel! {
            top: title_bar,
            fill: div()
                .size_full()
                .min_h_0()
                .flex()
                .flex_col()
                .child(
                    // Navigation and theme actions stay outside the page's scroll viewport.
                    div()
                        .id("radix-studio-control-bar")
                        .w_full()
                        .flex_none()
                        .px_8()
                        .py_2()
                        .border_b_1()
                        .border_color(bar_border)
                        .bg(bar_background)
                        .child(self.screen_nav.clone()),
                )
                .child(
                    div()
                        .flex_1()
                        .min_h_0()
                        .flex()
                        .flex_col()
                        .child(self.screens.read(cx).body()),
                )
                .child(self.swatch_overlay.clone()),
        }
        .into_element()
        .luma_focus_scope(&self.focus_scope)
        .size_full()
        .bg(page_background)
        .text_color(fg)
        .font_family("System UI")
    }
}

#[cfg(test)]
mod palette_edit_tests {
    use super::*;

    #[test]
    fn startup_and_reset_background_are_black() {
        let theme = Look::built_in();
        theme.set_mode(STARTUP_MODE);
        let edits = PaletteEdits::startup();
        assert_eq!(edits.colors(&theme).background, gpui::black());
        let draft = theme.fork();
        edits.apply(&theme, &draft);
        assert_eq!(draft.resolve_step(ScaleFamily::Gray, 1).hsla().l, 0.0);
        theme.set_mode(ThemeMode::Light);
        assert!(edits.colors(&theme).background.l > 0.9);
    }

    #[test]
    fn editor_inputs_regenerate_without_feedback_and_restore_per_mode() {
        let theme = Look::built_in();
        theme.set_mode(ThemeMode::Dark);
        let draft = theme.fork();
        let mut edits = PaletteEdits::default();
        let original = edits.colors(&theme);
        edits.set(ThemeMode::Dark, 0, gpui::rgb(0x941100).into());
        edits.apply(&theme, &draft);
        let inputs = edits.colors(&theme);
        assert_eq!(inputs.gray, original.gray);
        assert_eq!(inputs.background, original.background);
        assert_ne!(draft.resolve_step(ScaleFamily::Color, 9).hsla(), theme.resolve_step(ScaleFamily::Color, 9).hsla());
        let old_gray = draft.resolve_step(ScaleFamily::Gray, 8).hsla();
        edits.set(ThemeMode::Dark, 1, gpui::rgb(0x807060).into());
        edits.apply(&theme, &draft);
        assert_ne!(draft.resolve_step(ScaleFamily::Gray, 8).hsla(), old_gray);
        let old_background = draft.resolve_step(ScaleFamily::Color, 1).hsla();
        edits.set(ThemeMode::Dark, 2, gpui::black());
        edits.apply(&theme, &draft);
        assert_ne!(draft.resolve_step(ScaleFamily::Color, 1).hsla(), old_background);
        let dark_colors = (1..=12).map(|step| draft.resolve_step(ScaleFamily::Color, step).hsla()).collect::<Vec<_>>();
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            theme.set_mode(mode);
            draft.set_mode(mode);
            draft.set_palettes(Accent::Indigo, Gray::Auto);
            edits.apply(&theme, &draft);
        }
        assert_eq!(
            (1..=12).map(|step| draft.resolve_step(ScaleFamily::Color, step).hsla()).collect::<Vec<_>>(),
            dark_colors
        );
        edits = PaletteEdits::default();
        draft.set_palettes(Accent::Indigo, Gray::Auto);
        edits.apply(&theme, &draft);
        assert_eq!(draft.resolve_step(ScaleFamily::Color, 9).hsla(), theme.resolve_step(ScaleFamily::Color, 9).hsla());
    }

    #[test]
    fn all_three_inputs_survive_mode_changes_and_reset_clears_both_modes() {
        let mut edits = PaletteEdits::default();
        let dark = [gpui::rgb(0xe85454).into(), gpui::rgb(0x555555).into(), gpui::rgb(0x000000).into()];
        let light = [gpui::rgb(0x3264ff).into(), gpui::rgb(0xaaaaaa).into(), gpui::rgb(0xffffff).into()];
        for (index, color) in dark.into_iter().enumerate() {
            edits.set(ThemeMode::Dark, index, color);
        }
        assert_eq!(*edits.for_mode(ThemeMode::Light), [None; 3]);
        for (index, color) in light.into_iter().enumerate() {
            edits.set(ThemeMode::Light, index, color);
        }
        assert_eq!(*edits.for_mode(ThemeMode::Dark), dark.map(Some));
        assert_eq!(*edits.for_mode(ThemeMode::Light), light.map(Some));
        edits = PaletteEdits::default();
        assert_eq!(*edits.for_mode(ThemeMode::Dark), [None; 3]);
        assert_eq!(*edits.for_mode(ThemeMode::Light), [None; 3]);
    }
}

#[cfg(all(test, feature = "test-support"))]
mod screen_content_tests {
    use super::*;
    use gpui::TestAppContext;

    #[test]
    fn main_screen_slots_render_and_follow_programmatic_selection() {
        let mut app = TestAppContext::single();
        let (studio, cx) = app.add_window_view(RadixStudioApp::new);
        cx.simulate_resize(gpui::size(px(900.0), px(700.0)));
        cx.run_until_parked();
        let screens = cx.update(|_, cx| studio.read(cx).screens.clone());
        for (screen, id, _) in SCREEN_TABS {
            cx.update(|_, cx| screens.update(cx, |tabs, cx| tabs.set_active(id, cx)));
            cx.run_until_parked();
            cx.update(|_, cx| {
                let studio = studio.read(cx);
                assert_eq!(studio.active_tab(cx), screen);
                assert_eq!(studio.page_scroll.offset(), gpui::point(px(0.0), px(0.0)));
                if screen == RadixStudioTab::StyleGuide {
                    assert!(studio.style_guide.is_some());
                }
                if screen == RadixStudioTab::Developer {
                    assert!(studio.developer.is_some());
                }
            });
        }
    }
}
