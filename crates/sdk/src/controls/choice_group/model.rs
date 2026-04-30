use std::sync::Arc;

use gpui::{AppContext, Div, Entity, IntoElement, SharedString, Stateful, div, prelude::*, px};

use super::{ChoiceGroupTemplate, default_choice_group_template, template::template_with_modifier};
use super::control::ChoiceGroupControl;
use crate::controls::button_family::{ButtonKind as ChoiceGroupKind, ButtonSize as ChoiceGroupSize};
use crate::controls::choice_group::{ChoiceGroupItemState, ControlFocusState};
use crate::controls::command::button::{ButtonRenderModel, ButtonTemplate};
use crate::controls::content_presenter::{ControlContent, HasContent};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ChoiceGroupSelectionMode {
    #[default]
    Single,
    Multiple,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ChoiceGroupLayout {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ChoiceGroupStateMode {
    #[default]
    Unmanaged,
    Managed,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ChoiceGroupStylePreset {
    #[default]
    Default,
    IconButton,
    Radio,
}

#[derive(Clone, Debug)]
pub struct ChoiceGroupItem {
    pub(crate) id: SharedString,
    pub(crate) label: SharedString,
    pub(crate) value: SharedString,
    pub(crate) enabled: bool,
}

impl ChoiceGroupItem {
    pub fn new(id: impl Into<SharedString>, value: impl Into<SharedString>) -> Self {
        let id = id.into();
        Self { label: id.clone(), id, value: value.into(), enabled: true }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = value.into();
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn id(&self) -> &SharedString {
        &self.id
    }

    pub fn label_text(&self) -> &SharedString {
        &self.label
    }

    pub fn value_text(&self) -> &SharedString {
        &self.value
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

#[derive(Clone)]
pub struct ChoiceGroupModel {
    pub(crate) id: SharedString,
    pub(crate) items: Vec<ChoiceGroupItem>,

    /// Initial/fallback selected ids set through builder APIs.
    pub(crate) default_selected_ids: Vec<SharedString>,

    /// Externally owned source-of-truth selection.
    pub(crate) managed_selected_ids: Option<Vec<SharedString>>,

    pub(crate) active_id: Option<SharedString>,
    pub(crate) selection_mode: ChoiceGroupSelectionMode,
    pub(crate) layout: ChoiceGroupLayout,
    pub(crate) state_mode: ChoiceGroupStateMode,
    pub(crate) kind: ChoiceGroupKind,
    pub(crate) size: ChoiceGroupSize,
    pub(crate) enabled: bool,
    pub(crate) template: Arc<dyn ChoiceGroupTemplate>,
    pub(crate) content: ChoiceGroupContent,
    pub(crate) item_button_template: Option<ChoiceGroupItemButtonTemplate>,
}

impl ChoiceGroupModel {
    pub fn is_managed(&self) -> bool {
        self.state_mode == ChoiceGroupStateMode::Managed
    }

    pub fn managed_selected_ids(&self) -> Option<&[SharedString]> {
        self.managed_selected_ids.as_deref()
    }

    pub fn managed_selected_id(&self) -> Option<&SharedString> {
        self.managed_selected_ids.as_ref().and_then(|ids| ids.first())
    }

    pub fn default_selected_ids(&self) -> &[SharedString] {
        &self.default_selected_ids
    }

    pub fn default_selected_id(&self) -> Option<&SharedString> {
        self.default_selected_ids.first()
    }

    pub fn effective_selected_ids(&self) -> &[SharedString] {
        self.managed_selected_ids().unwrap_or(self.default_selected_ids.as_slice())
    }

    pub fn effective_selected_id(&self) -> Option<&SharedString> {
        self.effective_selected_ids().first()
    }

    pub fn set_managed_selected_ids<I, S>(&mut self, selected_ids: I)
    where
        I: IntoIterator<Item = S>,
        S: Into<SharedString>,
    {
        self.state_mode = ChoiceGroupStateMode::Managed;
        self.managed_selected_ids = Some(selected_ids.into_iter().map(Into::into).collect());
    }

    pub fn clear_managed_selected_ids(&mut self) {
        self.managed_selected_ids = None;
    }
}

pub struct ChoiceGroupRenderItem<'a> {
    pub id: &'a SharedString,
    pub label: &'a SharedString,
    pub value: &'a SharedString,
    pub selected: bool,
    pub enabled: bool,
    pub position: ChoiceGroupItemPosition,
    pub state: ChoiceGroupItemState,
    pub content_model: ChoiceGroupItemContentModel,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChoiceGroupItemPosition {
    Only,
    First,
    Middle,
    Last,
}

pub struct ChoiceGroupRenderModel<'a> {
    pub id: &'a SharedString,
    pub items: Vec<ChoiceGroupRenderItem<'a>>,
    pub content: &'a ChoiceGroupContent,
    pub item_button_template: Option<&'a ChoiceGroupItemButtonTemplate>,
    pub selected_ids: &'a [SharedString],
    pub active_id: Option<&'a SharedString>,
    pub selection_mode: ChoiceGroupSelectionMode,
    pub layout: ChoiceGroupLayout,
    pub state_mode: ChoiceGroupStateMode,
    pub kind: ChoiceGroupKind,
    pub size: ChoiceGroupSize,
    pub enabled: bool,
    pub focus: ControlFocusState,
}

/// Presenter model passed into `.content(...)`.
#[derive(Clone, Debug)]
pub struct ChoiceGroupItemContentModel {
    pub group_id: SharedString,
    pub item_id: SharedString,
    pub item_label: SharedString,
    pub item_value: SharedString,
    pub selected: bool,
    pub enabled: bool,
    pub position: ChoiceGroupItemPosition,
    pub state: ChoiceGroupItemState,
    pub selection_mode: ChoiceGroupSelectionMode,
    pub layout: ChoiceGroupLayout,
    pub group_enabled: bool,
}

pub type ChoiceGroupContent = ControlContent<ChoiceGroupItemContentModel>;

/// Primary item-template bridge for ChoiceGroup: reuses existing `Button<bool>` templates
/// (for example RadioButton templates) without duplicating visual code.
pub type ChoiceGroupItemButtonRenderModel = ButtonRenderModel<bool>;
pub type ChoiceGroupItemButtonTemplate = Arc<dyn ButtonTemplate<bool>>;

/// Legacy/advanced aliases for content-model-driven button rendering.
pub type ChoiceGroupItemContentButtonRenderModel = ButtonRenderModel<ChoiceGroupItemContentModel>;
pub type ChoiceGroupItemContentButtonTemplate = Arc<dyn ButtonTemplate<ChoiceGroupItemContentModel>>;

pub struct ChoiceGroupBuilder {
    pub(crate) model: ChoiceGroupModel,
}

impl ChoiceGroupBuilder {
    pub fn new(id: impl Into<SharedString>) -> Self {
        let id = id.into();

        Self {
            model: ChoiceGroupModel {
                id: id.clone(),
                items: Vec::new(),
                default_selected_ids: Vec::new(),
                managed_selected_ids: None,
                active_id: None,
                selection_mode: ChoiceGroupSelectionMode::Single,
                layout: ChoiceGroupLayout::Horizontal,
                state_mode: ChoiceGroupStateMode::Unmanaged,
                kind: ChoiceGroupKind::Standard,
                size: ChoiceGroupSize::Md,
                enabled: true,
                template: default_choice_group_template(),
                content: Arc::new(move |m, _| div().child(m.item_label.clone()).into_any_element()),
                item_button_template: None,
            },
        }
    }

    pub fn item(mut self, item: ChoiceGroupItem) -> Self {
        self.model.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = ChoiceGroupItem>) -> Self {
        self.model.items = items.into_iter().collect();
        self
    }

    pub fn selection_mode(mut self, selection_mode: ChoiceGroupSelectionMode) -> Self {
        self.model.selection_mode = selection_mode;
        self
    }

    pub fn single(self) -> Self {
        self.selection_mode(ChoiceGroupSelectionMode::Single)
    }

    pub fn multiple(self) -> Self {
        self.selection_mode(ChoiceGroupSelectionMode::Multiple)
    }

    pub fn layout(mut self, layout: ChoiceGroupLayout) -> Self {
        self.model.layout = layout;
        self
    }

    pub fn horizontal(self) -> Self {
        self.layout(ChoiceGroupLayout::Horizontal)
    }

    pub fn vertical(self) -> Self {
        self.layout(ChoiceGroupLayout::Vertical)
    }

    pub fn style_preset(self, preset: ChoiceGroupStylePreset) -> Self {
        match preset {
            ChoiceGroupStylePreset::Default => self,
            ChoiceGroupStylePreset::IconButton => self.icon_button_style(),
            ChoiceGroupStylePreset::Radio => self.radio_style(),
        }
    }

    /// Convenience preset for the most common editor-toolbar shape:
    /// horizontal, single-select, icon-button-like housing.
    pub fn toolbar_icons(self) -> Self {
        self.single().layout(ChoiceGroupLayout::Horizontal).style_preset(ChoiceGroupStylePreset::IconButton)
    }

    /// Convenience preset for multi-select icon toolbars.
    pub fn toolbar_icons_multiple(self) -> Self {
        self.multiple()
            .layout(ChoiceGroupLayout::Horizontal)
            .style_preset(ChoiceGroupStylePreset::IconButton)
    }

    /// Convenience preset for radio-group semantics and default horizontal layout.
    pub fn radio_group(self) -> Self {
        self.single().horizontal().style_preset(ChoiceGroupStylePreset::Radio)
    }

    /// Base radio styling used by radio-style presets.
    pub fn radio_style(self) -> Self {
        self.kind(ChoiceGroupKind::Standard).with_modifier(|element, model| match model.layout {
            ChoiceGroupLayout::Horizontal => element.gap(px(12.0)),
            ChoiceGroupLayout::Vertical => element.gap(px(0.0)),
        })
    }

    /// Base icon-button styling used by toolbar icon presets.
    pub fn icon_button_style(self) -> Self {
        self.kind(ChoiceGroupKind::Ghost).with_modifier(|element, model| {
            let base = element.rounded_full().gap(px(6.0));

            match model.layout {
                ChoiceGroupLayout::Horizontal => base.px(px(6.0)).py(px(4.0)),
                ChoiceGroupLayout::Vertical => base.px(px(4.0)).py(px(6.0)),
            }
        })
    }

    /// Initial/default selected id.
    /// In managed mode this remains a fallback value.
    pub fn selected(mut self, selected_id: impl Into<SharedString>) -> Self {
        self.model.default_selected_ids = vec![selected_id.into()];
        self
    }

    /// Initial/default selected ids.
    /// In managed mode this remains fallback/default only.
    pub fn selected_ids<I, S>(mut self, selected_ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<SharedString>,
    {
        self.model.default_selected_ids = selected_ids.into_iter().map(Into::into).collect();
        self
    }

    pub fn managed(mut self) -> Self {
        self.model.state_mode = ChoiceGroupStateMode::Managed;
        self
    }

    pub fn unmanaged(mut self) -> Self {
        self.model.state_mode = ChoiceGroupStateMode::Unmanaged;
        self.model.managed_selected_ids = None;
        self
    }

    pub fn managed_selected(mut self, selected_id: impl Into<SharedString>) -> Self {
        self.model.state_mode = ChoiceGroupStateMode::Managed;
        self.model.managed_selected_ids = Some(vec![selected_id.into()]);
        self
    }

    pub fn managed_selected_ids<I, S>(mut self, selected_ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<SharedString>,
    {
        self.model.state_mode = ChoiceGroupStateMode::Managed;
        self.model.managed_selected_ids = Some(selected_ids.into_iter().map(Into::into).collect());
        self
    }

    pub fn kind(mut self, kind: ChoiceGroupKind) -> Self {
        self.model.kind = kind;
        self
    }

    pub fn size(mut self, size: ChoiceGroupSize) -> Self {
        self.model.size = size;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn item_button_template(mut self, template: ChoiceGroupItemButtonTemplate) -> Self {
        self.model.item_button_template = Some(template);
        self
    }

    pub fn item_button_template_factory<F>(mut self, factory: F) -> Self
    where
        F: FnOnce(&ChoiceGroupModel) -> ChoiceGroupItemButtonTemplate,
    {
        self.model.item_button_template = Some(factory(&self.model));
        self
    }

    /// Explicit bool-template variant for readability at call sites.
    pub fn bool_button_template(self, template: ChoiceGroupItemButtonTemplate) -> Self {
        self.item_button_template(template)
    }

    /// Explicit bool-template factory variant for readability at call sites.
    pub fn bool_button_template_factory<F>(self, factory: F) -> Self
    where
        F: FnOnce(&ChoiceGroupModel) -> ChoiceGroupItemButtonTemplate,
    {
        self.item_button_template_factory(factory)
    }

    pub fn clear_item_button_template(mut self) -> Self {
        self.model.item_button_template = None;
        self
    }

    pub fn template(mut self, template: Arc<dyn ChoiceGroupTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn template_factory<F>(mut self, factory: F) -> Self
    where
        F: FnOnce(&ChoiceGroupModel) -> Arc<dyn ChoiceGroupTemplate>,
    {
        self.model.template = factory(&self.model);
        self
    }

    pub fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: for<'a> Fn(Stateful<Div>, &self::ChoiceGroupRenderModel<'a>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.model.template = template_with_modifier(self.model.template.clone(), modifier);
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<ChoiceGroupControl> {
        cx.new(|cx| ChoiceGroupControl::from_builder(self, cx))
    }
}

impl HasContent<ChoiceGroupItemContentModel> for ChoiceGroupBuilder {
    fn set_content(&mut self, content: ChoiceGroupContent) {
        self.model.content = content;
    }
}
