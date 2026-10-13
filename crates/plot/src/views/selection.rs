use std::rc::Rc;
use gpui::App;
/// Observation activation; persistent selection belongs to the host.
#[derive(Clone, Debug)]
pub enum SelectionEvent<T> {
    HoverChanged { target: Option<T> },
    Select { target: T },
}
type Callback<T> = dyn Fn(SelectionEvent<T>, &mut App);
#[derive(Clone)]
pub(crate) struct SelectionEvents<T>(pub Rc<Callback<T>>);
impl<T> SelectionEvents<T> {
    pub fn new(callback: impl Fn(SelectionEvent<T>, &mut App) + 'static) -> Self {
        Self(Rc::new(callback))
    }
}
