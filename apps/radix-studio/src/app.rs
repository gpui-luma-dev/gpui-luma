use std::sync::{Arc, Mutex};

use gpui::{
    App, Context, Entity, FocusHandle, Hsla, ImageSource, Render, RenderImage, SharedString, Subscription, Window, div,
    img, prelude::*, px,
};
use luma::controls::button::{Button, ButtonEvent};
use luma::controls::checkbox::{Checkbox, CheckboxEvent};
use luma::controls::overlay_window::{OverlayWindow, OverlayWindowMode, OverlayWindowPosition};
use luma::controls::popup_menu::PopupMenu;
use luma::controls::switch::Switch;
use luma::controls::tabs::{Tabs, TabsItem};
use luma::controls::textfield::TextField;
use luma::controls::toggle::{Toggle, ToggleEvent};
use luma::focus::LumaFocusScopeExt;
use luma::infra::ElementExt;
use luma::infra::menu_item::MenuItem;
use luma::infra::presenter::HasPresenter;
use luma::shell::TitleBar;
use luma::theme::ThemeMode;
use luma::{GridLayout, GridTrack, WideMiddle, WideMiddleLayout, dock_panel, hstack, spawn_wide_middle, vstack};
use luma_look_radix::{
    RadixLook, RadixLookControlExt, SCALE_LEN, ScaleFamily, SemanticRole, SignupMeshCacheKey, SignupStage,
    rasterize_signup_mesh_for_look,
};

use crate::color_hex::format_hex;
use crate::color_textfield::{ColorTextField, ColorTextFieldEvent};

/// Side columns stay compact; center is wider for the signup stage.
const PREVIEW_SIDE_COLUMN_MIN: f32 = 240.0;
const PREVIEW_SIDE_COLUMN_PREFERRED: f32 = 280.0;
/// Middle column hard cap in the three-column row (also signup glass width).
const PREVIEW_CENTER_COLUMN_MAX: f32 = 435.0;
const PREVIEW_COLUMN_GAP: f32 = 24.0;
const SIGNUP_MESH_MAX_SIDE: f32 = 960.0;
const SIGNUP_INNER_FORM_CARD_W: f32 = 340.0;
/// Form face alpha so mesh grads show through.
const SIGNUP_PANEL_ALPHA: f32 = 0.55;
const CONTENT_MAX_W: f32 = 1280.0;
#[derive(Clone, Debug)]
struct SwatchSelection {
    family: &'static str,
    step: u8,
    hex: String,
}

pub struct RadixStudioApp {
    focus_scope: FocusHandle,
    look: Arc<RadixLook>,
    light_toggle: Toggle,
    dark_toggle: Toggle,
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
    soft_demo: Entity<Button>,
    outline_demo: Entity<Button>,
    ghost_demo: Entity<Button>,
    preview_tabs: Entity<Tabs>,
    preview_switch: Switch,
    task_a: Checkbox,
    task_b: Checkbox,
    task_c: Checkbox,
    swatch_info: Arc<Mutex<Option<SwatchSelection>>>,
    swatch_overlay: OverlayWindow,
    preview_layout: Entity<WideMiddle>,
    signup_mesh_cache: Option<(SignupMeshCacheKey, Arc<RenderImage>)>,
    _subscriptions: Vec<Subscription>,
}

impl RadixStudioApp {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_scope = cx.focus_handle();
        let look = Arc::new(RadixLook::built_in());
        look.set_mode(ThemeMode::Dark);

        let light_toggle = look.toggle("mode-light").with_data(false).label("Light").spawn(cx);
        let dark_toggle = look.toggle("mode-dark").with_data(true).label("Dark").spawn(cx);

        let accent_color = look.resolve_role(SemanticRole::Primary).hsla();
        let gray_color = look.resolve_step(ScaleFamily::Gray, 8).hsla();
        let background_color = look.resolve_role(SemanticRole::Background).hsla();
        let accent_field = cx.new(|cx| {
            ColorTextField::new(Arc::clone(&look), "seed-accent", accent_color, format_hex(accent_color), cx)
        });
        let gray_field =
            cx.new(|cx| ColorTextField::new(Arc::clone(&look), "seed-gray", gray_color, format_hex(gray_color), cx));
        let background_field = cx.new(|cx| {
            ColorTextField::new(
                Arc::clone(&look),
                "seed-background",
                background_color,
                format_hex(background_color),
                cx,
            )
        });

