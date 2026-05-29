#[macro_export]
macro_rules! column {
    ($header:expr, width = $width:expr => |$item:ident| $body:expr) => {
        $crate::controls::list_view::ListViewColumn::fixed($header, ($width) as f32, move |$item: &_| $body)
    };
    ($header:expr => |$item:ident| $body:expr) => {
        $crate::controls::list_view::ListViewColumn::fill($header, move |$item: &_| $body)
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
        $( item_label = |$label_item:ident| $label_body:expr; )?
        $( item_enabled = |$enabled_item:ident| $enabled_body:expr; )?
        grid_view = {
            $( column!($header:expr, width = $width:expr => |$column_item:ident| $column_body:expr) ),+ $(,)?
        };
    ) => {{
        let builder = $crate::controls::list_view::new_typed($id).items($items);
        $( let builder = builder.theme($radix.list_view_theme()); )?
        $( let builder = builder.selection_mode($selection); )?
        $( let builder = builder.selected_index($selected_index); )?
        $( let builder = builder.active_index($active_index); )?
        $(
            let builder = builder.item_label(move |$label_item: &_| $label_body);
        )?
        $(
            let builder = builder.item_enabled(move |$enabled_item: &_| $enabled_body);
        )?
        let builder = builder
            .grid_columns()
            $(.column_fixed($header, ($width) as f32, move |$column_item| $column_body))+
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
        $( item_label = |$label_item:ident| $label_body:expr; )?
        $( item_enabled = |$enabled_item:ident| $enabled_body:expr; )?
        $( item_template = |$model:ident, $item:ident, $win:ident, $cx:ident| $body:expr; )?
        $( header_template = |$hdr_model:ident, $hdr_win:ident, $hdr_cx:ident| $hdr_body:expr; )?
    ) => {{
        let builder = $crate::controls::list_view::new_typed($id).items($items);
        $( let builder = builder.theme($radix.list_view_theme()); )?
        $( let builder = builder.selection_mode($selection); )?
        $( let builder = builder.selected_index($selected_index); )?
        $( let builder = builder.active_index($active_index); )?
        $(
            let builder = builder.item_label(move |$label_item: &_| $label_body);
        )?
        $(
            let builder = builder.item_enabled(move |$enabled_item: &_| $enabled_body);
        )?
        $(
            let builder = builder.with_item_template(move |$model, $win, $cx| {
                let $item = $model.item;
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
