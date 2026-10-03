use gpui::{
    App, Bounds, ElementInputHandler, GlobalElementId, IntoElement, LayoutId, PaintQuad, Pixels, ShapedLine, Style,
    TextAlign, TextRun, Window, fill, font, point, px, relative, size,
};

use super::layout::{TextAreaCachedLine, TextAreaLayoutCache};
use super::TextArea;
use crate::theme::{LayoutCacheKey, LumaLayoutCacheExt, StandardBoxScale};

pub(super) struct TextAreaElement {
    pub(super) input: gpui::Entity<TextArea>,
}

pub(super) struct TextAreaPrepaintState {
    lines: Vec<TextAreaCachedLine>,
    selection_quads: Vec<PaintQuad>,
    caret_quad: Option<PaintQuad>,
    placeholder_line: Option<ShapedLine>,
    line_height: Pixels,
    vertical_scroll: Pixels,
}
impl IntoElement for TextAreaElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl gpui::Element for TextAreaElement {
    type RequestLayoutState = ();
    type PrepaintState = TextAreaPrepaintState;

    fn id(&self) -> Option<gpui::ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let (theme, state, enabled, control_size, rows, look_override) = {
            let input = self.input.read(cx);
            (
                input.model.theme.clone(),
                input.state,
                input.model.enabled,
                input.model.size,
                input.model.rows.max(1),
                input.model.look_override.clone(),
            )
        };
        let scale_factor = window.scale_factor();
        let scale = cx.use_cached_layout(
            theme.metrics(),
            LayoutCacheKey { size: control_size, scale_factor_bits: scale_factor.to_bits() },
            |metrics| StandardBoxScale::compute(control_size, metrics, scale_factor),
        );
        let mut look = theme.resolve_look(state, enabled, control_size, &scale);
        if let Some(override_fn) = &look_override {
            look = override_fn(look);
        }
        let mut style = Style::default();
        style.size.width = relative(1.0).into();
        style.size.height = px(look.typography.line_height * rows as f32).into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let (theme, state, enabled, control_size, look_override) = {
            let input = self.input.read(cx);
            (
                input.model.theme.clone(),
                input.state,
                input.model.enabled,
                input.model.size,
                input.model.look_override.clone(),
            )
        };
        let scale_factor = window.scale_factor();
        let scale = cx.use_cached_layout(
            theme.metrics(),
            LayoutCacheKey { size: control_size, scale_factor_bits: scale_factor.to_bits() },
            |metrics| StandardBoxScale::compute(control_size, metrics, scale_factor),
        );
        let input = self.input.read(cx);
        let mut look = theme.resolve_look(state, enabled, control_size, &scale);
        if let Some(override_fn) = &look_override {
            look = override_fn(look);
        }
        let line_height = px(look.typography.line_height);
        let font_size = px(look.typography.size);
        let mut lines = Vec::new();
        let mut selection_quads = Vec::new();
        let mut caret_quad = None;
        let selection = input.state.selection_range();
        let cursor = input.state.cursor.min(input.model.value.chars().count());
        let show_placeholder = input.model.value.is_empty() && !input.state.focused;
        let font_family = look.font_family.clone();
        let font_weight = look.typography.weight;
        let run_for = move |len: usize, color| TextRun {
            len,
            font: {
                let mut font = font(font_family.clone());
                font.weight = font_weight;
                font
            },
            color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };

        let placeholder_line = if show_placeholder {
            let run = run_for(input.model.placeholder.len(), look.placeholder);
            Some(window.text_system().shape_line(input.model.placeholder.clone(), font_size, &[run], None))
        } else {
            None
        };

        let logical_lines = TextArea::logical_lines(input.model.value.as_ref());
        let wrap_width = bounds.size.width.max(px(1.0));

