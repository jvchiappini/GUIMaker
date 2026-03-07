use ferrous_app::{
    App, AppContext, AppMode, Color, CursorIcon, DrawContext, FerrousApp, KeyCode, MouseButton,
    WindowResizeDirection,
};

const TOP_H: f32 = 40.0;
const LEFT_W: f32 = 200.0;
const RIGHT_W: f32 = 220.0;

const C_TOP: [f32; 4] = [0.13, 0.13, 0.16, 1.0];
const C_LEFT: [f32; 4] = [0.11, 0.11, 0.14, 1.0];
const C_RIGHT: [f32; 4] = [0.11, 0.11, 0.14, 1.0];
const C_CANVAS: [f32; 4] = [0.08, 0.08, 0.10, 1.0];
const C_BORDER: [f32; 4] = [0.22, 0.22, 0.27, 1.0];
const C_GRID: [f32; 4] = [0.14, 0.14, 0.18, 1.0];

struct GUIMakerApp {
    // ── canvas ────────────────────────────────────────────────────────────────
    zoom: f32,
    pan_x: f32,
    pan_y: f32,
    last_mx: f32,
    last_my: f32,
    // ── drag de ventana ───────────────────────────────────────────────────────
    /// Offset ratón→esquina de la ventana cuando empezó el drag.
    drag_offset: Option<(i32, i32)>,
    /// Estado maximizado actual (para alternar el icono del botón).
    is_maximized: bool,
}

impl Default for GUIMakerApp {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
            last_mx: 0.0,
            last_my: 0.0,
            drag_offset: None,
            is_maximized: false,
        }
    }
}

