//! CSS / web font-family names matched against GPUI-resolvable system family names.

use gpui::SharedString;

/// Known web catalog names mapped to common installed family names.
const CSS_FONT_ALIASES: &[(&str, &str)] = &[
    ("Rajdhani", "Rajdhani Variable"),
    ("Inter", "Inter Variable"),
    ("Roboto", "Roboto Flex"),
    ("Geist", "Geist Sans"),
];

const TOKEN_SUFFIXES: &[&str] = &["variable", "vf", "pro", "mt", "std", "display", "regular", "book"];

pub fn clean_css_family_name(family: &str) -> String {
    family.trim().to_string()
}

pub fn match_css_named_family(family: &str, font_names: &[String]) -> Option<SharedString> {
    let family = clean_css_family_name(family);
    if family.is_empty() {
        return None;
    }

    for candidate in alias_candidates(&family) {
        if let Some(matched) = find_case_insensitive(candidate, font_names) {
            return Some(matched);
        }
        if let Some(matched) = fuzzy_family_match(candidate, font_names) {
            return Some(matched);
        }
    }

    find_case_insensitive(&family, font_names)
        .or_else(|| fuzzy_family_match(&family, font_names))
        .or_else(|| find_family_prefix(&family, font_names))
}

fn alias_candidates(family: &str) -> Vec<&str> {
    let mut candidates = vec![family];
    if let Some(alias) = lookup_alias(family) {
        if !candidates.contains(&alias) {
            candidates.push(alias);
        }
    }
    candidates
}

fn lookup_alias(family: &str) -> Option<&'static str> {
    CSS_FONT_ALIASES
        .iter()
        .find_map(|(web, system)| family.eq_ignore_ascii_case(web).then_some(*system))
}

fn find_case_insensitive(name: &str, font_names: &[String]) -> Option<SharedString> {
    font_names
        .iter()
        .find(|candidate| candidate.eq_ignore_ascii_case(name))
        .map(|candidate| SharedString::from(candidate.clone()))
}

fn find_family_prefix(prefix: &str, font_names: &[String]) -> Option<SharedString> {
    let prefix_lower = prefix.to_ascii_lowercase();
    font_names
        .iter()
        .find(|candidate| {
            let candidate_lower = candidate.to_ascii_lowercase();
            candidate_lower == prefix_lower || candidate_lower.starts_with(&format!("{prefix_lower} "))
        })
        .map(|candidate| SharedString::from(candidate.clone()))
}

fn fuzzy_family_match(query: &str, font_names: &[String]) -> Option<SharedString> {
    let query_core = core_family_tokens(query);
    if query_core.is_empty() {
        return None;
    }

    font_names.iter().find_map(|candidate| {
        let candidate_core = core_family_tokens(candidate);
        if query_core == candidate_core || tokens_prefix_equal(&query_core, &candidate_core) {
            Some(SharedString::from(candidate.clone()))
        } else {
            None
        }
    })
}

fn family_tokens(name: &str) -> Vec<String> {
    name.split_whitespace()
        .map(|token| token.trim_matches(|character: char| !character.is_alphanumeric()).to_ascii_lowercase())
        .filter(|token| !token.is_empty())
        .collect()
}

fn core_family_tokens(name: &str) -> Vec<String> {
    let mut tokens = family_tokens(name);
    while tokens.last().is_some_and(|token| TOKEN_SUFFIXES.contains(&token.as_str())) {
        tokens.pop();
    }
    tokens
}

fn tokens_prefix_equal(shorter: &[String], longer: &[String]) -> bool {
    if shorter.is_empty() || shorter.len() > longer.len() {
        return false;
    }

    shorter.iter().zip(longer.iter()).all(|(left, right)| left == right)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn css_named_family_matching_is_case_insensitive() {
        let font_names = vec!["inter".to_string()];
        let matched = match_css_named_family("Inter", &font_names).expect("match inter");
        assert_eq!(matched.as_ref(), "inter");
    }

    #[test]
    fn css_web_font_alias_maps_to_registered_family() {
        let font_names = vec!["Rajdhani Variable".to_string()];
        let matched = match_css_named_family("Rajdhani", &font_names).expect("match Rajdhani Variable");
        assert_eq!(matched.as_ref(), "Rajdhani Variable");
    }

    #[test]
    fn fuzzy_match_strips_variable_suffix() {
        let font_names = vec!["Outfit Variable".to_string()];
        let matched = match_css_named_family("Outfit", &font_names).expect("match outfit variable");
        assert_eq!(matched.as_ref(), "Outfit Variable");
    }

    #[test]
    fn fuzzy_match_allows_version_token_suffix() {
        let font_names = vec!["Source Serif 4".to_string()];
        let matched = match_css_named_family("Source Serif", &font_names).expect("match source serif 4");
        assert_eq!(matched.as_ref(), "Source Serif 4");
    }

    #[test]
    fn clean_css_family_name_preserves_web_primary_label() {
        assert_eq!(clean_css_family_name("  Rajdhani  "), "Rajdhani");
    }
}
