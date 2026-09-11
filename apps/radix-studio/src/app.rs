//! Radix Studio app shell — shared state, page routing, and window chrome.

use std::sync::{Arc, Mutex};

use gpui::{App, Context, Entity, FocusHandle, Hsla, Render, RenderImage, Subscription, Window, div, prelude::*, px};
use luma::controls::button::{Button, ButtonEvent};
use luma::controls::checkbox::{Checkbox, CheckboxEvent};
use luma::controls::overlay_window::{OverlayWindow, OverlayWindowMode, OverlayWindowPosition};
use luma::controls::popup_menu::PopupMenu;
use luma::controls::switch::Switch;
use luma::controls::tabs::{Tabs, TabsEvent, TabsItem, TabsWidthMode};
use luma::controls::textfield::TextField;
use luma::controls::toggle::{Toggle, ToggleEvent};
use luma::focus::LumaFocusScopeExt;
use luma::infra::menu_item::MenuItem;
use luma::infra::presenter::HasPresenter;
use luma::shell::TitleBar;
use luma::theme::ThemeMode;
use luma::{WideMiddle, dock_panel, spawn_wide_middle, vstack};
use luma_look_radix::{
    RadixAccent, RadixGray, RadixLook, RadixLookControlExt, ScaleFamily, SemanticRole, SignupMeshCacheKey,
    rasterize_signup_mesh_for_look,
};

use crate::color_hex::format_hex;
use crate::controls::{
    ClassicShadowEditor, ClassicShadowEditorEvent, ColorTextField, ColorTextFieldEvent, ScreenNav, ScreenNavEvent,
};
use crate::screens::{colors, custom_palette, developer, icons, style_guide};
use crate::tabs::RadixStudioTab;

const CONTENT_MAX_W: f32 = 1280.0;

#[derive(Clone, Debug)]
struct SwatchSelection {
    family: &'static str,
    step: u8,
    hex: String,
}

pub struct RadixStudioApp {
    focus_scope: FocusHandle,
    /// Named palettes every screen but Custom Palette paints from.
    theme: Arc<RadixLook>,
    /// Forked look the Custom Palette screen edits in isolation.
    draft: Arc<RadixLook>,
    theme_accent: RadixAccent,
    theme_gray: RadixGray,
    screen_nav: Entity<ScreenNav>,
    light_toggle: Toggle,
    dark_toggle: Toggle,
    accent_field: Entity<ColorTextField>,
    gray_field: Entity<ColorTextField>,
    background_field: Entity<ColorTextField>,
    copy_menu: Entity<PopupMenu>,
    palette_reset: Entity<Button>,
    search_field: TextField,
    search_submit: Entity<Button>,
    sign_up_name: TextField,
    sign_up_email: TextField,
    sign_up_password: TextField,
    create_account: Entity<Button>,
    continue_github: Entity<Button>,
    soft_demo: Entity<Button>,
    outline_demo: Entity<Button>,
    ghost_demo: Entity<Button>,
    preview_switch: Switch,
    task_a: Checkbox,
    task_b: Checkbox,
    task_c: Checkbox,
    shadow_editor: Entity<ClassicShadowEditor>,
    buttons_preview_tabs: Option<Entity<Tabs>>,
    checkboxes_preview_tabs: Option<Entity<Tabs>>,
    radios_preview_tabs: Option<Entity<Tabs>>,
    switches_preview_tabs: Option<Entity<Tabs>>,
    textfields_preview_tabs: Option<Entity<Tabs>>,
    textareas_preview_tabs: Option<Entity<Tabs>>,
    sliders_preview_tabs: Option<Entity<Tabs>>,
    swatch_info: Arc<Mutex<Option<SwatchSelection>>>,
    swatch_overlay: OverlayWindow,
    preview_layout: Entity<WideMiddle>,
    signup_mesh_cache: Option<(SignupMeshCacheKey, Arc<RenderImage>)>,
    _subscriptions: Vec<Subscription>,
}

impl RadixStudioApp {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_scope = cx.focus_handle();
        let theme_accent = RadixAccent::Indigo;
        let theme_gray = RadixGray::Auto;
        let theme = Arc::new(RadixLook::built_in());
        theme.set_mode(ThemeMode::Dark);
        // Custom Palette edits a fork, so seed changes never repaint the rest of the app.
        let draft = Arc::new(theme.fork());

        let light_toggle = draft.toggle("mode-light").with_data(false).label("Light").spawn(cx);
        let dark_toggle = draft.toggle("mode-dark").with_data(true).label("Dark").spawn(cx);
        let screen_nav = cx.new(|cx| ScreenNav::new(&theme, cx));

