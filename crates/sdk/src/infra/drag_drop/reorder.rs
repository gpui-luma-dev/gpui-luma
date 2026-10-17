use std::{collections::HashSet, hash::Hash};
use gpui::SharedString;

/// Resolve a stable-key gap before removing captured items. An anchor inside
/// the moving set is valid and can produce an unchanged commit.
pub(crate) fn reorder_indices<K: Eq + Hash>(
    keys: &[K],
    moving: &[K],
    before: Option<&K>,
) -> Result<Vec<usize>, SharedString> {
    let all: HashSet<_> = keys.iter().collect();
    let captured: HashSet<_> = moving.iter().collect();
    if all.len() != keys.len() || captured.len() != moving.len() || captured.is_empty() || !captured.is_subset(&all) {
        return Err("Item IDs must be unique and moving keys must still exist".into());
    }
    let gap = match before {
        Some(key) => keys
            .iter()
            .position(|id| id == key)
            .ok_or_else(|| SharedString::from("Insertion anchor disappeared"))?,
        None => keys.len(),
    };
    let index = keys[..gap].iter().filter(|key| !captured.contains(key)).count();
    let mut remaining: Vec<_> = (0..keys.len()).filter(|i| !captured.contains(&keys[*i])).collect();
    let moved = (0..keys.len()).filter(|i| captured.contains(&keys[*i]));
    remaining.splice(index..index, moved);
    Ok(remaining)
}
