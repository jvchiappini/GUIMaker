use ferrous_gui::{InteractiveButton as Button, GuiBatch, GuiQuad, TextBatch};
use ferrous_assets::Font;

use crate::model::CanvasState;

/// Controles del panel de propiedades.
pub struct PropertiesPanel {
    // Botones de ajuste de posición/tamaño
    pub btn_x_dec:      Button,
    pub btn_x_inc:      Button,
    pub btn_y_dec:      Button,
    pub btn_y_inc:      Button,
    pub btn_w_dec:      Button,
    pub btn_w_inc:      Button,
    pub btn_h_dec:      Button,
    pub btn_h_inc:      Button,
    pub btn_r_dec:      Button,
    pub btn_r_inc:      Button,
    pub btn_val_dec:    Button,
    pub btn_val_inc:    Button,
    pub btn_delete:     Button,
    pub btn_to_front:   Button,
    pub btn_to_back:    Button,
    /// Área X del panel en pantalla (para posicionar los botones).
    panel_x: f32,
    panel_y: f32,
}

const BTN_W: f32  = 28.0;
const BTN_H: f32  = 24.0;
const STEP: f32   = 4.0;
const STEP_LG: f32 = 20.0;

impl PropertiesPanel {
    pub fn new(panel_x: f32, panel_y: f32) -> Self {
        // Macro local para simplificar la creación de botones pequeños.
        let btn = |lbl: &str, x: f32, y: f32| {
            let _ = lbl; // usado solo para debug
            Button::new(panel_x + x, panel_y + y, BTN_W, BTN_H).with_radius(4.0)
        };

        // Disposición interna: dos columnas (dec | inc) por fila.
        //   X pos:   fila 0
        //   Y pos:   fila 1
        //   Width:   fila 2
        //   Height:  fila 3
        //   Radius:  fila 4
        //   Value:   fila 5
        let row = |r: f32| -> f32 { 80.0 + r * 34.0 };

        Self {
            btn_x_dec:    btn("-", 80.0, row(0.0)),
            btn_x_inc:    btn("+", 112.0, row(0.0)),
            btn_y_dec:    btn("-", 80.0, row(1.0)),
            btn_y_inc:    btn("+", 112.0, row(1.0)),
            btn_w_dec:    btn("-", 80.0, row(2.0)),
            btn_w_inc:    btn("+", 112.0, row(2.0)),
            btn_h_dec:    btn("-", 80.0, row(3.0)),
            btn_h_inc:    btn("+", 112.0, row(3.0)),
            btn_r_dec:    btn("-", 80.0, row(4.0)),
            btn_r_inc:    btn("+", 112.0, row(4.0)),
            btn_val_dec:  btn("-", 80.0, row(5.0)),
            btn_val_inc:  btn("+", 112.0, row(5.0)),
            btn_delete:   Button::new(panel_x + 8.0,  panel_y + row(7.0), 132.0, 28.0).with_radius(6.0),
            btn_to_front: Button::new(panel_x + 8.0,  panel_y + row(8.0), 60.0,  26.0).with_radius(5.0),
            btn_to_back:  Button::new(panel_x + 80.0, panel_y + row(8.0), 60.0,  26.0).with_radius(5.0),
            panel_x,
            panel_y,
        }
    }

    /// Registra los botones en el sistema de UI del engine.
    pub fn register(&self, ui: &mut ferrous_gui::Ui) {
        ui.add(self.btn_x_dec.clone());
        ui.add(self.btn_x_inc.clone());
        ui.add(self.btn_y_dec.clone());
        ui.add(self.btn_y_inc.clone());
        ui.add(self.btn_w_dec.clone());
        ui.add(self.btn_w_inc.clone());
        ui.add(self.btn_h_dec.clone());
        ui.add(self.btn_h_inc.clone());
        ui.add(self.btn_r_dec.clone());
        ui.add(self.btn_r_inc.clone());
        ui.add(self.btn_val_dec.clone());
        ui.add(self.btn_val_inc.clone());
        ui.add(self.btn_delete.clone());
        ui.add(self.btn_to_front.clone());
        ui.add(self.btn_to_back.clone());
    }

