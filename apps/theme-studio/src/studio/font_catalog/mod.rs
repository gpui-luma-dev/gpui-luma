//! System font catalog with suggested sans / serif / mono roles for typography pickers.
//!
//! Classification uses `font-kit` for enumeration and monospace detection, OS/2
//! `sFamilyClass` when available, and family-name keyword heuristics as a fallback.

mod classify;
mod error;

pub use classify::{FontSlot, SuggestedRole, classify_family, guess_role_from_name};
pub use error::FontCatalogError;

use std::collections::HashMap;

use font_kit::source::{Source, SystemSource};

/// A classified system font family.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FontFamily {
    pub name: String,
    pub suggested: SuggestedRole,
}

/// Cached system font families with suggested roles.
#[derive(Clone, Debug, Default)]
pub struct FontCatalog {
    families: Vec<FontFamily>,
    by_name: HashMap<String, SuggestedRole>,
}

impl FontCatalog {
    /// Enumerate installed system families and classify a representative face for each.
    pub fn load_system() -> Result<Self, FontCatalogError> {
        let source = SystemSource::new();
        let family_names = source.all_families()?;
        let mut families = Vec::with_capacity(family_names.len());

        for name in family_names {
            if should_exclude_family_name(&name) {
                continue;
            }

            let suggested = classify_system_family(&source, &name).unwrap_or(SuggestedRole::Unknown);
            families.push(FontFamily { name: name.clone(), suggested });
        }

        families.sort_by_key(|family| family.name.to_ascii_lowercase());

        let by_name = families.iter().map(|family| (family.name.clone(), family.suggested)).collect();

        Ok(Self { families, by_name })
    }

    #[cfg(test)]
    pub fn from_families(families: Vec<FontFamily>) -> Self {
        let by_name = families.iter().map(|family| (family.name.clone(), family.suggested)).collect();
        Self { families, by_name }
    }

    pub fn families(&self) -> &[FontFamily] {
        &self.families
    }

    pub fn suggested_role(&self, family_name: &str) -> Option<SuggestedRole> {
        self.by_name
            .get(family_name)
            .copied()
            .or_else(|| self.find_case_insensitive(family_name).map(|family| family.suggested))
    }

    pub fn is_suggested_for_slot(&self, family_name: &str, slot: FontSlot) -> bool {
        if self.families.is_empty() {
            return false;
        }

        self.suggested_role(family_name)
            .map(|role| slot.matches_suggestion(role))
            .unwrap_or(matches!(slot, FontSlot::Sans))
    }

    fn find_case_insensitive(&self, family_name: &str) -> Option<&FontFamily> {
        self.families.iter().find(|family| family.name.eq_ignore_ascii_case(family_name))
    }
}

fn classify_system_family(source: &impl Source, family_name: &str) -> Option<SuggestedRole> {
    let family = source.select_family_by_name(family_name).ok()?;
    let handle = family.fonts().first()?;
    let font = handle.load().ok()?;
    if let Some(role) = classify_family(&font)
        && role != SuggestedRole::Unknown
    {
        return Some(role);
    }

    let guessed = guess_role_from_name(family_name);
    if guessed != SuggestedRole::Unknown {
        return Some(guessed);
    }

    classify_family(&font).or(Some(SuggestedRole::Unknown))
}

fn should_exclude_family_name(name: &str) -> bool {
    if name.starts_with('.') {
        return true;
    }

    const EXCLUDED_SUBSTRINGS: &[&str] = &[
        "Color Emoji",
        "ColorEmoji",
        "Apple Symbols",
        "Segoe UI Symbol",
        "Segoe UI Emoji",
        "Noto Color Emoji",
        "Webdings",
        "Wingdings",
        "Symbol",
        "Dingbats",
        "Lucide",
    ];

    EXCLUDED_SUBSTRINGS.iter().any(|needle| name.contains(needle))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slot_filter_includes_unknown_for_sans_and_serif_only() {
        assert!(FontSlot::Sans.matches_suggestion(SuggestedRole::Unknown));
        assert!(FontSlot::Serif.matches_suggestion(SuggestedRole::Unknown));
        assert!(!FontSlot::Mono.matches_suggestion(SuggestedRole::Unknown));
    }

    #[test]
    fn slot_filter_mono_is_strict() {
        assert!(FontSlot::Mono.matches_suggestion(SuggestedRole::Mono));
        assert!(!FontSlot::Mono.matches_suggestion(SuggestedRole::Sans));
    }

    #[test]
    fn excludes_internal_gpui_family_names() {
        assert!(should_exclude_family_name(".SystemUIFont"));
        assert!(!should_exclude_family_name("Helvetica Neue"));
    }
}
