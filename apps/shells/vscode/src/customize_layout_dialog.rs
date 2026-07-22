use std::sync::Arc;

use gpui::{
    AnyElement, App, Context, Div, Entity, EventEmitter, FocusHandle, Hsla, IntoElement, Render, SharedString,
    Stateful, Window, div, prelude::*, px,
};
use gpui_luma::controls::command::button::{Button, ButtonRenderModel};
use gpui_luma::controls::command::button::ButtonTemplate;
use gpui_luma::controls::command::icon_button::IconButton;
use gpui_luma::controls::overlay_window::{
    OverlayWindow, OverlayWindowDismissPolicy, OverlayWindowEvent, OverlayWindowMode, OverlayWindowPosition,
    OverlayWindowRenderModel,
};
use gpui_luma::controls::presenter::HasPresenter;
use gpui_luma::theme::{ControlSize, InteractionState, LayoutCacheKey, LumaLayoutCacheExt, LumaTextStyle, StandardBoxScale};
use gpui_luma::{GridTrack, grid_layout, hstack, vstack};
use gpui_luma_look_shadcn::{LumaTypographyExt, ShadcnLook, ShadcnLookControlExt, ShadcnTextRole};
use lucide_icons::Icon as LucideIcon;

use crate::layout_config::{LayoutConfig, LayoutRegion, PanelAlignment, PrimarySideBarPosition};

const DIALOG_W: f32 = 600.0;
const HEADER_H: f32 = 38.0;
const ROW_H: f32 = 27.0;
const CONTENT_PAD_X: f32 = 18.0;
const CONTENT_PAD_TOP: f32 = 8.0;
const SECTION_GAP: f32 = 4.0;
const ROW_RADIUS: f32 = 4.0;
const ICON_COL_W: f32 = 30.0;
const SHORTCUT_COL_W: f32 = 96.0;
const EYE_COL_W: f32 = 30.0;
const POSITION_LABEL_COL_W: f32 = 250.0;
const LAYOUT_ICON_BORDER_W: f32 = 1.0;

struct MenuRowButtonTemplate {
    look: Arc<ShadcnLook>,
}

impl MenuRowButtonTemplate {
    fn new(look: Arc<ShadcnLook>) -> Self {
        Self { look }
    }
}

fn menu_row_template<D: 'static>(look: &Arc<ShadcnLook>) -> Arc<dyn ButtonTemplate<D>> {
    Arc::new(MenuRowButtonTemplate::new(look.clone()))
}

impl<D: 'static> ButtonTemplate<D> for MenuRowButtonTemplate {
    fn render(&self, model: &ButtonRenderModel<D>, window: &mut Window, cx: &mut App) -> Stateful<Div> {
        let scale_factor = window.scale_factor();
        let metrics = self.look.mode_tokens().metrics;
        let scale = cx.use_cached_layout(
            metrics,
            LayoutCacheKey { size: model.size, scale_factor_bits: scale_factor.to_bits() },
            |metrics| StandardBoxScale::compute(model.size, metrics, scale_factor),
        );
        let mut look = self.look.resolve_ghost_button(model.role, model.size, model.state);
        look.height = scale.height;
        look.padding_x = scale.padding_x;
        look.padding_y = scale.padding_y;
        look.radius = ROW_RADIUS;
        let border = if model.state.focused {
            look.focus_ring
        } else {
            gpui::hsla(0.0, 0.0, 0.0, 0.0)
        };

        let content = (model.content)(model, cx);
        let mut root = div()
            .id(model.id.clone())
            .relative()
            .w_full()
            .h(px(ROW_H))
            .flex()
            .items_center()
            .rounded(px(ROW_RADIUS))
            .border_1()
            .border_color(border)
            .bg(look.background)
            .text_color(look.foreground)
            .font_family(look.font_family.clone())
            .text_size(px(look.typography.size))
            .line_height(px(look.typography.line_height))
            .font_weight(look.typography.weight)
            .child(content);

        if model.state.disabled {
            root = root.opacity(0.56);
        } else {
            root = root.cursor_pointer();
        }

        root
    }
}