        let mut subscriptions = Vec::new();
        subscriptions.push(cx.subscribe(&accent_field, |this, _, event: &ColorTextFieldEvent, cx| {
            let ColorTextFieldEvent::Change { color } = event;
            this.look.set_accent_seed(*color);
            this.sync_seed_fields(cx);
            cx.notify();
        }));

        let copy_menu = look
            .solid_popup_menu("copy-palette")
            .label("Copy")
            .items([
                MenuItem::new("copy-css").label("Copy as CSS"),
                MenuItem::new("copy-json").label("Copy as JSON"),
                MenuItem::new("copy-hex").label("Copy hex values"),
            ])
            .spawn(cx);

        let search_field = look.textfield("preview-search").placeholder("Search…").spawn(cx);
        let search_submit = look.solid_button("preview-search-submit").label("Submit").spawn(cx);

        let sign_up_name = look.textfield("signup-name").placeholder("Full name").spawn(cx);
        let sign_up_email = look.textfield("signup-email").placeholder("Email").spawn(cx);
        let sign_up_password = look.textfield("signup-password").placeholder("Password").spawn(cx);
        let create_account = look.solid_button("signup-create").label("Create account").spawn(cx);
        let continue_github = look.outline_button("signup-github").label("Continue with GitHub").spawn(cx);
        let soft_demo = look.soft_button("demo-soft").label("Soft").spawn(cx);
        let outline_demo = look.outline_button("demo-outline").label("Outline").spawn(cx);
        let ghost_demo = look.ghost_button("demo-ghost").label("Ghost").spawn(cx);

        let preview_tabs = look
            .tabs("preview-tabs")
            .items([
                TabsItem::new("themes").label("Themes"),
                TabsItem::new("primitives").label("Primitives"),
                TabsItem::new("icons").label("Icons"),
                TabsItem::new("colors").label("Colors"),
            ])
            .active("colors")
            .spawn(cx);

        let preview_switch = look.switch("preview-switch").with_data(true).label("Notifications").spawn(cx);
        let task_a = look.checkbox("task-a").with_data(false).label("Respond to comment").spawn(cx);
        let task_b = look.checkbox("task-b").with_data(true).label("Close Q2 finances").spawn(cx);
        let task_c = look.checkbox("task-c").with_data(true).label("Review invoice #3456").spawn(cx);

        let swatch_info = Arc::new(Mutex::new(None::<SwatchSelection>));
        let swatch_close = look.ghost_button("swatch-info-close").label("Close").spawn(cx);
        let swatch_overlay = look
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
        subscriptions.push(cx.subscribe(&swatch_close, |this, _, event: &ButtonEvent, cx| {
            if matches!(event, ButtonEvent::Click) {
                this.swatch_overlay.update(cx, |overlay, cx| overlay.dismiss(cx));
            }
        }));
        // Keep checkboxes interactive (owned state); no app model required.
        for box_entity in [&task_a, &task_b, &task_c] {
            subscriptions.push(cx.subscribe(box_entity, |_this, _, _event: &CheckboxEvent, _cx| {}));
        }

        let preview_layout = spawn_wide_middle(
            WideMiddle::new()
                .gap(PREVIEW_COLUMN_GAP)
                .side_min_width(PREVIEW_SIDE_COLUMN_MIN)
                .side_preferred_width(PREVIEW_SIDE_COLUMN_PREFERRED)
                .middle_min_width(PREVIEW_CENTER_COLUMN_MAX)
                .middle_max_width(PREVIEW_CENTER_COLUMN_MAX),
            cx,
        );

