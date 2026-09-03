mod circular;
mod linear;

use std::sync::Arc;

use gpui::{App, Div, Stateful, Window};

use super::ProgressRenderModel;

pub use circular::{CircularProgressTemplate, ThemedProgressTemplate, default_circular_progress_template};
pub use linear::{LinearProgressTemplate, ThemedLinearProgressTemplate, default_linear_progress_template};

pub type ProgressTemplateModifier =
    Box<dyn Fn(Stateful<Div>, &ProgressRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static>;

pub trait ProgressTemplate: Send + Sync {
    fn render(&self, model: &ProgressRenderModel<'_>, window: &mut Window, cx: &mut App) -> Stateful<Div>;
}

struct ModifiedProgressTemplate {
    base: Arc<dyn ProgressTemplate>,
    modifiers: Vec<ProgressTemplateModifier>,
}

impl ModifiedProgressTemplate {
    fn new(base: Arc<dyn ProgressTemplate>) -> Self {
        Self { base, modifiers: Vec::new() }
    }

    fn with_modifier(mut self, modifier: ProgressTemplateModifier) -> Self {
        self.modifiers.push(modifier);
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<Div>, model: &ProgressRenderModel<'_>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = modifier(root, model);
        }
        root
    }
}

pub fn default_progress_template() -> Arc<dyn ProgressTemplate> {
    default_circular_progress_template()
}

pub(super) fn template_with_modifier<F>(template: Arc<dyn ProgressTemplate>, modifier: F) -> Arc<dyn ProgressTemplate>
where
    F: Fn(Stateful<Div>, &ProgressRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
{
    Arc::new(ModifiedProgressTemplate::new(template).with_modifier(Box::new(modifier)))
}

impl ProgressTemplate for ModifiedProgressTemplate {
    fn render(&self, model: &ProgressRenderModel<'_>, window: &mut Window, cx: &mut App) -> Stateful<Div> {
        let root = self.base.render(model, window, cx);
        self.apply_modifiers(root, model)
    }
}

pub(super) fn apply_template_modifiers(
    modifiers: &[ProgressTemplateModifier],
    mut root: Stateful<Div>,
    model: &ProgressRenderModel<'_>,
) -> Stateful<Div> {
    for modifier in modifiers {
        root = modifier(root, model);
    }
    root
}
