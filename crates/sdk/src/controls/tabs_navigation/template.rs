use std::cell::Cell;
use std::sync::{Arc, Mutex, OnceLock};

use gpui::{
    Anchor, AnyElement, App, Bounds, Div, MouseDownEvent, Pixels, Point, SharedString, Size, Stateful, TextRun, Window,
    anchored, deferred, div, font, hsla, point, px, prelude::*,
};

use super::indicator::TabsNavigationIndicatorMotion;
use super::{
    TabsNavigationItem, TabsNavigationItemAccessory, TabsNavigationRenderItem, TabsNavigationRenderModel,
    model::TabsNavigationWidthMode,
};
use crate::controls::button_family::{ButtonFamilyLook, ButtonFamilyRole, default_button_family_theme};
use crate::controls::color::style::ElementExt;
use crate::controls::command::button::{ButtonRenderModel, ButtonTemplate, DefaultButtonTemplate};
use crate::controls::control_group::{
    ControlGroupBoundsHandler, ControlGroupClickHandler, ControlGroupHoverHandler, ControlGroupItemHandlerExt,
    ControlGroupMouseDownHandler, ControlGroupMouseUpHandler, ControlGroupRenderModel, ControlGroupTemplate,
    ControlGroupTemplateHandlers,
};
use crate::controls::icon::{DisclosureIcons, IconSource, lucide_icon, render_disclosure_icon};
use crate::controls::overlay_presence::OverlayPresence;
use crate::controls::tabs_navigation::{TabsNavigationItemLook, TabsNavigationTheme, default_tabs_navigation_theme};
use crate::theme::{ControlSize, InteractionState};

const TAB_ACCESSORY_SIZE: f32 = 12.0;
const TAB_ACCESSORY_GAP: f32 = 4.0;

pub type TabsNavigationClickHandler = ControlGroupClickHandler;
pub type TabsNavigationBoundsHandler = ControlGroupBoundsHandler;
pub type TabsNavigationHoverHandler = ControlGroupHoverHandler;
pub type TabsNavigationMouseDownHandler = ControlGroupMouseDownHandler;
pub type TabsNavigationMouseUpHandler = ControlGroupMouseUpHandler;
pub type TabsNavigationTemplateHandlers = ControlGroupTemplateHandlers;

