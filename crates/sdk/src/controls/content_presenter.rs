use std::sync::Arc;

use gpui::{AnyElement, App, FocusHandle, IntoElement, SharedString, Window, div, prelude::*};

pub struct HostedContent {
    pub element: AnyElement,
    pub focus_handle: Option<FocusHandle>,
}

pub struct ContentPresenter<State> {
    presenter: Arc<dyn for<'a, 'b, 'c> Fn(&'a State, &'b mut Window, &'c mut App) -> HostedContent + 'static>,
}

impl<State> Clone for ContentPresenter<State> {
    fn clone(&self) -> Self {
        Self { presenter: self.presenter.clone() }
    }
}

impl<State> ContentPresenter<State> {
    pub fn new(
        presenter: impl for<'a, 'b, 'c> Fn(&'a State, &'b mut Window, &'c mut App) -> HostedContent + 'static,
    ) -> Self {
        Self { presenter: Arc::new(presenter) }
    }

    pub fn present(&self, state: &State, window: &mut Window, cx: &mut App) -> HostedContent {
        (self.presenter)(state, window, cx)
    }
}

pub trait IntoContentPresenter<State> {
    fn into_content_presenter(self) -> ContentPresenter<State>;
}

impl<State> IntoContentPresenter<State> for ContentPresenter<State> {
    fn into_content_presenter(self) -> ContentPresenter<State> {
        self
    }
}

impl<State, F> IntoContentPresenter<State> for F
where
    F: for<'a, 'b, 'c> Fn(&'a State, &'b mut Window, &'c mut App) -> HostedContent + 'static,
{
    fn into_content_presenter(self) -> ContentPresenter<State> {
        ContentPresenter::new(self)
    }
}

/// The lightweight presenter for control faces (labels, icons, etc.)
pub type ControlContent<M> = Arc<dyn Fn(&M, &mut App) -> AnyElement + Send + Sync + 'static>;

/// A trait for builders that can host control content.
pub trait HasContent<M> {
    fn set_content(&mut self, content: ControlContent<M>);

    /// The fluent API for setting custom content.
    fn content<F, E>(mut self, builder: F) -> Self
    where
        F: Fn(&M, &mut App) -> E + Send + Sync + 'static,
        E: IntoElement + 'static,
        Self: Sized,
    {
        self.set_content(Arc::new(move |m, cx| builder(m, cx).into_any_element()));
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
