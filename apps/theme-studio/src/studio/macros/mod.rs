//! Layout and form helpers copied from `apps/gallery/src/gallery/macros` (available via `#[macro_use]` on `studio`).

#![allow(unused_macros)]

macro_rules! declare_form {
    (
        $(#[$meta:meta])*
        $vis:vis struct $struct_name:ident {
            controls: {
                $(
                    $ctrl_name:ident : $ctrl_ty:ty = $builder:expr
                    $(=> $event_ty:ty |$this:ident, $event:ident, $cx:ident| $callback:block)?
                ),* $(,)?
            },
            args: {
                $( $arg_name:ident : $arg_ty:ty ),* $(,)?
            },
            fields: {
                $( $field_name:ident : $field_ty:ty = $field_init:expr ),* $(,)?
            }
        }
    ) => {
        $(#[$meta])*
        $vis struct $struct_name {
            $( $ctrl_name: $ctrl_ty, )*
            $( #[allow(dead_code)] $arg_name: $arg_ty, )*
            $( $field_name: $field_ty, )*
            _subscriptions: ::std::vec::Vec<::gpui::Subscription>,
        }

        impl $struct_name {
            $vis fn new(
                cx: &mut ::gpui::Context<Self>,
                $( $arg_name: $arg_ty ),*
            ) -> Self {
                $(
                    let $ctrl_name = $builder.spawn(cx);
                )*

                $(
                    let $field_name = $field_init;
                )*

                let mut _subscriptions = ::std::vec::Vec::new();
                $(
                    declare_form!(@push_subscription _subscriptions, cx, $ctrl_name $(=> $event_ty |$this, $event, $cx| $callback)?);
                )*

                Self {
                    $( $ctrl_name, )*
                    $( $arg_name, )*
                    $( $field_name, )*
                    _subscriptions,
                }
            }
        }
    };

    (@push_subscription $subscriptions:ident, $context:ident, $control:ident) => {};

    (@push_subscription $subscriptions:ident, $context:ident, $control:ident => $event_ty:ty |$this:ident, $event:ident, $cx:ident| $callback:block) => {
        $subscriptions.push($context.subscribe(&$control, |$this, _, $event: &$event_ty, $cx| $callback));
    };
}

macro_rules! vstack {
    (
        gap=$gap:tt align=$align:ident;
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex().flex_col().gap(::gpui::px($gap as f32));
        panel = vstack!(@align_items panel, $align);
        $( panel = panel.child($child); )*
        panel
    }};

    (
        gap=$gap:tt;
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex().flex_col().gap(::gpui::px($gap as f32));
        $( panel = panel.child($child); )*
        panel
    }};

    (
        align=$align:ident;
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex().flex_col();
        panel = vstack!(@align_items panel, $align);
        $( panel = panel.child($child); )*
        panel
    }};

    (
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex().flex_col();
        $( panel = panel.child($child); )*
        panel
    }};

    (@align_items $panel:ident, center) => { $panel.items_center() };
    (@align_items $panel:ident, start) => { $panel.items_start() };
    (@align_items $panel:ident, end) => { $panel.items_end() };
}

macro_rules! hstack {
    (
        gap=$gap:tt align=$align:ident;
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex().gap(::gpui::px($gap as f32));
        panel = hstack!(@align_items panel, $align);
        $( panel = panel.child($child); )*
        panel
    }};

    (
        gap=$gap:tt;
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex().gap(::gpui::px($gap as f32));
        $( panel = panel.child($child); )*
        panel
    }};

    (
        align=$align:ident;
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex();
        panel = hstack!(@align_items panel, $align);
        $( panel = panel.child($child); )*
        panel
    }};

    (
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex();
        $( panel = panel.child($child); )*
        panel
    }};

    (@align_items $panel:ident, center) => { $panel.items_center() };
    (@align_items $panel:ident, start) => { $panel.items_start() };
    (@align_items $panel:ident, end) => { $panel.items_end() };
}

macro_rules! wrappanel {
    // 1. Full parameters: orientation, gap, and cross-axis alignment
    (
        orientation=$orient:ident gap=$gap:tt align=$align:ident;
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex().flex_wrap().gap(::gpui::px($gap as f32));
        panel = wrappanel!(@orient panel, $orient);
        panel = wrappanel!(@align_items panel, $align);
        $( panel = panel.child($child); )*
        panel
    }};

    // 2. Orientation and gap
    (
        orientation=$orient:ident gap=$gap:tt;
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex().flex_wrap().gap(::gpui::px($gap as f32));
        panel = wrappanel!(@orient panel, $orient);
        $( panel = panel.child($child); )*
        panel
    }};

    // 3. Gap and alignment (defaults to horizontal)
    (
        gap=$gap:tt align=$align:ident;
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex().flex_wrap().gap(::gpui::px($gap as f32));
        panel = wrappanel!(@align_items panel, $align);
        $( panel = panel.child($child); )*
        panel
    }};

    // 4. Orientation only
    (
        orientation=$orient:ident;
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex().flex_wrap();
        panel = wrappanel!(@orient panel, $orient);
        $( panel = panel.child($child); )*
        panel
    }};

    // 5. Gap only (defaults to horizontal)
    (
        gap=$gap:tt;
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex().flex_wrap().gap(::gpui::px($gap as f32));
        $( panel = panel.child($child); )*
        panel
    }};

    // 6. Simple wrapper (defaults to horizontal wrap, no gap)
    (
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex().flex_wrap();
        $( panel = panel.child($child); )*
        panel
    }};

    // Helper macro rules to parse orientation
    (@orient $panel:ident, horizontal) => { $panel }; // GPUI default flex layout is row-based
    (@orient $panel:ident, vertical) => { $panel.flex_col() };
    (@orient $panel:ident, row) => { $panel };
    (@orient $panel:ident, col) => { $panel.flex_col() };

    // Helper macro rules to parse alignments
    (@align_items $panel:ident, center) => { $panel.items_center() };
    (@align_items $panel:ident, start) => { $panel.items_start() };
    (@align_items $panel:ident, end) => { $panel.items_end() };
}

macro_rules! flow {
    (
        gap=$gap:tt align=$align:ident;
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex().flex_wrap().gap(::gpui::px($gap as f32));
        panel = flow!(@align_items panel, $align);
        $( panel = panel.child($child); )*
        panel
    }};

    (
        gap=$gap:tt;
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex().flex_wrap().gap(::gpui::px($gap as f32));
        $( panel = panel.child($child); )*
        panel
    }};

    (
        align=$align:ident;
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex().flex_wrap();
        panel = flow!(@align_items panel, $align);
        $( panel = panel.child($child); )*
        panel
    }};

    (
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex().flex_wrap();
        $( panel = panel.child($child); )*
        panel
    }};

    (@align_items $panel:ident, center) => { $panel.items_center() };
    (@align_items $panel:ident, start) => { $panel.items_start() };
    (@align_items $panel:ident, end) => { $panel.items_end() };
}
