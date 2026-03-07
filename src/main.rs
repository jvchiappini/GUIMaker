mod codegen;
mod model;
mod properties;
mod toolbox;

use ferrous_app::{App, AppContext, AppMode, Color, DrawContext, FerrousApp, KeyCode, MouseButton};
use ferrous_gui::Ui;

use model::CanvasState;
use properties::PropertiesPanel;
use toolbox::Toolbox;

// ── Constantes de layout ──────────────────────────────────────────────────────

/// Ancho del panel izquierdo (toolbox).
const TOOLBOX_W: f32 = 160.0;
/// Ancho del panel derecho (propiedades).
const PROPS_W: f32 = 160.0;
/// Margen superior de ambos paneles.
const PANEL_MARGIN: f32 = 12.0;
/// Ancho de la barra de menú superior.
const MENU_H: f32 = 36.0;
/// Margen lateral de los paneles respecto a los bordes de ventana.
const PANEL_SIDE_MARGIN: f32 = 8.0;
/// Tamaño de la cuadrícula del canvas en píxeles.
const GRID_SIZE: f32 = 20.0;
/// Snap de posición al soltar un widget.
const SNAP: f32 = 4.0;

// ── GUIMakerApp ───────────────────────────────────────────────────────────────

struct GUIMakerApp {
    canvas: CanvasState,
    toolbox: Toolbox,
    properties: PropertiesPanel,
    /// Si hay texto en el portapapeles interno (el código generado visible).
    code_scroll: f32,
}

impl Default for GUIMakerApp {
    fn default() -> Self {
        // Los paneles llevan Constraint declarativo: las coordenadas absolutas
        // que se pasan aquí son sólo el origen inicial; Ui::resolve_constraints
        // las recalcula automáticamente cada frame.
        Self {
            canvas: CanvasState::default(),
            toolbox: Toolbox::new(PANEL_SIDE_MARGIN, MENU_H + PANEL_MARGIN),
            properties: PropertiesPanel::new(PANEL_SIDE_MARGIN, MENU_H + PANEL_MARGIN),
            code_scroll: 0.0,
        }
    }
}

impl GUIMakerApp {
    /// Snap al grid.
    fn snap(v: f32) -> f32 {
        (v / SNAP).round() * SNAP
    }
}

impl FerrousApp for GUIMakerApp {
    // ── Startup ───────────────────────────────────────────────────────────────
    fn configure_ui(&mut self, ui: &mut Ui) {
        self.toolbox.register(ui);
        self.properties.register(ui);
        // Resolver constraints con el tamaño inicial de la ventana.
        ui.resolve_constraints(1280.0, 720.0);
    }

    fn on_resize(&mut self, new_size: (u32, u32), _ctx: &mut AppContext) {
        // El runner de FerrousEngine llama Ui::resolve_constraints automáticamente
        // cada frame (implementado junto al sistema de Constraint en ferrous_gui).
        // No es necesario hacer nada aquí: los paneles con Constraint::pin_right /
        // Constraint::pin_left recalculan su posición en el próximo resolve_constraints.
        let _ = new_size;
    }

