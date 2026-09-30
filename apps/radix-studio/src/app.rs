//! Radix Studio app shell — shared state, page routing, and window chrome.

use std::sync::{Arc, Mutex};

use gpui::{App, Context, Entity, FocusHandle, Hsla, Render, RenderImage, Subscription, Window, div, prelude::*, px};
use luma::controls::button::{Button, ButtonEvent};
use luma::controls::overlay_window::{OverlayWindow, OverlayWindowMode, OverlayWindowPosition};
use luma::controls::popup_menu::PopupMenu;
use luma::controls::tabs::{Tabs, TabsEvent, TabsItem, TabsWidthMode};
use luma::controls::textfield::TextField;
use luma::controls::radio_group::{RadioGroup, RadioGroupItem, RadioGroupEvent};
use crate::controls::theme_mode;
use luma::controls::tree_view::TreeView;
use luma::controls::toolbar::Toolbar;
use luma::focus::LumaFocusScopeExt;
use luma::infra::menu_item::MenuItem;
use luma::infra::presenter::HasPresenter;
use luma::shell::TitleBar;
use luma::theme::ThemeMode;
use luma::{WideMiddle, dock_panel, spawn_wide_middle, vstack};
use luma_look_radix::{Accent, Gray, Look, LookControlExt, ScaleFamily, SemanticRole};
use luma_look_radix as radix;

use crate::color_hex::format_hex;
use crate::signup_mesh::{SignupMeshCacheKey, rasterize_signup_mesh_for_look};
use crate::controls::{
    ClassicShadowEditor, ClassicShadowEditorEvent, ColorTextField, ColorTextFieldEvent, ScreenNav, ScreenNavEvent,
};
use crate::screens::{colors, custom_palette, developer, icons, style_guide, tree_view};
use crate::tabs::RadixStudioTab;

