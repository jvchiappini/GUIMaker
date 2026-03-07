use ferrous_gui::{
    panel::ButtonHandle, Constraint, GuiBatch, Panel, PanelBuilder, RowItem, SizeExpr, TextBatch,
    Ui,
};

use crate::model::CanvasState;

// Índices en panel.buttons  [dec, inc]  para cada propiedad.
// Con add_row los pares ±  se insertan en orden: dec primero, inc segundo.
const IDX_X_DEC: usize = 0;
const IDX_X_INC: usize = 1;
const IDX_Y_DEC: usize = 2;
const IDX_Y_INC: usize = 3;
const IDX_W_DEC: usize = 4;
const IDX_W_INC: usize = 5;
const IDX_H_DEC: usize = 6;
const IDX_H_INC: usize = 7;
const IDX_R_DEC: usize = 8;
const IDX_R_INC: usize = 9;
const IDX_VAL_DEC: usize = 10;
const IDX_VAL_INC: usize = 11;
const IDX_DELETE: usize = 12;
const IDX_FRONT: usize = 13;
const IDX_BACK: usize = 14;

const STEP: f32 = 4.0;
const STEP_LG: f32 = 20.0;

/// Ancho del panel de propiedades.
const PANEL_W: f32 = 148.0;

/// Panel de propiedades construido con PanelBuilder + Constraint reactivo.
pub struct PropertiesPanel {
    /// Handles de todos los botones.
    buttons: Vec<ButtonHandle>,
    /// Panel original; se mueve a Ui en register() y queda None.
    _panel: Option<Panel>,
}

impl PropertiesPanel {
    /// `_panel_x` y `_panel_y` se ignoran: el Constraint se encarga de
    /// posicionar el panel en el borde derecho de la ventana automáticamente.
    pub fn new(_panel_x: f32, _panel_y: f32) -> Self {
        // Layout declarativo:
        //   • add_row([-,+]) genera un par de botones side-by-side sin aritmética manual.
        //   • Constraint::pin_right ancla el panel al borde derecho para cualquier
        //     tamaño de ventana; Ui::resolve_constraints lo recalcula cada frame.
        //
        // Orden de panel.buttons (igual que antes para no romper los índices):
        //   0  X−   1  X+
        //   2  Y−   3  Y+
        //   4  W−   5  W+
        //   6  H−   7  H+
        //   8  R−   9  R+
        //  10  V−  11  V+
        //  12  Eliminar
        //  13  Frente   14  Atrás
        let panel = PanelBuilder::column(0.0, 0.0, PANEL_W)
            .padding(8.0)
            .gap(6.0)
            .item_size(24.0)
            // Las 6 filas de ajuste: cada add_row produce 2 botones (dec, inc).
            .add_row(vec![
                RowItem::Button {
                    label: "-",
                    radius: 4.0,
                },
                RowItem::Button {
                    label: "+",
                    radius: 4.0,
                },
            ]) // X
            .add_row(vec![
                RowItem::Button {
                    label: "-",
                    radius: 4.0,
                },
                RowItem::Button {
                    label: "+",
                    radius: 4.0,
                },
            ]) // Y
            .add_row(vec![
                RowItem::Button {
                    label: "-",
                    radius: 4.0,
                },
                RowItem::Button {
                    label: "+",
                    radius: 4.0,
                },
            ]) // W
            .add_row(vec![
                RowItem::Button {
                    label: "-",
                    radius: 4.0,
                },
                RowItem::Button {
                    label: "+",
                    radius: 4.0,
                },
            ]) // H
            .add_row(vec![
                RowItem::Button {
                    label: "-",
                    radius: 4.0,
                },
                RowItem::Button {
                    label: "+",
                    radius: 4.0,
                },
            ]) // Radius
            .add_row(vec![
                RowItem::Button {
                    label: "-",
                    radius: 4.0,
                },
                RowItem::Button {
                    label: "+",
                    radius: 4.0,
                },
            ]) // Value
            // Botones de acción en filas individuales.
            .add_button_with_radius("Eliminar", 6.0)
            .add_row(vec![
                RowItem::Button {
                    label: "Frente",
                    radius: 5.0,
                },
                RowItem::Button {
                    label: "Atras",
                    radius: 5.0,
                },
            ])
            // Anclaje reactivo: siempre a 8 px del borde derecho, 48 px del borde superior.
            .with_constraint(Constraint::pin_right(8.0, SizeExpr::px(48.0), PANEL_W, 0.0))
            .build();

        // Personalizar colores de los botones de acción.
        panel.buttons[IDX_DELETE].borrow_mut().label_color = [1.0, 0.4, 0.4, 1.0];
        panel.buttons[IDX_FRONT].borrow_mut().label_color = [0.75, 0.75, 0.75, 1.0];
        panel.buttons[IDX_BACK].borrow_mut().label_color = [0.75, 0.75, 0.75, 1.0];

        let buttons = panel.buttons.clone();
        Self {
            buttons,
            _panel: Some(panel),
        }
    }

    /// Registra el panel (y todos sus hijos) en el sistema de UI.
    pub fn register(&mut self, ui: &mut Ui) {
        if let Some(panel) = self._panel.take() {
            ui.add(panel);
        }
    }

    // ── Acceso a handles individuales ──────────────────────────────────────
    fn btn(&self, idx: usize) -> &ButtonHandle {
        &self.buttons[idx]
    }
    fn pressed_consume(&self, idx: usize) -> bool {
        let mut b = self.btn(idx).borrow_mut();
        if b.pressed {
            b.pressed = false;
            true
        } else {
            false
        }
    }

