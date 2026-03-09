use ferrous_app::DrawContext;

use crate::{c_border, TOP_H};

/// Dibuja solo el borde separador derecho del panel izquierdo.
/// El fondo y los widgets del panel son gestionados por `UiTree`.
pub fn draw_border(dc: &mut DrawContext<'_, '_>, left_w: f32) {
    let (_, win_h) = dc.ctx.window_size;
    let canvas_h = win_h as f32 - TOP_H;

    // Línea de borde entre el panel izquierdo y el canvas
    dc.gui.rect(left_w - 2.0, TOP_H, 2.0, canvas_h, c_border());
}
