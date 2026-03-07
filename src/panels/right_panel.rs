use ferrous_app::DrawContext;

use crate::{c_border, c_right, RIGHT_W, TOP_H};

pub fn draw(dc: &mut DrawContext<'_, '_>) {
    let (win_w, win_h) = dc.ctx.window_size;
    let ww = win_w as f32;
    let canvas_h = win_h as f32 - TOP_H;
    let canvas_y = TOP_H;

    let right_x = ww - RIGHT_W;

    dc.gui.rect(right_x, TOP_H, RIGHT_W, canvas_h, c_right());
    dc.gui.rect(right_x, TOP_H, 2.0, canvas_h, c_border());

    dc.text.draw_text(
        dc.font,
        "Propiedades",
        [right_x + 12.0, canvas_y + 11.0],
        11.0,
        ferrous_app::Color::hex("#BBBBBB").to_linear_f32(),
    );
}
