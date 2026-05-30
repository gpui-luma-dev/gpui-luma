#[macro_export]
macro_rules! column {
    ($header:expr, width = $width:expr => $render_fn:expr) => {
        $crate::controls::list_view::ListViewColumn::fixed($header, ($width) as f32, $render_fn)
    };
    ($header:expr => $render_fn:expr) => {
        $crate::controls::list_view::ListViewColumn::fill($header, $render_fn)
    };
}

#[macro_export]
macro_rules! column_text {
    ($header:expr, width = $width:expr => $render_fn:expr) => {
        $crate::controls::list_view::ListViewColumn::fixed(
            $header,
            ($width) as f32,
            $crate::controls::list_view::default_text_column_template($render_fn),
        )
    };
    ($header:expr => $render_fn:expr) => {
        $crate::controls::list_view::ListViewColumn::fill(
            $header,
            $crate::controls::list_view::default_text_column_template($render_fn),
        )
    };
}

#[macro_export]
macro_rules! column_muted {
    ($header:expr, width = $width:expr => $render_fn:expr) => {
        $crate::controls::list_view::ListViewColumn::fixed(
            $header,
            ($width) as f32,
            $crate::controls::list_view::default_muted_column_template($render_fn),
        )
    };
    ($header:expr => $render_fn:expr) => {
        $crate::controls::list_view::ListViewColumn::fill(
            $header,
            $crate::controls::list_view::default_muted_column_template($render_fn),
        )
    };
}

#[macro_export]
macro_rules! column_emphasis {
    ($header:expr, width = $width:expr => $render_fn:expr) => {
        $crate::controls::list_view::ListViewColumn::fixed(
            $header,
            ($width) as f32,
            $crate::controls::list_view::default_emphasis_column_template($render_fn),
        )
    };
    ($header:expr => $render_fn:expr) => {
        $crate::controls::list_view::ListViewColumn::fill(
            $header,
            $crate::controls::list_view::default_emphasis_column_template($render_fn),
        )
    };
}

#[macro_export]
macro_rules! column_numeric {
    ($header:expr, width = $width:expr => $render_fn:expr) => {
        $crate::controls::list_view::ListViewColumn::fixed(
            $header,
            ($width) as f32,
            $crate::controls::list_view::default_numeric_column_template($render_fn),
        )
    };
    ($header:expr => $render_fn:expr) => {
        $crate::controls::list_view::ListViewColumn::fill(
            $header,
            $crate::controls::list_view::default_numeric_column_template($render_fn),
        )
    };
}

