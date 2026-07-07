use font_kit::loader::Loader;

const OS2_TABLE_TAG: u32 = u32::from_be_bytes(*b"OS/2");
const OS2_FAMILY_CLASS_OFFSET: usize = 30;

/// Typography token slot used by theme editors (`--font-sans`, `--font-serif`, `--font-mono`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FontSlot {
    Sans,
    Serif,
    Mono,
}

impl FontSlot {
    pub fn matches_suggestion(self, role: SuggestedRole) -> bool {
        match self {
            Self::Sans => matches!(role, SuggestedRole::Sans | SuggestedRole::Unknown),
            Self::Serif => matches!(role, SuggestedRole::Serif | SuggestedRole::Unknown),
            Self::Mono => role == SuggestedRole::Mono,
        }
    }
}

/// Suggested role derived from font metadata. Used for dropdown filtering, not enforcement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SuggestedRole {
    Sans,
    Serif,
    Mono,
    Cursive,
    Fantasy,
    Unknown,
}

pub fn classify_family(font: &impl Loader) -> Option<SuggestedRole> {
    if font.is_monospace() {
        return Some(SuggestedRole::Mono);
    }

    let os2_class = font.load_font_table(OS2_TABLE_TAG).and_then(|table| os2_family_class(table.as_ref()));

    if let Some(class) = os2_class {
        if class == 11 {
            return None;
        }
        let role = role_from_os2_class(class);
        if role != SuggestedRole::Unknown {
            return Some(role);
        }
    }

    None
}

/// Keyword fallback when OS/2 metadata is missing or reports class 0.
pub fn guess_role_from_name(name: &str) -> SuggestedRole {
    let name_lower = name.to_ascii_lowercase();
    if name_lower.contains("mono") || name_lower.contains("code") || name_lower.contains("console") {
        SuggestedRole::Mono
    } else if name_lower.contains("sans") || name_lower.contains("gothic") || name_lower.contains("ui") {
        SuggestedRole::Sans
    } else if name_lower.contains("serif") || name_lower.contains("roman") || name_lower.contains("slab") {
        SuggestedRole::Serif
    } else {
        SuggestedRole::Unknown
    }
}

pub fn os2_family_class(table: &[u8]) -> Option<u8> {
    if table.len() <= OS2_FAMILY_CLASS_OFFSET + 1 {
        return None;
    }

    let raw = i16::from_be_bytes([table[OS2_FAMILY_CLASS_OFFSET], table[OS2_FAMILY_CLASS_OFFSET + 1]]);
    Some(((raw >> 8) & 0xFF) as u8)
}

pub fn role_from_os2_class(class: u8) -> SuggestedRole {
    match class {
        1..=7 => SuggestedRole::Serif,
        8 | 14 | 15 | 16 => SuggestedRole::Sans,
        9 => SuggestedRole::Fantasy,
        10 => SuggestedRole::Cursive,
        _ => SuggestedRole::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn os2_class_mapping_matches_ibm_table() {
        assert_eq!(role_from_os2_class(1), SuggestedRole::Serif);
        assert_eq!(role_from_os2_class(8), SuggestedRole::Sans);
        assert_eq!(role_from_os2_class(9), SuggestedRole::Fantasy);
        assert_eq!(role_from_os2_class(10), SuggestedRole::Cursive);
        assert_eq!(role_from_os2_class(0), SuggestedRole::Unknown);
    }

    #[test]
    fn os2_family_class_reads_high_byte() {
        let mut table = vec![0_u8; 32];
        table[OS2_FAMILY_CLASS_OFFSET] = 0x08;
        table[OS2_FAMILY_CLASS_OFFSET + 1] = 0x03;
        assert_eq!(os2_family_class(&table), Some(8));
    }

    #[test]
    fn guess_role_from_name_uses_keywords_when_os2_unknown() {
        assert_eq!(guess_role_from_name("JetBrains Mono"), SuggestedRole::Mono);
        assert_eq!(guess_role_from_name("Fira Code"), SuggestedRole::Mono);
        assert_eq!(guess_role_from_name("Segoe UI"), SuggestedRole::Sans);
        assert_eq!(guess_role_from_name("Noto Sans"), SuggestedRole::Sans);
        assert_eq!(guess_role_from_name("Roboto Slab"), SuggestedRole::Serif);
        assert_eq!(guess_role_from_name("Helvetica Neue"), SuggestedRole::Unknown);
    }
}
