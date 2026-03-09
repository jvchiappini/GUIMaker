use ferrous_app::DrawContext;

use crate::{c_border, TOP_H};

/// Dibuja solo el borde separador izquierdo del panel derecho.
/// El fondo y los widgets del panel son gestionados por `UiTree`.
pub fn draw_border(dc: &mut DrawContext<'_, '_>, right_w: f32) {
    let (win_w, win_h) = dc.ctx.window_size;
    let ww = win_w as f32;
    let canvas_h = win_h as f32 - TOP_H;

    let right_x = ww - right_w;

    // Línea de borde entre el canvas y el panel derecho
    dc.gui.rect(right_x, TOP_H, 2.0, canvas_h, c_border());
}