#[macro_export]
macro_rules! list_view {
    // Branch 1: grid_view + row_template together
    (
        $( radix = $radix:expr; )?
        id = $id:expr;
        items = $items:expr;
        $( selection = $selection:expr; )?
        $( selected_index = $selected_index:expr; )?
        $( active_index = $active_index:expr; )?
        $( row_label = |$label_row:ident| $label_body:expr; )?
        $( row_enabled = |$enabled_row:ident| $enabled_body:expr; )?
        $( visible_rows = $visible_rows:expr; )?
        $( page_size = $page_size:expr; )?
        $( scroll_snap = $scroll_snap:expr; )?
        $( paging_toolbar_template = $paging_toolbar_template:expr; )?
        grid_view = { $($col:expr),* $(,)? };
        row_template = |$model:ident, $cells:ident, $win:ident, $cx:ident| $body:expr $(;)?
    ) => {{
        let builder = $crate::controls::list_view::new_typed($id).items($items);
        $( let builder = builder.theme($radix.list_view_theme()); )?
        $( let builder = builder.selection_mode($selection); )?
        $( let builder = builder.selected_index($selected_index); )?
        $( let builder = builder.active_index($active_index); )?
        $(
            let builder = builder.row_label(move |$label_row: &_| $label_body);
        )?
        $(
            let builder = builder.row_enabled(move |$enabled_row: &_| $enabled_body);
        )?
        let builder = builder.grid_view(vec![$($col),*]);
        let builder = builder.row_template(std::sync::Arc::new(move |$model, $cells, $win, $cx| {
            $body
        }));
        $( let builder = builder.visible_rows($visible_rows); )?
        $( let builder = builder.paged($page_size); )?
        $( let builder = builder.scroll_snap($scroll_snap); )?
        $( let builder = builder.paging_toolbar_template(std::sync::Arc::new($paging_toolbar_template)); )?
        builder
    }};

    // Branch 2: grid_view only (uses internal default row template)
    (
        $( radix = $radix:expr; )?
        id = $id:expr;
        items = $items:expr;
        $( selection = $selection:expr; )?
        $( selected_index = $selected_index:expr; )?
        $( active_index = $active_index:expr; )?
        $( row_label = |$label_row:ident| $label_body:expr; )?
        $( row_enabled = |$enabled_row:ident| $enabled_body:expr; )?
        $( visible_rows = $visible_rows:expr; )?
        $( page_size = $page_size:expr; )?
        $( scroll_snap = $scroll_snap:expr; )?
        $( paging_toolbar_template = $paging_toolbar_template:expr; )?
        grid_view = { $($col:expr),* $(,)? } $(;)?
    ) => {{
        let builder = $crate::controls::list_view::new_typed($id).items($items);
        $( let builder = builder.theme($radix.list_view_theme()); )?
        $( let builder = builder.selection_mode($selection); )?
        $( let builder = builder.selected_index($selected_index); )?
        $( let builder = builder.active_index($active_index); )?
        $(
            let builder = builder.row_label(move |$label_row: &_| $label_body);
        )?
        $(
            let builder = builder.row_enabled(move |$enabled_row: &_| $enabled_body);
        )?
        let builder = builder.grid_view(vec![$($col),*]);
        $( let builder = builder.visible_rows($visible_rows); )?
        $( let builder = builder.paged($page_size); )?
        $( let builder = builder.scroll_snap($scroll_snap); )?
        $( let builder = builder.paging_toolbar_template(std::sync::Arc::new($paging_toolbar_template)); )?
        builder
    }};

    // Branch 3: row_template only
    (
        $( radix = $radix:expr; )?
        id = $id:expr;
        items = $items:expr;
        $( selection = $selection:expr; )?
        $( selected_index = $selected_index:expr; )?
        $( active_index = $active_index:expr; )?
        $( row_label = |$label_row:ident| $label_body:expr; )?
        $( row_enabled = |$enabled_row:ident| $enabled_body:expr; )?
        $( visible_rows = $visible_rows:expr; )?
        $( page_size = $page_size:expr; )?
        $( scroll_snap = $scroll_snap:expr; )?
        $( paging_toolbar_template = $paging_toolbar_template:expr; )?
        row_template = |$model:ident, $cells:ident, $win:ident, $cx:ident| $body:expr $(;)?
    ) => {{
        let builder = $crate::controls::list_view::new_typed($id).items($items);
        $( let builder = builder.theme($radix.list_view_theme()); )?
        $( let builder = builder.selection_mode($selection); )?
        $( let builder = builder.selected_index($selected_index); )?
        $( let builder = builder.active_index($active_index); )?
        $(
            let builder = builder.row_label(move |$label_row: &_| $label_body);
        )?
        $(
            let builder = builder.row_enabled(move |$enabled_row: &_| $enabled_body);
        )?
        let builder = builder.row_template(std::sync::Arc::new(move |$model, $cells, $win, $cx| {
            $body
        }));
        $( let builder = builder.visible_rows($visible_rows); )?
        $( let builder = builder.paged($page_size); )?
        $( let builder = builder.scroll_snap($scroll_snap); )?
        $( let builder = builder.paging_toolbar_template(std::sync::Arc::new($paging_toolbar_template)); )?
        builder
    }};
}