        Self {
            focus_scope,
            look,
            light_toggle,
            dark_toggle,
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
            soft_demo,
            outline_demo,
            ghost_demo,
            preview_tabs,
            preview_switch,
            task_a,
            task_b,
            task_c,
            swatch_info,
            swatch_overlay,
            preview_layout,
            signup_mesh_cache: None,
            _subscriptions: subscriptions,
        }
    }

    fn set_mode(&mut self, mode: ThemeMode, cx: &mut Context<Self>) {
        self.look.set_mode(mode);
        self.signup_mesh_cache = None;
        let light_on = matches!(mode, ThemeMode::Light);
        self.light_toggle.update(cx, |toggle, cx| toggle.set_data(light_on, cx));
        self.dark_toggle.update(cx, |toggle, cx| toggle.set_data(!light_on, cx));
        self.sync_seed_fields(cx);
        cx.notify();
    }

    fn ensure_signup_mesh_cache(&mut self, width: u32, height: u32) -> bool {
        if width == 0 || height == 0 {
            return false;
        }
        let key = SignupMeshCacheKey::for_look(&self.look, width, height);
        if self.signup_mesh_cache.as_ref().is_some_and(|(cached, _)| *cached == key) {
            return false;
        }
        let Some(image) = rasterize_signup_mesh_for_look(&self.look, width, height) else {
            return false;
        };
        self.signup_mesh_cache = Some((key, image));
        true
    }

    fn cached_signup_mesh(&self) -> Option<Arc<RenderImage>> {
        self.signup_mesh_cache.as_ref().map(|(_, image)| Arc::clone(image))
    }
    fn sync_seed_fields(&self, cx: &mut Context<Self>) {
        let accent = format_hex(self.look.resolve_role(SemanticRole::Primary).hsla());
        let gray = format_hex(self.look.resolve_step(ScaleFamily::Gray, 8).hsla());
        let background = format_hex(self.look.resolve_role(SemanticRole::Background).hsla());
        self.accent_field.update(cx, |field, cx| {
            field.set_value(accent.clone(), cx);
            field.set_color(self.look.resolve_role(SemanticRole::Primary).hsla(), cx);
        });
        self.gray_field.update(cx, |field, cx| {
            field.set_value(gray.clone(), cx);
            field.set_color(self.look.resolve_step(ScaleFamily::Gray, 8).hsla(), cx);
        });
        self.background_field.update(cx, |field, cx| {
            field.set_value(background.clone(), cx);
            field.set_color(self.look.resolve_role(SemanticRole::Background).hsla(), cx);
        });
    }

    fn open_swatch_info(&mut self, family: ScaleFamily, step: u8, cx: &mut Context<Self>) {
        let color = self.look.resolve_step(family, step).hsla();
        if let Ok(mut guard) = self.swatch_info.lock() {
            *guard = Some(SwatchSelection { family: family.as_str(), step, hex: format_hex(color) });
        }
        let focus = self.focus_scope.clone();
        self.swatch_overlay.update(cx, |overlay, cx| {
            overlay.open_from(Some(focus), cx);
            cx.notify();
        });
        cx.notify();
    }

    fn surface(&self) -> Hsla {
        self.look.resolve_role(SemanticRole::Surface).hsla()
    }

    fn border(&self) -> Hsla {
        self.look.resolve_role(SemanticRole::Border).hsla()
    }

    fn fg(&self) -> Hsla {
        self.look.resolve_role(SemanticRole::Foreground).hsla()
    }

    fn muted(&self) -> Hsla {
        self.look.resolve_role(SemanticRole::MutedForeground).hsla()
    }

    fn accent(&self) -> Hsla {
        self.look.resolve_role(SemanticRole::Primary).hsla()
    }
}

