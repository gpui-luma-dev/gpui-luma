#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BuiltInTheme {
    pub id: &'static str,
    pub css: &'static str,
    pub requires_rajdhani_font: bool,
}

impl BuiltInTheme {
    pub fn display_name(&self) -> String {
        humanize_theme_id(self.id)
    }
}

include!(concat!(env!("OUT_DIR"), "/built_in_themes.rs"));

pub fn built_in_themes() -> &'static [BuiltInTheme] {
    BUILT_IN_THEMES
}

pub fn built_in_theme(id: &str) -> Option<&'static BuiltInTheme> {
    built_in_themes().iter().find(|theme| theme.id.eq_ignore_ascii_case(id))
}

fn humanize_theme_id(id: &str) -> String {
    let mut words = Vec::new();

    for segment in id.split('-').filter(|segment| !segment.is_empty()) {
        let mut chars = segment.chars();
        let Some(first) = chars.next() else {
            continue;
        };
        let mut word = String::new();
        word.extend(first.to_uppercase());
        word.push_str(chars.as_str());
        words.push(word);
    }

    words.join(" ")
}

#[cfg(test)]
mod tests {
    use super::{built_in_theme, built_in_themes};

    #[test]
    fn built_in_themes_are_sorted_by_id() {
        let ids: Vec<&str> = built_in_themes().iter().map(|theme| theme.id).collect();
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        assert_eq!(ids, sorted);
    }

    #[test]
    fn built_in_theme_lookup_is_case_insensitive() {
        assert_eq!(built_in_theme("Retro-Arcade").expect("theme").display_name(), "Retro Arcade");
    }

    #[test]
    fn jarvis_marks_required_font() {
        assert!(built_in_theme("jarvis").expect("theme").requires_rajdhani_font);
        assert!(!built_in_theme("retro-arcade").expect("theme").requires_rajdhani_font);
    }

    #[test]
    fn humanized_names_split_kebab_case() {
        assert_eq!(
            built_in_theme("monochrome-amber-mono-2.0").expect("theme").display_name(),
            "Monochrome Amber Mono 2.0"
        );
    }
}
