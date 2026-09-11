//! Developer tab screen: live tuning tools that are not part of the design docs.

use std::sync::Arc;

use gpui::{AnyElement, Entity, IntoElement, prelude::*};
use luma::vstack;
use luma_look_radix::{RadixLook, SemanticRole};

use super::section::section;
use crate::controls::ClassicShadowEditor;

pub fn page(look: &Arc<RadixLook>, shadow_editor: Entity<ClassicShadowEditor>) -> AnyElement {
    let fg = look.resolve_role(SemanticRole::Foreground).hsla();
    let muted = look.resolve_role(SemanticRole::MutedForeground).hsla();
    let border = look.resolve_role(SemanticRole::Border).hsla();

    vstack! {
        gap=28;
        section(
            "Classic Button",
            "Retunes the raised bubble live. Every Classic button in the app follows.",
            fg,
            muted,
            border,
            shadow_editor.into_any_element(),
        ),
    }
    .w_full()
    .into_any_element()
}