pub type TabsNavigationTemplateModifier =
    Box<dyn Fn(Stateful<Div>, &TabsNavigationRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static>;

#[derive(Clone)]
pub struct TabsNavigationOverlayState {
    trigger_bounds: Arc<Mutex<Option<Bounds<Pixels>>>>,
    presence: Arc<Mutex<OverlayPresence>>,
    last_overlay: Arc<Mutex<Option<TabsNavigationCachedOverlay>>>,
}

#[derive(Clone)]
struct TabsNavigationCachedOverlay {
    content_size: Size<Pixels>,
    placement: TabsNavigationOverlayPlacement,
    offset_y: Pixels,
    window_margin: Pixels,
}

impl Default for TabsNavigationOverlayState {
    fn default() -> Self {
        Self {
            trigger_bounds: Arc::new(Mutex::new(None)),
            presence: Arc::new(Mutex::new(OverlayPresence::new(false, true))),
            last_overlay: Arc::new(Mutex::new(None)),
        }
    }
}

impl TabsNavigationOverlayState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn trigger_bounds(&self) -> Option<Bounds<Pixels>> {
        *self.trigger_bounds.lock().expect("tabs navigation overlay trigger bounds lock")
    }

    pub fn set_animated(&self, animated: bool) {
        self.presence.lock().expect("tabs navigation overlay presence lock").set_animated(animated);
    }

    /// Sync presence for the current frame. Returns `true` while animating.
    /// Hosting controls should call this from `Render` and `cx.notify()` while it returns true.
    pub fn sync_for_frame(&self) -> bool {
        self.presence.lock().expect("tabs navigation overlay presence lock").sync()
    }

    pub fn is_animating(&self) -> bool {
        self.presence.lock().expect("tabs navigation overlay presence lock").is_animating()
    }

    pub fn should_paint(&self) -> bool {
        self.presence.lock().expect("tabs navigation overlay presence lock").should_paint()
    }

    fn set_overlay_open(&self, open: bool) {
        self.presence.lock().expect("tabs navigation overlay presence lock").set_open(open);
    }

    fn presence_snapshot(&self) -> OverlayPresence {
        *self.presence.lock().expect("tabs navigation overlay presence lock")
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TabsNavigationOverlayPlacement {
    BelowStart,
    BelowCenter,
}

pub struct TabsNavigationItemOverlay {
    pub content: AnyElement,
    pub content_size: Size<Pixels>,
    pub placement: TabsNavigationOverlayPlacement,
    pub offset_y: Pixels,
    pub window_margin: Pixels,
}

#[derive(Clone, Copy)]
struct ResolvedTabsNavigationOverlayPlacement {
    anchor: Anchor,
    position: Point<Pixels>,
    offset: Point<Pixels>,
}

#[derive(Clone)]
struct TabsNavigationButtonData {
    look: TabsNavigationItemLook,
    disclosure_icons: DisclosureIcons,
}

#[derive(Clone)]
pub struct TabsNavigationItemButtonStyle {
    pub look: TabsNavigationItemLook,
    pub font_family: SharedString,
    pub size: ControlSize,
    pub disclosure_icons: DisclosureIcons,
}

impl TabsNavigationItemButtonStyle {
    pub fn new(look: TabsNavigationItemLook, font_family: SharedString, size: ControlSize) -> Self {
        Self {
            look,
            font_family,
            size,
            disclosure_icons: DisclosureIcons::new(
                lucide_svg_static::Icon::ChevronUp,
                lucide_svg_static::Icon::ChevronDown,
            ),
        }
    }

    pub fn disclosure_icons(mut self, icons: DisclosureIcons) -> Self {
        self.disclosure_icons = icons;
        self
    }
}

pub trait TabsNavigationTemplate: Send + Sync {
    fn render(
        &self,
        model: &TabsNavigationRenderModel<'_>,
        handlers: TabsNavigationTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}

pub struct ThemedTabsNavigationTemplate {
    theme: Arc<dyn TabsNavigationTheme>,
    modifiers: Vec<TabsNavigationTemplateModifier>,
}

impl ThemedTabsNavigationTemplate {
    pub fn new(theme: Arc<dyn TabsNavigationTheme>) -> Self {
        Self { theme, modifiers: Vec::new() }
    }

    pub fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &TabsNavigationRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<Div>, model: &TabsNavigationRenderModel<'_>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = modifier(root, model);
        }
        root
    }
}

struct ModifiedTabsNavigationTemplate {
    base: Arc<dyn TabsNavigationTemplate>,
    modifiers: Vec<TabsNavigationTemplateModifier>,
}

impl ModifiedTabsNavigationTemplate {
    fn new(base: Arc<dyn TabsNavigationTemplate>) -> Self {
        Self { base, modifiers: Vec::new() }
    }

    fn with_modifier(mut self, modifier: TabsNavigationTemplateModifier) -> Self {
        self.modifiers.push(modifier);
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<Div>, model: &TabsNavigationRenderModel<'_>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = modifier(root, model);
        }
        root
    }
}

pub fn default_tabs_navigation_template() -> Arc<dyn TabsNavigationTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn TabsNavigationTemplate>> = OnceLock::new();

    TEMPLATE
        .get_or_init(|| Arc::new(ThemedTabsNavigationTemplate::new(default_tabs_navigation_theme())))
        .clone()
}

