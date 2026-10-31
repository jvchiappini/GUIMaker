use ferrous_engine::{AppContext, DrawContext, MouseButton};
use crate::theme::{Theme, ThemeRegistry};

// ── Dimensiones del modal ────────────────────────────────────────────────────
const MODAL_W: f32 = 400.0;
const MODAL_H: f32 = 320.0;
const TITLE_H: f32 = 42.0;
const RADIUS: f32 = 8.0;

fn hit(mx: f32, my: f32, x: f32, y: f32, w: f32, h: f32) -> bool {
    mx >= x && mx <= x + w && my >= y && my <= y + h
}

pub fn update(
    ctx: &mut AppContext,
    preview_w: &mut f32,
    preview_h: &mut f32,
    show_modal: &mut bool,
    theme_registry: &mut ThemeRegistry,
) {
    let (win_w, win_h) = ctx.window_size;
    let ww = win_w as f32;
    let wh = win_h as f32;
    let (mx, my) = ctx.input.mouse_pos_f32();

    let dx = (ww - MODAL_W) * 0.5;
    let dy = (wh - MODAL_H) * 0.5;

    // Cerrar con ESC
    if ctx.input.just_pressed(ferrous_engine::KeyCode::Escape) {
        *show_modal = false;
        return;
    }

    let lmb = ctx.input.button_just_pressed(MouseButton::Left);
    if !lmb {
        return;
    }

    // Clic fuera → cerrar
    if !hit(mx, my, dx, dy, MODAL_W, MODAL_H) {
        *show_modal = false;
        return;
    }

    // Botón [×] en la barra de título
    let close_x = dx + MODAL_W - 32.0;
    let close_y = dy + 7.0;
    if hit(mx, my, close_x, close_y, 28.0, 28.0) {
        *show_modal = false;
        return;
    }

    // ── Controles de contenido ────────────────────────────────────────────────
    let content_y = dy + TITLE_H;
    let pad = 24.0;

    // Fila Ancho
    let w_row_y = content_y + 16.0;
    let w_box = (dx + MODAL_W * 0.5, w_row_y + 2.0, 90.0, 28.0);
    if hit(mx, my, w_box.0, w_box.1, w_box.2, w_box.3) {
        *preview_w += 10.0;
    }

    // Fila Alto
    let h_row_y = w_row_y + 44.0;
    let h_box = (dx + MODAL_W * 0.5, h_row_y + 2.0, 90.0, 28.0);
    if hit(mx, my, h_box.0, h_box.1, h_box.2, h_box.3) {
        *preview_h += 10.0;
    }

    // ── Theme Selector ────────────────────────────────────────────────────────
    let theme_row_y = h_row_y + 60.0;
    let theme_names = [("Dark", "dark"), ("Light", "light"), ("Ocean", "ocean")];
    let btn_w = (MODAL_W - pad * 2.0) / 3.0 - 4.0;

    for (i, (label, key)) in theme_names.iter().enumerate() {
        let bx = dx + pad + i as f32 * (btn_w + 6.0);
        if hit(mx, my, bx, theme_row_y + 20.0, btn_w, 32.0) {
            theme_registry.set_current(key);
        }
    }
}