impl FerrousApp for GUIMakerApp {
    fn update(&mut self, ctx: &mut AppContext) {
        if ctx.input.just_pressed(KeyCode::Escape) {
            ctx.request_exit();
        }

        let (win_w, win_h) = ctx.window_size;
        let ww = win_w as f32;
        let (mx, my) = ctx.input.mouse_pos_f32();

        // ── Botón cerrar (esquina superior derecha) ───────────────────────────
        // Área: últimos 40×40 px de la barra superior
        let close_x = ww - TOP_H;
        let over_close = mx >= close_x && mx <= ww && my >= 0.0 && my <= TOP_H;
        if over_close && ctx.input.button_just_pressed(MouseButton::Left) {
            ctx.request_exit();
        }

        // ── Botón maximizar (entre minimizar y cerrar) ───────────────────────
        let max_x = ww - TOP_H * 2.0;
        let over_max = mx >= max_x && mx < close_x && my >= 0.0 && my <= TOP_H;
        if over_max && ctx.input.button_just_pressed(MouseButton::Left) {
            self.is_maximized = !self.is_maximized;
            ctx.window.set_maximized(self.is_maximized);
        }

        // ── Botón minimizar (justo a la izquierda del maximizar) ──────────────
        let min_x = ww - TOP_H * 3.0;
        let over_min = mx >= min_x && mx < max_x && my >= 0.0 && my <= TOP_H;
        if over_min && ctx.input.button_just_pressed(MouseButton::Left) {
            ctx.window.set_minimized(true);
        }

        // ── Resize desde bordes (solo si no está maximizada) ─────────────────────────
        if !self.is_maximized {
            let resize_dir = resize_direction(mx, my, win_w, win_h);

            // Cambiar el cursor según la zona de resize
            let icon =
                match resize_dir {
                    Some(WindowResizeDirection::North) | Some(WindowResizeDirection::South) => {
                        CursorIcon::NsResize
                    }
                    Some(WindowResizeDirection::East) | Some(WindowResizeDirection::West) => {
                        CursorIcon::EwResize
                    }
                    Some(WindowResizeDirection::NorthWest)
                    | Some(WindowResizeDirection::SouthEast) => CursorIcon::NwseResize,
                    Some(WindowResizeDirection::NorthEast)
                    | Some(WindowResizeDirection::SouthWest) => CursorIcon::NeswResize,
                    None => CursorIcon::Default,
                };
            ctx.window.set_cursor(icon);

            if ctx.input.button_just_pressed(MouseButton::Left) {
                if let Some(dir) = resize_dir {
                    ctx.start_window_resize(dir);
                }
            }
        } else {
            ctx.window.set_cursor(CursorIcon::Default);
        }

        // ── Drag de la barra superior ─────────────────────────────────────────
        // Zona arrastrable: toda la barra salvo los botones de control
        let drag_area_w = ww - TOP_H * 3.0;
        let over_drag = mx >= 0.0
            && mx < drag_area_w
            && my >= 0.0
            && my <= TOP_H
            && resize_direction(mx, my, win_w, win_h).is_none();

        if ctx.input.button_just_pressed(MouseButton::Left) && over_drag {
            // Guardar offset cursor → posición actual de la ventana
            let win_pos = ctx.window_position().unwrap_or((0, 0));
            let screen_mx = win_pos.0 + mx as i32;
            let screen_my = win_pos.1 + my as i32;
            self.drag_offset = Some((screen_mx - win_pos.0, screen_my - win_pos.1));
        }
        if ctx.input.button_just_released(MouseButton::Left) {
            self.drag_offset = None;
        }
        if ctx.input.is_button_down(MouseButton::Left) {
            if let Some(offset) = self.drag_offset {
                let win_pos = ctx.window_position().unwrap_or((0, 0));
                // Posición global del cursor = posición ventana + cursor local
                let screen_mx = win_pos.0 + mx as i32;
                let screen_my = win_pos.1 + my as i32;
                ctx.set_window_position(screen_mx - offset.0, screen_my - offset.1);
            }
        }

        // ── Zoom y paneo del canvas ───────────────────────────────────────────
        let wh = win_h as f32;
        let canvas_x = LEFT_W;
        let canvas_y = TOP_H;
        let canvas_w = ww - LEFT_W - RIGHT_W;
        let canvas_h = wh - TOP_H;
        let over_canvas = mx >= canvas_x
            && mx <= canvas_x + canvas_w
            && my >= canvas_y
            && my <= canvas_y + canvas_h;
        if over_canvas {
            let scroll = ctx.input.scroll_delta().1;
            if scroll != 0.0 {
                let factor = if scroll > 0.0 { 1.1_f32 } else { 1.0 / 1.1 };
                let cx = mx - canvas_x;
                let cy = my - canvas_y;
                self.pan_x = cx - (cx - self.pan_x) * factor;
                self.pan_y = cy - (cy - self.pan_y) * factor;
                self.zoom = (self.zoom * factor).clamp(0.1, 10.0);
            }
        }
        let panning = ctx.input.is_button_down(MouseButton::Middle)
            || ctx.input.is_button_down(MouseButton::Right);
        if panning && over_canvas {
            self.pan_x += mx - self.last_mx;
            self.pan_y += my - self.last_my;
        }
        self.last_mx = mx;
        self.last_my = my;
    }
    fn draw_ui(&mut self, dc: &mut DrawContext<'_, '_>) {
        let (win_w, win_h) = dc.ctx.window_size;
        let ww = win_w as f32;
        let wh = win_h as f32;
        let canvas_x = LEFT_W;
        let canvas_y = TOP_H;
        let canvas_w = ww - LEFT_W - RIGHT_W;
        let canvas_h = wh - TOP_H;

        // ── 1. Barra superior ─────────────────────────────────────────────────
        dc.gui.rect(0.0, 0.0, ww, TOP_H, C_TOP);
        dc.gui.rect(0.0, TOP_H - 1.0, ww, 1.0, C_BORDER);

        // Logo / título
        dc.text.draw_text(
            dc.font,
            "GUIMaker",
            [12.0, 12.0],
            15.0,
            [0.95, 0.75, 0.3, 1.0],
        );

        // Indicador zoom
        let zoom_label = format!("zoom  {:.0}%", self.zoom * 100.0);
        dc.text.draw_text(
            dc.font,
            &zoom_label,
            [canvas_x + 8.0, 13.0],
            11.0,
            [0.55, 0.65, 0.75, 1.0],
        );

        // ── Botones de control de ventana (esquina derecha) ───────────────────
        let (mx, my) = dc.ctx.input.mouse_pos_f32();

        // Cerrar  [×]
        let close_x = ww - TOP_H;
        let hover_close = mx >= close_x && mx <= ww && my >= 0.0 && my <= TOP_H;
        let c_close = if hover_close {
            [0.85, 0.25, 0.25, 1.0]
        } else {
            [0.25, 0.18, 0.18, 1.0]
        };
        dc.gui.rect_r(close_x, 0.0, TOP_H, TOP_H, c_close, 0.0);
        dc.text.draw_text(
            dc.font,
            "×",
            [close_x + TOP_H * 0.28, 9.0],
            18.0,
            [0.9, 0.9, 0.9, 1.0],
        );

        // Maximizar  [□ / ❐]
        let max_x = ww - TOP_H * 2.0;
        let hover_max = mx >= max_x && mx < close_x && my >= 0.0 && my <= TOP_H;
        let c_max = if hover_max {
            [0.28, 0.28, 0.35, 1.0]
        } else {
            [0.18, 0.18, 0.22, 1.0]
        };
        dc.gui.rect(max_x, 0.0, TOP_H, TOP_H, c_max);
        let max_icon = if self.is_maximized { "❐" } else { "□" };
        dc.text.draw_text(
            dc.font,
            max_icon,
            [max_x + TOP_H * 0.22, 8.0],
            16.0,
            [0.9, 0.9, 0.9, 1.0],
        );

        // Minimizar  [–]
        let min_x = ww - TOP_H * 3.0;
        let hover_min = mx >= min_x && mx < max_x && my >= 0.0 && my <= TOP_H;
        let c_min = if hover_min {
            [0.28, 0.28, 0.35, 1.0]
        } else {
            [0.18, 0.18, 0.22, 1.0]
        };
        dc.gui.rect(min_x, 0.0, TOP_H, TOP_H, c_min);
        dc.text.draw_text(
            dc.font,
            "–",
            [min_x + TOP_H * 0.28, 9.0],
            18.0,
            [0.9, 0.9, 0.9, 1.0],
        );

        // ── 2. Panel izquierdo ────────────────────────────────────────────────
        dc.gui.rect(0.0, TOP_H, LEFT_W, canvas_h, C_LEFT);
        dc.gui.rect(LEFT_W - 1.0, TOP_H, 1.0, canvas_h, C_BORDER);
        dc.text.draw_text(
            dc.font,
            "Herramientas",
            [10.0, canvas_y + 10.0],
            11.0,
            [0.6, 0.6, 0.7, 1.0],
        );

        // ── 3. Panel derecho ──────────────────────────────────────────────────
        let right_x = ww - RIGHT_W;
        dc.gui.rect(right_x, TOP_H, RIGHT_W, canvas_h, C_RIGHT);
        dc.gui.rect(right_x, TOP_H, 1.0, canvas_h, C_BORDER);
        dc.text.draw_text(
            dc.font,
            "Propiedades",
            [right_x + 10.0, canvas_y + 10.0],
            11.0,
            [0.6, 0.6, 0.7, 1.0],
        );

        // ── 4. Previsualizador ────────────────────────────────────────────────
        dc.gui
            .rect(canvas_x, canvas_y, canvas_w, canvas_h, C_CANVAS);

        let grid = 20.0 * self.zoom;
        let offset_x = self.pan_x % grid;
        let offset_y = self.pan_y % grid;
        let mut gx = canvas_x + offset_x;
        while gx < canvas_x + canvas_w {
            dc.gui.rect(gx, canvas_y, 1.0, canvas_h, C_GRID);
            gx += grid;
        }
        let mut gy = canvas_y + offset_y;
        while gy < canvas_y + canvas_h {
            dc.gui.rect(canvas_x, gy, canvas_w, 1.0, C_GRID);
            gy += grid;
        }

        let origin_x = canvas_x + canvas_w * 0.5 + self.pan_x;
        let origin_y = canvas_y + canvas_h * 0.5 + self.pan_y;
        dc.gui.rect(
            origin_x - 0.5,
            canvas_y,
            1.0,
            canvas_h,
            [0.25, 0.35, 0.50, 0.5],
        );
        dc.gui.rect(
            canvas_x,
            origin_y - 0.5,
            canvas_w,
            1.0,
            [0.25, 0.35, 0.50, 0.5],
        );

        dc.text.draw_text(
            dc.font,
            "Rueda: zoom  |  Btn.medio / Btn.derecho: paneo",
            [canvas_x + 8.0, canvas_y + canvas_h - 18.0],
            10.0,
            [0.35, 0.35, 0.45, 1.0],
        );
    }
}

