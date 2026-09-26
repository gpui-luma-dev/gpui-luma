#[macro_export]
macro_rules! column {
    ($header:expr, width = $width:expr => $render_fn:expr) => {
        $crate::controls::table::TableColumn::fixed($header, ($width) as f32, $render_fn)
    };
    ($header:expr => $render_fn:expr) => {
        $crate::controls::table::TableColumn::fill($header, $render_fn)
    };
}

#[macro_export]
macro_rules! column_text {
    ($header:expr, width = $width:expr => $render_fn:expr) => {
        $crate::controls::table::TableColumn::fixed(
            $header,
            ($width) as f32,
            $crate::controls::table::default_text_column_template($render_fn),
        )
    };
    ($header:expr => $render_fn:expr) => {
        $crate::controls::table::TableColumn::fill(
            $header,
            $crate::controls::table::default_text_column_template($render_fn),
        )
    };
}

#[macro_export]
macro_rules! column_muted {
    ($header:expr, width = $width:expr => $render_fn:expr) => {
        $crate::controls::table::TableColumn::fixed(
            $header,
            ($width) as f32,
            $crate::controls::table::default_muted_column_template($render_fn),
        )
    };
    ($header:expr => $render_fn:expr) => {
        $crate::controls::table::TableColumn::fill(
            $header,
            $crate::controls::table::default_muted_column_template($render_fn),
        )
    };
}

#[macro_export]
macro_rules! column_emphasis {
    ($header:expr, width = $width:expr => $render_fn:expr) => {
        $crate::controls::table::TableColumn::fixed(
            $header,
            ($width) as f32,
            $crate::controls::table::default_emphasis_column_template($render_fn),
        )
    };
    ($header:expr => $render_fn:expr) => {
        $crate::controls::table::TableColumn::fill(
            $header,
            $crate::controls::table::default_emphasis_column_template($render_fn),
        )
    };
}

#[macro_export]
macro_rules! column_numeric {
    ($header:expr, width = $width:expr => $render_fn:expr) => {
        $crate::controls::table::TableColumn::fixed(
            $header,
            ($width) as f32,
            $crate::controls::table::default_numeric_column_template($render_fn),
        )
    };
    ($header:expr => $render_fn:expr) => {
        $crate::controls::table::TableColumn::fill(
            $header,
            $crate::controls::table::default_numeric_column_template($render_fn),
        )
    };
}

#[macro_export]
macro_rules! table {
    // Branch 1: grid_view + row_template together
    (
        $( table_theme = $table_theme:expr; )?
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
        grid_view = { $($col:expr),* $(,)? };
        row_template = |$model:ident, $cells:ident, $win:ident, $cx:ident| $body:expr $(;)?
    ) => {{
        let builder = $crate::controls::table::new_typed($id).items($items);
        $( let builder = builder.theme($table_theme); )?
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
        let builder = builder.with_row_template(move |$model, $cells, $win, $cx| {
            $body
        });
        $( let builder = builder.visible_rows($visible_rows); )?
        $( let builder = builder.paged($page_size); )?
        $( let builder = builder.scroll_snap($scroll_snap); )?
        builder
    }};

    // Branch 2: grid_view only (uses internal default row template)
    (
        $( table_theme = $table_theme:expr; )?
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
        grid_view = { $($col:expr),* $(,)? } $(;)?
    ) => {{
        let builder = $crate::controls::table::new_typed($id).items($items);
        $( let builder = builder.theme($table_theme); )?
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
        builder
    }};

    // Branch 3: row_template only
    (
        $( table_theme = $table_theme:expr; )?
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
        row_template = |$model:ident, $cells:ident, $win:ident, $cx:ident| $body:expr $(;)?
    ) => {{
        let builder = $crate::controls::table::new_typed($id).items($items);
        $( let builder = builder.theme($table_theme); )?
        $( let builder = builder.selection_mode($selection); )?
        $( let builder = builder.selected_index($selected_index); )?
        $( let builder = builder.active_index($active_index); )?
        $(
            let builder = builder.row_label(move |$label_row: &_| $label_body);
        )?
        $(
            let builder = builder.row_enabled(move |$enabled_row: &_| $enabled_body);
        )?
        let builder = builder.with_row_template(move |$model, $cells, $win, $cx| {
            $body
        });
        $( let builder = builder.visible_rows($visible_rows); )?
        $( let builder = builder.paged($page_size); )?
        $( let builder = builder.scroll_snap($scroll_snap); )?
        builder
    }};
}