    // ── Lógica de frame ───────────────────────────────────────────────────────
    fn update(&mut self, ctx: &mut AppContext) {
        // Salir con Escape
        if ctx.input.just_pressed(KeyCode::Escape) {
            ctx.request_exit();
        }

        // Teclas de acceso rápido
        if ctx.input.just_pressed(KeyCode::Backspace) {
            self.canvas.delete_selected();
        }

        let win_w = ctx.window_size.0;

        // ── Toolbox: añadir widget ────────────────────────────────────────────
        if let Some(kind) = self.toolbox.consume_pressed() {
            // Colocar el nuevo widget en el centro visible del canvas.
            let canvas_w = win_w as f32 - TOOLBOX_W - PROPS_W - 32.0;
            let canvas_cx = canvas_w / 2.0;
            let canvas_cy = 200.0;
            let id = self
                .canvas
                .add_widget(kind, Self::snap(canvas_cx), Self::snap(canvas_cy));
            self.canvas.selected_id = Some(id);
        }

        // ── Generar código ───────────────────────────────────────────────────────
        if self.toolbox.btn_generate().borrow().pressed {
            self.toolbox.btn_generate().borrow_mut().pressed = false;
            self.canvas.generated_code = codegen::generate(&self.canvas);
            self.canvas.show_code_panel = true;
            self.code_scroll = 0.0;
        }

        // ── Limpiar canvas ────────────────────────────────────────────────────
        if self.toolbox.btn_clear().borrow().pressed {
            self.toolbox.btn_clear().borrow_mut().pressed = false;
            self.canvas.widgets.clear();
            self.canvas.selected_id = None;
            self.canvas.generated_code.clear();
            self.canvas.show_code_panel = false;
        }

        // ── Cerrar panel de código ────────────────────────────────────────────────────
        if self.toolbox.btn_close_code().borrow().pressed {
            self.toolbox.btn_close_code().borrow_mut().pressed = false;
            self.canvas.show_code_panel = false;
        }

        // ── Panel de propiedades ──────────────────────────────────────────────
        self.properties.apply(&mut self.canvas);
        if self.properties.delete_pressed() {
            self.canvas.delete_selected();
        }

        // ── Ratón: selección / arrastre en el canvas ──────────────────────────
        let (mx, my) = ctx.input.mouse_pos_f32();
        let (cx, cy) = (mx - (TOOLBOX_W + 16.0), my - (MENU_H + 8.0));

        if ctx.input.button_just_pressed(MouseButton::Left) {
            // ¿Pulsamos sobre un widget?
            if let Some(id) = self.canvas.hit_test(cx, cy) {
                self.canvas.selected_id = Some(id);
                self.canvas.drag_widget_id = Some(id);
                let w = self.canvas.get(id).unwrap();
                self.canvas.drag_offset = (cx - w.x, cy - w.y);
            } else if cx > 0.0 && cy > 0.0 {
                // Click en zona vacía: deseleccionar
                self.canvas.selected_id = None;
            }
        }

        if ctx.input.is_button_down(MouseButton::Left) {
            if let Some(drag_id) = self.canvas.drag_widget_id {
                let offset = self.canvas.drag_offset;
                if let Some(w) = self.canvas.get_mut(drag_id) {
                    w.x = Self::snap(cx - offset.0).max(0.0);
                    w.y = Self::snap(cy - offset.1).max(0.0);
                }
            }
        }

        if ctx.input.button_just_released(MouseButton::Left) {
            self.canvas.drag_widget_id = None;
        }

        // Scroll del panel de código
        let scroll = ctx.input.scroll_delta();
        if self.canvas.show_code_panel {
            self.code_scroll = (self.code_scroll - scroll.1 * 20.0).max(0.0);
        }
    }