pub fn draw(dc: &mut DrawContext<'_, '_>, preview_w: f32, preview_h: f32, theme: &Theme, theme_registry: &ThemeRegistry) {
    let (win_w, win_h) = dc.ctx.window_size;
    let ww = win_w as f32;
    let wh = win_h as f32;
    let (mx, my) = dc.ctx.input.mouse_pos_f32();

    let dx = (ww - MODAL_W) * 0.5;
    let dy = (wh - MODAL_H) * 0.5;

    // ── Palette ───────────────────────────────────────────────────────────────
    let c_backdrop = [0.0, 0.0, 0.0, 0.7];
    let c_surface = ThemeRegistry::hex_to_f32(&theme.colors.surface);
    let c_secondary = ThemeRegistry::hex_to_f32(&theme.colors.secondary);
    let c_accent = ThemeRegistry::hex_to_f32(&theme.colors.accent);
    let c_text = ThemeRegistry::hex_to_f32(&theme.colors.text);
    let c_text_dim = ThemeRegistry::hex_to_f32(&theme.colors.text_dim);
    let c_error = ThemeRegistry::hex_to_f32(&theme.colors.error);
    let c_border = ThemeRegistry::hex_to_f32(&theme.colors.border);

    // ── Backdrop ──────────────────────────────────────────────────────────────
    dc.gui.rect(0.0, 0.0, ww, wh, c_backdrop);

    // ── Shadow ────────────────────────────────────────────────────────────────
    dc.gui.rect_r(dx + 4.0, dy + 6.0, MODAL_W, MODAL_H, [0.0, 0.0, 0.0, 0.4], RADIUS);

    // ── Panel principal ───────────────────────────────────────────────────────
    dc.gui.rect_r(dx, dy, MODAL_W, MODAL_H, c_surface, RADIUS);

    // ── Barra de título ───────────────────────────────────────────────────────
    dc.gui.rect_r(dx, dy, MODAL_W, TITLE_H, c_secondary, RADIUS);
    dc.gui.rect(dx, dy + TITLE_H - 10.0, MODAL_W, 10.0, c_secondary); // Square bottom corners
    
    // Bottom border for title bar
    dc.gui.rect(dx, dy + TITLE_H - 1.0, MODAL_W, 1.0, c_border);

    dc.gui.draw_text(
        dc.font,
        "CONFIGURACIÓN",
        [dx + 20.0, dy + 15.0],
        12.0,
        c_text,
    );

    // Botón [×]
    let close_x = dx + MODAL_W - 32.0;
    let close_y = dy + 7.0;
    let over_close = hit(mx, my, close_x, close_y, 28.0, 28.0);
    if over_close {
        dc.gui.rect_r(close_x, close_y, 28.0, 28.0, c_error, 4.0);
    }
    dc.gui.draw_text(dc.font, "×", [close_x + 8.0, close_y + 6.0], 16.0, if over_close { [1.0; 4] } else { c_text_dim });

    // ── Área de contenido ─────────────────────────────────────────────────────
    let content_y = dy + TITLE_H;
    let pad = 24.0;
    let label_x = dx + pad;
    let input_x = dx + MODAL_W * 0.5;
    let input_w = MODAL_W * 0.5 - pad;

    // Ancho
    let w_row_y = content_y + 24.0;
    dc.gui.draw_text(dc.font, "Ancho Canvas (px)", [label_x, w_row_y + 8.0], 11.0, c_text_dim);
    draw_input(dc, input_x, w_row_y + 2.0, input_w, 28.0, &format!("{:.0}", preview_w), &c_text, &c_border, &c_secondary);

    // Alto
    let h_row_y = w_row_y + 44.0;
    dc.gui.draw_text(dc.font, "Alto Canvas (px)", [label_x, h_row_y + 8.0], 11.0, c_text_dim);
    draw_input(dc, input_x, h_row_y + 2.0, input_w, 28.0, &format!("{:.0}", preview_h), &c_text, &c_border, &c_secondary);

    // ── Theme Selector ────────────────────────────────────────────────────────
    let theme_row_y = h_row_y + 60.0;
    dc.gui.draw_text(dc.font, "TEMA VISUAL", [label_x, theme_row_y], 10.0, c_accent);
    
    let theme_names = [("Dark", "dark"), ("Light", "light"), ("Ocean", "ocean")];
    let btn_w = (MODAL_W - pad * 2.0) / 3.0 - 4.0;

    for (i, (label, key)) in theme_names.iter().enumerate() {
        let bx = dx + pad + i as f32 * (btn_w + 6.0);
        let by = theme_row_y + 20.0;
        let is_active = theme_registry.current == *key;
        let over = hit(mx, my, bx, by, btn_w, 32.0);
        
        let mut bg = if is_active { c_accent } else { c_secondary };
        if over && !is_active {
            bg[0] += 0.05; bg[1] += 0.05; bg[2] += 0.05;
        }

        dc.gui.rect_r(bx, by, btn_w, 32.0, bg, 4.0);
        
        let tw = ferrous_gui::GuiBatch::measure_text(dc.font, label, 11.0);
        dc.gui.draw_text(
            dc.font,
            label,
            [bx + (btn_w - tw) * 0.5, by + 10.0],
            11.0,
            if is_active { [1.0; 4] } else { c_text },
        );
    }

    // Footnote
    dc.gui.draw_text(
        dc.font,
        "GUIMaker v0.1.0 · Modular Theming System",
        [dx + pad, dy + MODAL_H - 24.0],
        9.0,
        c_text_dim,
    );
}

fn draw_input(dc: &mut DrawContext<'_, '_>, x: f32, y: f32, w: f32, h: f32, value: &str, c_text: &[f32; 4], c_border: &[f32; 4], c_bg: &[f32; 4]) {
    dc.gui.rect_r(x, y, w, h, *c_bg, 4.0);
    // Border
    dc.gui.rect(x, y, w, 1.0, *c_border);
    dc.gui.rect(x, y + h - 1.0, w, 1.0, *c_border);
    dc.gui.rect(x, y, 1.0, h, *c_border);
    dc.gui.rect(x + w - 1.0, y, 1.0, h, *c_border);

    dc.gui.draw_text(dc.font, value, [x + 8.0, y + 8.0], 11.0, *c_text);
}