const CONTENT_MAX_W: f32 = 1280.0;
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
    mode_selector: RadioGroup<RadioGroupItem>,
    accent_field: Entity<ColorTextField>,
    gray_field: Entity<ColorTextField>,
    background_field: Entity<ColorTextField>,
    copy_menu: Entity<PopupMenu>,
    search_field: TextField,
    search_submit: Entity<Button>,
    sign_up_name: TextField,
    sign_up_email: TextField,
    sign_up_password: TextField,
    create_account: Entity<Button>,
    continue_github: Entity<Button>,
    icon_samples: crate::screens::icon_samples::IconSamples,
    task_samples: crate::screens::task_samples::TaskSamples,
    preview_tree: TreeView<()>,
    preview_toolbar: Toolbar,
    preview_actions: Entity<PopupMenu>,
    preview_tabs: Entity<Tabs>,
    guide_tabs: style_guide::TabsExamples,
    guide_tree: TreeView<()>,
    guide_tree_disabled: TreeView<()>,
    shadow_editor: Entity<ClassicShadowEditor>,
    avatars_preview_tabs: Option<Entity<Tabs>>,
    badges_preview_tabs: Option<Entity<Tabs>>,
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
        let screen_nav = cx.new(|cx| ScreenNav::new(&theme, cx));
        let preview_tree = tree_view::spawn("palette-tree", &draft, true, cx);
        let (preview_toolbar, preview_actions) = crate::screens::preview_toolbar::spawn(&draft, cx);
        let preview_tabs = radix::Tabs::new("palette-preview-tabs")
            .look(&draft)
            .line()
            .with_template_modifier(|root, _| root.w_full())
            .items([
                TabsItem::new("themes").label("Themes"),
                TabsItem::new("primitives").label("Primitives"),
                TabsItem::new("icons").label("Icons"),
                TabsItem::new("colors").label("Colors"),
            ])
            .active("themes")
            .spawn(cx);
        let guide_tabs = style_guide::TabsExamples::spawn(&theme, cx);
        let guide_tree = tree_view::spawn("guide-tree", &theme, true, cx);
        let guide_tree_disabled = tree_view::spawn("guide-tree-disabled", &theme, false, cx);

        let accent_color = seed_colors.accent;
        let gray_color = seed_colors.gray;
        let background_color = seed_colors.background;
        let accent_field = cx.new(|cx| ColorTextField::new(Arc::clone(&draft), "seed-accent", accent_color, cx));
        let gray_field = cx.new(|cx| ColorTextField::new(Arc::clone(&draft), "seed-gray", gray_color, cx));
        let background_field =
            cx.new(|cx| ColorTextField::new(Arc::clone(&draft), "seed-background", background_color, cx));

        let mut subscriptions = Vec::new();
        for (index, field) in [(0, &accent_field), (1, &gray_field), (2, &background_field)] {
            subscriptions.push(cx.subscribe(field, move |this, _, event: &ColorTextFieldEvent, cx| {
                let ColorTextFieldEvent::Change { color } = event;
                this.palette_edits.set(this.draft.mode(), index, *color);
                this.palette_edits.apply(&this.theme, &this.draft);
                this.signup_mesh_cache = None;
                this.notify_preview_controls(cx);
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

        let search_field = radix::TextField::new("preview-search").look(&draft).placeholder("Search…").spawn(cx);
        let search_submit = radix::Button::new("preview-search-submit").look(&draft).solid().label("Submit").spawn(cx);

        let sign_up_name = radix::TextField::new("signup-name").look(&draft).placeholder("Full name").spawn(cx);
        let sign_up_email = radix::TextField::new("signup-email").look(&draft).placeholder("Email").spawn(cx);
        let sign_up_password = radix::TextField::new("signup-password").look(&draft).placeholder("Password").spawn(cx);
        let create_account = radix::Button::new("signup-create").look(&draft).solid().label("Create account").spawn(cx);
        let continue_github = radix::Button::new("signup-github")
            .look(&draft)
            .outline()
            .content(|model, _| {
                div()
                    .flex()
                    .items_center()
                    .gap(px(model.look.gap))
                    .child(
                        gpui::svg()
                            .path("assets/react-icons/github-logo.svg")
                            .size(px(model.look.icon_size))
                            .text_color(model.look.foreground),
                    )
                    .child("Continue with GitHub")
            })
            .spawn(cx);
        let icon_samples = crate::screens::icon_samples::IconSamples::spawn(&draft, cx);
        let task_samples = crate::screens::task_samples::TaskSamples::spawn(&draft, cx);

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
            ScreenNavEvent::Change { .. } => cx.notify(),
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
            mode_selector,
            accent_field,
            gray_field,
            background_field,
            copy_menu,
            search_field,
            search_submit,
            sign_up_name,
            sign_up_email,
            sign_up_password,
            create_account,
            continue_github,
            icon_samples,
            task_samples,
            preview_tree,
            preview_toolbar,
            preview_actions,
            preview_tabs,
            guide_tabs,
            guide_tree,
            guide_tree_disabled,
            shadow_editor,
            avatars_preview_tabs: None,
            badges_preview_tabs: None,
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

    fn avatars_preview_tabs(&mut self, cx: &mut Context<Self>) -> Entity<Tabs> {
        if let Some(tabs) = self.avatars_preview_tabs.clone() {
            return tabs;
        }

        let tabs = radix::Tabs::new("radix-studio-avatars-preview-tabs")
            .look(&self.theme)
            .items([
                TabsItem::new("template-preview").label("Template Preview"),
                TabsItem::new("colors").label("Colors"),
                TabsItem::new("all-sizes").label("All Sizes"),
            ])
            .active("template-preview")
            .width_mode(TabsWidthMode::Intrinsic)
            .with_template_modifier(|root, _| root.w_full())
            .spawn(cx);

        self._subscriptions.push(cx.subscribe(&tabs, |_, _, _: &TabsEvent, cx| {
            cx.notify();
        }));

        self.avatars_preview_tabs = Some(tabs.clone());
        tabs
    }

    fn badges_preview_tabs(&mut self, cx: &mut Context<Self>) -> Entity<Tabs> {
        if let Some(tabs) = self.badges_preview_tabs.clone() {
            return tabs;
        }

        let tabs = radix::Tabs::new("radix-studio-badges-preview-tabs")
            .look(&self.theme)
            .items([
                TabsItem::new("template-preview").label("Template Preview"),
                TabsItem::new("colors").label("Colors"),
                TabsItem::new("all-sizes").label("All Sizes"),
            ])
            .active("template-preview")
            .width_mode(TabsWidthMode::Intrinsic)
            .with_template_modifier(|root, _| root.w_full())
            .spawn(cx);

        self._subscriptions.push(cx.subscribe(&tabs, |_, _, _: &TabsEvent, cx| {
            cx.notify();
        }));

        self.badges_preview_tabs = Some(tabs.clone());
        tabs
    }

    fn buttons_preview_tabs(&mut self, cx: &mut Context<Self>) -> Entity<Tabs> {
        if let Some(tabs) = self.buttons_preview_tabs.clone() {
            return tabs;
        }

        let tabs = radix::Tabs::new("radix-studio-buttons-preview-tabs")
            .look(&self.theme)
            .items([
                TabsItem::new("template-preview").label("Template Preview"),
                TabsItem::new("colors").label("Colors"),
                TabsItem::new("all-sizes").label("All Sizes"),
            ])
            .active("template-preview")
            .width_mode(TabsWidthMode::Intrinsic)
            .with_template_modifier(|root, _| root.w_full())
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

        let tabs = radix::Tabs::new("radix-studio-checkboxes-preview-tabs")
            .look(&self.theme)
            .items([
                TabsItem::new("template-preview").label("Template Preview"),
                TabsItem::new("colors").label("Colors"),
                TabsItem::new("all-sizes").label("All Sizes"),
            ])
            .active("template-preview")
            .width_mode(TabsWidthMode::Intrinsic)
            .with_template_modifier(|root, _| root.w_full())
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

        let tabs = radix::Tabs::new("radix-studio-radios-preview-tabs")
            .look(&self.theme)
            .items([
                TabsItem::new("template-preview").label("Template Preview"),
                TabsItem::new("colors").label("Colors"),
                TabsItem::new("all-sizes").label("All Sizes"),
            ])
            .active("template-preview")
            .width_mode(TabsWidthMode::Intrinsic)
            .with_template_modifier(|root, _| root.w_full())
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

        let tabs = radix::Tabs::new("radix-studio-switches-preview-tabs")
            .look(&self.theme)
            .items([
                TabsItem::new("template-preview").label("Template Preview"),
                TabsItem::new("colors").label("Colors"),
                TabsItem::new("all-sizes").label("All Sizes"),
            ])
            .active("template-preview")
            .width_mode(TabsWidthMode::Intrinsic)
            .with_template_modifier(|root, _| root.w_full())
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

        let tabs = radix::Tabs::new("radix-studio-textfields-preview-tabs")
            .look(&self.theme)
            .items([
                TabsItem::new("template-preview").label("Template Preview"),
                TabsItem::new("colors").label("Colors"),
                TabsItem::new("all-sizes").label("All Sizes"),
            ])
            .active("template-preview")
            .width_mode(TabsWidthMode::Intrinsic)
            .with_template_modifier(|root, _| root.w_full())
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

        let tabs = radix::Tabs::new("radix-studio-textareas-preview-tabs")
            .look(&self.theme)
            .items([
                TabsItem::new("template-preview").label("Template Preview"),
                TabsItem::new("colors").label("Colors"),
                TabsItem::new("all-sizes").label("All Sizes"),
            ])
            .active("template-preview")
            .width_mode(TabsWidthMode::Intrinsic)
            .with_template_modifier(|root, _| root.w_full())
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

        let tabs = radix::Tabs::new("radix-studio-sliders-preview-tabs")
            .look(&self.theme)
            .items([
                TabsItem::new("template-preview").label("Template Preview"),
                TabsItem::new("colors").label("Colors"),
                TabsItem::new("all-sizes").label("All Sizes"),
            ])
            .active("template-preview")
            .width_mode(TabsWidthMode::Intrinsic)
            .with_template_modifier(|root, _| root.w_full())
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
        self.draft.set_palettes(self.theme_accent, self.theme_gray);
        self.palette_edits.apply(&self.theme, &self.draft);
        self.signup_mesh_cache = None;
        self.mode_selector.update(cx, |group, cx| group.set_selected(theme_mode::mode_id(mode), cx));
        self.screen_nav.update(cx, |nav, cx| nav.mode_changed(cx));
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

    fn notify_preview_controls(&self, cx: &mut Context<Self>) {
        self.icon_samples.notify(cx);
        self.task_samples.notify(cx);
        self.preview_toolbar.update(cx, |toolbar, cx| toolbar.notify_items(cx));
        self.preview_actions.update(cx, |_, cx| cx.notify());
        self.preview_tabs.update(cx, |_, cx| cx.notify());
    }

    fn sync_seed_fields(&self, cx: &mut Context<Self>) {
        self.notify_preview_controls(cx);
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

impl Render for RadixStudioApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let surface = self.surface();
        let border = self.border();
        let fg = self.fg();
        let muted = self.muted();
        let draft_chrome = self.draft_chrome();
        let active_tab = self.active_tab(cx);
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
                                        mode_selector: self.mode_selector.clone(),
                                        accent_field: self.accent_field.clone(),
                                        gray_field: self.gray_field.clone(),
                                        background_field: self.background_field.clone(),
                                        copy_menu: self.copy_menu.clone(),
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
                                            icon_samples: self.icon_samples.clone(),
                                            task_samples: self.task_samples.clone(),
                                            tree: self.preview_tree.clone(),
                                            toolbar: self.preview_toolbar.clone(),
                                            actions: self.preview_actions.clone(),
                                            tabs: self.preview_tabs.clone(),
                                        },
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
                                    let tabs = style_guide::PreviewTabs {
                                        avatars: self.avatars_preview_tabs(cx),
                                        badges: self.badges_preview_tabs(cx),
                                        examples: self.guide_tabs.clone(),
                                        buttons: self.buttons_preview_tabs(cx),
                                        checkboxes: self.checkboxes_preview_tabs(cx),
                                        radios: self.radios_preview_tabs(cx),
                                        switches: self.switches_preview_tabs(cx),
                                        textfields: self.textfields_preview_tabs(cx),
                                        textareas: self.textareas_preview_tabs(cx),
                                        sliders: self.sliders_preview_tabs(cx),
                                    };
                                    style_guide::page(&self.theme, tabs, self.guide_tree.clone(), self.guide_tree_disabled.clone(), window, cx)
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