#[derive(Clone, Copy)]
pub(crate) struct VisibilityRowData {
    region: LayoutRegion,
    visible: bool,
    show_visibility_header: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct PositionRowData {
    position: PrimarySideBarPosition,
    selected: bool,
    show_position_header: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct PanelAlignmentRowData {
    alignment: PanelAlignment,
    selected: bool,
    show_alignment_header: bool,
}

pub struct CustomizeLayoutDialog {
    look: Arc<ShadcnLook>,
    overlay: OverlayWindow,
    close_button: IconButton,
    reset_button: IconButton,
    visibility_rows: [Entity<Button<VisibilityRowData>>; 6],
    primary_side_bar_position_rows: [Entity<Button<PositionRowData>>; 2],
    panel_alignment_rows: [Entity<Button<PanelAlignmentRowData>>; 4],
}

impl EventEmitter<OverlayWindowEvent> for CustomizeLayoutDialog {}

impl CustomizeLayoutDialog {
    pub fn new(look: Arc<ShadcnLook>, cx: &mut Context<Self>) -> Self {
        let dialog = cx.entity().clone();
        let config = LayoutConfig::default();
        let close_button =
            look.ghost_icon_button("customize-layout-close", LucideIcon::X).size(ControlSize::Sm).spawn(cx);
        let reset_button = look
            .ghost_icon_button("customize-layout-reset", LucideIcon::RotateCcw)
            .size(ControlSize::Sm)
            .spawn(cx);

        let visibility_rows = [
            spawn_visibility_button(
                &look,
                LayoutRegion::ActivityBar,
                true,
                config.region_visible(LayoutRegion::ActivityBar),
                cx,
            ),
            spawn_visibility_button(
                &look,
                LayoutRegion::SecondaryActivityBar,
                false,
                config.region_visible(LayoutRegion::SecondaryActivityBar),
                cx,
            ),
            spawn_visibility_button(
                &look,
                LayoutRegion::PrimarySideBar,
                false,
                config.region_visible(LayoutRegion::PrimarySideBar),
                cx,
            ),
            spawn_visibility_button(
                &look,
                LayoutRegion::SecondarySideBar,
                false,
                config.region_visible(LayoutRegion::SecondarySideBar),
                cx,
            ),
            spawn_visibility_button(&look, LayoutRegion::Panel, false, config.region_visible(LayoutRegion::Panel), cx),
            spawn_visibility_button(
                &look,
                LayoutRegion::StatusBar,
                false,
                config.region_visible(LayoutRegion::StatusBar),
                cx,
            ),
        ];

        let primary_side_bar_position_rows = [
            spawn_primary_side_bar_position_button(
                &look,
                PrimarySideBarPosition::Left,
                config.primary_side_bar_position,
                cx,
            ),
            spawn_primary_side_bar_position_button(
                &look,
                PrimarySideBarPosition::Right,
                config.primary_side_bar_position,
                cx,
            ),
        ];

        let panel_alignment_rows = [
            spawn_panel_alignment_button(&look, PanelAlignment::Left, config.panel_alignment, cx),
            spawn_panel_alignment_button(&look, PanelAlignment::Right, config.panel_alignment, cx),
            spawn_panel_alignment_button(&look, PanelAlignment::Center, config.panel_alignment, cx),
            spawn_panel_alignment_button(&look, PanelAlignment::Justify, config.panel_alignment, cx),
        ];

        let overlay_look = look.clone();

        let overlay = look
            .overlay_window("customize-layout")
            .mode(OverlayWindowMode::Modeless)
            .position(OverlayWindowPosition::Center)
            .size(ControlSize::Sm)
            .dismiss_policy(OverlayWindowDismissPolicy::CloseOnClickAway)
            .width(DIALOG_W)
            .with_template_modifier(move |shell, _| {
                let dialog_background = overlay_look
                    .resolve_secondary_button(Default::default(), ControlSize::Sm, InteractionState::default())
                    .background;
                shell.p_0().overflow_hidden().bg(dialog_background)
            })
            .content(move |overlay_model, window, app| dialog.read(app).render_body(overlay_model, window, app))
            .spawn(cx);

        cx.subscribe(&overlay, |_, _, event: &OverlayWindowEvent, cx| {
            cx.emit(event.clone());
        })
        .detach();

        Self {
            look,
            overlay,
            close_button,
            reset_button,
            visibility_rows,
            primary_side_bar_position_rows,
            panel_alignment_rows,
        }
    }

    pub fn close_button(&self) -> IconButton {
        self.close_button.clone()
    }

    pub fn reset_button(&self) -> IconButton {
        self.reset_button.clone()
    }

    pub fn visibility_row(&self, region: LayoutRegion) -> Entity<Button<VisibilityRowData>> {
        self.visibility_rows[region.index()].clone()
    }

    pub fn primary_side_bar_position_row(&self, position: PrimarySideBarPosition) -> Entity<Button<PositionRowData>> {
        self.primary_side_bar_position_rows[position.index()].clone()
    }

    pub fn panel_alignment_row(&self, alignment: PanelAlignment) -> Entity<Button<PanelAlignmentRowData>> {
        self.panel_alignment_rows[alignment.index()].clone()
    }

    pub fn sync_from_config(&self, config: &LayoutConfig, cx: &mut App) {
        self.sync_visibility_rows(config, cx);
        self.sync_position_rows(config, cx);
        self.sync_panel_alignment_rows(config, cx);
    }

    pub fn refresh_theme(&self, cx: &mut App) {
        self.overlay.update(cx, |_, cx| cx.notify());
        self.close_button.update(cx, |_, cx| cx.notify());
        self.reset_button.update(cx, |_, cx| cx.notify());

        for row in &self.visibility_rows {
            row.update(cx, |_, cx| cx.notify());
        }

        for row in &self.primary_side_bar_position_rows {
            row.update(cx, |_, cx| cx.notify());
        }

        for row in &self.panel_alignment_rows {
            row.update(cx, |_, cx| cx.notify());
        }
    }

    pub fn open(&mut self, opener: Option<FocusHandle>, cx: &mut App) {
        self.overlay.update(cx, |overlay, cx| overlay.open_from(opener, cx));
    }

    pub fn dismiss(&mut self, cx: &mut App) {
        self.overlay.update(cx, |overlay, cx| overlay.dismiss(cx));
    }

    fn sync_visibility_rows(&self, config: &LayoutConfig, cx: &mut App) {
        for (index, region) in VISIBILITY_REGIONS.iter().enumerate() {
            self.visibility_rows[index].update(cx, |button, cx| {
                button.set_data(
                    VisibilityRowData {
                        region: *region,
                        visible: config.region_visible(*region),
                        show_visibility_header: index == 0,
                    },
                    cx,
                );
            });
        }
    }

    fn sync_position_rows(&self, config: &LayoutConfig, cx: &mut App) {
        for position in [PrimarySideBarPosition::Left, PrimarySideBarPosition::Right] {
            self.primary_side_bar_position_rows[position.index()].update(cx, |button, cx| {
                button.set_data(
                    PositionRowData {
                        position,
                        selected: config.primary_side_bar_position == position,
                        show_position_header: position == PrimarySideBarPosition::Left,
                    },
                    cx,
                );
            });
        }
    }

    fn sync_panel_alignment_rows(&self, config: &LayoutConfig, cx: &mut App) {
        for alignment in [PanelAlignment::Left, PanelAlignment::Right, PanelAlignment::Center, PanelAlignment::Justify]
        {
            self.panel_alignment_rows[alignment.index()].update(cx, |button, cx| {
                button.set_data(
                    PanelAlignmentRowData {
                        alignment,
                        selected: config.panel_alignment == alignment,
                        show_alignment_header: alignment == PanelAlignment::Left,
                    },
                    cx,
                );
            });
        }
    }

    fn render_body(
        &self,
        _overlay_model: &OverlayWindowRenderModel<'_>,
        _window: &mut Window,
        _app: &App,
    ) -> AnyElement {
        let chrome = self.look.chrome();

        vstack! {
            dialog_header(&self.look, self.reset_button.clone(), self.close_button.clone()),
            vstack! {
                gap=SECTION_GAP;
                visibility_section(&self.visibility_rows),
                section_divider(chrome.border),
                positions_section(
                    &self.primary_side_bar_position_rows,
                    &self.panel_alignment_rows,
                    chrome.border,
                ),
            },
        }
        .w_full()
        .into_any_element()
    }
}

impl Render for CustomizeLayoutDialog {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.overlay.clone()
    }
}

const VISIBILITY_REGIONS: [LayoutRegion; 6] = [
    LayoutRegion::ActivityBar,
    LayoutRegion::SecondaryActivityBar,
    LayoutRegion::PrimarySideBar,
    LayoutRegion::SecondarySideBar,
    LayoutRegion::Panel,
    LayoutRegion::StatusBar,
];

fn spawn_visibility_button(
    look: &Arc<ShadcnLook>,
    region: LayoutRegion,
    show_visibility_header: bool,
    visible: bool,
    cx: &mut Context<CustomizeLayoutDialog>,
) -> Entity<Button<VisibilityRowData>> {
    let look = look.clone();
    let id: SharedString = format!("customize-layout-visibility-{}", region_id(region)).into();

    look.ghost_button(id)
        .typed(VisibilityRowData { region, visible, show_visibility_header })
        .template(menu_row_template(&look))
        .compact()
        .content(move |model, _| visibility_row_content(&look, model))
        .spawn(cx)
}

fn spawn_primary_side_bar_position_button(
    look: &Arc<ShadcnLook>,
    position: PrimarySideBarPosition,
    selected: PrimarySideBarPosition,
    cx: &mut Context<CustomizeLayoutDialog>,
) -> Entity<Button<PositionRowData>> {
    spawn_position_button(
        look,
        format!("customize-layout-primary-side-bar-position-{}", position.id()),
        PositionRowData {
            position,
            selected: position == selected,
            show_position_header: position == PrimarySideBarPosition::Left,
        },
        cx,
    )
}

fn spawn_panel_alignment_button(
    look: &Arc<ShadcnLook>,
    alignment: PanelAlignment,
    selected: PanelAlignment,
    cx: &mut Context<CustomizeLayoutDialog>,
) -> Entity<Button<PanelAlignmentRowData>> {
    let look = look.clone();

    look.ghost_button(format!("customize-layout-panel-alignment-{}", alignment.id()))
        .typed(PanelAlignmentRowData {
            alignment,
            selected: alignment == selected,
            show_alignment_header: alignment == PanelAlignment::Left,
        })
        .template(menu_row_template(&look))
        .compact()
        .content(move |model, _| panel_alignment_row_content(&look, model))
        .spawn(cx)
}

fn spawn_position_button(
    look: &Arc<ShadcnLook>,
    id: impl Into<SharedString>,
    data: PositionRowData,
    cx: &mut Context<CustomizeLayoutDialog>,
) -> Entity<Button<PositionRowData>> {
    let look = look.clone();

    look.ghost_button(id)
        .typed(data)
        .template(menu_row_template(&look))
        .compact()
        .content(move |model, _| position_row_content(&look, model))
        .spawn(cx)
}

fn region_id(region: LayoutRegion) -> &'static str {
    match region {
        LayoutRegion::ActivityBar => "activity-bar",
        LayoutRegion::SecondaryActivityBar => "secondary-activity-bar",
        LayoutRegion::PrimarySideBar => "primary-side-bar",
        LayoutRegion::SecondarySideBar => "secondary-side-bar",
        LayoutRegion::Panel => "panel",
        LayoutRegion::StatusBar => "status-bar",
    }
}

