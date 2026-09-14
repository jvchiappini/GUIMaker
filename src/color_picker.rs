use ferrous_engine::{DrawContext, MouseButton};
use crate::theme::{Theme, ThemeRegistry};
use crate::panels::widgets::shared::parse_hex_or;

fn with_alpha(color: [f32; 4], alpha: f32) -> [f32; 4] {
    [color[0], color[1], color[2], color[3] * alpha]
}

#[derive(Clone, Debug)]
pub struct ColorPickerState {
    pub active: bool,
    pub target_field: String,
    pub widget_id: Option<u32>,
    pub current_color: [f32; 4],
    pub screen_pos: (f32, f32),
    pub drag_mode: DragMode,
    pub drag_offset: (f32, f32),
    pub just_opened: bool,
    pub original_color: [f32; 4],
    pub h: f32,
    pub s: f32,
    pub v: f32,
    pub alpha: f32,
    pub open_anim: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DragMode {
    None,
    Window,
    Wheel,
    Brightness,
    Alpha,
}

impl Default for ColorPickerState {
    fn default() -> Self {
        Self {
            active: false,
            target_field: String::new(),
            widget_id: None,
            current_color: [1.0, 1.0, 1.0, 1.0],
            screen_pos: (0.0, 0.0),
            drag_mode: DragMode::None,
            drag_offset: (0.0, 0.0),
            just_opened: false,
            original_color: [1.0, 1.0, 1.0, 1.0],
            h: 0.0,
            s: 0.0,
            v: 1.0,
            alpha: 1.0,
            open_anim: 0.0,
        }
    }
}

pub fn draw_picker_modal(
    dc: &mut DrawContext<'_, '_>,
    state: &mut ColorPickerState,
    current_hex: String,
    theme: &Theme,
) -> (Option<String>, bool) {
    if !state.active {
        return (None, false);
    }

    let (mx, my) = dc.ctx.input.mouse_pos_f32();
    let clicked = dc.ctx.input.button_just_pressed(MouseButton::Left);
    let pressed = dc.ctx.input.is_button_down(MouseButton::Left);

    let base_w = 280.0;
    let base_h = 520.0; // más holgura para controles + paleta

    let mut x = state.screen_pos.0;
    let mut y = state.screen_pos.1;

    if state.open_anim < 1.0 {
        state.open_anim = (state.open_anim + 0.03).min(1.0);
    }
    let anim = state.open_anim;
    let ease = 1.0 - (1.0 - anim).powi(3);
    let scale = 0.85 + 0.15 * ease;
    let alpha = anim;

    if state.just_opened {
        let (win_w, win_h) = dc.ctx.window_size;
        let center_x = (win_w as f32 - base_w) * 0.5;
        let center_y = (win_h as f32 - base_h) * 0.5;
        state.screen_pos = (center_x, center_y);
        x = center_x;
        y = center_y;
    }

    let w = base_w * scale;
    let h = base_h * scale;
    let center_x = x + w * 0.5;
    let center_y = y + h * 0.5;
    x = center_x + (x - center_x) * scale;
    y = center_y + (y - center_y) * scale;

    let s = |v: f32| -> f32 { v * scale };

    let header_rect = [x, y, w, s(30.0)];
    let wheel_size = s(180.0);
    let wheel_x = x + (w - wheel_size) * 0.5;
    let wheel_y = y + s(40.0);

    let slider_y = wheel_y + wheel_size + s(20.0);
    let slider_w = w - s(40.0);
    let slider_h = s(10.0);
    let slider_x = x + s(20.0);

    let alpha_y = slider_y + s(28.0);
    let alpha_h = s(10.0);
    let alpha_x = x + s(20.0);

    if state.just_opened {
        let parsed = parse_hex_or(&current_hex, [1.0, 1.0, 1.0, 1.0]);
        state.current_color = parsed;
        let (h, s, v) = rgb_to_hsv(parsed[0], parsed[1], parsed[2]);
        state.h = h;
        state.s = s;
        state.v = v;
        if (state.alpha - 1.0).abs() < f32::EPSILON {
            state.alpha = parsed[3];
        }
        state.current_color = [parsed[0], parsed[1], parsed[2], state.alpha];
        state.original_color = state.current_color;
    }

    if state.drag_mode != DragMode::None {
        if !pressed {
            state.drag_mode = DragMode::None;
        } else {
            match state.drag_mode {
                DragMode::Window => {
                    x = mx - state.drag_offset.0;
                    y = my - state.drag_offset.1;
                    x = x.clamp(0.0, dc.ctx.window_size.0 as f32 - w);
                    y = y.clamp(0.0, dc.ctx.window_size.1 as f32 - h);
                    state.screen_pos = (x, y);
                }
                DragMode::Wheel => {
                    let cx = wheel_x + wheel_size * 0.5;
                    let cy = wheel_y + wheel_size * 0.5;
                    let dx = (mx - cx) / (wheel_size * 0.5);
                    let dy = (my - cy) / (wheel_size * 0.5);
                    let dist = (dx * dx + dy * dy).sqrt().clamp(0.0, 1.0);
                    let hue = ((dy.atan2(dx) / (2.0 * std::f32::consts::PI)) + 1.0) % 1.0;
                    state.h = hue;
                    state.s = dist;
                }
                DragMode::Brightness => {
                    let v_val = ((mx - slider_x) / slider_w).clamp(0.0, 1.0);
                    state.v = v_val;
                }
                DragMode::Alpha => {
                    let a_val = ((mx - alpha_x) / slider_w).clamp(0.0, 1.0);
                    state.alpha = a_val;
                }
                DragMode::None => {}
            }
        }
    } else if clicked {
        if mx >= header_rect[0] && mx < header_rect[0] + header_rect[2] && my >= header_rect[1] && my < header_rect[1] + header_rect[3] {
            if mx < header_rect[0] + header_rect[2] - 30.0 {
                state.drag_mode = DragMode::Window;
                state.drag_offset = (mx - x, my - y);
            }
        }

        let wheel_in = mx >= wheel_x && mx < wheel_x + wheel_size && my >= wheel_y && my < wheel_y + wheel_size;
        if wheel_in {
            state.drag_mode = DragMode::Wheel;
        }

        let slider_in = mx >= slider_x && mx < slider_x + slider_w && my >= slider_y && my < slider_y + slider_h;
        if slider_in {
            state.drag_mode = DragMode::Brightness;
        }

        let alpha_in = mx >= alpha_x && mx < alpha_x + slider_w && my >= alpha_y && my < alpha_y + alpha_h;
        if alpha_in {
            state.drag_mode = DragMode::Alpha;
        }
    }

    dc.gui.rect(x + 4.0, y + 4.0, w, h, with_alpha([0.0, 0.0, 0.0, 0.2], alpha));
    dc.gui.rect(x - 1.0, y - 1.0, w + 2.0, h + 2.0, with_alpha(ThemeRegistry::hex_to_f32(&theme.colors.border), alpha));
    dc.gui.rect(x, y, w, h, with_alpha(ThemeRegistry::hex_to_f32(&theme.colors.surface), alpha));

    dc.gui.rect(x, y, w, 30.0, with_alpha(ThemeRegistry::hex_to_f32(&theme.colors.background), alpha));
    dc.gui.draw_text(dc.font, "COLOR PICKER", [x + 12.0, y + 10.0], 12.0, with_alpha(ThemeRegistry::hex_to_f32(&theme.colors.text_dim), alpha));

    let close_btn_s = 24.0;
    let close_btn_x = x + w - close_btn_s - 3.0;
    let close_btn_y = y + 3.0;
    let close_hover = mx >= close_btn_x && mx < close_btn_x + close_btn_s && my >= close_btn_y && my < close_btn_y + close_btn_s;
    let close_clicked = clicked && close_hover;
    dc.gui.rect(close_btn_x, close_btn_y, close_btn_s, close_btn_s, with_alpha(if close_hover { [0.8, 0.2, 0.2, 1.0] } else { [0.0, 0.0, 0.0, 0.0] }, alpha));
    dc.gui.draw_text(dc.font, "X", [close_btn_x + 8.0, close_btn_y + 6.0], 12.0, with_alpha([1.0, 1.0, 1.0, 1.0], alpha));

    dc.gui.push_quad(ferrous_gui::GuiQuad {
        pos: [wheel_x, wheel_y],
        size: [wheel_size, wheel_size],
        uv0: [0.0, 0.0],
        uv1: [1.0, 1.0],
        color: [1.0, 1.0, 1.0, 1.0],
        color_b: [0.0; 4],
        radii: [wheel_size * 0.5; 4],
        tex_index: 0,
        flags: 1,
        z_order: 0.0,
        node_id: 0,
    });

    let wheel_area = mx >= wheel_x && mx < wheel_x + wheel_size && my >= wheel_y && my < wheel_y + wheel_size;
    if pressed && wheel_area {
        let dx = (mx - (wheel_x + wheel_size * 0.5)) / (wheel_size * 0.5);
        let dy = (my - (wheel_y + wheel_size * 0.5)) / (wheel_size * 0.5);
        let dist = (dx * dx + dy * dy).sqrt().clamp(0.0, 1.0);
        if dist <= 1.0 {
            state.h = ((dy.atan2(dx) / (2.0 * std::f32::consts::PI)) + 1.0) % 1.0;
            state.s = dist;
        }
    }

    let [r, g, b] = hsv_to_rgb(state.h, state.s, state.v);
    state.current_color = [r, g, b, state.alpha];

    dc.gui.draw_text(dc.font, "BRIGHTNESS", [slider_x, slider_y - 12.0], 8.0, with_alpha(ThemeRegistry::hex_to_f32(&theme.colors.text_dim), alpha));
    dc.gui.rect(slider_x, slider_y, slider_w, slider_h, with_alpha([0.0, 0.0, 0.0, 1.0], alpha));
    for i in 0..20 {
        let step_w = slider_w / 20.0;
        let v_step = i as f32 / 20.0;
        let c = hsv_to_rgb(state.h, state.s, v_step);
        dc.gui.rect(slider_x + i as f32 * step_w, slider_y, step_w, slider_h, [c[0], c[1], c[2], 1.0]);
    }

    if pressed && mx >= slider_x && mx < slider_x + slider_w && my >= slider_y && my < slider_y + slider_h {
        state.v = ((mx - slider_x) / slider_w).clamp(0.0, 1.0);
    }

    let slider_handle_x = slider_x + state.v * slider_w;
    let accent = with_alpha(ThemeRegistry::hex_to_f32(&theme.colors.accent), alpha);
    dc.gui.rect(slider_handle_x - 1.0, slider_y - 2.0, 2.0, slider_h + 4.0, accent);

    dc.gui.draw_text(dc.font, "ALPHA", [alpha_x, alpha_y - 12.0], 8.0, with_alpha(ThemeRegistry::hex_to_f32(&theme.colors.text_dim), alpha));
    dc.gui.rect(alpha_x, alpha_y, slider_w, alpha_h, with_alpha([0.0, 0.0, 0.0, 1.0], alpha));
    for i in 0..20 {
        let step_w = slider_w / 20.0;
        let alpha_step = i as f32 / 20.0;
        dc.gui.rect(alpha_x + i as f32 * step_w, alpha_y, step_w, alpha_h, [1.0, 1.0, 1.0, alpha_step]);
    }

    if pressed && mx >= alpha_x && mx < alpha_x + slider_w && my >= alpha_y && my < alpha_y + alpha_h {
        state.alpha = ((mx - alpha_x) / slider_w).clamp(0.0, 1.0);
    }

    let alpha_handle_x = alpha_x + state.alpha * slider_w;
    dc.gui.rect(alpha_handle_x - 1.0, alpha_y - 2.0, 2.0, alpha_h + 4.0, accent);

    let marker_radius = state.s * wheel_size * 0.5;
    let marker_angle = state.h * std::f32::consts::TAU;
    let marker_x = wheel_x + wheel_size * 0.5 + marker_radius * marker_angle.cos();
    let marker_y = wheel_y + wheel_size * 0.5 + marker_radius * marker_angle.sin();
    let marker_size = 8.0;
    dc.gui.rect(marker_x - marker_size * 0.5, marker_y - marker_size * 0.5, marker_size, marker_size, with_alpha([1.0, 1.0, 1.0, 0.8], alpha));
    dc.gui.rect(marker_x - (marker_size * 0.5 - 1.0), marker_y - (marker_size * 0.5 - 1.0), marker_size - 2.0, marker_size - 2.0, accent);

    let preview_y = alpha_y + s(40.0);
    let controls_y = preview_y + s(60.0);
    dc.gui.rect(x + 20.0, preview_y, 40.0, 40.0, with_alpha(state.current_color, alpha));
    dc.gui.rect(x + 20.0, preview_y, 40.0, 2.0, with_alpha([1.0, 1.0, 1.0, 0.2], alpha));
    dc.gui.draw_text(dc.font, &rgba_to_hex([r, g, b, state.alpha]).to_uppercase(), [x + 70.0, preview_y + 12.0], 14.0, with_alpha(ThemeRegistry::hex_to_f32(&theme.colors.text), alpha));

    let apply_btn_x = x + s(20.0);
    let apply_btn_y = controls_y;
    let apply_btn_w = s(100.0);
    let apply_btn_h = s(26.0);
    let cancel_btn_x = x + w - s(20.0) - apply_btn_w;
    let cancel_btn_y = apply_btn_y;
    let cancel_btn_w = apply_btn_w;
    let cancel_btn_h = apply_btn_h;
    let apply_hover = mx >= apply_btn_x && mx < apply_btn_x + apply_btn_w && my >= apply_btn_y && my < apply_btn_y + apply_btn_h;
    let cancel_hover = mx >= cancel_btn_x && mx < cancel_btn_x + cancel_btn_w && my >= cancel_btn_y && my < cancel_btn_y + cancel_btn_h;

    dc.gui.rect(apply_btn_x, apply_btn_y, apply_btn_w, apply_btn_h, if apply_hover { accent } else { [0.2, 0.6, 0.8, 1.0] });
    dc.gui.draw_text(dc.font, "Apply", [apply_btn_x + 28.0, apply_btn_y + 6.0], 12.0, with_alpha([1.0, 1.0, 1.0, 1.0], alpha));
    dc.gui.rect(cancel_btn_x, cancel_btn_y, cancel_btn_w, cancel_btn_h, if cancel_hover { [0.8, 0.2, 0.2, 1.0] } else { [0.5, 0.2, 0.2, 1.0] });
    dc.gui.draw_text(dc.font, "Cancel", [cancel_btn_x + 24.0, cancel_btn_y + 6.0], 12.0, with_alpha([1.0, 1.0, 1.0, 1.0], alpha));

    let palette_y = controls_y + s(40.0);
    let chip_s = s(25.0);
    let palette_bottom = palette_y + chip_s;
    if palette_bottom > y + h - s(10.0) {
        // Keep palette inside panel if h is small during opening animation.
        let offset = palette_bottom - (y + h - s(10.0));
        // compact controls slightly when needed.
        state.screen_pos.1 -= offset / scale;
    }
    let colors = vec!["#FFFFFF", "#000000", "#6C63FF", "#FF63A5", "#FFD93D", "#6CCF70", "#3498DB", "#E74C3C"];
    let chip_s = s(25.0);
    for (i, c) in colors.iter().enumerate() {
        let cx = x + 20.0 + i as f32 * (chip_s + 5.0);
        let cy = palette_y;
        let col = parse_hex_or(c, [1.0, 1.0, 1.0, 1.0]);
        dc.gui.rect(cx, cy, chip_s, chip_s, col);
        if clicked && mx >= cx && mx < cx + chip_s && my >= cy && my < cy + chip_s {
            let parsed = parse_hex_or(c, [1.0, 1.0, 1.0, 1.0]);
            state.current_color = parsed;
            let (h2, s2, v2) = rgb_to_hsv(parsed[0], parsed[1], parsed[2]);
            state.h = h2;
            state.s = s2;
            state.v = v2;
            state.alpha = parsed[3];
        }
    }

    let mut should_close = false;
    let mut out = Some(rgba_to_hex([r, g, b, state.alpha]));
    let in_modal = mx >= x && mx <= x + w && my >= y && my <= y + h;

    if clicked {
        if state.just_opened {
            state.just_opened = false;
        } else if close_clicked {
            should_close = true;
            out = None;
            state.current_color = state.original_color;
        } else if apply_hover {
            should_close = true;
            out = Some(rgba_to_hex([r, g, b, state.alpha]));
        } else if cancel_hover {
            should_close = true;
            out = None;
            state.current_color = state.original_color;
        } else if !in_modal {
            should_close = true;
            out = Some(rgba_to_hex([r, g, b, state.alpha]));
        }
    } else {
        state.just_opened = false;
    }

    (out, should_close)
}
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [f32; 3] {
    let h_scaled = h * 6.0;
    let i = h_scaled.floor() as i32;
    let f = h_scaled - i as f32;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);
    let (r, g, b) = match i.rem_euclid(6) {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        5 => (v, p, q),
        _ => (v, v, v),
    };
    [r, g, b]
}

fn rgb_to_hsv(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let d = max - min;
    let h = if d == 0.0 {
        0.0
    } else if max == r {
        ((g - b) / d).rem_euclid(6.0) / 6.0
    } else if max == g {
        ((b - r) / d + 2.0) / 6.0
    } else {
        ((r - g) / d + 4.0) / 6.0
    };
    let s = if max == 0.0 { 0.0 } else { d / max };
    let v = max;
    (h, s, v)
}

fn rgba_to_hex(rgba: [f32; 4]) -> String {
    let clamp_u8 = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    let r = clamp_u8(rgba[0]);
    let g = clamp_u8(rgba[1]);
    let b = clamp_u8(rgba[2]);
    let a = clamp_u8(rgba[3]);
    if a < 255 {
        format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, a)
    } else {
        format!("#{:02X}{:02X}{:02X}", r, g, b)
    }
}