        let accent_color = draft.resolve_role(SemanticRole::Primary).hsla();
        let gray_color = draft.resolve_step(ScaleFamily::Gray, 8).hsla();
        let background_color = draft.resolve_role(SemanticRole::Background).hsla();
        let accent_field = cx.new(|cx| {
            ColorTextField::new(Arc::clone(&draft), "seed-accent", accent_color, format_hex(accent_color), cx)
        });
        let gray_field =
            cx.new(|cx| ColorTextField::new(Arc::clone(&draft), "seed-gray", gray_color, format_hex(gray_color), cx));
        let background_field = cx.new(|cx| {
            ColorTextField::new(
                Arc::clone(&draft),
                "seed-background",
                background_color,
                format_hex(background_color),
                cx,
            )
        });

        let mut subscriptions = Vec::new();
        subscriptions.push(cx.subscribe(&accent_field, |this, _, event: &ColorTextFieldEvent, cx| {
            let ColorTextFieldEvent::Change { color } = event;
            this.draft.set_accent_seed(*color);
            this.signup_mesh_cache = None;
            this.sync_seed_fields(cx);
            cx.notify();
        }));

        let palette_reset = draft.ghost_button("palette-reset").label("Reset to theme").spawn(cx);
        subscriptions.push(cx.subscribe(&palette_reset, |this, _, event: &ButtonEvent, cx| {
            if matches!(event, ButtonEvent::Click) {
                this.reset_draft(cx);
            }
        }));

        let copy_menu = draft
            .solid_popup_menu("copy-palette")
            .label("Copy")
            .items([
                MenuItem::new("copy-css").label("Copy as CSS"),
                MenuItem::new("copy-json").label("Copy as JSON"),
                MenuItem::new("copy-hex").label("Copy hex values"),
            ])
            .spawn(cx);

        let search_field = draft.textfield("preview-search").placeholder("Search…").spawn(cx);
        let search_submit = draft.solid_button("preview-search-submit").label("Submit").spawn(cx);

        let sign_up_name = draft.textfield("signup-name").placeholder("Full name").spawn(cx);
        let sign_up_email = draft.textfield("signup-email").placeholder("Email").spawn(cx);
        let sign_up_password = draft.textfield("signup-password").placeholder("Password").spawn(cx);
        let create_account = draft.solid_button("signup-create").label("Create account").spawn(cx);
        let continue_github = draft.outline_button("signup-github").label("Continue with GitHub").spawn(cx);
        let soft_demo = draft.soft_button("demo-soft").label("Soft").spawn(cx);
        let outline_demo = draft.outline_button("demo-outline").label("Outline").spawn(cx);
        let ghost_demo = draft.ghost_button("demo-ghost").label("Ghost").spawn(cx);

        let preview_switch = draft.switch("preview-switch").with_data(true).label("Notifications").spawn(cx);
        let task_a = draft.checkbox("task-a").with_data(false).label("Respond to comment").spawn(cx);
        let task_b = draft.checkbox("task-b").with_data(true).label("Close Q2 finances").spawn(cx);
        let task_c = draft.checkbox("task-c").with_data(true).label("Review invoice #3456").spawn(cx);

        let swatch_info = Arc::new(Mutex::new(None::<SwatchSelection>));
        let swatch_close = draft.ghost_button("swatch-info-close").label("Close").spawn(cx);
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