    // ── Dibujo ────────────────────────────────────────────────────────────────
    fn draw_ui(&mut self, dc: &mut DrawContext<'_, '_>) {
        let (win_w, win_h) = dc.ctx.window_size;
        let ww = win_w as f32;
        let wh = win_h as f32;

        let canvas_x = TOOLBOX_W + 16.0;
        let canvas_y = MENU_H + 8.0;
        let canvas_w = ww - TOOLBOX_W - PROPS_W - 32.0;
        let canvas_h = wh - MENU_H - 16.0;

        // ── Barra de menú superior ───────────────────────────────────────────────────────
        dc.gui.rect(0.0, 0.0, ww, MENU_H, [0.15, 0.15, 0.18, 1.0]);
        dc.text.draw_text(
            dc.font,
            "GUIMaker",
            [10.0, 10.0],
            16.0,
            [0.95, 0.75, 0.3, 1.0],
        );
        dc.text.draw_text(
            dc.font,
            "FerrousEngine GUI Builder",
            [170.0, 12.0],
            12.0,
            [0.5, 0.5, 0.6, 1.0],
        );
        let info = format!(
            "widgets: {}  |  proyecto: \"{}\"",
            self.canvas.widgets.len(),
            self.canvas.project_name
        );
        dc.text.draw_text(
            dc.font,
            &info,
            [ww - 380.0, 12.0],
            11.0,
            [0.5, 0.6, 0.5, 1.0],
        );

        // ── Área del canvas ───────────────────────────────────────────────────
        dc.gui.rect(
            canvas_x,
            canvas_y,
            canvas_w,
            canvas_h,
            [0.10, 0.10, 0.13, 1.0],
        );

        // Cuadrícula
        self.draw_grid(dc.gui, canvas_x, canvas_y, canvas_w, canvas_h);

        // Widgets del usuario
        for w in &self.canvas.widgets {
            let wx = canvas_x + w.x;
            let wy = canvas_y + w.y;
            let color = w.kind.preview_color();

            // Sombra suave (con radio → GuiQuad directo)
            dc.gui.push(ferrous_gui::GuiQuad {
                pos: [wx + 3.0, wy + 3.0],
                size: [w.width, w.height],
                color: [0.0, 0.0, 0.0, 0.35],
                radii: [w.radius; 4],
                flags: 0,
            });

            // Relleno del widget (con radio → GuiQuad directo)
            dc.gui.push(ferrous_gui::GuiQuad {
                pos: [wx, wy],
                size: [w.width, w.height],
                color,
                radii: [w.radius; 4],
                flags: 0,
            });

            // Contorno de selección
            if self.canvas.selected_id == Some(w.id) {
                let t = 2.0;
                let oc = [1.0, 0.85, 0.2, 1.0];
                dc.gui.rect(wx - t, wy - t, w.width + t * 2.0, t, oc);
                dc.gui.rect(wx - t, wy + w.height, w.width + t * 2.0, t, oc);
                dc.gui.rect(wx - t, wy, t, w.height, oc);
                dc.gui.rect(wx + w.width, wy, t, w.height, oc);
                self.draw_selection_handles(dc.gui, wx, wy, w.width, w.height);
            }

            // Etiqueta dentro del widget
            let font_size = (w.height * 0.45).clamp(10.0, 18.0);
            let tx = wx + 6.0;
            let ty = wy + (w.height - font_size) * 0.5;
            dc.text.draw_text(
                dc.font,
                &w.label,
                [tx, ty],
                font_size,
                [0.05, 0.05, 0.05, 0.9],
            );
        }

        // Tooltip con nombre del widget al pasar el ratón
        let (mx, my) = dc.ctx.input.mouse_pos_f32();
        let cx2 = mx - canvas_x;
        let cy2 = my - canvas_y;
        if let Some(hovered) = self.canvas.hit_test(cx2, cy2) {
            if let Some(w) = self.canvas.get(hovered) {
                let label = format!("{} ({})", w.var_name, w.kind.display_name());
                dc.gui.push(ferrous_gui::GuiQuad {
                    pos: [mx + 12.0, my - 6.0],
                    size: [label.len() as f32 * 7.5 + 8.0, 20.0],
                    color: [0.05, 0.05, 0.08, 0.92],
                    radii: [4.0; 4],
                    flags: 0,
                });
                dc.text.draw_text(
                    dc.font,
                    &label,
                    [mx + 16.0, my - 2.0],
                    11.0,
                    [0.9, 0.9, 0.5, 1.0],
                );
            }
        }

        // ── Toolbox ───────────────────────────────────────────────────────────
        self.toolbox.draw(dc.gui, dc.text, dc.font);

        // ── Propiedades ───────────────────────────────────────────────────────
        self.properties.draw(dc.gui, dc.text, dc.font, &self.canvas);

        // ── Panel de código generado ──────────────────────────────────────────
        if self.canvas.show_code_panel && !self.canvas.generated_code.is_empty() {
            self.draw_code_panel(dc, ww, wh);
        }
    }
}

// ── Helpers de dibujo (no son callbacks de FerrousApp) ────────────────────────

impl GUIMakerApp {
    fn draw_grid(&self, gui: &mut ferrous_gui::GuiBatch, cx: f32, cy: f32, cw: f32, ch: f32) {
        let color = [0.18, 0.18, 0.22, 1.0];
        let mut x = cx;
        while x < cx + cw {
            gui.rect(x, cy, 1.0, ch, color);
            x += GRID_SIZE;
        }
        let mut y = cy;
        while y < cy + ch {
            gui.rect(cx, y, cw, 1.0, color);
            y += GRID_SIZE;
        }
    }

    fn draw_selection_handles(
        &self,
        gui: &mut ferrous_gui::GuiBatch,
        wx: f32,
        wy: f32,
        ww: f32,
        wh: f32,
    ) {
        let hs = 6.0;
        let color = [1.0, 0.85, 0.2, 1.0];
        let corners = [
            (wx - hs * 0.5, wy - hs * 0.5),
            (wx + ww - hs * 0.5, wy - hs * 0.5),
            (wx - hs * 0.5, wy + wh - hs * 0.5),
            (wx + ww - hs * 0.5, wy + wh - hs * 0.5),
        ];
        for (hx, hy) in corners {
            // Handles con radio → GuiQuad
            gui.push(ferrous_gui::GuiQuad {
                pos: [hx, hy],
                size: [hs, hs],
                color,
                radii: [1.0; 4],
                flags: 0,
            });
        }
    }

