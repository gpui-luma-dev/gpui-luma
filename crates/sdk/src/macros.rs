//! Layout and form helper macros shared across apps.

/// Spawns a vertical stack flexbox layout with custom gap and cross-axis alignment.
#[macro_export]
macro_rules! vstack {
    // 1. Gap, cross-axis alignment, and main-axis justification
    (
        gap=$gap:tt align=$align:ident justify=$justify:ident;
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex().flex_col().gap(::gpui::px($gap as f32));
        panel = $crate::vstack!(@align_items panel, $align);
        panel = $crate::vstack!(@justify_content panel, $justify);
        $( panel = panel.child($child); )*
        panel
    }};

    // 2. Gap and cross-axis alignment
    (
        gap=$gap:tt align=$align:ident;
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex().flex_col().gap(::gpui::px($gap as f32));
        panel = $crate::vstack!(@align_items panel, $align);
        $( panel = panel.child($child); )*
        panel
    }};

    // 3. Gap and main-axis justification
    (
        gap=$gap:tt justify=$justify:ident;
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex().flex_col().gap(::gpui::px($gap as f32));
        panel = $crate::vstack!(@justify_content panel, $justify);
        $( panel = panel.child($child); )*
        panel
    }};

    // 4. Gap only
    (
        gap=$gap:tt;
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex().flex_col().gap(::gpui::px($gap as f32));
        $( panel = panel.child($child); )*
        panel
    }};

    // 5. Cross-axis alignment only
    (
        align=$align:ident;
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex().flex_col();
        panel = $crate::vstack!(@align_items panel, $align);
        $( panel = panel.child($child); )*
        panel
    }};

    // 6. Simple column wrap
    (
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex().flex_col();
        $( panel = panel.child($child); )*
        panel
    }};

    // Internal resolution rules
    (@align_items $panel:ident, center) => { $panel.items_center() };
    (@align_items $panel:ident, start) => { $panel.items_start() };
    (@align_items $panel:ident, end) => { $panel.items_end() };

    (@justify_content $panel:ident, start) => { $panel.justify_start() };
    (@justify_content $panel:ident, center) => { $panel.justify_center() };
    (@justify_content $panel:ident, end) => { $panel.justify_end() };
    (@justify_content $panel:ident, between) => { $panel.justify_between() };
    (@justify_content $panel:ident, around) => { $panel.justify_around() };
    (@justify_content $panel:ident, evenly) => { $panel.justify_evenly() };
}

/// Spawns a horizontal stack flexbox layout with custom gap, alignment, and justification.
#[macro_export]
macro_rules! hstack {
    // 1. Gap, cross-axis alignment, and main-axis justification
    (
        gap=$gap:tt align=$align:ident justify=$justify:ident;
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex().gap(::gpui::px($gap as f32));
        panel = $crate::hstack!(@align_items panel, $align);
        panel = $crate::hstack!(@justify_content panel, $justify);
        $( panel = panel.child($child); )*
        panel
    }};

    // 2. Gap and cross-axis alignment
    (
        gap=$gap:tt align=$align:ident;
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex().gap(::gpui::px($gap as f32));
        panel = $crate::hstack!(@align_items panel, $align);
        $( panel = panel.child($child); )*
        panel
    }};

    // 3. Gap and main-axis justification
    (
        gap=$gap:tt justify=$justify:ident;
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex().gap(::gpui::px($gap as f32));
        panel = $crate::hstack!(@justify_content panel, $justify);
        $( panel = panel.child($child); )*
        panel
    }};

    // 4. Gap only
    (
        gap=$gap:tt;
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex().gap(::gpui::px($gap as f32));
        $( panel = panel.child($child); )*
        panel
    }};

    // 5. Cross-axis alignment only
    (
        align=$align:ident;
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex();
        panel = $crate::hstack!(@align_items panel, $align);
        $( panel = panel.child($child); )*
        panel
    }};

    // 6. Simple row wrap
    (
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex();
        $( panel = panel.child($child); )*
        panel
    }};

    // Internal resolution rules
    (@align_items $panel:ident, center) => { $panel.items_center() };
    (@align_items $panel:ident, start) => { $panel.items_start() };
    (@align_items $panel:ident, end) => { $panel.items_end() };

    (@justify_content $panel:ident, start) => { $panel.justify_start() };
    (@justify_content $panel:ident, center) => { $panel.justify_center() };
    (@justify_content $panel:ident, end) => { $panel.justify_end() };
    (@justify_content $panel:ident, between) => { $panel.justify_between() };
    (@justify_content $panel:ident, around) => { $panel.justify_around() };
    (@justify_content $panel:ident, evenly) => { $panel.justify_evenly() };
}

/// Spawns a flow wrapping flex layout (similar to a WPF WrapPanel).
#[macro_export]
macro_rules! wrappanel {
    // 1. Full parameters: orientation, gap, and cross-axis alignment
    (
        orientation=$orient:ident gap=$gap:tt align=$align:ident;
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex().flex_wrap().gap(::gpui::px($gap as f32));
        panel = $crate::wrappanel!(@orient panel, $orient);
        panel = $crate::wrappanel!(@align_items panel, $align);
        $( panel = panel.child($child); )*
        panel
    }};

    // 2. Orientation and gap
    (
        orientation=$orient:ident gap=$gap:tt;
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex().flex_wrap().gap(::gpui::px($gap as f32));
        panel = $crate::wrappanel!(@orient panel, $orient);
        $( panel = panel.child($child); )*
        panel
    }};

    // 3. Gap and alignment (defaults to horizontal)
    (
        gap=$gap:tt align=$align:ident;
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex().flex_wrap().gap(::gpui::px($gap as f32));
        panel = $crate::wrappanel!(@align_items panel, $align);
        $( panel = panel.child($child); )*
        panel
    }};

    // 4. Orientation only
    (
        orientation=$orient:ident;
        $( $child:expr ),* $(,)?
    ) => {{
        let mut panel = ::gpui::div().flex().flex_wrap();
        panel = $crate::wrappanel!(@orient panel, $orient);
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

    // Helpers
    (@orient $panel:ident, horizontal) => { $panel };
    (@orient $panel:ident, vertical) => { $panel.flex_col() };
    (@orient $panel:ident, row) => { $panel };
    (@orient $panel:ident, col) => { $panel.flex_col() };

    (@align_items $panel:ident, center) => { $panel.items_center() };
    (@align_items $panel:ident, start) => { $panel.items_start() };
    (@align_items $panel:ident, end) => { $panel.items_end() };
}

/// Legacy wrapper alias for horizontal wrapping (retained for backward compatibility).
#[macro_export]
macro_rules! flow {
    ( $($args:tt)* ) => {
        $crate::wrappanel!(orientation=horizontal $($args)*)
    };
}

/// Spawns interactive forms and handles state/subscription mapping.
#[macro_export]
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
                    $crate::declare_form!(@push_subscription _subscriptions, cx, $ctrl_name $(=> $event_ty |$this, $event, $cx| $callback)?);
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

/// Utility macro to easily wrap a control in a column with a text label and standard gap.
///
/// Requires `field_label` to be in scope at the call site (typically app-specific chrome helpers).
#[macro_export]
macro_rules! form_field {
    ($label:expr, $chrome:expr; $control:expr) => {
        ::gpui::div()
            .flex()
            .flex_col()
            .gap(::gpui::px(4.0))
            .child(field_label($label, $chrome.body_text))
            .child($control)
    };
}
