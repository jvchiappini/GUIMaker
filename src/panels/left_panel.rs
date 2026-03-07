use ferrous_app::DrawContext;

use crate::{c_border, c_left, TOP_H};

pub fn draw(dc: &mut DrawContext<'_, '_>, left_w: f32) {
    let (_, win_h) = dc.ctx.window_size;
    let canvas_h = win_h as f32 - TOP_H;
    let canvas_y = TOP_H;

    dc.gui.rect(0.0, TOP_H, left_w, canvas_h, c_left());
    dc.gui.rect(left_w - 2.0, TOP_H, 2.0, canvas_h, c_border());

    dc.text.draw_text(
        dc.font,
        "Herramientas",
        [12.0, canvas_y + 11.0],
        11.0,
        ferrous_app::Color::hex("#BBBBBB").to_linear_f32(),
    );
}