pub(super) fn template_with_modifier<F>(
    template: Arc<dyn TabsNavigationTemplate>,
    modifier: F,
) -> Arc<dyn TabsNavigationTemplate>
where
    F: Fn(Stateful<Div>, &TabsNavigationRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
{
    Arc::new(ModifiedTabsNavigationTemplate::new(template).with_modifier(Box::new(modifier)))
}

impl TabsNavigationTemplate for ModifiedTabsNavigationTemplate {
    fn render(
        &self,
        model: &TabsNavigationRenderModel<'_>,
        handlers: TabsNavigationTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let root = self.base.render(model, handlers, window, cx);
        self.apply_modifiers(root, model)
    }
}

impl TabsNavigationTemplate for ThemedTabsNavigationTemplate {
    fn render(
        &self,
        model: &TabsNavigationRenderModel<'_>,
        handlers: TabsNavigationTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let list_look = self.theme.resolve_list(model.enabled, model.size);
        let uniform_width = resolve_tabs_navigation_uniform_item_width(model, self.theme.as_ref(), model.size, window);

        let active_item = model.items.iter().find(|item| item.active);
        let active_look =
            active_item.map(|item| self.theme.resolve_item(true, item.state.interaction_state(), model.size));
        if let Some(motion) = model.indicator_motion {
            let look =
                active_look.unwrap_or_else(|| self.theme.resolve_item(true, InteractionState::default(), model.size));
            motion.set_metrics(look.padding_x, look.indicator_height, look.indicator);
        }

        let mut root = div()
            .id(model.id.clone())
            .relative()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(list_look.gap))
            .p(px(list_look.padding))
            .rounded(px(list_look.radius));

        if let Some(motion) = model.indicator_motion.cloned() {
            root = root.on_prepaint(move |bounds, _, _| {
                motion.set_list_bounds(bounds);
            });
        }

        if let Some(background) = list_look.background {
            root = root.bg(background);
        }

        if let Some(border) = list_look.border {
            root = root.border_1().border_color(border);
        }

        for (item, item_handlers) in model.items.iter().zip(handlers.into_item_handlers()) {
            let look = self.theme.resolve_item(item.active, item.state.interaction_state(), model.size);
            let mut tab = render_tabs_navigation_item_button_with_style(
                model.id,
                item,
                TabsNavigationItemButtonStyle::new(look, self.theme.font_family(), model.size)
                    .disclosure_icons(model.disclosure_icons.clone()),
                window,
                cx,
            )
            .control_group_item_handlers(item_handlers);

            if let Some(width) = uniform_width {
                tab = tab.w(px(width)).flex_none();
            }

            if !item.state.disabled {
                tab = tab.cursor_pointer();
            }

            root = root.child(tab);
        }

        if let Some(indicator) =
            model.indicator.or_else(|| model.indicator_motion.and_then(TabsNavigationIndicatorMotion::paint))
        {
            root = root.child(
                div()
                    .absolute()
                    .left(px(indicator.left))
                    .bottom(px(0.0))
                    .w(px(indicator.width))
                    .h(px(indicator.height))
                    .rounded(px(indicator.height))
                    .bg(indicator.color),
            );
        }

        self.apply_modifiers(root, model)
    }
}

