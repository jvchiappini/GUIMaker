//! Hot/active interaction model: press captures the pointer, click fires on
//! release inside the same widget, drags need a small threshold.

use crate::backend::{Input, PointerButton};
use crate::context::{UiMemory, WidgetId};
use crate::layout::Rect;

pub const DRAG_THRESHOLD: f32 = 4.0;
pub const DOUBLE_CLICK_TIME: f64 = 0.35;
pub const DOUBLE_CLICK_DISTANCE: f32 = 6.0;

#[derive(Clone, Debug, Default)]
pub struct Response {
    pub id: WidgetId,
    pub rect: Rect,
    pub hovered: bool,
    pub pressed: bool,
    pub released: bool,
    pub clicked: bool,
    pub double_clicked: bool,
    pub secondary_clicked: bool,
    pub dragged: bool,
    pub drag_delta: [f32; 2],
    pub changed: bool,
    pub has_focus: bool,
    pub lost_focus: bool,
    pub submitted: bool,
    pub cancelled: bool,
    pub enabled: bool,
    pub tooltip: Option<String>,
    pub shortcut: Option<String>,
}

impl Response {
    pub fn changed(&self) -> bool {
        self.changed
    }

    pub fn mark_changed(mut self) -> Self {
        self.changed = true;
        self
    }

    pub fn tooltip(mut self, text: impl Into<String>) -> Self {
        self.tooltip = Some(text.into());
        self
    }

