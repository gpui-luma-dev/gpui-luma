use super::super::*;

pub(in crate::studio::style::style_guide) fn render_sidebar_template_section(
    sidebar: Entity<NavigationSidebar>,
    look: &ShadcnLook,
) -> AnyElement {
    let chrome = look.chrome();

    section_shell_with_width(
        960.0,
        "Sidebar",
        "Navigation sidebar template preview.",
        chrome.title_text,
        chrome.muted_text,
        chrome.border,
        chrome.panel_background,
        div()
            .w_full()
            .flex()
            .justify_start()
            .child(div().w(px(300.0)).h(px(520.0)).overflow_hidden().child(sidebar))
            .into_any_element(),
    )
}
