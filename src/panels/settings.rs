use ferrous_app::{AppContext, DrawContext, MouseButton};

// ── Dimensiones del modal ────────────────────────────────────────────────────
const MODAL_W: f32 = 360.0;
const MODAL_H: f32 = 290.0;
const TITLE_H: f32 = 42.0;
const RADIUS: f32 = 6.0;

// ── Colores (VS Code Dark+ palette) ─────────────────────────────────────────
const C_BACKDROP:     [f32; 4] = [0.0,  0.0,  0.0,  0.60];
const C_SHADOW:       [f32; 4] = [0.0,  0.0,  0.0,  0.40];
const C_SURFACE:      [f32; 4] = [0.145, 0.145, 0.149, 1.0]; // #252527
const C_TITLE_BAR:    [f32; 4] = [0.18,  0.18,  0.184, 1.0]; // #2E2E2F
const C_ACCENT:       [f32; 4] = [0.0,   0.478, 0.800, 1.0]; // #007ACC
const C_TEXT:         [f32; 4] = [0.867, 0.867, 0.867, 1.0]; // #DDDDDD
const C_TEXT_MUTED:   [f32; 4] = [0.60,  0.60,  0.60,  1.0]; // #999999
const C_INPUT_BG:     [f32; 4] = [0.098, 0.098, 0.098, 1.0]; // #191919
const C_INPUT_BORDER: [f32; 4] = [0.267, 0.267, 0.267, 1.0]; // #444444
const C_HOVER_CLOSE:  [f32; 4] = [0.80,  0.22,  0.22,  1.0]; // red hover
const C_CHECK_ON:     [f32; 4] = [0.0,   0.478, 0.800, 1.0]; // accent
const C_CHECK_OFF:    [f32; 4] = [0.14,  0.14,  0.14,  1.0];
const C_DIVIDER:      [f32; 4] = [0.22,  0.22,  0.22,  1.0];

fn hit(mx: f32, my: f32, x: f32, y: f32, w: f32, h: f32) -> bool {
    mx >= x && mx <= x + w && my >= y && my <= y + h
}

pub fn update(
    ctx: &mut AppContext,
    preview_w: &mut f32,
    preview_h: &mut f32,
    responsive: &mut bool,
    show_modal: &mut bool,
) {
    let (win_w, win_h) = ctx.window_size;
    let ww = win_w as f32;
    let wh = win_h as f32;
    let (mx, my) = ctx.input.mouse_pos_f32();

    let dx = (ww - MODAL_W) * 0.5;
    let dy = (wh - MODAL_H) * 0.5;

    // Cerrar con ESC
    if ctx.input.just_pressed(ferrous_app::KeyCode::Escape) {
        *show_modal = false;
        return;
    }

    let lmb = ctx.input.button_just_pressed(MouseButton::Left);
    if !lmb { return; }

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
    let pad = 20.0;

    // Fila Ancho — click +/- en la caja de valor
    let w_row_y = content_y + 16.0;
    let w_box = (dx + MODAL_W * 0.5, w_row_y + 2.0, 90.0, 28.0);
    if hit(mx, my, w_box.0, w_box.1, w_box.2, w_box.3) {
        *preview_w += 10.0;
    }

    // Fila Alto
    let h_row_y = w_row_y + 52.0;
    let h_box = (dx + MODAL_W * 0.5, h_row_y + 2.0, 90.0, 28.0);
    if hit(mx, my, h_box.0, h_box.1, h_box.2, h_box.3) {
        *preview_h += 10.0;
    }

    // Checkbox Responsive
    let resp_y = h_row_y + 52.0 + 10.0;
    let cb = (dx + pad, resp_y, 18.0, 18.0);
    if hit(mx, my, cb.0, cb.1, cb.2, cb.3) {
        *responsive = !*responsive;
    }
}