fn section_divider(color: Hsla) -> impl IntoElement {
    div().w_full().h(px(1.0)).mx(px(CONTENT_PAD_X)).bg(color)
}

fn dialog_header(look: &ShadcnLook, reset_button: IconButton, close_button: IconButton) -> impl IntoElement {
    let chrome = look.chrome();
    let title_style = look.typography_role(ShadcnTextRole::H4);

    div()
        .h(px(HEADER_H))
        .w_full()
        .flex()
        .items_center()
        .justify_center()
        .bg(chrome.panel_background)
        .text_color(chrome.title_text)
        .relative()
        .child(div().typography_style(title_style).child("Customize Layout"))
        .child(
            hstack! {
                gap=2;
                reset_button,
                close_button,
            }
            .absolute()
            .right(px(10.0))
            .top(px(5.0)),
        )
}

fn visibility_section(rows: &[Entity<Button<VisibilityRowData>>; 6]) -> impl IntoElement {
    vstack! {
        rows[0].clone(),
        rows[1].clone(),
        rows[2].clone(),
        rows[3].clone(),
        rows[4].clone(),
        rows[5].clone(),
    }
    .w_full()
    .px(px(CONTENT_PAD_X))
    .pt(px(CONTENT_PAD_TOP))
}

fn positions_section(
    primary_rows: &[Entity<Button<PositionRowData>>; 2],
    panel_alignment_rows: &[Entity<Button<PanelAlignmentRowData>>; 4],
    border: Hsla,
) -> impl IntoElement {
    vstack! {
        gap=SECTION_GAP;
        vstack! {
            primary_rows[0].clone(),
            primary_rows[1].clone(),
        },
        div().w_full().h(px(1.0)).bg(border),
        vstack! {
            panel_alignment_rows[0].clone(),
            panel_alignment_rows[1].clone(),
            panel_alignment_rows[2].clone(),
            panel_alignment_rows[3].clone(),
        },
        div().w_full().h(px(1.0)).bg(border),
    }
    .w_full()
    .px(px(CONTENT_PAD_X))
}