// ── Helper: detectar zona de resize en los bordes de la ventana ──────────────
fn resize_direction(mx: f32, my: f32, win_w: u32, win_h: u32) -> Option<WindowResizeDirection> {
    const E: f32 = 6.0; // grosor del borde de resize en px
    let (w, h) = (win_w as f32, win_h as f32);
    // Ignorar zona de barra superior (la usa el drag/botones)
    if my < TOP_H && mx > E && mx < w - E {
        return None;
    }
    match (mx < E, mx > w - E, my < E, my > h - E) {
        (true, false, true, false) => Some(WindowResizeDirection::NorthWest),
        (false, true, true, false) => Some(WindowResizeDirection::NorthEast),
        (true, false, false, true) => Some(WindowResizeDirection::SouthWest),
        (false, true, false, true) => Some(WindowResizeDirection::SouthEast),
        (true, false, false, false) => Some(WindowResizeDirection::West),
        (false, true, false, false) => Some(WindowResizeDirection::East),
        (false, false, true, false) => Some(WindowResizeDirection::North),
        (false, false, false, true) => Some(WindowResizeDirection::South),
        _ => None,
    }
}

fn main() {
    App::new(GUIMakerApp::default())
        .with_title("GUIMaker")
        .with_size(1280, 720)
        .with_mode(AppMode::Desktop2D)
        .with_decorations(false)
        .with_resizable(true)
        .with_background_color(Color::rgb(0.08, 0.08, 0.10))
        .with_font("assets/fonts/Roboto-Regular.ttf")
        .run();
}
