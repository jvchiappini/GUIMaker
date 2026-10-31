//! Immediate-mode widgets built on `Ui` + `Backend`.

pub mod text_edit;

use text_edit::{TextEditState, TextFilter};

use crate::backend::PointerButton;
use crate::context::{Ui, WidgetId};
use crate::interact::Response;
use crate::layout::{Rect, TextAlign};
use crate::style::Color;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonKind {
    Primary,
    Secondary,
    Ghost,
    Danger,
}

impl ButtonKind {
    fn colors(self, ui: &Ui<'_>) -> (Color, Color) {
        let p = &ui.style().palette;
        match self {
            ButtonKind::Primary => (p.accent, p.text),
            ButtonKind::Danger => (p.danger, p.text),
            ButtonKind::Ghost => (Color::TRANSPARENT, p.text_dim),
            ButtonKind::Secondary => (p.surface_raised, p.text),
        }
    }
}

impl Ui<'_> {
    fn text_color(&self) -> Color {
        self.style().palette.text
    }

    pub fn label(&mut self, rect: Rect, text: &str, font_size: f32, color: Color) {
        self.text(rect, text, font_size, color, TextAlign::LEFT_CENTER);
    }

    pub fn heading(&mut self, rect: Rect, text: &str) {
        let color = self.style().palette.text_muted;
        self.text(rect, text, self.style().type_scale.caption, color, TextAlign::LEFT_CENTER);
    }

    pub fn section(&mut self, rect: Rect, title: &str) {
        let color = self.style().palette.text_muted;
        let size = self.style().type_scale.caption;
        self.text(rect, title, size, color, TextAlign::LEFT_CENTER);
        let width = self.measure_text(title, size);
        let rule_x = rect.x + width + self.style().spacing.md;
        let y = rect.y + rect.height * 0.5;
        self.rect(
            Rect::new(rule_x, y - 0.5, (rect.right() - rule_x).max(0.0), 1.0),
            self.style().palette.border,
        );
    }

    pub fn separator(&mut self, rect: Rect) {
        self.rect(rect, self.style().palette.border);
    }

    pub fn badge(&mut self, rect: Rect, text: &str, color: Color) {
        let radii = [rect.height * 0.5; 4];
        self.rect_radii(rect, color.with_alpha(color.a * 0.18), radii);
        self.text(rect, text, self.style().type_scale.caption, color, TextAlign::CENTER);
    }

    pub fn kbd(&mut self, rect: Rect, text: &str) {
        let p = self.style().palette;
        self.rect_radii(rect, p.surface_raised, [self.style().radius.sm; 4]);
        self.text(rect, text, self.style().type_scale.caption, p.text_dim, TextAlign::CENTER);
    }

    pub fn hint(&mut self, rect: Rect, text: &str) {
        let color = self.style().palette.text_muted;
        self.text(rect, text, self.style().type_scale.caption, color, TextAlign::LEFT_TOP);
    }

    pub fn progress(&mut self, rect: Rect, value: f32, color: Color) {
        let p = self.style().palette;
        let t = value.clamp(0.0, 1.0);
        self.rect_radii(rect, p.surface_raised, [rect.height * 0.5; 4]);
        if t > 0.0 {
            self.rect_radii(
                Rect::new(rect.x, rect.y, rect.width * t, rect.height),
                color,
                [rect.height * 0.5; 4],
            );
        }
    }

    pub fn button(&mut self, label: &str, rect: Rect, kind: ButtonKind) -> Response {
        self.button_id(label, label, rect, kind)
    }

    pub fn button_id(&mut self, id_label: &str, label: &str, rect: Rect, kind: ButtonKind) -> Response {
        let id = self.id(id_label);
        let response = self.interact(id, rect, true);
        let (base, text) = kind.colors(self);
        let p = self.style().palette;
        let active = self.memory.active == Some(id);
        let bg = if active {
            base.with_alpha((base.a * 0.85).max(0.85))
        } else if response.hovered {
            match kind {
                ButtonKind::Ghost => p.surface_hover,
                _ => base.with_alpha((base.a * 0.75).max(0.35)),
            }
        } else {
            base
        };
        let radii = [self.style().radius.md; 4];
        if bg.a > 0.0 {
            self.rect_radii(rect, bg, radii);
        }
        if kind == ButtonKind::Secondary && !response.hovered {
            self.border(rect, p.border, 1.0, radii);
        }
        self.text(
            rect,
            label,
            self.style().type_scale.body,
            text,
            TextAlign::CENTER,
        );
        response
    }

    pub fn glyph_button(&mut self, label: &str, rect: Rect) -> Response {
        self.glyph_button_id(label, label, rect)
    }

    pub fn glyph_button_id(&mut self, id_label: &str, glyph: &str, rect: Rect) -> Response {
        let id = self.id(id_label);
        let response = self.interact(id, rect, true);
        let p = self.style().palette;
        let bg = if response.hovered { p.surface_hover } else { p.surface_raised };
        self.rect_radii(rect, bg, [self.style().radius.md; 4]);
        self.text(rect, glyph, self.style().type_scale.label, p.text, TextAlign::CENTER);
        response
    }

    pub fn toggle(&mut self, label: &str, rect: Rect, value: &mut bool) -> Response {
        let id = self.id(label);
        let mut response = self.interact(id, rect, true);
        if response.clicked {
            *value = !*value;
            response.changed = true;
        }
        let p = self.style().palette;
        let pill_w = 38.0;
        let pill_h = (rect.height - 6.0).min(20.0).max(14.0);
        let pill = Rect::new(rect.right() - pill_w, rect.y + (rect.height - pill_h) * 0.5, pill_w, pill_h);
        self.text(
            Rect::new(rect.x, rect.y, (rect.width - pill_w - 8.0).max(0.0), rect.height),
            label,
            self.style().type_scale.body,
            self.text_color(),
            TextAlign::LEFT_CENTER,
        );
        let bg = if *value { p.success } else { p.surface_raised };
        self.rect_radii(pill, bg, [pill_h * 0.5; 4]);
        let knob = pill_h - 4.0;
        let kx = if *value { pill.right() - knob - 2.0 } else { pill.x + 2.0 };
        self.rect_radii(
            Rect::new(kx, pill.y + 2.0, knob, knob),
            Color::WHITE,
            [knob * 0.5; 4],
        );
        response
    }

    pub fn checkbox(&mut self, label: &str, rect: Rect, value: &mut bool) -> Response {
        let id = self.id(label);
        let mut response = self.interact(id, rect, true);
        if response.clicked {
            *value = !*value;
            response.changed = true;
        }
        let p = self.style().palette;
        let box_size = (rect.height - 6.0).min(16.0).max(12.0);
        let bx = rect.x;
        let by = rect.y + (rect.height - box_size) * 0.5;
        let bg = if *value { p.accent } else { p.surface_raised };
        self.rect_radii(Rect::new(bx, by, box_size, box_size), bg, [3.0; 4]);
        if *value {
            self.text(
                Rect::new(bx, by, box_size, box_size),
                "x",
                box_size * 0.75,
                Color::WHITE,
                TextAlign::CENTER,
            );
        }
        self.text(
            Rect::new(bx + box_size + 8.0, rect.y, rect.width - box_size - 8.0, rect.height),
            label,
            self.style().type_scale.body,
            self.text_color(),
            TextAlign::LEFT_CENTER,
        );
        response
    }

    pub fn radio(&mut self, label: &str, rect: Rect, selected: bool) -> Response {
        let id = self.id(label);
        let response = self.interact(id, rect, true);
        let p = self.style().palette;
        let size = (rect.height - 6.0).min(16.0).max(12.0);
        let cx = rect.x + size * 0.5;
        let cy = rect.y + rect.height * 0.5;
        let circle = Rect::new(cx - size * 0.5, cy - size * 0.5, size, size);
        self.rect_radii(circle, if selected { p.accent } else { p.surface_raised }, [size * 0.5; 4]);
        if selected {
            let inner = size * 0.38;
            self.rect_radii(
                Rect::new(cx - inner * 0.5, cy - inner * 0.5, inner, inner),
                Color::WHITE,
                [inner * 0.5; 4],
            );
        }
        self.text(
            Rect::new(rect.x + size + 8.0, rect.y, rect.width - size - 8.0, rect.height),
            label,
            self.style().type_scale.body,
            self.text_color(),
            TextAlign::LEFT_CENTER,
        );
        response
    }

    pub fn segmented(&mut self, label: &str, rect: Rect, options: &[&str], active: usize) -> Option<usize> {
        let n = options.len();
        if n == 0 {
            return None;
        }
        let width = rect.width / n as f32;
        let p = self.style().palette;
        let mut clicked = None;
        for (i, option) in options.iter().enumerate() {
            let cell = Rect::new(rect.x + i as f32 * width, rect.y, width, rect.height);
            let id = self.id(&format!("{label}.{i}"));
            let response = self.interact(
                id,
                Rect::new(cell.x + 1.0, cell.y, (cell.width - 2.0).max(0.0), cell.height),
                true,
            );
            let bg = if i == active {
                p.accent
            } else if response.hovered {
                p.surface_hover
            } else {
                p.surface_raised
            };
            self.rect_radii(
                Rect::new(cell.x + 1.0, cell.y, (cell.width - 2.0).max(0.0), cell.height),
                bg,
                [self.style().radius.sm; 4],
            );
            self.text(
                cell,
                option,
                self.style().type_scale.body,
                if i == active { Color::WHITE } else { p.text_dim },
                TextAlign::CENTER,
            );
            if response.clicked {
                clicked = Some(i);
            }
        }
        clicked
    }

    pub fn chips(&mut self, label: &str, rect: Rect, options: &[&str], active: usize) -> Option<usize> {
        let n = options.len();
        if n == 0 {
            return None;
        }
        let gap = self.style().spacing.xs;
        let width = (rect.width - gap * (n.saturating_sub(1)) as f32) / n as f32;
        let p = self.style().palette;
        let mut clicked = None;
        for (i, option) in options.iter().enumerate() {
            let cell = Rect::new(rect.x + i as f32 * (width + gap), rect.y, width, rect.height);
            let id = self.id(&format!("{label}.{i}"));
            let response = self.interact(id, cell, true);
            let bg = if i == active {
                p.accent
            } else if response.hovered {
                p.surface_hover
            } else {
                p.surface_raised
            };
            self.rect_radii(cell, bg, [self.style().radius.sm; 4]);
            self.text(
                cell,
                option,
                self.style().type_scale.caption,
                if i == active { Color::WHITE } else { p.text_dim },
                TextAlign::CENTER,
            );
            if response.clicked {
                clicked = Some(i);
            }
        }
        clicked
    }

    pub fn slider(&mut self, label: &str, rect: Rect, value: &mut f32, from: Color, to: Color) -> Response {
        let id = self.id(label);
        let mut response = self.interact(id, rect, true);
        if response.dragged || response.pressed {
            let t = ((self.input.pointer[0] - rect.x) / rect.width.max(1.0)).clamp(0.0, 1.0);
            if (*value - t).abs() > 1e-4 {
                *value = t;
                response.changed = true;
            }
        }
        let radii = [rect.height * 0.5; 4];
        self.gradient(rect, from, to, false, radii);
        self.border(rect, self.style().palette.border, 1.0, radii);
        let kx = rect.x + value.clamp(0.0, 1.0) * rect.width;
        self.rect_radii(
            Rect::new(kx - 1.0, rect.y - 1.0, 3.0, rect.height + 2.0),
            Color::WHITE,
            [2.0; 4],
        );
        response
    }

    pub fn color_swatch(&mut self, label: &str, rect: Rect, color: Color) -> Response {
        let id = self.id(label);
        let response = self.interact(id, rect, true);
        let border = if response.hovered {
            self.style().palette.border_strong
        } else {
            self.style().palette.border
        };
        self.rect_radii(rect, color.with_alpha(1.0), [self.style().radius.sm; 4]);
        self.border(rect, border, 1.0, [self.style().radius.sm; 4]);
        response
    }

    fn text_state_for(&mut self, id: WidgetId, value: &str) -> TextEditState {
        self.memory.text_states.remove(&id).unwrap_or_else(|| {
            let mut state = TextEditState::new();
            state.set_text(value.to_string(), false);
            state
        })
    }

    pub fn text_edit(&mut self, label: &str, rect: Rect, value: &mut String, filter: TextFilter) -> Response {
        let id = self.id(label);
        let mut state = self.text_state_for(id, value);
        let was_focused = self.memory.focus == Some(id);
        let blur_requested = self.memory.blur_requested == Some(id);
        let mut response = self.interact(id, rect, true);
        let pressed_inside = response.pressed;
        let pointer = self.input.pointer;

        if pressed_inside {
            if !was_focused {
                state.set_text(value.clone(), true);
                self.memory.focus = Some(id);
                self.memory.focus_rect = rect;
            } else {
                let rel = (pointer[0] - rect.x - 8.0 + state.scroll).max(0.0);
                let mut measure = |s: &str| self.measure_text(s, self.style().type_scale.body);
                state.move_caret(state.index_at_x(rel, self.style().type_scale.body, &mut measure), false);
            }
        }

        if blur_requested {
            *value = state.text().to_string();
            response.lost_focus = true;
        }

        let focused = self.memory.focus == Some(id);
        if was_focused && !focused {
            response.lost_focus = true;
        }

        let mut event = text_edit::TextEvent::default();
        if focused {
            self.memory.focus_seen = true;
            self.set_wants_keyboard(true);
            if self.input.button_pressed(PointerButton::Left) && !rect.contains(pointer) {
                *value = state.text().to_string();
                self.memory.focus = None;
                response.lost_focus = true;
            } else {
                event = self.handle_text_keys(&mut state, filter);
                if event.changed {
                    *value = state.text().to_string();
                }
                if event.submitted {
                    *value = state.text().to_string();
                    self.memory.focus = None;
                }
                if event.cancelled {
                    state.restore_seed();
                    *value = state.text().to_string();
                    self.memory.focus = None;
                }
            }
        }

        let focused = self.memory.focus == Some(id);
        self.draw_text_field(rect, value, &mut state, focused);
        response.changed |= event.changed;
        response.submitted |= event.submitted;
        response.cancelled |= event.cancelled;
        response.has_focus = focused;
        self.memory.text_states.insert(id, state);
        response
    }

    fn handle_text_keys(&mut self, state: &mut TextEditState, filter: TextFilter) -> text_edit::TextEvent {
        let mut event = text_edit::TextEvent::default();
        let ctrl = self.input.ctrl();
        let shift = self.input.shift();
        let size = self.style().type_scale.body;

        if ctrl {
            if self.input.key_pressed(crate::backend::Key::A) {
                state.select_all();
            }
            if self.input.key_pressed(crate::backend::Key::C) {
                if let Some((a, b)) = state.selection() {
                    let text = state.slice(a, b);
                    if let Some(backend) = self.backend.as_mut() {
                        backend.clipboard_write(&text);
                    }
                }
            }
            if self.input.key_pressed(crate::backend::Key::X) {
                if let Some((a, b)) = state.selection() {
                    let text = state.slice(a, b);
                    if let Some(backend) = self.backend.as_mut() {
                        backend.clipboard_write(&text);
                    }
                    if state.delete_selection() {
                        event.changed = true;
                    }
                }
            }
            if self.input.key_pressed(crate::backend::Key::V) {
                if let Some(backend) = self.backend.as_mut() {
                    backend.clipboard_request_read();
                }
                state.pending_paste = true;
            }
            if self.input.key_pressed(crate::backend::Key::Z) {
                let changed = if shift { state.redo() } else { state.undo() };
                event.changed |= changed;
            }
            if self.input.key_pressed(crate::backend::Key::Y) {
                event.changed |= state.redo();
            }
        }

        let _ = size;
        if self.input.key_pressed(crate::backend::Key::ArrowLeft) {
            let to = if ctrl { state.prev_word(state.caret()) } else { state.caret().saturating_sub(1) };
            state.move_caret(to, shift);
        }
        if self.input.key_pressed(crate::backend::Key::ArrowRight) {
            let to = if ctrl { state.next_word(state.caret()) } else { state.caret() + 1 };
            state.move_caret(to, shift);
        }
        if self.input.key_pressed(crate::backend::Key::Home) {
            state.move_caret(0, shift);
        }
        if self.input.key_pressed(crate::backend::Key::End) {
            let n = state.char_len();
            state.move_caret(n, shift);
        }
        if self.input.key_pressed(crate::backend::Key::Backspace) {
            event.changed |= state.delete_backward(ctrl);
        }
        if self.input.key_pressed(crate::backend::Key::Delete) {
            event.changed |= state.delete_forward(ctrl);
        }

        if self.input.key_pressed(crate::backend::Key::Enter)
            || self.input.key_pressed(crate::backend::Key::NumpadEnter)
        {
            event.submitted = true;
        }
        if self.input.key_pressed(crate::backend::Key::Escape) {
            event.cancelled = true;
        }

        let typed: String = self
            .input
            .typed_chars
            .iter()
            .copied()
            .filter(|c| filter.accepts(*c))
            .collect();
        if !typed.is_empty() && state.apply_insert(&typed, filter) {
            event.changed = true;
        }

        if state.pending_paste {
            let ready = self.backend.as_mut().and_then(|backend| backend.clipboard_take());
            if let Some(text) = ready {
                if state.apply_insert(&text, filter) {
                    event.changed = true;
                }
                state.pending_paste = false;
            } else {
                state.tick_paste();
            }
        }
        event
    }

    fn draw_text_field(&mut self, rect: Rect, value: &str, state: &mut TextEditState, focused: bool) {
        let p = self.style().palette;
        let size = self.style().type_scale.body;
        let pad = 8.0;
        let bg = if focused {
            p.selection
        } else if rect.contains(self.input.pointer) {
            p.surface_hover
        } else {
            p.surface_raised
        };
        self.rect_radii(rect, bg, [self.style().radius.sm; 4]);
        if focused {
            self.border(rect, p.accent.with_alpha(0.8), 1.5, [self.style().radius.sm; 4]);
        }
        let inner = Rect::new(rect.x + pad, rect.y, (rect.width - pad * 2.0).max(1.0), rect.height);
        self.push_clip(inner);

        let shown: String;
        if focused {
            shown = state.text().to_string();
        } else if value.is_empty() {
            shown = String::new();
        } else {
            shown = value.to_string();
        }

        let x_at = |state: &TextEditState, idx: usize, ui: &mut Ui<'_>| -> f32 {
            let mut measure = |s: &str| ui.measure_text(s, size);
            state.x_at_char(idx, size, &mut measure)
        };

        if focused {
            let caret_x = x_at(state, state.caret(), self);
            let text_w = self.measure_text(&shown, size);
            let max_scroll = (text_w - inner.width).max(0.0);
            if caret_x - state.scroll > inner.width {
                state.scroll = caret_x - inner.width;
            }
            if caret_x - state.scroll < 0.0 {
                state.scroll = caret_x;
            }
            state.scroll = state.scroll.clamp(0.0, max_scroll);
        } else {
            state.scroll = 0.0;
        }

        let origin = inner.x - state.scroll;
        if focused {
            if let Some((a, b)) = state.selection() {
                let (x0, x1) = {
                    let ax = x_at(state, a, self);
                    let bx = x_at(state, b, self);
                    ((origin + ax).max(inner.x), (origin + bx).min(inner.right()))
                };
                if x1 > x0 {
                    self.rect(
                        Rect::new(x0, rect.y + 2.0, x1 - x0, rect.height - 4.0),
                        p.accent.with_alpha(0.30),
                    );
                }
            }
        }
        let ty = rect.y + (rect.height - size) * 0.5;
        let color = if shown.is_empty() { p.text_muted } else { p.text };
        self.text(Rect::new(origin, ty, shown.len() as f32 * size, size), &shown, size, color, TextAlign::LEFT_TOP);
        if focused {
            let blink = (self.input.time * 2.0) as i64 % 2 == 0;
            if blink {
                let cx = (origin + x_at(state, state.caret(), self)).clamp(inner.x, inner.right());
                self.rect(Rect::new(cx, rect.y + 3.0, 1.5, rect.height - 6.0), p.text);
            }
        }
        self.pop_clip();
    }


    pub fn dropdown(&mut self, label: &str, rect: Rect, options: &[&str], active: usize) -> Option<usize> {
        let id = self.id(label);
        let response = self.interact(id, rect, true);
        let mut open = self.memory.open_popup == Some(id);
        if response.clicked {
            open = !open;
            self.memory.open_popup = if open { Some(id) } else { None };
        }
        let p = self.style().palette;
        self.rect_radii(
            rect,
            if response.hovered { p.surface_hover } else { p.surface_raised },
            [self.style().radius.sm; 4],
        );
        self.text(
            Rect::new(rect.x + 8.0, rect.y, (rect.width - 28.0).max(0.0), rect.height),
            options.get(active).copied().unwrap_or(""),
            self.style().type_scale.body,
            p.text,
            TextAlign::LEFT_CENTER,
        );
        self.text(
            Rect::new(rect.right() - 22.0, rect.y, 18.0, rect.height),
            if open { "^" } else { "v" },
            self.style().type_scale.caption,
            p.text_dim,
            TextAlign::CENTER,
        );

        if !open {
            return None;
        }

        let item_h = rect.height.max(22.0);
        let list_h = item_h * options.len() as f32;
        let below = rect.bottom() + 2.0;
        let above = rect.y - list_h - 2.0;
        let ly = if below + list_h <= self.input.window_size[1] { below } else { above };
        let list = Rect::new(rect.x, ly, rect.width, list_h);
        self.shadow(list, Color::linear(0.0, 0.0, 0.0, 0.35), 8.0, 0.0, [0.0, 3.0], [self.style().radius.md; 4]);
        self.rect_radii(list, p.surface_raised, [self.style().radius.md; 4]);
        self.border(list, p.border, 1.0, [self.style().radius.md; 4]);

        let mut selected = None;
        for (i, option) in options.iter().enumerate() {
            let item = Rect::new(list.x, list.y + i as f32 * item_h, list.width, item_h);
            let item_id = self.id(&format!("{label}.option.{i}"));
            let item_response = self.interact(item_id, item, true);
            if i == active || item_response.hovered {
                self.rect_radii(
                    item,
                    if i == active { p.selection } else { p.surface_hover },
                    [self.style().radius.sm; 4],
                );
            }
            self.text(
                Rect::new(item.x + 8.0, item.y, (item.width - 16.0).max(0.0), item.height),
                option,
                self.style().type_scale.body,
                p.text,
                TextAlign::LEFT_CENTER,
            );
            if item_response.clicked {
                selected = Some(i);
            }
        }

        let [mx, my] = self.input.pointer;
        if self.input.button_pressed(PointerButton::Left)
            && !rect.contains([mx, my])
            && !list.contains([mx, my])
        {
            self.memory.open_popup = None;
        }
        if selected.is_some() {
            self.memory.open_popup = None;
        }
        selected
    }

    pub fn number(
        &mut self,
        label: &str,
        rect: Rect,
        value: &mut f32,
        min: f32,
        max: f32,
        decimals: usize,
        unit: &str,
    ) -> Response {
        let id = self.id(label);
        let focused = self.memory.focus == Some(id);
        let mut buffer = if focused {
            self.memory.text_buffers.remove(&id).unwrap_or_else(|| format!("{:.*}", decimals, *value))
        } else {
            format!("{:.*}", decimals, *value)
        };
        let response = self.text_edit(label, rect, &mut buffer, TextFilter::Numeric);
        if response.has_focus || response.changed {
            self.memory.text_buffers.insert(id, buffer.clone());
        }
        if response.changed {
            if let Ok(parsed) = buffer.parse::<f32>() {
                *value = parsed;
            }
        }
        if response.submitted || response.lost_focus {
            if let Ok(parsed) = buffer.parse::<f32>() {
                *value = parsed.clamp(min, max);
            }
            self.memory.text_buffers.remove(&id);
        }
        if !unit.is_empty() && !focused {
            let hint_rect = Rect::new(rect.right() - 24.0, rect.y, 20.0, rect.height);
            self.text(hint_rect, unit, self.style().type_scale.caption, self.style().palette.text_muted, TextAlign::RIGHT_CENTER);
        }
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn button_click_requires_release_inside() {
        use crate::backend::HeadlessBackend;
        use crate::context::UiMemory;
        use crate::style::Style;

        let style = Style::koncreta_dark();
        let mut memory = UiMemory::new();
        let mut backend = HeadlessBackend::with_window(200.0, 100.0);
        backend.set_pointer(20.0, 20.0);
        backend.press(PointerButton::Left);
        let mut ui = crate::Ui::begin(&mut backend, &style, &mut memory);
        let _ = ui.button("ok", Rect::new(0.0, 0.0, 80.0, 40.0), ButtonKind::Primary);
        let _ = ui.end();

        let mut backend = HeadlessBackend::with_window(200.0, 100.0);
        backend.set_pointer(20.0, 20.0);
        backend.release(PointerButton::Left);
        let mut ui = crate::Ui::begin(&mut backend, &style, &mut memory);
        let response = ui.button("ok", Rect::new(0.0, 0.0, 80.0, 40.0), ButtonKind::Primary);
        assert!(response.clicked);
    }
}
