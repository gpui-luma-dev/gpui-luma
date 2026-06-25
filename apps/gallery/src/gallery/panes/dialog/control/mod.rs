#[allow(clippy::module_inception)]
mod control;
mod model;
mod template;

pub(in crate::gallery) use control::Dialog;
pub(in crate::gallery) use template::default_dialog_template;