fn visibility_row_content(look: &ShadcnLook, model: &ButtonRenderModel<VisibilityRowData>) -> AnyElement {
    let data = model.data;
    let chrome = look.chrome();
    let body_style = look.typography_role(ShadcnTextRole::P);
    let muted_style = look.typography_role(ShadcnTextRole::P);
    let row_foreground = look.resolve_ghost_button(model.role, model.size, model.state).foreground;

    div()
        .w_full()
        .px(px(12.0))
        .child(grid_layout! {
            rows: 1,
            columns: [
                GridTrack::Px(ICON_COL_W),
                GridTrack::Star(1.0),
                GridTrack::Px(SHORTCUT_COL_W),
                GridTrack::Px(EYE_COL_W),
            ];
            [0, 0] => layout_region_icon(data.region, row_foreground),
            [0, 1] => div()
                .min_w_0()
                .typography_style(body_style)
                .text_color(row_foreground)
                .truncate()
                .child(data.region.label()),
            [0, 2] => shortcut_column(
                data.region,
                data.show_visibility_header,
                chrome.border,
                chrome.muted_text,
                muted_style,
            ),
            [0, 3] => visibility_eye(data.visible, chrome.muted_text),
        })
        .into_any_element()
}

fn position_row_content(look: &ShadcnLook, model: &ButtonRenderModel<PositionRowData>) -> AnyElement {
    let data = model.data;
    let chrome = look.chrome();
    let body_style = look.typography_role(ShadcnTextRole::P);
    let muted_style = look.typography_role(ShadcnTextRole::P);
    let row_foreground = look.resolve_ghost_button(model.role, model.size, model.state).foreground;
    let label = data.position.label();
    let icon = primary_side_bar_position_icon(data.position);

    div()
        .w_full()
        .px(px(12.0))
        .child(grid_layout! {
            rows: 1,
            columns: [
                GridTrack::Px(ICON_COL_W),
                GridTrack::Star(1.0),
                GridTrack::Px(POSITION_LABEL_COL_W),
                GridTrack::Px(EYE_COL_W),
            ];
            [0, 0] => layout_option_icon(icon, row_foreground),
            [0, 1] => hstack! {
                gap=8 align=center;
                div()
                    .typography_style(body_style)
                    .text_color(row_foreground)
                    .child(label),
                selected_check(data.selected, row_foreground),
            },
            [0, 2] => position_header(
                data.show_position_header,
                "Primary Side Bar Position",
                muted_style,
                chrome.muted_text,
            ),
            [0, 3] => div().w(px(EYE_COL_W)),
        })
        .into_any_element()
}

