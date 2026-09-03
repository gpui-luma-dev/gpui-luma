//! Text family: fields plus the shared editing engine.

mod editing;
mod state;

pub mod textarea;
pub mod textfield;

pub(crate) use editing::{FocusNavigation, handle_key_down, word_cluster_range};
pub(crate) use state::{EditableTextPolicy, TextSelectionState, select_all};
