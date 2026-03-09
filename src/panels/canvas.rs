// No changes required in the update function signature.
use ferrous_app::{AppContext, DrawContext, MouseButton};

use crate::{c_canvas, c_grid, TOP_H};

// ── Update ────────────────────────────────────────────────────────────────────

/// Gestiona el zoom con la rueda y el paneo con botón medio / derecho.
pub fn update(
    ctx: &mut AppContext,
    zoom: &mut f32,
    pan_x: &mut f32,
    pan_y: &mut f32,
    last_mx: &mut f32,
    last_my: &mut f32,
    left_w: f32,
    right_w: f32,
) {
    let (win_w, win_h) = ctx.window_size;
    let ww = win_w as f32;
    let wh = win_h as f32;
    let (mx, my) = ctx.input.mouse_pos_f32();

    let canvas_x = left_w;
    let canvas_y = TOP_H;
    let canvas_w = ww - left_w - right_w;
    let canvas_h = wh - TOP_H;

    let over_canvas = mx >= canvas_x
        && mx <= canvas_x + canvas_w
        && my >= canvas_y
        && my <= canvas_y + canvas_h;

    // Zoom con rueda
    if over_canvas {
        let scroll = ctx.input.scroll_delta().1;
        if scroll != 0.0 {
            let factor = if scroll > 0.0 { 1.1_f32 } else { 1.0 / 1.1 };
            let cx = mx - canvas_x;
            let cy = my - canvas_y;
            *pan_x = cx - (cx - *pan_x) * factor;
            *pan_y = cy - (cy - *pan_y) * factor;
            *zoom = (*zoom * factor).clamp(0.1, 10.0);
        }
    }

    // Paneo con botón medio o derecho
    let panning = ctx.input.is_button_down(MouseButton::Middle)
        || ctx.input.is_button_down(MouseButton::Right);
    if panning && over_canvas {
        *pan_x += mx - *last_mx;
        *pan_y += my - *last_my;
    }

    *last_mx = mx;
    *last_my = my;
}

// ── Draw ──────────────────────────────────────────────────────────────────────

pub fn draw(
    dc: &mut DrawContext<'_, '_>,
    zoom: f32,
    pan_x: f32,
    pan_y: f32,
    left_w: f32,
    right_w: f32,
    preview_w: f32,
    preview_h: f32,
    preview_responsive: bool,
) {
    let (win_w, win_h) = dc.ctx.window_size;
    let ww = win_w as f32;
    let wh = win_h as f32;

    let canvas_x = left_w;
    let canvas_y = TOP_H;
    let canvas_w = ww - left_w - right_w;
    let canvas_h = wh - TOP_H;

    // Fondo del canvas
    dc.gui.rect(canvas_x, canvas_y, canvas_w, canvas_h, c_canvas());

    // Grid de líneas
    let grid = 20.0 * zoom;
    let offset_x = pan_x % grid;
    let offset_y = pan_y % grid;

    let color_grid = c_grid();
    let mut gx = canvas_x + offset_x;
    while gx < canvas_x + canvas_w {
        dc.gui.rect(gx, canvas_y, 1.0, canvas_h, color_grid);
        gx += grid;
    }
    let mut gy = canvas_y + offset_y;
    while gy < canvas_y + canvas_h {
        dc.gui.rect(canvas_x, gy, canvas_w, 1.0, color_grid);
        gy += grid;
    }

    // Ejes de origen (azulados semitransparentes)
    let origin_x = canvas_x + canvas_w * 0.5 + pan_x;
    let origin_y = canvas_y + canvas_h * 0.5 + pan_y;
    dc.gui.rect(
        origin_x - 0.5,
        canvas_y,
        1.0,
        canvas_h,
        ferrous_app::Color::hex("#007ACC60").to_linear_f32(),
    );
    dc.gui.rect(
        canvas_x,
        origin_y - 0.5,
        canvas_w,
        1.0,
        ferrous_app::Color::hex("#007ACC60").to_linear_f32(),
    );

    // Hint de controles
    dc.gui.draw_text(
        dc.font,
        "Rueda: zoom  |  Btn.medio / Btn.derecho: paneo",
        [canvas_x + 8.0, canvas_y + canvas_h - 18.0],
        10.0,
        ferrous_app::Color::hex("#555555").to_linear_f32(),
    );

    // Si no es responsive, dibujamos un borde que representa la "ventana de preview" con las medidas especificadas
    if !preview_responsive {
        let pw = preview_w.min(canvas_w);
        let ph = preview_h.min(canvas_h);
        let px = canvas_x + (canvas_w - pw) * 0.5;
        let py = canvas_y + (canvas_h - ph) * 0.5;
        // borde blanco semitransparente
        dc.gui.rect(px, py, pw, ph, [1.0, 1.0, 1.0, 0.3]);
        dc.gui.rect(px + 1.0, py + 1.0, pw - 2.0, ph - 2.0, [0.0, 0.0, 0.0, 0.0]);
    }
}