fn panel_alignment_row_content(look: &ShadcnLook, model: &ButtonRenderModel<PanelAlignmentRowData>) -> AnyElement {
    let data = model.data;
    let chrome = look.chrome();
    let body_style = look.typography_role(ShadcnTextRole::P);
    let muted_style = look.typography_role(ShadcnTextRole::P);
    let row_foreground = look.resolve_ghost_button(model.role, model.size, model.state).foreground;
    let label = data.alignment.label();
    let icon = panel_alignment_icon(data.alignment);

    div()
        .w_full()
        .px(px(12.0))
        .child(grid_layout! {
            rows: 1,
            columns: [
                GridTrack::Px(ICON_COL_W),
                GridTrack::Star(1.0),
                GridTrack::Px(POSITION_LABEL_COL_W),
                GridTrack::Px(EYE_COL_W),
            ];
            [0, 0] => layout_option_icon(icon, row_foreground),
            [0, 1] => hstack! {
                gap=8 align=center;
                div()
                    .typography_style(body_style)
                    .text_color(row_foreground)
                    .child(label),
                selected_check(data.selected, row_foreground),
            },
            [0, 2] => position_header(data.show_alignment_header, "Panel Alignment", muted_style, chrome.muted_text),
            [0, 3] => div().w(px(EYE_COL_W)),
        })
        .into_any_element()
}