pub fn resolve_tabs_navigation_uniform_item_width(
    model: &TabsNavigationRenderModel<'_>,
    theme: &dyn TabsNavigationTheme,
    size: ControlSize,
    window: &mut Window,
) -> Option<f32> {
    if model.width_mode != TabsNavigationWidthMode::Uniform {
        return None;
    }

    let font_family = theme.font_family();
    let mut max_width = 0.0_f32;

    for item in &model.items {
        let look = theme.resolve_item(item.active, item.state.interaction_state(), size);
        let run = TextRun {
            len: item.label.len(),
            font: {
                let mut font = font(font_family.clone());
                font.weight = look.label_typography.weight;
                font
            },
            color: look.label_color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let line = window.text_system().shape_line(item.label.clone(), px(look.label_typography.size), &[run], None);
        let accessory_count =
            usize::from(item.leading_accessory.is_some()) + usize::from(item.trailing_accessory.is_some());
        let accessory_width = accessory_count as f32 * TAB_ACCESSORY_SIZE
            + if accessory_count > 0 {
                accessory_count as f32 * TAB_ACCESSORY_GAP
            } else {
                0.0
            };
        let width = line.x_for_index(item.label.len()).as_f32() + look.padding_x * 2.0 + accessory_width;
        max_width = max_width.max(width);
    }

    Some(max_width)
}

pub fn render_tabs_navigation_item_overlay_host<F>(
    id: impl Into<SharedString>,
    mut trigger: Stateful<Div>,
    width: Option<Pixels>,
    overlay_state: TabsNavigationOverlayState,
    overlay: Option<TabsNavigationItemOverlay>,
    on_mouse_down_out: F,
) -> AnyElement
where
    F: Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
{
    let id = id.into();
    let mut root = div()
        .on_children_prepainted({
            let trigger_bounds = overlay_state.trigger_bounds.clone();
            move |bounds, _, _| {
                if let Some(bounds) = bounds.first() {
                    *trigger_bounds.lock().expect("tabs navigation overlay trigger bounds lock") = Some(*bounds);
                }
            }
        })
        .id(id)
        .relative()
        .on_mouse_down_out(on_mouse_down_out);

    if let Some(width) = width {
        trigger = trigger.w_full();
        root = root.w(width).flex_none();
    }

    root = root.child(trigger);

    match overlay {
        Some(overlay) => {
            *overlay_state.last_overlay.lock().expect("tabs navigation overlay cache lock") =
                Some(TabsNavigationCachedOverlay {
                    content_size: overlay.content_size,
                    placement: overlay.placement,
                    offset_y: overlay.offset_y,
                    window_margin: overlay.window_margin,
                });
            overlay_state.set_overlay_open(true);
            let presence = overlay_state.presence_snapshot();
            let placement = resolve_tabs_navigation_overlay_placement(
                overlay_state.trigger_bounds(),
                overlay.placement,
                overlay.content_size,
                overlay.offset_y,
            );
            let offset = presence.adjust_offset(placement.offset, overlay.content_size);
            let content = div().opacity(presence.opacity()).child(overlay.content);
            let anchored_overlay = anchored()
                .snap_to_window_with_margin(overlay.window_margin)
                .anchor(placement.anchor)
                .position(placement.position)
                .offset(offset)
                .child(content);
            root = root.child(deferred(anchored_overlay).with_priority(1));
        }
        None => {
            overlay_state.set_overlay_open(false);
            let presence = overlay_state.presence_snapshot();
            if presence.should_paint()
                && let Some(cached) =
                    overlay_state.last_overlay.lock().expect("tabs navigation overlay cache lock").clone()
            {
                let placement = resolve_tabs_navigation_overlay_placement(
                    overlay_state.trigger_bounds(),
                    cached.placement,
                    cached.content_size,
                    cached.offset_y,
                );
                let offset = presence.adjust_offset(placement.offset, cached.content_size);
                // Exit keeps a sized shell while content is unavailable (`AnyElement` is not cloneable).
                let content =
                    div().w(cached.content_size.width).h(cached.content_size.height).opacity(presence.opacity());
                let anchored_overlay = anchored()
                    .snap_to_window_with_margin(cached.window_margin)
                    .anchor(placement.anchor)
                    .position(placement.position)
                    .offset(offset)
                    .child(content);
                root = root.child(deferred(anchored_overlay).with_priority(1));
            }
        }
    }

    root.into_any_element()
}

fn resolve_tabs_navigation_overlay_placement(
    trigger_bounds: Option<Bounds<Pixels>>,
    placement: TabsNavigationOverlayPlacement,
    content_size: Size<Pixels>,
    offset_y: Pixels,
) -> ResolvedTabsNavigationOverlayPlacement {
    let trigger_bounds = trigger_bounds
        .unwrap_or_else(|| Bounds::new(point(px(0.0), px(0.0)), Size { width: px(0.0), height: px(0.0) }));

    match placement {
        TabsNavigationOverlayPlacement::BelowStart => ResolvedTabsNavigationOverlayPlacement {
            anchor: Anchor::TopLeft,
            position: point(trigger_bounds.left(), trigger_bounds.bottom()),
            offset: point(px(0.0), offset_y),
        },
        TabsNavigationOverlayPlacement::BelowCenter => ResolvedTabsNavigationOverlayPlacement {
            anchor: Anchor::TopLeft,
            position: point(trigger_bounds.center().x, trigger_bounds.bottom()),
            offset: point(-(content_size.width * 0.5), offset_y),
        },
    }
}

fn tabs_navigation_item_button_template() -> Arc<dyn ButtonTemplate<TabsNavigationButtonData>> {
    static TEMPLATE: OnceLock<Arc<dyn ButtonTemplate<TabsNavigationButtonData>>> = OnceLock::new();

    TEMPLATE
        .get_or_init(|| Arc::new(DefaultButtonTemplate::<TabsNavigationButtonData>::new(default_button_family_theme())))
        .clone()
}

pub fn render_tabs_navigation_item_button(
    navigation_id: &SharedString,
    item: &TabsNavigationRenderItem<'_>,
    look: TabsNavigationItemLook,
    font_family: SharedString,
    size: ControlSize,
    window: &mut Window,
    cx: &mut App,
) -> Stateful<Div> {
    render_tabs_navigation_item_button_with_style(
        navigation_id,
        item,
        TabsNavigationItemButtonStyle::new(look, font_family, size),
        window,
        cx,
    )
}

pub fn render_tabs_navigation_item_button_with_style(
    navigation_id: &SharedString,
    item: &TabsNavigationRenderItem<'_>,
    style: TabsNavigationItemButtonStyle,
    window: &mut Window,
    cx: &mut App,
) -> Stateful<Div> {
    tabs_navigation_item_button_template().render(
        &tabs_navigation_button_model(
            navigation_id,
            item,
            style.look,
            style.font_family,
            style.size,
            style.disclosure_icons,
        ),
        window,
        cx,
    )
}

fn tabs_navigation_button_model(
    navigation_id: &SharedString,
    item: &TabsNavigationRenderItem<'_>,
    look: TabsNavigationItemLook,
    font_family: SharedString,
    size: ControlSize,
    disclosure_icons: DisclosureIcons,
) -> ButtonRenderModel<TabsNavigationButtonData> {
    let label = item.label.clone();
    let leading_accessory = item.leading_accessory.cloned();
    let trailing_accessory = item.trailing_accessory.cloned();
    let disclosure_progress = item.disclosure_progress;

    ButtonRenderModel {
        id: format!("{}-tab-{}", navigation_id, item.id).into(),
        data: TabsNavigationButtonData { look, disclosure_icons },
        icon: None,
        content: Arc::new(move |model, _| {
            let mut label_content = div().flex().items_center().gap(px(TAB_ACCESSORY_GAP));
            if let Some(accessory) = &leading_accessory {
                label_content = label_content.child(render_accessory(
                    accessory,
                    model.data.look.label_color,
                    &model.data.disclosure_icons,
                    disclosure_progress,
                ));
            }
            label_content = label_content.child(label.clone());
            if let Some(accessory) = &trailing_accessory {
                label_content = label_content.child(render_accessory(
                    accessory,
                    model.data.look.label_color,
                    &model.data.disclosure_icons,
                    disclosure_progress,
                ));
            }
            label_content.into_any_element()
        }),
        role: ButtonFamilyRole::Toggle { selected: item.active },
        size,
        state: item.state.interaction_state(),
        round: false,
        radius_override: Cell::new(Some(look.radius)),
        elevation: false,
        compact: false,
        switch_track_width_extra: 0.0,
        switch_orientation: crate::controls::switch::SwitchOrientation::Horizontal,
        switch_track_content: None,
        switch_thumb_content: None,
        look: Some(Arc::new(move |model| tabs_navigation_button_look(model.data.look, font_family.clone()))),
    }
}

fn tabs_navigation_button_look(look: TabsNavigationItemLook, font_family: SharedString) -> ButtonFamilyLook {
    ButtonFamilyLook {
        background: hsla(0.0, 0.0, 0.0, 0.0),
        foreground: look.label_color,
        border: None,
        typography: look.label_typography,
        font_family,
        radius: look.radius,
        padding_x: look.padding_x,
        padding_y: 0.0,
        gap: TAB_ACCESSORY_GAP,
        height: look.height,
        icon_size: TAB_ACCESSORY_SIZE,
        shadow: None,
    }
}

fn render_accessory(
    accessory: &TabsNavigationItemAccessory,
    color: gpui::Hsla,
    disclosure_icons: &crate::controls::icon::DisclosureIcons,
    disclosure_progress: f32,
) -> gpui::AnyElement {
    match accessory {
        TabsNavigationItemAccessory::Icon(icon) => render_tab_icon_source(icon, color),
        TabsNavigationItemAccessory::Disclosure { .. } => {
            render_disclosure_icon(disclosure_icons, disclosure_progress, color, TAB_ACCESSORY_SIZE)
        }
    }
}

fn render_tab_icon_source(icon: &IconSource, color: gpui::Hsla) -> gpui::AnyElement {
    match icon {
        IconSource::Lucide(icon) => lucide_icon(*icon, color, TAB_ACCESSORY_SIZE),
        IconSource::SvgPath(_) => div().size(px(TAB_ACCESSORY_SIZE)).into_any_element(),
    }
}

pub(crate) fn tabs_navigation_control_group_template(
    size: crate::theme::ControlSize,
    width_mode: TabsNavigationWidthMode,
    template: Arc<dyn TabsNavigationTemplate>,
    indicator_motion: TabsNavigationIndicatorMotion,
    disclosure_icons: crate::controls::icon::DisclosureIcons,
    disclosure_progress: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<SharedString, f32>>>,
) -> ControlGroupTemplate<TabsNavigationItem> {
    Arc::new(move |model, handlers, window, cx| {
        let tabs_model = tabs_navigation_render_model(
            model,
            size,
            width_mode,
            &indicator_motion,
            &disclosure_icons,
            &disclosure_progress,
        );
        template.render(&tabs_model, handlers, window, cx)
    })
}

fn tabs_navigation_render_model<'a>(
    model: &'a ControlGroupRenderModel<'a, TabsNavigationItem>,
    size: crate::theme::ControlSize,
    width_mode: TabsNavigationWidthMode,
    indicator_motion: &'a TabsNavigationIndicatorMotion,
    disclosure_icons: &'a crate::controls::icon::DisclosureIcons,
    disclosure_progress: &std::sync::Arc<std::sync::Mutex<std::collections::HashMap<SharedString, f32>>>,
) -> TabsNavigationRenderModel<'a> {
    let progress = disclosure_progress.lock().expect("tabs disclosure progress lock");
    TabsNavigationRenderModel {
        id: model.id,
        size,
        width_mode,
        items: model
            .items
            .iter()
            .map(|item| TabsNavigationRenderItem {
                id: item.item.id(),
                label: item.item.label_text(),
                trigger_kind: item.item.trigger_kind_value(),
                leading_accessory: item.item.leading_accessory_ref(),
                trailing_accessory: item.item.trailing_accessory_ref(),
                active: item.selected,
                enabled: item.enabled,
                state: item.state,
                disclosure_progress: progress.get(item.item.id()).copied().unwrap_or(item.item.disclosure_progress),
            })
            .collect(),
        active_id: model.selected_ids.first(),
        enabled: model.enabled,
        focus: model.focus,
        indicator: indicator_motion.paint(),
        indicator_motion: Some(indicator_motion),
        disclosure_icons,
    }
}