#[macro_export]
macro_rules! scrolling_table {
    // grid_view + row_template
    (
        table_theme = $table_theme:expr;
        id = $id:expr;
        items = $items:expr;
        $( selection = $selection:expr; )?
        $( selected_index = $selected_index:expr; )?
        $( active_index = $active_index:expr; )?
        $( row_label = |$label_row:ident| $label_body:expr; )?
        $( row_enabled = |$enabled_row:ident| $enabled_body:expr; )?
        $( visible_rows = $visible_rows:expr; )?
        $( scroll_snap = $scroll_snap:expr; )?
        grid_view = { $($col:expr),* $(,)? };
        row_template = |$model:ident, $cells:ident, $win:ident, $cx:ident| $body:expr $(;)?
    ) => {{
        let builder = $crate::controls::table::new_typed($id).items($items);
        let builder = builder.theme($table_theme);
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
        let builder = builder.with_row_template(move |$model, $cells, $win, $cx| {
            $body
        });
        $( let builder = builder.visible_rows($visible_rows); )?
        $( let builder = builder.scroll_snap($scroll_snap); )?
        builder
    }};

    // grid_view only
    (
        table_theme = $table_theme:expr;
        id = $id:expr;
        items = $items:expr;
        $( selection = $selection:expr; )?
        $( selected_index = $selected_index:expr; )?
        $( active_index = $active_index:expr; )?
        $( row_label = |$label_row:ident| $label_body:expr; )?
        $( row_enabled = |$enabled_row:ident| $enabled_body:expr; )?
        $( visible_rows = $visible_rows:expr; )?
        $( scroll_snap = $scroll_snap:expr; )?
        grid_view = { $($col:expr),* $(,)? } $(;)?
    ) => {{
        let builder = $crate::controls::table::new_typed($id).items($items);
        let builder = builder.theme($table_theme);
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
        $( let builder = builder.scroll_snap($scroll_snap); )?
        builder
    }};
}

#[macro_export]
macro_rules! paging_table {
    // grid_view + row_template
    (
        table_theme = $table_theme:expr;
        id = $id:expr;
        items = $items:expr;
        page_size = $page_size:expr;
        $( pager = $pager:expr; )?
        $( selection = $selection:expr; )?
        $( selected_index = $selected_index:expr; )?
        $( active_index = $active_index:expr; )?
        $( row_label = |$label_row:ident| $label_body:expr; )?
        $( row_enabled = |$enabled_row:ident| $enabled_body:expr; )?
        $( visible_rows = $visible_rows:expr; )?
        grid_view = { $($col:expr),* $(,)? };
        row_template = |$model:ident, $cells:ident, $rwin:ident, $rcx:ident| $body:expr $(;)?
    ) => {{
        let builder = $crate::controls::table::new_typed($id).items($items);
        let builder = builder.theme($table_theme);
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
        let builder = builder.with_row_template(move |$model, $cells, $rwin, $rcx| {
            $body
        });
        $( let builder = builder.visible_rows($visible_rows); )?

        let pager_builder = $crate::controls::pager::new(format!("{}-pager", $id))
            .style($crate::controls::pager::PagerStyle::MinimalEdge)
            .page_size($page_size);
        $( let pager_builder = $pager; )?

        $crate::controls::table::PagingTableBuilder::new(
            builder.paged($page_size),
            pager_builder,
        )
    }};

    // grid_view only
    (
        table_theme = $table_theme:expr;
        id = $id:expr;
        items = $items:expr;
        page_size = $page_size:expr;
        $( pager = $pager:expr; )?
        $( selection = $selection:expr; )?
        $( selected_index = $selected_index:expr; )?
        $( active_index = $active_index:expr; )?
        $( row_label = |$label_row:ident| $label_body:expr; )?
        $( row_enabled = |$enabled_row:ident| $enabled_body:expr; )?
        $( visible_rows = $visible_rows:expr; )?
        grid_view = { $($col:expr),* $(,)? } $(;)?
    ) => {{
        let builder = $crate::controls::table::new_typed($id).items($items);
        let builder = builder.theme($table_theme);
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

        let pager_builder = $crate::controls::pager::new(format!("{}-pager", $id))
            .style($crate::controls::pager::PagerStyle::MinimalEdge)
            .page_size($page_size);
        $( let pager_builder = $pager; )?

        $crate::controls::table::PagingTableBuilder::new(
            builder.paged($page_size),
            pager_builder,
        )
    }};
}