fn position_header(show: bool, label: &'static str, style: LumaTextStyle, color: Hsla) -> impl IntoElement {
    hstack! {
        justify=end align=center;
        div()
            .typography_style(style)
            .text_color(color)
            .child(if show {
                label
            } else {
                ""
            }),
    }
    .w_full()
}

fn selected_check(selected: bool, color: Hsla) -> impl IntoElement {
    div()
        .w(px(18.0))
        .h(px(18.0))
        .flex()
        .items_center()
        .justify_center()
        .font_family("lucide")
        .text_size(px(15.0))
        .text_color(color)
        .child(if selected {
            char::from(LucideIcon::Check).to_string()
        } else {
            String::new()
        })
}

fn visibility_eye(visible: bool, muted_text: Hsla) -> impl IntoElement {
    let eye_icon = if visible { LucideIcon::Eye } else { LucideIcon::EyeOff };

    div()
        .w(px(EYE_COL_W))
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center()
        .font_family("lucide")
        .text_size(px(13.0))
        .text_color(muted_text)
        .child(char::from(eye_icon).to_string())
}

fn shortcut_column(
    region: LayoutRegion,
    show_visibility_header: bool,
    border: Hsla,
    muted_text: Hsla,
    muted_style: LumaTextStyle,
) -> AnyElement {
    if show_visibility_header {
        return hstack! {
            justify=end align=center;
            div().typography_style(muted_style).text_color(muted_text).child("Visibility"),
        }
        .w_full()
        .into_any_element();
    }

    if let Some(keys) = region.shortcut_keys() {
        return hstack! {
            justify=end align=center gap=2;
            shortcut_key(border, muted_text, keys[0]),
            shortcut_key(border, muted_text, keys[1]),
        }
        .w_full()
        .into_any_element();
    }

    div().w_full().into_any_element()
}

