use std::sync::Arc;

use gpui::{AnyElement, App, FocusHandle, Window};

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