    /// Aplica los botones pulsados al estado del canvas.
    /// Devuelve `true` si se realizó algún cambio.
    pub fn apply(&mut self, state: &mut CanvasState) -> bool {
        let Some(sel) = state.selected_id else {
            return false;
        };
        let Some(w) = state.get_mut(sel) else {
            return false;
        };
        let mut changed = false;

        macro_rules! adj {
            ($idx:expr, $field:expr, $delta:expr, $min:expr) => {
                if self.pressed_consume($idx) {
                    $field = ($field + $delta).max($min);
                    changed = true;
                }
            };
        }

        adj!(IDX_X_DEC, w.x, -STEP_LG, 0.0);
        adj!(IDX_X_INC, w.x, STEP_LG, 0.0);
        adj!(IDX_Y_DEC, w.y, -STEP_LG, 0.0);
        adj!(IDX_Y_INC, w.y, STEP_LG, 0.0);
        adj!(IDX_W_DEC, w.width, -STEP_LG, 20.0);
        adj!(IDX_W_INC, w.width, STEP_LG, 20.0);
        adj!(IDX_H_DEC, w.height, -STEP_LG, 12.0);
        adj!(IDX_H_INC, w.height, STEP_LG, 12.0);
        adj!(IDX_R_DEC, w.radius, -STEP, 0.0);
        adj!(IDX_R_INC, w.radius, STEP, 0.0);

        if self.pressed_consume(IDX_VAL_DEC) {
            w.value = (w.value - 0.05).clamp(0.0, 1.0);
            changed = true;
        }
        if self.pressed_consume(IDX_VAL_INC) {
            w.value = (w.value + 0.05).clamp(0.0, 1.0);
            changed = true;
        }

        if self.pressed_consume(IDX_FRONT) {
            if let Some(idx) = state.widgets.iter().position(|w| w.id == sel) {
                let last = state.widgets.len() - 1;
                if idx < last {
                    state.widgets.swap(idx, idx + 1);
                }
            }
            changed = true;
        }
        if self.pressed_consume(IDX_BACK) {
            if let Some(idx) = state.widgets.iter().position(|w| w.id == sel) {
                if idx > 0 {
                    state.widgets.swap(idx, idx - 1);
                }
            }
            changed = true;
        }

        changed
    }

    /// Devuelve true si se pulsó el botón de borrar.
    pub fn delete_pressed(&mut self) -> bool {
        self.pressed_consume(IDX_DELETE)
    }

    /// Dibuja el panel lateral de propiedades.
    /// Obtiene la posición actual del primer botón para derivar el origen del panel,
    /// ya que el Constraint puede haberlo reposicionado respecto al frame anterior.
    pub fn draw(
        &self,
        gui: &mut GuiBatch,
        text: &mut TextBatch,
        font: &ferrous_assets::Font,
        state: &CanvasState,
    ) {
        // La posición del panel se deriva del primer botón (siempre existe).
        // Constraint::pin_right reposicionó sus rects, así que los leemos aquí.
        let first_rect = self.buttons[0].borrow().rect;
        // El origen del fondo es el borde izquierdo del panel (padding = 8).
        let px = first_rect[0] - 8.0;
        let py = 0.0; // el fondo cubre toda la altura de la ventana

        // Fondo del panel.
        gui.rect(px, py, PANEL_W + 16.0, 9999.0, [0.13, 0.13, 0.16, 0.95]);

        // El header se posiciona justo encima del primer botón.
        // first_rect[1] es la Y resuelta del primer botón por el constraint.
        let header_y = first_rect[1] - 80.0; // 80px de espacio para título/var
        let txt_x = px + 8.0;

        let Some(sel) = state.selected_id else {
            text.draw_text(
                font,
                "(ningun widget",
                [txt_x, header_y + 20.0],
                13.0,
                [0.5, 0.5, 0.5, 1.0],
            );
            text.draw_text(
                font,
                "seleccionado)",
                [txt_x, header_y + 36.0],
                13.0,
                [0.5, 0.5, 0.5, 1.0],
            );
            return;
        };
        let Some(w) = state.get(sel) else {
            return;
        };

        // Título.
        text.draw_text(
            font,
            "PROPIEDADES",
            [txt_x, header_y],
            14.0,
            [0.9, 0.75, 0.3, 1.0],
        );
        text.draw_text(
            font,
            w.kind.display_name(),
            [txt_x, header_y + 20.0],
            12.0,
            [0.7, 0.7, 0.9, 1.0],
        );
        text.draw_text(
            font,
            &format!("var: {}", w.var_name),
            [txt_x, header_y + 36.0],
            11.0,
            [0.5, 0.8, 0.5, 1.0],
        );

        // Etiquetas de propiedad alineadas con cada fila de botones ±.
        // Cada fila add_row ocupa item_size(24) + gap(6) = 30px.
        const ROW_STEP: f32 = 30.0;
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
            // first_rect[1] es la Y del primer par (X±)
            let row_y = first_rect[1] + i as f32 * ROW_STEP + 4.0;
            text.draw_text(font, label, [txt_x, row_y], 12.0, lbl_color);
            text.draw_text(font, value, [txt_x + 36.0, row_y], 12.0, val_color);
        }

        // Separador entre ajustes y acciones.
        let sep_y = first_rect[1] + 6.0 * ROW_STEP + 4.0;
        gui.rect(px, sep_y, PANEL_W + 16.0, 2.0, [0.3, 0.3, 0.35, 1.0]);

        // Todos los botones con sus labels centrados.
        for btn_handle in &self.buttons {
            let btn = btn_handle.borrow();
            btn.draw_with_text(gui, text, Some(font));
        }
    }
}