#[cfg(test)]
mod tests {
    use gpui::{Bounds, point, px, size};

    use super::*;

    #[test]
    fn overlay_below_start_anchors_to_trigger_bottom_left() {
        let placement = resolve_tabs_navigation_overlay_placement(
            Some(Bounds::new(point(px(40.0), px(10.0)), size(px(80.0), px(32.0)))),
            TabsNavigationOverlayPlacement::BelowStart,
            size(px(200.0), px(120.0)),
            px(6.0),
        );

        assert_eq!(placement.anchor, Anchor::TopLeft);
        assert_eq!(placement.position, point(px(40.0), px(42.0)));
        assert_eq!(placement.offset, point(px(0.0), px(6.0)));
    }

    #[test]
    fn overlay_below_center_anchors_to_trigger_center_with_content_offset() {
        let placement = resolve_tabs_navigation_overlay_placement(
            Some(Bounds::new(point(px(40.0), px(10.0)), size(px(80.0), px(32.0)))),
            TabsNavigationOverlayPlacement::BelowCenter,
            size(px(200.0), px(120.0)),
            px(6.0),
        );

        assert_eq!(placement.anchor, Anchor::TopLeft);
        assert_eq!(placement.position, point(px(80.0), px(42.0)));
        assert_eq!(placement.offset, point(px(-100.0), px(6.0)));
    }
}
