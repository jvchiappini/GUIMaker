use ferrous_app::{AppContext, DrawContext, MouseButton};

use crate::{c_top, TOP_H};

// ── Update ────────────────────────────────────────────────────────────────────

/// Gestiona los botones de control de ventana (cerrar, maximizar, minimizar)
/// y el drag de la barra superior.
/// Devuelve `true` si se solicitó salir de la aplicación.
pub fn update(
    ctx: &mut AppContext,
    drag_offset: &mut Option<(i32, i32)>,
    is_maximized: &mut bool,
) -> bool {
    let (win_w, win_h) = ctx.window_size;
    let ww = win_w as f32;
    let (mx, my) = ctx.input.mouse_pos_f32();

    // ── Botón cerrar [×] ─────────────────────────────────────────────────────
    let close_x = ww - TOP_H;
    let over_close = mx >= close_x && mx <= ww && my >= 0.0 && my <= TOP_H;
    if over_close && ctx.input.button_just_pressed(MouseButton::Left) {
        return true;
    }

    // ── Botón maximizar [□] ──────────────────────────────────────────────────
    let max_x = ww - TOP_H * 2.0;
    let over_max = mx >= max_x && mx < close_x && my >= 0.0 && my <= TOP_H;
    if over_max && ctx.input.button_just_pressed(MouseButton::Left) {
        *is_maximized = !*is_maximized;
        ctx.window.set_maximized(*is_maximized);
    }

    // ── Botón minimizar [–] ──────────────────────────────────────────────────
    let min_x = ww - TOP_H * 3.0;
    let over_min = mx >= min_x && mx < max_x && my >= 0.0 && my <= TOP_H;
    if over_min && ctx.input.button_just_pressed(MouseButton::Left) {
        ctx.window.set_minimized(true);
    }

    // ── Drag de la barra ─────────────────────────────────────────────────────
    let drag_area_w = ww - TOP_H * 3.0;
    let over_drag = mx >= 0.0
        && mx < drag_area_w
        && my >= 0.0
        && my <= TOP_H
        && crate::resize_direction(mx, my, win_w, win_h).is_none();

    if ctx.input.button_just_pressed(MouseButton::Left) && over_drag {
        let win_pos = ctx.window_position().unwrap_or((0, 0));
        let screen_mx = win_pos.0 + mx as i32;
        let screen_my = win_pos.1 + my as i32;
        *drag_offset = Some((screen_mx - win_pos.0, screen_my - win_pos.1));
    }
    if ctx.input.button_just_released(MouseButton::Left) {
        *drag_offset = None;
    }
    if ctx.input.is_button_down(MouseButton::Left) {
        if let Some(offset) = *drag_offset {
            let win_pos = ctx.window_position().unwrap_or((0, 0));
            let screen_mx = win_pos.0 + mx as i32;
            let screen_my = win_pos.1 + my as i32;
            ctx.set_window_position(screen_mx - offset.0, screen_my - offset.1);
        }
    }

    false
}

// ── Draw ──────────────────────────────────────────────────────────────────────

pub fn draw(dc: &mut DrawContext<'_, '_>, zoom: f32, is_maximized: bool) {
    let (win_w, _) = dc.ctx.window_size;
    let ww = win_w as f32;
    let canvas_x = crate::LEFT_W;
    let (mx, my) = dc.ctx.input.mouse_pos_f32();

    dc.gui.rect(0.0, 0.0, ww, TOP_H, c_top());
    // Línea de acento inferior: azul VSCode #007ACC, 2px
    dc.gui.rect(
        0.0,
        TOP_H - 2.0,
        ww,
        2.0,
        ferrous_app::Color::hex("#007ACC").to_linear_f32(),
    );

    // Logo / título — blanco VSCode
    dc.text.draw_text(
        dc.font,
        "GUIMaker",
        [14.0, 12.0],
        15.0,
        ferrous_app::Color::hex("#CCCCCC").to_linear_f32(),
    );

    // Indicador de zoom — azul acento VSCode
    let zoom_label = format!("zoom  {:.0}%", zoom * 100.0);
    dc.text.draw_text(
        dc.font,
        &zoom_label,
        [canvas_x + 8.0, 13.0],
        11.0,
        ferrous_app::Color::hex("#3DC9B0").to_linear_f32(),
    );

    // ── Cerrar [×] ───────────────────────────────────────────────────────────
    let close_x = ww - TOP_H;
    let hover_close = mx >= close_x && mx <= ww && my >= 0.0 && my <= TOP_H;
    let c_close = if hover_close {
        ferrous_app::Color::hex("#F14C4C").to_linear_f32()
    } else {
        c_top()
    };
    dc.gui.rect_r(close_x, 0.0, TOP_H, TOP_H, c_close, 0.0);
    dc.text.draw_text(
        dc.font,
        "×",
        [close_x + TOP_H * 0.28, 9.0],
        18.0,
        [1.0, 0.70, 0.78, 1.0], // rosa claro
    );

    // ── Maximizar [□ / ❐] ────────────────────────────────────────────────────
    let max_x = ww - TOP_H * 2.0;
    let hover_max = mx >= max_x && mx < close_x && my >= 0.0 && my <= TOP_H;
    let c_max = if hover_max {
        ferrous_app::Color::hex("#505050").to_linear_f32()
    } else {
        c_top()
    };
    dc.gui.rect(max_x, 0.0, TOP_H, TOP_H, c_max);
    let max_icon = if is_maximized { "❐" } else { "□" };
    dc.text.draw_text(
        dc.font,
        max_icon,
        [max_x + TOP_H * 0.22, 8.0],
        16.0,
        ferrous_app::Color::hex("#CCCCCC").to_linear_f32(),
    );

    // ── Minimizar [–] ────────────────────────────────────────────────────────
    let min_x = ww - TOP_H * 3.0;
    let hover_min = mx >= min_x && mx < max_x && my >= 0.0 && my <= TOP_H;
    let c_min = if hover_min {
        ferrous_app::Color::hex("#505050").to_linear_f32()
    } else {
        c_top()
    };
    dc.gui.rect(min_x, 0.0, TOP_H, TOP_H, c_min);
    dc.text.draw_text(
        dc.font,
        "–",
        [min_x + TOP_H * 0.28, 9.0],
        18.0,
        ferrous_app::Color::hex("#CCCCCC").to_linear_f32(),
    );
}