        for (hard_start, _hard_end, text) in logical_lines.into_iter() {
            let full_run = run_for(text.len(), look.foreground);
            let full_shaped = window.text_system().shape_line(text.clone().into(), font_size, &[full_run], None);
            let line_chars = text.chars().collect::<Vec<_>>();
            let char_count = line_chars.len();

            if char_count == 0 {
                lines.push(TextAreaCachedLine {
                    start: hard_start,
                    end: hard_start,
                    text: String::new(),
                    line: full_shaped,
                });
                continue;
            }

            let mut byte_offsets = Vec::with_capacity(char_count + 1);
            for char_offset in 0..=char_count {
                byte_offsets.push(TextArea::char_to_byte_offset(&text, char_offset));
            }

            let mut local_start = 0usize;
            while local_start < char_count {
                let start_x = full_shaped.x_for_index(byte_offsets[local_start]);
                let mut fit_end = local_start + 1;
                for (probe, _) in byte_offsets.iter().enumerate().take(char_count + 1).skip(local_start + 1) {
                    let probe_x = full_shaped.x_for_index(byte_offsets[probe]);
                    if probe_x - start_x <= wrap_width {
                        fit_end = probe;
                    } else {
                        break;
                    }
                }

                let mut local_end = fit_end;
                if fit_end < char_count {
                    for probe in ((local_start + 1)..=fit_end).rev() {
                        if line_chars[probe - 1].is_whitespace() {
                            local_end = probe;
                            break;
                        }
                    }
                    local_end = local_end.max(local_start + 1);
                }

                let segment_text = line_chars[local_start..local_end].iter().collect::<String>();
                let runs = crate::controls::text::runs::colored_runs(
                    &segment_text,
                    hard_start + local_start,
                    selection,
                    input.marked_range.as_ref(),
                    {
                        let mut font = font(look.font_family.clone());
                        font.weight = font_weight;
                        font
                    },
                    look.foreground,
                    look.selection_foreground,
                );
                let shaped = window.text_system().shape_line(segment_text.clone().into(), font_size, &runs, None);

                lines.push(TextAreaCachedLine {
                    start: hard_start + local_start,
                    end: hard_start + local_end,
                    text: segment_text,
                    line: shaped,
                });

                local_start = local_end;
            }
        }

        let content_height = line_height * lines.len() as f32;
        let max_vertical_scroll = (content_height - bounds.size.height).max(px(0.0));
        let vertical_scroll = input.vertical_scroll.max(px(0.0)).min(max_vertical_scroll);

        for (line_ix, line) in lines.iter().enumerate() {
            let top = bounds.top() + line_height * line_ix as f32 - vertical_scroll;
            let bottom = top + line_height;

            if bottom >= bounds.top() && top <= bounds.bottom() {
                let start = line.start;
                let end = line.end;

                if let Some((selection_start, selection_end)) = selection {
                    let local_start = selection_start.max(start).min(end).saturating_sub(start);
                    let local_end = selection_end.max(start).min(end).saturating_sub(start);
                    if local_start < local_end || (selection_start <= end && selection_end > end && end == start) {
                        let x1 = line.line.x_for_index(TextArea::char_to_byte_offset(&line.text, local_start));
                        let x2 = line.line.x_for_index(TextArea::char_to_byte_offset(&line.text, local_end));
                        selection_quads.push(fill(
                            Bounds::from_corners(
                                point(bounds.left() + x1, top),
                                point(bounds.left() + x2.max(x1), bottom),
                            ),
                            look.selection_background,
                        ));
                    }
                }

                if selection.is_none() && cursor >= start && cursor <= end {
                    let local_cursor = cursor.saturating_sub(start);
                    let x = line.line.x_for_index(TextArea::char_to_byte_offset(&line.text, local_cursor));
                    caret_quad =
                        Some(fill(Bounds::new(point(bounds.left() + x, top), size(px(1.5), line_height)), look.caret));
                }
            }
        }

        TextAreaPrepaintState { lines, selection_quads, caret_quad, placeholder_line, line_height, vertical_scroll }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus_handle = self.input.read(cx).focus_handle.clone();
        window.handle_input(&focus_handle, ElementInputHandler::new(bounds, self.input.clone()), cx);

        for selection in prepaint.selection_quads.drain(..) {
            window.paint_quad(selection);
        }

        if let Some(line) = prepaint.placeholder_line.take() {
            line.paint(bounds.origin, prepaint.line_height, TextAlign::Left, None, window, cx).ok();
        } else {
            for (line_ix, line) in prepaint.lines.iter().enumerate() {
                let origin = point(
                    bounds.left(),
                    bounds.top() + prepaint.line_height * line_ix as f32 - prepaint.vertical_scroll,
                );
                if origin.y + prepaint.line_height >= bounds.top() && origin.y <= bounds.bottom() {
                    line.line.paint(origin, prepaint.line_height, TextAlign::Left, None, window, cx).ok();
                }
            }
        }

        let focused = focus_handle.is_focused(window);
        if focused && let Some(caret) = prepaint.caret_quad.take() {
            window.paint_quad(caret);
        }

        self.input.update(cx, |input, cx| {
            input.vertical_scroll = prepaint.vertical_scroll;
            input.layout_cache = Some(TextAreaLayoutCache {
                text_viewport: bounds,
                line_height: prepaint.line_height,
                lines: prepaint.lines.clone(),
            });
            input.clamp_vertical_scroll_to_cache();
            input.sync_scrollbar(cx);
        });
    }
}
