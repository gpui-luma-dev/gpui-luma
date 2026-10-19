//! Look-neutral interactive sample support. No dependency on a Studio shell or look crate.
pub mod event_log;
pub mod inspector;
pub mod native;
pub mod shell;
pub mod switch_sample;
use gpui_luma::controls::switch::SwitchEvent;

pub fn format_switch_event(event: &SwitchEvent) -> String {
    match event {
        SwitchEvent::Change { on } => format!("SwitchEvent::Change {{ on: {on} }}"),
        SwitchEvent::FocusChanged { focused } => format!("SwitchEvent::FocusChanged {{ focused: {focused} }}"),
        SwitchEvent::EnabledChanged { enabled } => format!("SwitchEvent::EnabledChanged {{ enabled: {enabled} }}"),
        SwitchEvent::HoverChanged { hovered } => format!("SwitchEvent::HoverChanged {{ hovered: {hovered} }}"),
        _ => "SwitchEvent::(unknown)".to_string(),
    }
}
