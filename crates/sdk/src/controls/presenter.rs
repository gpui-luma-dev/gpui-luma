use std::sync::Arc;

use gpui::{AnyElement, App, FocusHandle, IntoElement, SharedString, Window, div, prelude::*};

pub struct HostedContent {
    pub element: AnyElement,
    pub focus_handle: Option<FocusHandle>,
}

pub struct Presenter<State> {
    presenter: Arc<dyn for<'a, 'b, 'c> Fn(&'a State, &'b mut Window, &'c mut App) -> HostedContent + 'static>,
}

impl<State> Clone for Presenter<State> {
    fn clone(&self) -> Self {
        Self { presenter: self.presenter.clone() }
    }
}

impl<State> Presenter<State> {
    pub fn new(
        presenter: impl for<'a, 'b, 'c> Fn(&'a State, &'b mut Window, &'c mut App) -> HostedContent + 'static,
    ) -> Self {
        Self { presenter: Arc::new(presenter) }
    }

    pub fn present(&self, state: &State, window: &mut Window, cx: &mut App) -> HostedContent {
        (self.presenter)(state, window, cx)
    }
}

pub trait IntoPresenter<State> {
    fn into_presenter(self) -> Presenter<State>;
}

impl<State> IntoPresenter<State> for Presenter<State> {
    fn into_presenter(self) -> Presenter<State> {
        self
    }
}

impl<State, F> IntoPresenter<State> for F
where
    F: for<'a, 'b, 'c> Fn(&'a State, &'b mut Window, &'c mut App) -> HostedContent + 'static,
{
    fn into_presenter(self) -> Presenter<State> {
        Presenter::new(self)
    }
}

/// The lightweight presenter for control faces (labels, icons, etc.)
pub type ControlPresenter<M> = Arc<dyn Fn(&M, &mut App) -> AnyElement + Send + Sync + 'static>;

/// A trait for builders that can host control content.
pub trait HasPresenter<M> {
    fn set_presenter(&mut self, content: ControlPresenter<M>);

    /// The fluent API for setting custom content.
    fn content<F, E>(mut self, builder: F) -> Self
    where
        F: Fn(&M, &mut App) -> E + Send + Sync + 'static,
        E: IntoElement + 'static,
        Self: Sized,
    {
        self.set_presenter(Arc::new(move |m, cx| builder(m, cx).into_any_element()));
        self
    }

    /// Convenience helper for simple text labels.
    fn label(self, text: impl Into<SharedString>) -> Self
    where
        Self: Sized,
    {
        let text = text.into();
        self.content(move |_, _| div().child(text.clone()))
    }
}