    pub fn shortcut(mut self, text: impl Into<String>) -> Self {
        self.shortcut = Some(text.into());
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn clicked(&self) -> bool {
        self.clicked
    }

    pub fn double_clicked(&self) -> bool {
        self.double_clicked
    }

    pub fn secondary_clicked(&self) -> bool {
        self.secondary_clicked
    }

    pub fn dragged(&self) -> bool {
        self.dragged
    }

    pub fn drag_delta(&self) -> [f32; 2] {
        self.drag_delta
    }

    pub fn hovered(&self) -> bool {
        self.hovered
    }

    pub fn has_focus(&self) -> bool {
        self.has_focus
    }

    pub fn submitted(&self) -> bool {
        self.submitted
    }

    pub fn cancelled(&self) -> bool {
        self.cancelled
    }
}

pub(crate) fn interact(
    memory: &mut UiMemory,
    input: &Input,
    id: WidgetId,
    rect: Rect,
    enabled: bool,
) -> Response {
    let pointer = input.pointer;
    let inside = rect.contains(pointer);
    let was_active = memory.active == Some(id);
    let another_active = memory.active.is_some() && !was_active;

    let hovered = enabled && inside && !another_active;
    if hovered {
        memory.hot = Some(id);
    }

    let left_pressed = input.button_pressed(PointerButton::Left);
    let left_released = input.button_released(PointerButton::Left);
    let left_down = input.button_down(PointerButton::Left);

    let mut pressed = false;
    if enabled && hovered && left_pressed && memory.active.is_none() {
        memory.active = Some(id);
        memory.active_started = input.time;
        memory.drag_origin = Some(pointer);
        memory.drag_started = false;
        pressed = true;
    }

    let active = memory.active == Some(id);

    let drag_delta = match (active, memory.drag_origin) {
        (true, Some(origin)) => [pointer[0] - origin[0], pointer[1] - origin[1]],
        _ => [0.0; 2],
    };
    let drag_distance = (drag_delta[0] * drag_delta[0] + drag_delta[1] * drag_delta[1]).sqrt();
    let dragged = active && left_down && (memory.drag_started || drag_distance > DRAG_THRESHOLD);
    if dragged {
        memory.drag_started = true;
    }

    let clicked = active && enabled && left_released && inside;
    let mut double_clicked = false;
    if clicked {
        if let Some((last_id, last_time, last_pos)) = memory.last_click {
            let dx = pointer[0] - last_pos[0];
            let dy = pointer[1] - last_pos[1];
            let close = (dx * dx + dy * dy).sqrt() <= DOUBLE_CLICK_DISTANCE;
            if last_id == id && input.time - last_time <= DOUBLE_CLICK_TIME && close {
                double_clicked = true;
            }
        }
        memory.last_click = Some((id, input.time, pointer));
    }

    let secondary_clicked = enabled
        && hovered
        && input.button_pressed(PointerButton::Right)
        && memory.active.is_none();

    let released = active && left_released;

    if active && left_released {
        memory.active = None;
        memory.drag_origin = None;
        memory.drag_started = false;
    }

    Response {
        id,
        rect,
        hovered,
        pressed,
        released,
        clicked,
        double_clicked,
        secondary_clicked,
        dragged,
        drag_delta,
        changed: false,
        has_focus: false,
        lost_focus: false,
        submitted: false,
        cancelled: false,
        enabled,
        tooltip: None,
        shortcut: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input_at(x: f32, y: f32) -> Input {
        let mut input = Input::default();
        input.pointer = [x, y];
        input
    }

    #[test]
    fn press_then_release_inside_is_a_click() {
        let mut memory = UiMemory::new();
        let rect = Rect::new(0.0, 0.0, 100.0, 40.0);
        let id = WidgetId::new(7);

        let mut down = input_at(20.0, 20.0);
        down.buttons_down[0] = true;
        down.buttons_pressed[0] = true;
        down.time = 1.0;
        let first = interact(&mut memory, &down, id, rect, true);
        assert!(first.pressed);
        assert!(!first.clicked);

        let mut up = input_at(20.0, 20.0);
        up.time = 1.1;
        up.buttons_released[0] = true;
        let second = interact(&mut memory, &up, id, rect, true);
        assert!(second.clicked);
        assert!(second.released);
    }

    #[test]
    fn release_outside_cancels_the_click() {
        let mut memory = UiMemory::new();
        let rect = Rect::new(0.0, 0.0, 100.0, 40.0);
        let id = WidgetId::new(8);

        let mut down = input_at(20.0, 20.0);
        down.buttons_down[0] = true;
        down.buttons_pressed[0] = true;
        interact(&mut memory, &down, id, rect, true);

        let mut up = input_at(160.0, 20.0);
        up.buttons_released[0] = true;
        let second = interact(&mut memory, &up, id, rect, true);
        assert!(!second.clicked);
    }

    #[test]
    fn drag_needs_the_threshold() {
        let mut memory = UiMemory::new();
        let rect = Rect::new(0.0, 0.0, 100.0, 40.0);
        let id = WidgetId::new(9);

        let mut down = input_at(20.0, 20.0);
        down.buttons_down[0] = true;
        down.buttons_pressed[0] = true;
        interact(&mut memory, &down, id, rect, true);

        let mut move_small = input_at(22.0, 20.0);
        move_small.buttons_down[0] = true;
        assert!(!interact(&mut memory, &move_small, id, rect, true).dragged);

        let mut move_far = input_at(40.0, 20.0);
        move_far.buttons_down[0] = true;
        let response = interact(&mut memory, &move_far, id, rect, true);
        assert!(response.dragged);
        assert_eq!(response.drag_delta, [20.0, 0.0]);
    }

    #[test]
    fn active_widget_captures_the_pointer() {
        let mut memory = UiMemory::new();
        let first = Rect::new(0.0, 0.0, 100.0, 40.0);
        let second = Rect::new(0.0, 0.0, 100.0, 40.0);

        let mut down = input_at(20.0, 20.0);
        down.buttons_down[0] = true;
        down.buttons_pressed[0] = true;
        interact(&mut memory, &down, WidgetId::new(1), first, true);

        let hover_other = interact(&mut memory, &down, WidgetId::new(2), second, true);
        assert!(!hover_other.hovered);
    }
}
