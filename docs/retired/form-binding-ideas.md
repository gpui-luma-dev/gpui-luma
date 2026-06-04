# Declarative Form Binding Ideas

This document preserves the design patterns and ideas discussed for making form building and event propagation in GPUI-Luma more declarative. 

---

## Pattern 1: Macro-Driven DSL (`declare_form!`)

This pattern uses a declarative macro to define a panel struct, its child control fields, and its event subscription callbacks in a single block of code.

### The Macro Definition

```rust
macro_rules! declare_form {
    (
        $(#[$meta:meta])*
        $vis:vis struct $struct_name:ident {
            controls: {
                $(
                    $ctrl_name:ident : $ctrl_ty:ty = $builder:expr
                    => |$this:ident, $event:pat, $cx:ident| $callback:block
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
        // 1. Struct Definition
        $(#[$meta])*
        $vis struct $struct_name {
            $( $ctrl_name : $ctrl_ty , )*
            $( $arg_name : $arg_ty , )*
            $( $field_name : $field_ty , )*
            _subscriptions: Vec<gpui::Subscription>,
        }

        impl $struct_name {
            // 2. Generated Constructor
            $vis fn new(
                cx: &mut gpui::Context<Self>,
                $( $arg_name : $arg_ty ),*
            ) -> Self {
                // Spawn controls
                $(
                    let $ctrl_name = $builder.spawn(cx);
                )*

                // Initialize fields
                $(
                    let $field_name = $field_init;
                )*

                // Subscribe to events
                let mut _subscriptions = Vec::new();
                $(
                    _subscriptions.push(cx.subscribe(&$ctrl_name, move |$this, _, event, $cx| {
                        let $event = event;
                        $callback
                    }));
                )*

                Self {
                    $( $ctrl_name , )*
                    $( $arg_name , )*
                    $( $field_name , )*
                    _subscriptions,
                }
            }
        }
    };
}
```

### Usage Example

```rust
declare_form! {
    pub(super) struct PaymentPanel {
        controls: {
            submit_button: Entity<Button> = radix_theme.primary_button("intro-submit").label("Submit")
                => |this, _: &ButtonEvent, cx| {
                    this.event_bus.update(cx, |_, cx| cx.emit(AppEvent::PaymentSubmit));
                },

            name_field: TextField = radix_theme.textfield("intro-name").placeholder("Name on card")
                => |this, event: &TextFieldEvent, cx| {
                    if let TextFieldEvent::Change { value } | TextFieldEvent::Submit { value } = event {
                        this.name_value = value.clone().into();
                        this.emit_change("TextField::Change", cx);
                    }
                },
        },
        args: {
            radix_theme: Arc<RadixTheme>,
            event_bus: Entity<EventBus>,
        },
        fields: {
            name_value: SharedString = SharedString::default(),
        }
    }
}
```

---

## Pattern 2: Elm/Redux-Style Action Dispatcher

This pattern translates all control events into a single enum of **Actions**, using a lightweight `bind_events!` macro to forward control events to a single centralized `dispatch` reducer function.

### The Event Binding Macro

```rust
macro_rules! bind_events {
    ($cx:expr, $subs:expr; { $($control:expr => $event_ty:ty : |$ev:pat| $action:expr),* $(,)? }) => {
        $(
            $subs.push($cx.subscribe(&$control, |this, _, event: &$event_ty, cx| {
                if let Some(act) = { let $ev = event; $action } {
                    this.dispatch(act, cx);
                }
            }));
        )*
    };
}
```

### Usage Example

First, define the actions and the reducer:
```rust
enum FormAction {
    Submit,
    UpdateName(SharedString),
}

impl PaymentPanel {
    fn dispatch(&mut self, action: FormAction, cx: &mut Context<Self>) {
        match action {
            FormAction::Submit => {
                self.event_bus.update(cx, |_, cx| cx.emit(AppEvent::PaymentSubmit));
            }
            FormAction::UpdateName(name) => {
                self.name_value = name;
                self.emit_change("TextField::Change", cx);
            }
        }
    }
}
```

Then bind the actions in the constructor:
```rust
let mut subscriptions = Vec::new();
bind_events!(cx, subscriptions; {
    submit_button => ButtonEvent: |_| Some(FormAction::Submit),
    name_field => TextFieldEvent: |ev| match ev {
        TextFieldEvent::Change { value } | TextFieldEvent::Submit { value } => Some(FormAction::UpdateName(value.clone().into())),
        _ => None,
    },
});
```

---

## Pattern 3: Fluent `FormBinder` Builder API

This pattern uses a helper struct with method chaining to map controls directly to state properties or closures.

```rust
pub struct FormBinder<'a, T: 'static> {
    cx: &'a mut Context<T>,
    subscriptions: &'a mut Vec<Subscription>,
}

impl<'a, T: 'static> FormBinder<'a, T> {
    pub fn new(cx: &'a mut Context<T>, subscriptions: &'a mut Vec<Subscription>) -> Self {
        Self { cx, subscriptions }
    }

    pub fn bind_button(
        &mut self,
        button: &Entity<Button>,
        mut action: impl FnMut(&mut T, &mut Context<T>) + 'static,
    ) -> &mut Self {
        self.subscriptions.push(self.cx.subscribe(button, move |this, _, _: &ButtonEvent, cx| {
            action(this, cx);
        }));
        self
    }

    pub fn bind_text<F>(
        &mut self,
        field: &TextField,
        mut getter: F,
        mut on_change: impl FnMut(&mut T, &mut Context<T>) + 'static,
    ) -> &mut Self
    where
        F: FnMut(&mut T) -> &mut SharedString + 'static,
    {
        self.subscriptions.push(self.cx.subscribe(field, move |this, _, event, cx| {
            if let TextFieldEvent::Change { value } | TextFieldEvent::Submit { value } = event {
                *getter(this) = value.clone().into();
                on_change(this, cx);
            }
        }));
        self
    }
}
```