fn shortcut_key(border: Hsla, text_color: Hsla, label: &str) -> impl IntoElement {
    div()
        .min_w(px(18.0))
        .h(px(18.0))
        .px(px(4.0))
        .flex()
        .items_center()
        .justify_center()
        .flex_shrink_0()
        .rounded(px(3.0))
        .border_1()
        .border_color(border)
        .text_size(px(11.0))
        .text_color(text_color)
        .child(label.to_string())
}

fn layout_region_icon(region: LayoutRegion, foreground: Hsla) -> impl IntoElement {
    let icon = div()
        .w(px(18.0))
        .h(px(18.0))
        .border(px(LAYOUT_ICON_BORDER_W))
        .border_color(foreground)
        .rounded(px(2.0))
        .overflow_hidden();

    match region {
        LayoutRegion::ActivityBar => icon.child(div().h_full().w(px(4.0)).bg(foreground)),
        LayoutRegion::SecondaryActivityBar => icon.flex().justify_end().child(div().h_full().w(px(4.0)).bg(foreground)),
        LayoutRegion::PrimarySideBar => icon.child(div().h_full().w(px(7.0)).bg(foreground)),
        LayoutRegion::SecondarySideBar => icon.flex().justify_end().child(div().h_full().w(px(7.0)).bg(foreground)),
        LayoutRegion::Panel => icon.flex().flex_col().justify_end().child(div().w_full().h(px(5.0)).bg(foreground)),
        LayoutRegion::StatusBar => icon.flex().flex_col().justify_end().child(div().w_full().h(px(3.0)).bg(foreground)),
    }
}

#[derive(Clone, Copy)]
enum LayoutOptionIcon {
    PrimaryLeft,
    PrimaryRight,
    PanelLeft,
    PanelRight,
    PanelCenter,
    PanelJustify,
}

fn primary_side_bar_position_icon(position: PrimarySideBarPosition) -> LayoutOptionIcon {
    match position {
        PrimarySideBarPosition::Left => LayoutOptionIcon::PrimaryLeft,
        PrimarySideBarPosition::Right => LayoutOptionIcon::PrimaryRight,
    }
}

fn panel_alignment_icon(alignment: PanelAlignment) -> LayoutOptionIcon {
    match alignment {
        PanelAlignment::Left => LayoutOptionIcon::PanelLeft,
        PanelAlignment::Right => LayoutOptionIcon::PanelRight,
        PanelAlignment::Center => LayoutOptionIcon::PanelCenter,
        PanelAlignment::Justify => LayoutOptionIcon::PanelJustify,
    }
}

fn layout_option_icon(option: LayoutOptionIcon, foreground: Hsla) -> impl IntoElement {
    let icon = div()
        .w(px(18.0))
        .h(px(18.0))
        .border(px(LAYOUT_ICON_BORDER_W))
        .border_color(foreground)
        .rounded(px(2.0))
        .overflow_hidden();

    match option {
        LayoutOptionIcon::PrimaryLeft => icon.child(div().h_full().w(px(7.0)).bg(foreground)),
        LayoutOptionIcon::PrimaryRight => icon.flex().justify_end().child(div().h_full().w(px(7.0)).bg(foreground)),
        LayoutOptionIcon::PanelLeft => {
            icon.flex().flex_col().justify_end().child(div().w(px(8.0)).h(px(5.0)).bg(foreground))
        }
        LayoutOptionIcon::PanelRight => {
            icon.flex().flex_col().justify_end().items_end().child(div().w(px(8.0)).h(px(5.0)).bg(foreground))
        }
        LayoutOptionIcon::PanelCenter => icon
            .flex()
            .flex_col()
            .justify_end()
            .items_center()
            .child(div().w(px(8.0)).h(px(5.0)).bg(foreground)),
        LayoutOptionIcon::PanelJustify => {
            icon.flex().flex_col().justify_end().child(div().w_full().h(px(5.0)).bg(foreground))
        }
    }
}