    /// Aplica los botones pulsados al estado del canvas.
    /// Devuelve `true` si se realizó algún cambio.
    pub fn apply(&mut self, state: &mut CanvasState) -> bool {
        let Some(sel) = state.selected_id else { return false; };
        let Some(w) = state.get_mut(sel) else { return false; };
        let mut changed = false;

        macro_rules! adj {
            ($btn:expr, $field:expr, $delta:expr, $min:expr) => {
                if $btn.pressed {
                    $btn.pressed = false;
                    $field = ($field + $delta).max($min);
                    changed = true;
                }
            };
        }

        adj!(self.btn_x_dec,   w.x,      -STEP_LG, 0.0);
        adj!(self.btn_x_inc,   w.x,       STEP_LG, 0.0);
        adj!(self.btn_y_dec,   w.y,      -STEP_LG, 0.0);
        adj!(self.btn_y_inc,   w.y,       STEP_LG, 0.0);
        adj!(self.btn_w_dec,   w.width,  -STEP_LG, 20.0);
        adj!(self.btn_w_inc,   w.width,   STEP_LG, 20.0);
        adj!(self.btn_h_dec,   w.height, -STEP_LG, 12.0);
        adj!(self.btn_h_inc,   w.height,  STEP_LG, 12.0);
        adj!(self.btn_r_dec,   w.radius, -STEP,     0.0);
        adj!(self.btn_r_inc,   w.radius,  STEP,     0.0);

        if self.btn_val_dec.pressed {
            self.btn_val_dec.pressed = false;
            w.value = (w.value - 0.05).clamp(0.0, 1.0);
            changed = true;
        }
        if self.btn_val_inc.pressed {
            self.btn_val_inc.pressed = false;
            w.value = (w.value + 0.05).clamp(0.0, 1.0);
            changed = true;
        }

        // Botón delete: se procesa fuera después de soltar el borrow
        if self.btn_to_front.pressed {
            self.btn_to_front.pressed = false;
            if let Some(idx) = state.widgets.iter().position(|w| w.id == sel) {
                let last = state.widgets.len() - 1;
                if idx < last { state.widgets.swap(idx, idx + 1); }
            }
            changed = true;
        }
        if self.btn_to_back.pressed {
            self.btn_to_back.pressed = false;
            if let Some(idx) = state.widgets.iter().position(|w| w.id == sel) {
                if idx > 0 { state.widgets.swap(idx, idx - 1); }
            }
            changed = true;
        }

        changed
    }

    /// Devuelve true si se pulsó el botón de borrar.
    pub fn delete_pressed(&mut self) -> bool {
        if self.btn_delete.pressed {
            self.btn_delete.pressed = false;
            return true;
        }
        false
    }

    /// Dibuja el panel lateral de propiedades.
    pub fn draw(
        &self,
        gui:   &mut GuiBatch,
        text:  &mut TextBatch,
        font:  Option<&Font>,
        state: &CanvasState,
    ) {
        let px = self.panel_x;
        let py = self.panel_y;

        // Fondo del panel
        gui.push(GuiQuad {
            pos:   [px - 8.0, py - 8.0],
            size:  [160.0, 460.0],
            color: [0.13, 0.13, 0.16, 0.95],
            radii: [0.0; 4],
            flags: 0,
        });

        let Some(f) = font else { return; };
        let Some(sel) = state.selected_id else {
            text.draw_text(f, "(ningun widget", [px, py], 13.0, [0.5, 0.5, 0.5, 1.0]);
            text.draw_text(f, "seleccionado)", [px, py + 16.0], 13.0, [0.5, 0.5, 0.5, 1.0]);
            return;
        };
        let Some(w) = state.get(sel) else { return; };

        // Titulo
        text.draw_text(f, "PROPIEDADES", [px, py], 14.0, [0.9, 0.75, 0.3, 1.0]);
        text.draw_text(f, w.kind.display_name(), [px, py + 20.0], 12.0, [0.7, 0.7, 0.9, 1.0]);
        text.draw_text(f, &format!("var: {}", w.var_name), [px, py + 36.0], 11.0, [0.5, 0.8, 0.5, 1.0]);

        let row = |r: f32| -> f32 { py + 80.0 + r * 34.0 };

        // Etiquetas + valores
        let lbl_color = [0.75, 0.75, 0.75, 1.0];
        let val_color = [1.0, 1.0, 1.0, 1.0];

        let rows: &[(&str, String)] = &[
            ("X", format!("{:.0}", w.x)),
            ("Y", format!("{:.0}", w.y)),
            ("W", format!("{:.0}", w.width)),
            ("H", format!("{:.0}", w.height)),
            ("Radius", format!("{:.0}", w.radius)),
            ("Value", format!("{:.2}", w.value)),
        ];

        for (i, (label, value)) in rows.iter().enumerate() {
            let y = row(i as f32);
            text.draw_text(f, label, [px,        y + 4.0], 12.0, lbl_color);
            text.draw_text(f, value, [px + 44.0, y + 4.0], 12.0, val_color);
        }

        // Botones extra
        self.btn_delete.draw(gui);
        text.draw_text(f, "Eliminar", [px + 12.0, row(7.0) + 5.0], 12.0, [1.0, 0.4, 0.4, 1.0]);

        self.btn_to_front.draw(gui);
        text.draw_text(f, "Frente", [px + 14.0, row(8.0) + 4.0], 11.0, lbl_color);

        self.btn_to_back.draw(gui);
        text.draw_text(f, "Atras", [px + 86.0, row(8.0) + 4.0], 11.0, lbl_color);

        // Dibujar todos los botones de ajuste
        let btns: [&Button; 12] = [
            &self.btn_x_dec, &self.btn_x_inc,
            &self.btn_y_dec, &self.btn_y_inc,
            &self.btn_w_dec, &self.btn_w_inc,
            &self.btn_h_dec, &self.btn_h_inc,
            &self.btn_r_dec, &self.btn_r_inc,
            &self.btn_val_dec, &self.btn_val_inc,
        ];

        for pair in btns.chunks(2) {
            pair[0].draw(gui);
            pair[1].draw(gui);
        }

        // Línea de separación
        gui.push(GuiQuad {
            pos:   [px - 8.0, row(6.5) - 4.0],
            size:  [160.0, 2.0],
            color: [0.3, 0.3, 0.35, 1.0],
            radii: [0.0; 4],
            flags: 0,
        });
    }
}