pub fn draw(
    dc: &mut DrawContext<'_, '_>,
    preview_w: f32,
    preview_h: f32,
    responsive: bool,
) {
    let (win_w, win_h) = dc.ctx.window_size;
    let ww = win_w as f32;
    let wh = win_h as f32;
    let (mx, my) = dc.ctx.input.mouse_pos_f32();

    let dx = (ww - MODAL_W) * 0.5;
    let dy = (wh - MODAL_H) * 0.5;

    // ── Backdrop ──────────────────────────────────────────────────────────────
    dc.gui.rect(0.0, 0.0, ww, wh, C_BACKDROP);

    // ── Sombra del panel ──────────────────────────────────────────────────────
    dc.gui.rect_r(dx + 6.0, dy + 10.0, MODAL_W, MODAL_H, C_SHADOW, RADIUS + 2.0);

    // ── Panel principal ───────────────────────────────────────────────────────
    dc.gui.rect_r(dx, dy, MODAL_W, MODAL_H, C_SURFACE, RADIUS);

    // ── Línea de acento superior (azul) ───────────────────────────────────────
    dc.gui.rect_r(dx, dy, MODAL_W, 2.0, C_ACCENT, RADIUS);

    // ── Barra de título ───────────────────────────────────────────────────────
    dc.gui.rect_r(dx, dy, MODAL_W, TITLE_H, C_TITLE_BAR, RADIUS);
    // Corregir esquinas inferiores de la barra (rect sólido sobre el redondeado)
    dc.gui.rect(dx, dy + RADIUS, MODAL_W, TITLE_H - RADIUS, C_TITLE_BAR);

    dc.gui.draw_text(
        dc.font,
        "Configuración de preview",
        [dx + 16.0, dy + 13.0],
        13.0,
        C_TEXT,
    );

    // Botón [×]
    let close_x = dx + MODAL_W - 32.0;
    let close_y = dy + 7.0;
    let over_close = hit(mx, my, close_x, close_y, 28.0, 28.0);
    dc.gui.rect_r(
        close_x, close_y, 28.0, 28.0,
        if over_close { C_HOVER_CLOSE } else { [0.0, 0.0, 0.0, 0.0] },
        5.0,
    );
    dc.gui.draw_text(dc.font, "×", [close_x + 7.0, close_y + 6.0], 16.0, C_TEXT_MUTED);

    // ── Divisor bajo la barra ─────────────────────────────────────────────────
    dc.gui.rect(dx, dy + TITLE_H, MODAL_W, 1.0, C_DIVIDER);

    // ── Área de contenido ─────────────────────────────────────────────────────
    let content_y = dy + TITLE_H;
    let pad = 20.0;
    let label_x = dx + pad;
    let input_x = dx + MODAL_W * 0.5;
    let input_w = MODAL_W * 0.5 - pad;

    // ── Fila: Ancho ───────────────────────────────────────────────────────────
    let w_row_y = content_y + 16.0;
    dc.gui.draw_text(dc.font, "Ancho (px)", [label_x, w_row_y + 8.0], 12.0, C_TEXT);
    draw_input(dc, input_x, w_row_y + 2.0, input_w, 28.0, &format!("{:.0}", preview_w));

    // ── Fila: Alto ────────────────────────────────────────────────────────────
    let h_row_y = w_row_y + 52.0;
    dc.gui.draw_text(dc.font, "Alto (px)", [label_x, h_row_y + 8.0], 12.0, C_TEXT);
    draw_input(dc, input_x, h_row_y + 2.0, input_w, 28.0, &format!("{:.0}", preview_h));

    // Separador sutil entre inputs y checkbox
    dc.gui.rect(dx + pad, h_row_y + 46.0, MODAL_W - pad * 2.0, 1.0, C_DIVIDER);

    // ── Fila: Responsive (checkbox) ───────────────────────────────────────────
    let resp_y = h_row_y + 52.0 + 10.0;
    draw_checkbox(dc, dx + pad, resp_y, responsive);
    dc.gui.draw_text(dc.font, "Diseño responsive", [dx + pad + 26.0, resp_y + 3.0], 12.0, C_TEXT);

    // Nota de ayuda
    let note_y = resp_y + 26.0;
    dc.gui.draw_text(
        dc.font,
        "Clic en valor para +10  ·  Clic derecho para −10",
        [dx + pad, note_y],
        10.0,
        C_TEXT_MUTED,
    );
}

fn draw_input(dc: &mut DrawContext<'_, '_>, x: f32, y: f32, w: f32, h: f32, value: &str) {
    // Borde
    dc.gui.rect_r(x - 1.0, y - 1.0, w + 2.0, h + 2.0, C_INPUT_BORDER, 4.0);
    // Fondo
    dc.gui.rect_r(x, y, w, h, C_INPUT_BG, 4.0);
    // Valor
    dc.gui.draw_text(dc.font, value, [x + 8.0, y + 7.0], 12.0, C_TEXT);
}

fn draw_checkbox(dc: &mut DrawContext<'_, '_>, x: f32, y: f32, checked: bool) {
    let color = if checked { C_CHECK_ON } else { C_CHECK_OFF };
    // Borde
    dc.gui.rect_r(x - 1.0, y - 1.0, 20.0, 20.0, C_INPUT_BORDER, 4.0);
    // Fondo
    dc.gui.rect_r(x, y, 18.0, 18.0, color, 4.0);
    if checked {
        // Checkmark "✓"
        dc.gui.draw_text(dc.font, "✓", [x + 2.0, y + 1.0], 13.0, [1.0, 1.0, 1.0, 1.0]);
    }
}