        subscriptions.push(cx.subscribe(&light_toggle, |this, _, event: &ToggleEvent, cx| {
            if let ToggleEvent::Change { selected: true } = event {
                this.set_mode(ThemeMode::Light, cx);
            }
        }));
        subscriptions.push(cx.subscribe(&dark_toggle, |this, _, event: &ToggleEvent, cx| {
            if let ToggleEvent::Change { selected: true } = event {
                this.set_mode(ThemeMode::Dark, cx);
            }
        }));
        subscriptions.push(cx.subscribe(&screen_nav, |this, _, event: &ScreenNavEvent, cx| match event {
            ScreenNavEvent::Change { .. } => cx.notify(),
            ScreenNavEvent::ModeChange { mode } => this.set_mode(*mode, cx),
        }));
        subscriptions.push(cx.subscribe(&swatch_close, |this, _, event: &ButtonEvent, cx| {
            if matches!(event, ButtonEvent::Click) {
                this.swatch_overlay.update(cx, |overlay, cx| overlay.dismiss(cx));
            }
        }));
        for box_entity in [&task_a, &task_b, &task_c] {
            subscriptions.push(cx.subscribe(box_entity, |_this, _, _event: &CheckboxEvent, _cx| {}));
        }

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
            screen_nav,
            light_toggle,
            dark_toggle,
            accent_field,
            gray_field,
            background_field,
            copy_menu,
            palette_reset,
            search_field,
            search_submit,
            sign_up_name,
            sign_up_email,
            sign_up_password,
            create_account,
            continue_github,
            soft_demo,
            outline_demo,
            ghost_demo,
            preview_switch,
            task_a,
            task_b,
            task_c,
            shadow_editor,
            buttons_preview_tabs: None,
            checkboxes_preview_tabs: None,
            radios_preview_tabs: None,
            switches_preview_tabs: None,
            textfields_preview_tabs: None,
            textareas_preview_tabs: None,
            sliders_preview_tabs: None,
            swatch_info,
            swatch_overlay,
            preview_layout,
            signup_mesh_cache: None,
            _subscriptions: subscriptions,
        }
    }

    fn active_tab(&self, cx: &App) -> RadixStudioTab {
        self.screen_nav.read(cx).active()
    }

    fn buttons_preview_tabs(&mut self, cx: &mut Context<Self>) -> Entity<Tabs> {
        if let Some(tabs) = self.buttons_preview_tabs.clone() {
            return tabs;
        }

        let tabs = self
            .theme
            .tabs("radix-studio-buttons-preview-tabs")
            .items([
                TabsItem::new("template-preview").label("Template Preview"),
                TabsItem::new("colors").label("Colors"),
                TabsItem::new("all-sizes").label("All Sizes"),
            ])
            .active("template-preview")
            .width_mode(TabsWidthMode::Intrinsic)
            .spawn(cx);

        self._subscriptions.push(cx.subscribe(&tabs, |_, _, _: &TabsEvent, cx| {
            cx.notify();
        }));

        self.buttons_preview_tabs = Some(tabs.clone());
        tabs
    }

    fn checkboxes_preview_tabs(&mut self, cx: &mut Context<Self>) -> Entity<Tabs> {
        if let Some(tabs) = self.checkboxes_preview_tabs.clone() {
            return tabs;
        }

        let tabs = self
            .theme
            .tabs("radix-studio-checkboxes-preview-tabs")
            .items([
                TabsItem::new("template-preview").label("Template Preview"),
                TabsItem::new("colors").label("Colors"),
                TabsItem::new("all-sizes").label("All Sizes"),
            ])
            .active("template-preview")
            .width_mode(TabsWidthMode::Intrinsic)
            .spawn(cx);

        self._subscriptions.push(cx.subscribe(&tabs, |_, _, _: &TabsEvent, cx| {
            cx.notify();
        }));

        self.checkboxes_preview_tabs = Some(tabs.clone());
        tabs
    }

    fn radios_preview_tabs(&mut self, cx: &mut Context<Self>) -> Entity<Tabs> {
        if let Some(tabs) = self.radios_preview_tabs.clone() {
            return tabs;
        }

        let tabs = self
            .theme
            .tabs("radix-studio-radios-preview-tabs")
            .items([
                TabsItem::new("template-preview").label("Template Preview"),
                TabsItem::new("colors").label("Colors"),
                TabsItem::new("all-sizes").label("All Sizes"),
            ])
            .active("template-preview")
            .width_mode(TabsWidthMode::Intrinsic)
            .spawn(cx);

        self._subscriptions.push(cx.subscribe(&tabs, |_, _, _: &TabsEvent, cx| {
            cx.notify();
        }));

        self.radios_preview_tabs = Some(tabs.clone());
        tabs
    }

    fn switches_preview_tabs(&mut self, cx: &mut Context<Self>) -> Entity<Tabs> {
        if let Some(tabs) = self.switches_preview_tabs.clone() {
            return tabs;
        }

        let tabs = self
            .theme
            .tabs("radix-studio-switches-preview-tabs")
            .items([
                TabsItem::new("template-preview").label("Template Preview"),
                TabsItem::new("colors").label("Colors"),
                TabsItem::new("all-sizes").label("All Sizes"),
            ])
            .active("template-preview")
            .width_mode(TabsWidthMode::Intrinsic)
            .spawn(cx);

        self._subscriptions.push(cx.subscribe(&tabs, |_, _, _: &TabsEvent, cx| {
            cx.notify();
        }));

        self.switches_preview_tabs = Some(tabs.clone());
        tabs
    }

    fn textfields_preview_tabs(&mut self, cx: &mut Context<Self>) -> Entity<Tabs> {
        if let Some(tabs) = self.textfields_preview_tabs.clone() {
            return tabs;
        }

        let tabs = self
            .theme
            .tabs("radix-studio-textfields-preview-tabs")
            .items([
                TabsItem::new("template-preview").label("Template Preview"),
                TabsItem::new("colors").label("Colors"),
                TabsItem::new("all-sizes").label("All Sizes"),
            ])
            .active("template-preview")
            .width_mode(TabsWidthMode::Intrinsic)
            .spawn(cx);

        self._subscriptions.push(cx.subscribe(&tabs, |_, _, _: &TabsEvent, cx| {
            cx.notify();
        }));

        self.textfields_preview_tabs = Some(tabs.clone());
        tabs
    }

    fn textareas_preview_tabs(&mut self, cx: &mut Context<Self>) -> Entity<Tabs> {
        if let Some(tabs) = self.textareas_preview_tabs.clone() {
            return tabs;
        }

        let tabs = self
            .theme
            .tabs("radix-studio-textareas-preview-tabs")
            .items([
                TabsItem::new("template-preview").label("Template Preview"),
                TabsItem::new("colors").label("Colors"),
                TabsItem::new("all-sizes").label("All Sizes"),
            ])
            .active("template-preview")
            .width_mode(TabsWidthMode::Intrinsic)
            .spawn(cx);

        self._subscriptions.push(cx.subscribe(&tabs, |_, _, _: &TabsEvent, cx| {
            cx.notify();
        }));

        self.textareas_preview_tabs = Some(tabs.clone());
        tabs
    }

    fn sliders_preview_tabs(&mut self, cx: &mut Context<Self>) -> Entity<Tabs> {
        if let Some(tabs) = self.sliders_preview_tabs.clone() {
            return tabs;
        }

        let tabs = self
            .theme
            .tabs("radix-studio-sliders-preview-tabs")
            .items([
                TabsItem::new("template-preview").label("Template Preview"),
                TabsItem::new("colors").label("Colors"),
                TabsItem::new("all-sizes").label("All Sizes"),
            ])
            .active("template-preview")
            .width_mode(TabsWidthMode::Intrinsic)
            .spawn(cx);

        self._subscriptions.push(cx.subscribe(&tabs, |_, _, _: &TabsEvent, cx| {
            cx.notify();
        }));

        self.sliders_preview_tabs = Some(tabs.clone());
        tabs
    }

    fn set_mode(&mut self, mode: ThemeMode, cx: &mut Context<Self>) {
        self.theme.set_mode(mode);
        self.draft.set_mode(mode);
        self.signup_mesh_cache = None;
        let light_on = matches!(mode, ThemeMode::Light);
        self.light_toggle.update(cx, |toggle, cx| toggle.set_data(light_on, cx));
        self.dark_toggle.update(cx, |toggle, cx| toggle.set_data(!light_on, cx));
        self.screen_nav.update(cx, |nav, cx| nav.mode_changed(cx));
        self.sync_seed_fields(cx);
        cx.notify();
    }

    /// Restores the draft to the app's named palettes, discarding seed edits.
    fn reset_draft(&mut self, cx: &mut Context<Self>) {
        self.draft.set_palettes(self.theme_accent, self.theme_gray);
        self.signup_mesh_cache = None;
        self.sync_seed_fields(cx);
        cx.notify();
    }

    pub fn ensure_signup_mesh_cache(&mut self, width: u32, height: u32) -> bool {
        if width == 0 || height == 0 {
            return false;
        }
        let key = SignupMeshCacheKey::for_look(&self.draft, width, height);
        if self.signup_mesh_cache.as_ref().is_some_and(|(cached, _)| *cached == key) {
            return false;
        }
        let Some(image) = rasterize_signup_mesh_for_look(&self.draft, width, height) else {
            return false;
        };
        self.signup_mesh_cache = Some((key, image));
        true
    }

    fn cached_signup_mesh(&self) -> Option<Arc<RenderImage>> {
        self.signup_mesh_cache.as_ref().map(|(_, image)| Arc::clone(image))
    }

    fn sync_seed_fields(&self, cx: &mut Context<Self>) {
        let accent = format_hex(self.draft.resolve_role(SemanticRole::Primary).hsla());
        let gray = format_hex(self.draft.resolve_step(ScaleFamily::Gray, 8).hsla());
        let background = format_hex(self.draft.resolve_role(SemanticRole::Background).hsla());
        self.accent_field.update(cx, |field, cx| {
            field.set_value(accent.clone(), cx);
            field.set_color(self.draft.resolve_role(SemanticRole::Primary).hsla(), cx);
        });
        self.gray_field.update(cx, |field, cx| {
            field.set_value(gray.clone(), cx);
            field.set_color(self.draft.resolve_step(ScaleFamily::Gray, 8).hsla(), cx);
        });
        self.background_field.update(cx, |field, cx| {
            field.set_value(background.clone(), cx);
            field.set_color(self.draft.resolve_role(SemanticRole::Background).hsla(), cx);
        });
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

impl Render for RadixStudioApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let surface = self.surface();
        let border = self.border();
        let fg = self.fg();
        let muted = self.muted();
        let is_dark = matches!(self.theme.mode(), ThemeMode::Dark);
        let draft_chrome = self.draft_chrome();
        let active_tab = self.active_tab(cx);
        let page_background = match active_tab {
            RadixStudioTab::Colors => colors::page_background(self.theme.mode()),
            // The draft owns this page, so its wash tracks the palette being edited.
            RadixStudioTab::CustomPalette => self.draft.page_background(),
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
                    .id("radix-studio-scroll")
                    .relative()
                    .size_full()
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .overflow_y_scroll()
                    .when(active_tab == RadixStudioTab::Icons, |d| {
                        d.child(icons::hero_decoration(fg))
                    })
                    .child(
                        // Full-window width so the theme toggle sits at the far right edge,
                        // outside the max-width content column below.
                        div()
                            .relative()
                            .w_full()
                            .px_8()
                            .pt_2()
                            .pb(px(28.0))
                            .child(self.screen_nav.clone()),
                    )
                    .child(
                        vstack! {
                            gap=28;
                            match active_tab {
                                RadixStudioTab::CustomPalette => custom_palette::page(
                                    custom_palette::PageArgs {
                                        look: &self.draft,
                                        light_toggle: self.light_toggle.clone(),
                                        dark_toggle: self.dark_toggle.clone(),
                                        accent_field: self.accent_field.clone(),
                                        gray_field: self.gray_field.clone(),
                                        background_field: self.background_field.clone(),
                                        copy_menu: self.copy_menu.clone(),
                                        palette_reset: self.palette_reset.clone(),
                                        preview_layout: self.preview_layout.clone(),
                                        mesh_image: self.cached_signup_mesh(),
                                        app: cx.entity(),
                                        controls: custom_palette::PreviewControls {
                                            search_field: self.search_field.clone(),
                                            search_submit: self.search_submit.clone(),
                                            sign_up_name: self.sign_up_name.clone(),
                                            sign_up_email: self.sign_up_email.clone(),
                                            sign_up_password: self.sign_up_password.clone(),
                                            create_account: self.create_account.clone(),
                                            continue_github: self.continue_github.clone(),
                                            soft_demo: self.soft_demo.clone(),
                                            outline_demo: self.outline_demo.clone(),
                                            ghost_demo: self.ghost_demo.clone(),
                                            preview_switch: self.preview_switch.clone(),
                                            task_a: self.task_a.clone(),
                                            task_b: self.task_b.clone(),
                                            task_c: self.task_c.clone(),
                                        },
                                        surface: draft_chrome.surface,
                                        border: draft_chrome.border,
                                        muted: draft_chrome.muted,
                                        fg: draft_chrome.fg,
                                        accent: draft_chrome.accent,
                                        is_dark,
                                    },
                                    cx,
                                ),
                                RadixStudioTab::Colors => colors::page(&self.theme, fg, muted),
                                RadixStudioTab::Icons => icons::page(fg, muted, surface),
                                RadixStudioTab::StyleGuide => {
                                    let tabs = style_guide::PreviewTabs {
                                        buttons: self.buttons_preview_tabs(cx),
                                        checkboxes: self.checkboxes_preview_tabs(cx),
                                        radios: self.radios_preview_tabs(cx),
                                        switches: self.switches_preview_tabs(cx),
                                        textfields: self.textfields_preview_tabs(cx),
                                        textareas: self.textareas_preview_tabs(cx),
                                        sliders: self.sliders_preview_tabs(cx),
                                    };
                                    style_guide::page(&self.theme, tabs, window, cx)
                                }
                                RadixStudioTab::Developer => {
                                    developer::page(&self.theme, self.shadow_editor.clone())
                                }
                            },
                        }
                        .relative()
                        .w_full()
                        .max_w(px(CONTENT_MAX_W))
                    .mx_auto()
                    .px_8()
                    .pb_8(),
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
