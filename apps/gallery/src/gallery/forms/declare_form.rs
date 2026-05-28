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

pub(crate) use declare_form;