    fn draw_code_panel(&self, dc: &mut DrawContext<'_, '_>, ww: f32, wh: f32) {
        let pw = (ww * 0.70).min(860.0);
        let ph = wh * 0.75;
        let px = (ww - pw) * 0.5;
        let py = (wh - ph) * 0.5;

        // Fondo semitransparente
        dc.gui.rect(0.0, 0.0, ww, wh, [0.0, 0.0, 0.0, 0.55]);
        dc.gui.push(ferrous_gui::GuiQuad {
            pos: [px, py],
            size: [pw, ph],
            color: [0.08, 0.10, 0.12, 0.98],
            radii: [10.0; 4],
            flags: 0,
        });
        // Borde simulado con 4 rects
        dc.gui
            .rect(px - 2.0, py - 2.0, pw + 4.0, 2.0, [0.3, 0.6, 0.9, 0.8]);
        dc.gui
            .rect(px - 2.0, py + ph, pw + 4.0, 2.0, [0.3, 0.6, 0.9, 0.8]);
        dc.gui.rect(px - 2.0, py, 2.0, ph, [0.3, 0.6, 0.9, 0.8]);
        dc.gui.rect(px + pw, py, 2.0, ph, [0.3, 0.6, 0.9, 0.8]);

        dc.text.draw_text(
            dc.font,
            "Codigo Rust generado",
            [px + 16.0, py + 10.0],
            15.0,
            [0.9, 0.75, 0.3, 1.0],
        );
        dc.text.draw_text(
            dc.font,
            "(copia y pega en src/main.rs de tu proyecto)",
            [px + 16.0, py + 28.0],
            10.0,
            [0.5, 0.5, 0.6, 1.0],
        );

        // Renderizar líneas de código con scroll
        let line_h = 15.0;
        let code_y0 = py + 50.0;
        let max_lines = ((ph - 60.0) / line_h) as usize;
        let lines: Vec<&str> = self.canvas.generated_code.lines().collect();
        let start = (self.code_scroll / line_h) as usize;

        for (i, line) in lines.iter().skip(start).take(max_lines).enumerate() {
            let ly = code_y0 + i as f32 * line_h;
            let color = if line.trim_start().starts_with("//") {
                [0.45, 0.65, 0.45, 1.0]
            } else if line.contains("fn ")
                || line.contains("struct ")
                || line.contains("impl ")
                || line.contains("use ")
            {
                [0.55, 0.78, 0.98, 1.0]
            } else if line.contains("pub ") {
                [0.80, 0.60, 0.98, 1.0]
            } else if line.contains("TODO") {
                [1.0, 0.85, 0.3, 1.0]
            } else {
                [0.85, 0.85, 0.85, 1.0]
            };
            dc.text
                .draw_text(dc.font, line, [px + 16.0, ly], 11.5, color);
        }

        // Indicador de scroll
        if lines.len() > max_lines {
            let scroll_ratio = start as f32 / (lines.len() - max_lines).max(1) as f32;
            let bar_h = ph - 60.0;
            let thumb_h = (max_lines as f32 / lines.len() as f32 * bar_h).max(20.0);
            let thumb_y = py + 50.0 + (bar_h - thumb_h) * scroll_ratio;
            dc.gui.push(ferrous_gui::GuiQuad {
                pos: [px + pw - 10.0, py + 50.0],
                size: [6.0, bar_h],
                color: [0.2, 0.2, 0.25, 1.0],
                radii: [3.0; 4],
                flags: 0,
            });
            dc.gui.push(ferrous_gui::GuiQuad {
                pos: [px + pw - 10.0, thumb_y],
                size: [6.0, thumb_h],
                color: [0.5, 0.6, 0.8, 1.0],
                radii: [3.0; 4],
                flags: 0,
            });
        }

        dc.text.draw_text(
            dc.font,
            "[ scroll para leer  ·  boton Cerrar Codigo o haz click fuera ]",
            [px + 16.0, py + ph - 20.0],
            10.0,
            [0.4, 0.4, 0.5, 1.0],
        );
    }
}

// ── main ──────────────────────────────────────────────────────────────────────

fn main() {
    App::new(GUIMakerApp::default())
        .with_title("GUIMaker — FerrousEngine GUI Builder")
        .with_size(1280, 720)
        .with_mode(AppMode::Desktop2D)
        .with_background_color(Color::rgb(0.08, 0.08, 0.10))
        .with_font("assets/fonts/Roboto-Regular.ttf")
        .with_idle_timeout(Some(0.1)) // ahorra CPU cuando no hay input
        .run();
}