impl Render for RadixStudioApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let surface = self.surface();
        let border = self.border();
        let fg = self.fg();
        let muted = self.muted();
        let accent = self.accent();
        let is_dark = matches!(self.look.mode(), ThemeMode::Dark);
        // Color #3 → Gray #1 (look-owned dual-space page recipe).
        let page_background = self.look.page_background();

        let title_bar = TitleBar::new().background_color(surface).border_color(border).text_color(fg).child(
            hstack! {
                gap=0 align=center;
                div().id("radix-studio-titlebar").h_full().w_full().px_3().text_color(fg).child("Radix Studio"),
            }
            .h_full()
            .w_full(),
        );

        dock_panel! {
            top: title_bar,
            fill: div()
                .id("radix-studio-scroll")
                .size_full()
                .min_h_0()
                .overflow_y_scroll()
                .child(
                    vstack! {
                        gap=28;
                        header_block(fg),
                        mode_toggle(self.light_toggle.clone(), self.dark_toggle.clone(), surface, border, is_dark, accent),
                        seed_row(
                            self.accent_field.clone(),
                            self.gray_field.clone(),
                            self.background_field.clone(),
                            self.copy_menu.clone(),
                            accent,
                            self.look.resolve_step(ScaleFamily::Gray, 8).hsla(),
                            self.look.resolve_step(ScaleFamily::Gray, 1).hsla(),
                            border,
                            muted,
                        ),
                        scale_section(&self.look, muted, cx),
                        preview_section(
                            &self.look,
                            self.cached_signup_mesh(),
                            cx.entity(),
                            self.preview_layout.clone(),
                            cx,
                            PreviewControls {
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
                                preview_tabs: self.preview_tabs.clone(),
                                preview_switch: self.preview_switch.clone(),
                                task_a: self.task_a.clone(),
                                task_b: self.task_b.clone(),
                                task_c: self.task_c.clone(),
                            },
                            surface,
                            border,
                            muted,
                            fg,
                            accent,
                        ),
                    }
                    .w_full()
                    .max_w(px(CONTENT_MAX_W))
                    .mx_auto()
                    .px_8()
                    .py_8(),
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

fn header_block(fg: Hsla) -> impl IntoElement {
    hstack! {
        justify=center;
        div().text_3xl().font_weight(gpui::FontWeight::BOLD).text_color(fg).child("Create a custom palette"),
    }
    .w_full()
}

fn mode_toggle(
    light: Toggle,
    dark: Toggle,
    surface: Hsla,
    border: Hsla,
    is_dark: bool,
    accent: Hsla,
) -> impl IntoElement {
    let _ = (is_dark, accent);
    hstack! {
        gap=4 align=center;
        light,
        dark,
    }
    .mx_auto()
    .p_1()
    .rounded_lg()
    .border_1()
    .border_color(border)
    .bg(surface)
}

#[allow(clippy::too_many_arguments)]
fn seed_row(
    accent_field: Entity<ColorTextField>,
    gray_field: Entity<ColorTextField>,
    background_field: Entity<ColorTextField>,
    copy_menu: Entity<PopupMenu>,
    accent: Hsla,
    gray: Hsla,
    background: Hsla,
    border: Hsla,
    muted: Hsla,
) -> impl IntoElement {
    hstack! {
        gap=16 align=end justify=center;
        seed_field("Accent", accent_field, accent, border, muted),
        seed_field("Gray", gray_field, gray, border, muted),
        seed_field("Background", background_field, background, border, muted),
        div().pt_5().child(copy_menu),
    }
    .w_full()
}

fn seed_field(
    label: &'static str,
    field: Entity<ColorTextField>,
    _swatch: Hsla,
    _border: Hsla,
    muted: Hsla,
) -> impl IntoElement {
    vstack! {
        gap=6;
        div().text_xs().text_color(muted).child(label),
        div().w(px(180.0)).child(field),
    }
}

/// Gutter between shade cells and between Color / Gray rows (Radix Colors grid).
const SCALE_SWATCH_GAP: f32 = 3.0;

const SCALE_LEGEND_GROUPS: &[(&str, usize)] = &[
    ("Backgrounds", 2),
    ("Interactive components", 3),
    ("Borders and separators", 3),
    ("Solid colors", 2),
    ("Accessible text", 2),
];

fn scale_section(look: &RadixLook, muted: Hsla, cx: &mut Context<RadixStudioApp>) -> impl IntoElement {
    let columns = vec![GridTrack::Star(1.0); SCALE_LEN];
    let mut grid = GridLayout::new().columns(columns).gap_x(SCALE_SWATCH_GAP).gap_y(SCALE_SWATCH_GAP);

    let mut col = 0usize;
    for &(label, span) in SCALE_LEGEND_GROUPS {
        grid = grid.child_with_span(legend_group(label, muted), 0, col, span);
        col += span;
    }

    for step in 1..=SCALE_LEN as u8 {
        grid = grid.child(scale_step_number(step, muted), 1, (step - 1) as usize);
    }

    for step in 1..=SCALE_LEN as u8 {
        grid = grid.child(scale_swatch(ScaleFamily::Color, step, look, cx), 2, (step - 1) as usize);
    }

    for step in 1..=SCALE_LEN as u8 {
        grid = grid.child(scale_swatch(ScaleFamily::Gray, step, look, cx), 3, (step - 1) as usize);
    }

    grid.into_element().w_full()
}

fn scale_swatch(family: ScaleFamily, step: u8, look: &RadixLook, cx: &mut Context<RadixStudioApp>) -> impl IntoElement {
    let color = look.resolve_step(family, step).hsla();
    div()
        .id(SharedString::from(format!("swatch-{}-{step}", family.as_str())))
        .w_full()
        .h(px(44.0))
        .bg(color)
        .cursor_pointer()
        .on_click(cx.listener(move |this, _, _, cx| {
            this.open_swatch_info(family, step, cx);
        }))
}

fn legend_group(label: &'static str, muted: Hsla) -> impl IntoElement {
    vstack! {
        gap=4 align=center;
        div()
            .w_full()
            .text_xs()
            .font_weight(gpui::FontWeight::MEDIUM)
            .text_color(muted)
            .text_center()
            .truncate()
            .child(label),
        div().w_full().h(px(2.0)).rounded_full().bg(muted.opacity(0.45)),
    }
    .w_full()
}

fn scale_step_number(step: u8, muted: Hsla) -> impl IntoElement {
    hstack! {
        justify=center;
        div()
            .text_xs()
            .font_weight(gpui::FontWeight::MEDIUM)
            .text_color(muted)
            .child(format!("{step}")),
    }
    .w_full()
}

struct PreviewControls {
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
    preview_tabs: Entity<Tabs>,
    preview_switch: Switch,
    task_a: Checkbox,
    task_b: Checkbox,
    task_c: Checkbox,
}

#[allow(clippy::too_many_arguments)]
fn preview_section(
    look: &RadixLook,
    mesh_image: Option<Arc<RenderImage>>,
    app: Entity<RadixStudioApp>,
    preview_layout: Entity<WideMiddle>,
    cx: &App,
    controls: PreviewControls,
    surface: Hsla,
    border: Hsla,
    muted: Hsla,
    fg: Hsla,
    accent: Hsla,
) -> impl IntoElement {
    // WideMiddle: L|C|R when roomy; else L|R on row 1 and full-width signup below.
    WideMiddleLayout::new(
        preview_layout,
        preview_column_left(
            controls.search_field,
            controls.search_submit,
            controls.preview_switch,
            controls.task_a,
            controls.task_b,
            controls.task_c,
            surface,
            border,
            muted,
            fg,
            accent,
        ),
        preview_column_center(
            look,
            mesh_image,
            app,
            controls.sign_up_name,
            controls.sign_up_email,
            controls.sign_up_password,
            controls.create_account,
            controls.continue_github,
            border,
            muted,
            fg,
        ),
        preview_column_right(
            controls.preview_tabs,
            controls.soft_demo,
            controls.outline_demo,
            controls.ghost_demo,
            muted,
            fg,
            accent,
        ),
    )
    .build(cx)
}

#[allow(clippy::too_many_arguments)]
fn preview_column_left(
    search_field: TextField,
    search_submit: Entity<Button>,
    preview_switch: Switch,
    task_a: Checkbox,
    task_b: Checkbox,
    task_c: Checkbox,
    surface: Hsla,
    border: Hsla,
    muted: Hsla,
    fg: Hsla,
    accent: Hsla,
) -> impl IntoElement {
    vstack! {
        gap=14;
        hstack! {
            gap=8 align=center;
            div().flex_1().min_w_0().child(search_field),
            search_submit,
        }
        .w_full(),
        alert_card(accent, muted, fg),
        div()
            .w_full()
            .rounded_lg()
            .border_1()
            .border_color(border)
            .bg(surface)
            .p_3()
            .child(
                vstack! {
                    gap=10;
                    preview_switch,
                    task_a,
                    task_b,
                    task_c,
                }
                .w_full(),
            ),
    }
    .w_full()
}

#[allow(clippy::too_many_arguments)]
fn preview_column_center(
    look: &RadixLook,
    mesh_image: Option<Arc<RenderImage>>,
    app: Entity<RadixStudioApp>,
    name: TextField,
    email: TextField,
    password: TextField,
    create: Entity<Button>,
    github: Entity<Button>,
    border: Hsla,
    muted: Hsla,
    fg: Hsla,
) -> impl IntoElement {
    let stage = SignupStage::default();
    let stage_fill = stage.fill(look);
    let panel_fill = {
        let mut c = stage.card(look);
        c.a = SIGNUP_PANEL_ALPHA;
        c
    };
    let mesh_opacity = stage.mesh_opacity;

    let inner_form_card = div()
        .id("inner_form_card")
        .flex_none()
        .w(px(SIGNUP_INNER_FORM_CARD_W))
        .min_w(px(SIGNUP_INNER_FORM_CARD_W))
        .max_w(px(SIGNUP_INNER_FORM_CARD_W))
        .overflow_hidden()
        .rounded_xl()
        .border_1()
        .border_color(border)
        .bg(panel_fill)
        .p_6()
        .child(
            vstack! {
                gap=16;
                labeled_field("Full name", name, muted),
                labeled_field("Email", email, muted),
                labeled_field("Password", password, muted),
                create,
                hstack! {
                    gap=12 align=center;
                    div().flex_1().h(px(1.0)).bg(border),
                    div().text_xs().text_color(muted).child("OR"),
                    div().flex_1().h(px(1.0)).bg(border),
                }
                .w_full(),
                github,
            }
            .w_full(),
        );

    let content = vstack! {
        gap=16 align=center justify=center;
        div()
            .flex_none()
            .text_xl()
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .text_color(fg)
            .child("Sign up"),
        div()
            .w_full()
            .flex()
            .justify_center()
            .child(inner_form_card),
    }
    .relative()
    .w_full()
    .px_4()
    .pt_6()
    .pb(px(45.0));

    div()
        .id("signup_outer_card")
        .w_full()
        .overflow_hidden()
        .relative()
        .bg(stage_fill)
        .on_prepaint(move |bounds, window, cx| {
            let width_px = bounds.size.width.as_f32().max(1.0);
            let height_px = bounds.size.height.as_f32().max(1.0);
            let scale = (SIGNUP_MESH_MAX_SIDE / width_px.max(height_px)).min(1.0);
            let width = (width_px * scale).round().max(1.0) as u32;
            let height = (height_px * scale).round().max(1.0) as u32;
            app.update(cx, |this, cx| {
                if this.ensure_signup_mesh_cache(width, height) {
                    window.refresh();
                    cx.notify();
                }
            });
        })
        .when_some(mesh_image, |this, image| {
            this.child(
                img(ImageSource::Render(image))
                    .absolute()
                    .inset_0()
                    .size_full()
                    .object_fit(gpui::ObjectFit::Cover)
                    .opacity(mesh_opacity),
            )
        })
        .child(content)
}

fn preview_column_right(
    tabs: Entity<Tabs>,
    soft: Entity<Button>,
    outline: Entity<Button>,
    ghost: Entity<Button>,
    muted: Hsla,
    fg: Hsla,
    accent: Hsla,
) -> impl IntoElement {
    vstack! {
        gap=14;
        tabs,
        typography_block(muted, fg, accent),
        hstack! {
            gap=8;
            soft,
            outline,
            ghost,
        }
        .w_full(),
    }
    .w_full()
}

fn labeled_field(label: &'static str, field: TextField, muted: Hsla) -> impl IntoElement {
    vstack! {
        gap=4;
        div().text_xs().text_color(muted).child(label),
        field,
    }
    .w_full()
}

fn alert_card(accent: Hsla, muted: Hsla, fg: Hsla) -> impl IntoElement {
    div()
        .w_full()
        .rounded_lg()
        .p_3()
        .bg(gpui::hsla(accent.h, accent.s * 0.35, (accent.l * 0.35).clamp(0.12, 0.35), 1.0))
        .border_1()
        .border_color(accent)
        .child(hstack! {
            gap=10 align=start;
            div().text_sm().text_color(accent).child("i"),
            vstack! {
                gap=2;
                div().text_sm().text_color(fg).child("Please upgrade to the new version."),
                div().text_xs().text_color(muted).child("Accent soft surface preview"),
            },
        })
}

fn typography_block(muted: Hsla, fg: Hsla, accent: Hsla) -> impl IntoElement {
    vstack! {
        gap=8;
        div().text_sm().text_color(fg).child(
            "Susan Kare is an artist and graphic designer known for creating many of the interface elements for the Apple Macintosh in the 1980s."
        ),
        div().text_sm().text_color(muted).child(
            "She also designed icons and typefaces for NeXT, IBM, and Microsoft."
        ),
        div().text_sm().text_color(accent).child("Learn more →"),
    }
    .w_full()
}
