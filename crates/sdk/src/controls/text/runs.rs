use std::ops::Range;

use gpui::{Font, Hsla, TextRun, UnderlineStyle, px};

/// Split at selection and composition boundaries, keeping lengths in UTF-8 bytes.
pub(super) fn colored_runs(
    text: &str,
    global_offset: usize,
    selection: Option<(usize, usize)>,
    marked_range: Option<&Range<usize>>,
    font: Font,
    foreground: Hsla,
    selection_foreground: Hsla,
) -> Vec<TextRun> {
    let mut runs: Vec<TextRun> = Vec::new();
    let mut previous = None;
    for (index, ch) in text.chars().enumerate() {
        let offset = global_offset + index;
        let selected = selection.is_some_and(|(start, end)| start <= offset && offset < end);
        let marked = marked_range.is_some_and(|range| range.contains(&offset));
        let state = (selected, marked);
        if previous == Some(state) {
            if let Some(run) = runs.last_mut() {
                run.len += ch.len_utf8();
            }
        } else {
            runs.push(TextRun {
                len: ch.len_utf8(),
                font: font.clone(),
                color: if selected { selection_foreground } else { foreground },
                background_color: None,
                underline: marked.then_some(UnderlineStyle { thickness: px(1.0), color: None, wavy: false }),
                strikethrough: None,
            });
            previous = Some(state);
        }
    }
    runs
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{font, hsla};

    #[test]
    fn composition_and_selection_use_character_ranges_and_byte_lengths() {
        let foreground = hsla(0.0, 0.0, 0.0, 1.0);
        let selected = hsla(0.0, 0.0, 1.0, 1.0);
        let runs = colored_runs("aé中🙂z", 0, Some((2, 4)), Some(&(1..3)), font("test"), foreground, selected);
        assert_eq!(runs.iter().map(|run| run.len).collect::<Vec<_>>(), vec![1, 2, 3, 4, 1]);
        assert_eq!(
            runs.iter().map(|run| run.underline.is_some()).collect::<Vec<_>>(),
            vec![false, true, true, false, false]
        );
        assert_eq!(
            runs.iter().map(|run| run.color).collect::<Vec<_>>(),
            vec![foreground, foreground, selected, selected, foreground]
        );
        assert!(runs.iter().filter_map(|run| run.underline).all(|style| !style.wavy && style.color.is_none()));
    }

    #[test]
    fn composition_spans_wrapped_lines_and_disappears_when_unmarked() {
        let color = hsla(0.0, 0.0, 0.0, 1.0);
        for (text, offset, expected) in
            [("ab", 0, vec![false, true]), ("中🙂", 2, vec![true]), ("ef", 4, vec![true, false])]
        {
            let runs = colored_runs(text, offset, None, Some(&(1..5)), font("test"), color, color);
            assert_eq!(runs.iter().map(|run| run.underline.is_some()).collect::<Vec<_>>(), expected);
            assert_eq!(runs.iter().map(|run| run.len).sum::<usize>(), text.len());
            let unmarked = colored_runs(text, offset, None, None, font("test"), color, color);
            assert_eq!(unmarked.len(), 1);
            assert!(unmarked[0].underline.is_none());
        }
    }
}
