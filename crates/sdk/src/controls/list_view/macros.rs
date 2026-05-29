#[macro_export]
macro_rules! column {
    ($header:expr, width = $width:expr => |$row:ident| $body:expr) => {
        $crate::controls::list_view::ListViewColumn::fixed($header, ($width) as f32, move |$row: &_| $body)
    };
    ($header:expr => |$row:ident| $body:expr) => {
        $crate::controls::list_view::ListViewColumn::fill($header, move |$row: &_| $body)
    };
}

#[macro_export]
macro_rules! list_view {
    (
        $( radix = $radix:expr; )?
        id = $id:expr;
        items = $items:expr;
        $( selection = $selection:expr; )?
        $( selected_index = $selected_index:expr; )?
        $( active_index = $active_index:expr; )?
        $( row_label = |$label_row:ident| $label_body:expr; )?
        $( row_enabled = |$enabled_row:ident| $enabled_body:expr; )?
        grid_view = {
            $( column!($header:expr, width = $width:expr => |$column_row:ident| $column_body:expr) ),+ $(,)?
        };
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
        let builder = builder
            .grid_columns()
            $(.column_fixed($header, ($width) as f32, move |$column_row| $column_body))+
            .finish();
        builder
    }};
    (
        $( radix = $radix:expr; )?
        id = $id:expr;
        items = $items:expr;
        $( selection = $selection:expr; )?
        $( selected_index = $selected_index:expr; )?
        $( active_index = $active_index:expr; )?
        $( row_label = |$label_row:ident| $label_body:expr; )?
        $( row_enabled = |$enabled_row:ident| $enabled_body:expr; )?
        $( row_template = |$model:ident, $row:ident, $win:ident, $cx:ident| $body:expr; )?
        $( header_template = |$hdr_model:ident, $hdr_win:ident, $hdr_cx:ident| $hdr_body:expr; )?
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
        $(
            let builder = builder.with_row_template(move |$model, $win, $cx| {
                let $row = $model.row;
                $body
            });
        )?
        $(
            let builder = builder.with_header_template(move |$hdr_model, $hdr_win, $hdr_cx| {
                $hdr_body
            });
        )?
        builder
    }};
}
