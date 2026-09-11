//! No-op template handlers for static style-guide previews.

use std::sync::Arc;

use gpui::{
    App, Bounds, ClickEvent, DragMoveEvent, KeyDownEvent, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, Window,
};
use luma::controls::slider::{
    SliderBoundsHandler, SliderDrag, SliderHoverHandler, SliderMouseDownHandler, SliderMouseMoveHandler,
    SliderMouseUpHandler, SliderTemplateHandlers, ThumbId,
};
use luma::controls::textarea::{
    TextAreaClickHandler, TextAreaDrag, TextAreaHoverHandler, TextAreaKeyDownHandler, TextAreaMouseDownHandler,
    TextAreaMouseMoveHandler, TextAreaMouseUpHandler, TextAreaTemplateHandlers,
};
use luma::controls::textfield::{
    TextFieldClickHandler, TextFieldDrag, TextFieldDragMoveHandler, TextFieldHoverHandler, TextFieldKeyDownHandler,
    TextFieldMouseDownHandler, TextFieldMouseMoveHandler, TextFieldMouseUpHandler, TextFieldTemplateHandlers,
};

fn noop_bounds(_: &Bounds<Pixels>, _: &mut Window, _: &mut App) {}
fn noop_hover(_: &bool, _: &mut Window, _: &mut App) {}
fn noop_mouse_down(_: &MouseDownEvent, _: &mut Window, _: &mut App) {}
fn noop_mouse_move(_: &MouseMoveEvent, _: &mut Window, _: &mut App) {}
fn noop_mouse_up(_: &MouseUpEvent, _: &mut Window, _: &mut App) {}
fn noop_click(_: &ClickEvent, _: &mut Window, _: &mut App) {}
fn noop_key_down(_: &KeyDownEvent, _: &mut Window, _: &mut App) {}
fn noop_slider_drag(_: &DragMoveEvent<SliderDrag>, _: &mut Window, _: &mut App) {}
fn noop_thumb_mouse_down(_: &ThumbId, _: &MouseDownEvent, _: &mut Window, _: &mut App) {}
fn noop_textarea_drag(_: &DragMoveEvent<TextAreaDrag>, _: &mut Window, _: &mut App) {}
fn noop_textfield_drag(_: &DragMoveEvent<TextFieldDrag>, _: &mut Window, _: &mut App) {}

pub fn textfield_handlers() -> TextFieldTemplateHandlers {
    TextFieldTemplateHandlers {
        hover: Box::new(noop_hover) as TextFieldHoverHandler,
        mouse_down: Box::new(noop_mouse_down) as TextFieldMouseDownHandler,
        mouse_move: Box::new(noop_mouse_move) as TextFieldMouseMoveHandler,
        mouse_up: Box::new(noop_mouse_up) as TextFieldMouseUpHandler,
        mouse_up_out: Box::new(noop_mouse_up) as TextFieldMouseUpHandler,
        click: Box::new(noop_click) as TextFieldClickHandler,
        key_down: Box::new(noop_key_down) as TextFieldKeyDownHandler,
        drag_move: Box::new(noop_textfield_drag) as TextFieldDragMoveHandler,
    }
}

pub fn textarea_handlers() -> TextAreaTemplateHandlers {
    TextAreaTemplateHandlers {
        hover: Box::new(noop_hover) as TextAreaHoverHandler,
        mouse_down: Box::new(noop_mouse_down) as TextAreaMouseDownHandler,
        mouse_move: Box::new(noop_mouse_move) as TextAreaMouseMoveHandler,
        mouse_up: Box::new(noop_mouse_up) as TextAreaMouseUpHandler,
        mouse_up_out: Box::new(noop_mouse_up) as TextAreaMouseUpHandler,
        click: Box::new(noop_click) as TextAreaClickHandler,
        key_down: Box::new(noop_key_down) as TextAreaKeyDownHandler,
        drag_move: Box::new(noop_textarea_drag),
    }
}

pub fn slider_handlers() -> SliderTemplateHandlers {
    SliderTemplateHandlers {
        track_bounds: Box::new(noop_bounds) as SliderBoundsHandler,
        hover: Box::new(noop_hover) as SliderHoverHandler,
        mouse_down: Box::new(noop_mouse_down) as SliderMouseDownHandler,
        mouse_move: Box::new(noop_mouse_move) as SliderMouseMoveHandler,
        mouse_up: Box::new(noop_mouse_up) as SliderMouseUpHandler,
        mouse_up_out: Box::new(noop_mouse_up) as SliderMouseUpHandler,
        drag_move: Arc::new(noop_slider_drag),
        thumb_mouse_down: Arc::new(noop_thumb_mouse_down),
    }
}
