//! Controls for browsing and manipulating collections of records.
//!
//! [`listbox`] presents a flat collection, [`table`] presents tabular records,
//! and [`tree_view`] presents a hierarchy. Selection, navigation, and layout are
//! capabilities of these controls; each retains its own implementation.
//!
//! Value pickers remain in [`super::selection`], while controls for moving
//! between destinations or sections belong to [`super::navigation`].
//!
//! The short paths (`controls::table`, `controls::tree_view`, `controls::listbox`)
//! and the previous nested paths remain available as re-exports.

pub mod listbox;
pub mod table;
pub mod tree_view;

#[cfg(test)]
mod tests {
    use super::{listbox, table, tree_view};
    use crate::controls;

    #[test]
    fn existing_imports_refer_to_the_same_collection_types() {
        let table: table::TableBuilder<String> = table::new_typed("table");
        let table: controls::selection::table::TableBuilder<String> = table;
        let _: controls::table::TableBuilder<String> = table;

        let tree: tree_view::TreeViewBuilder<()> = tree_view::new("tree");
        let tree: controls::navigation::tree_view::TreeViewBuilder<()> = tree;
        let _: controls::tree_view::TreeViewBuilder<()> = tree;

        let mode: listbox::SelectionMode = listbox::SelectionMode::Extended;
        let mode: controls::selection::listbox::SelectionMode = mode;
        let _: controls::listbox::SelectionMode = mode;
    }
}
