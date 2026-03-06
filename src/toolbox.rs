use ferrous_gui::{InteractiveButton as Button, GuiBatch, TextBatch, GuiQuad, Ui};
use ferrous_assets::Font;

use crate::model::WidgetKind;

/// Un botón de la toolbox asociado a un WidgetKind.
struct ToolEntry {
    kind:   WidgetKind,
    button: Button,
}

/// Panel de herramientas: contiene un botón por cada tipo de widget
/// y los botones de acción global (Generar código, Limpiar, etc.).
pub struct Toolbox {
    tools:             Vec<ToolEntry>,
    pub btn_generate:  Button,
    pub btn_clear:     Button,
    pub btn_close_code: Button,
    panel_x: f32,
    panel_y: f32,
}

const TOOL_W: f32   = 140.0;
const TOOL_H: f32   = 30.0;
const TOOL_GAP: f32 = 6.0;

impl Toolbox {
    pub fn new(panel_x: f32, panel_y: f32) -> Self {
        let kinds = WidgetKind::all();
        let mut tools = Vec::with_capacity(kinds.len());

        for (i, kind) in kinds.into_iter().enumerate() {
            let y = panel_y + 50.0 + i as f32 * (TOOL_H + TOOL_GAP);
            let btn = Button::new(panel_x + 8.0, y, TOOL_W, TOOL_H).with_radius(6.0);
            tools.push(ToolEntry { kind, button: btn });
        }

        let action_y_base = panel_y + 50.0
            + WidgetKind::all().len() as f32 * (TOOL_H + TOOL_GAP)
            + 20.0;

        let btn_generate  = Button::new(panel_x + 8.0, action_y_base,        TOOL_W, 34.0).with_radius(8.0);
        let btn_clear     = Button::new(panel_x + 8.0, action_y_base + 46.0, TOOL_W, 28.0).with_radius(6.0);
        let btn_close_code = Button::new(
            panel_x + 8.0, action_y_base + 86.0, TOOL_W, 28.0
        ).with_radius(6.0);

        Self {
            tools,
            btn_generate,
            btn_clear,
            btn_close_code,
            panel_x,
            panel_y,
        }
    }

    /// Registra todos los botones en el sistema de UI del engine.
    pub fn register(&self, ui: &mut Ui) {
        for t in &self.tools {
            ui.add(t.button.clone());
        }
        ui.add(self.btn_generate.clone());
        ui.add(self.btn_clear.clone());
        ui.add(self.btn_close_code.clone());
    }

    /// Devuelve el WidgetKind del botón pulsado (si alguno lo fue)
    /// y consume el evento.
    pub fn consume_pressed(&mut self) -> Option<WidgetKind> {
        for t in &mut self.tools {
            if t.button.pressed {
                t.button.pressed = false;
                return Some(t.kind.clone());
            }
        }
        None
    }

    /// Dibuja el panel completo de la toolbox.
    pub fn draw(
        &self,
        gui:  &mut GuiBatch,
        text: &mut TextBatch,
        font: Option<&Font>,
    ) {
        let px = self.panel_x;
        let py = self.panel_y;

        // Fondo del panel
        gui.push(GuiQuad {
            pos:   [px - 8.0, py - 8.0],
            size:  [162.0, 700.0],
            color: [0.12, 0.12, 0.15, 0.95],
            radii: [0.0; 4],
            flags: 0,
        });

        if let Some(f) = font {
            text.draw_text(f, "WIDGETS", [px, py], 14.0, [0.9, 0.75, 0.3, 1.0]);
            text.draw_text(f, "(click para añadir)", [px, py + 18.0], 10.0, [0.5, 0.5, 0.5, 1.0]);
        }

        for t in &self.tools {
            // Dibujar el botón
            t.button.draw(gui);

            // Etiqueta dentro del botón
            if let Some(f) = font {
                let color = t.kind.preview_color();
                text.draw_text(
                    f,
                    t.kind.display_name(),
                    [t.button.rect[0] + 10.0, t.button.rect[1] + 8.0],
                    13.0,
                    color,
                );
            }
        }

        // Separador
        let sep_y = self.tools.last().map_or(py + 50.0, |t| t.button.rect[1] + TOOL_H + 10.0);
        gui.push(GuiQuad {
            pos:   [px - 8.0, sep_y],
            size:  [162.0, 2.0],
            color: [0.3, 0.3, 0.35, 1.0],
            radii: [0.0; 4],
            flags: 0,
        });

        // Botones de acción
        self.btn_generate.draw(gui);
        self.btn_clear.draw(gui);
        self.btn_close_code.draw(gui);

        if let Some(f) = font {
            text.draw_text(f, "Generar Codigo",  [px + 8.0, self.btn_generate.rect[1] + 9.0],   13.0, [0.3, 1.0, 0.5, 1.0]);
            text.draw_text(f, "Limpiar Todo",    [px + 8.0, self.btn_clear.rect[1] + 7.0],      12.0, [1.0, 0.5, 0.4, 1.0]);
            text.draw_text(f, "Cerrar Codigo",   [px + 8.0, self.btn_close_code.rect[1] + 7.0], 12.0, [0.7, 0.7, 0.7, 1.0]);
        }
    }
}
