use ferrous_gui::{
    panel::ButtonHandle, Constraint, GuiBatch, Panel, PanelBuilder, SizeExpr, TextBatch, Ui,
};

use crate::model::WidgetKind;

/// Panel de herramientas construido con PanelBuilder.
pub struct Toolbox {
    /// Handles de todos los botones (widgets + acciones).
    buttons: Vec<ButtonHandle>,
    /// Número de tipos de widget.
    tool_count: usize,
    /// Panel original; se mueve a Ui en register() y queda None.
    _panel: Option<Panel>,
}

impl Toolbox {
    pub fn new(panel_x: f32, panel_y: f32) -> Self {
        let kinds = WidgetKind::all();
        let tool_count = kinds.len();

        let mut builder = PanelBuilder::column(panel_x + 8.0, panel_y + 40.0, 140.0)
            .padding(0.0)
            .gap(6.0)
            .item_size(30.0)
            .with_background([0.12, 0.12, 0.15, 0.95])
            // Anclaje reactivo: siempre a 8 px del borde izquierdo, 48 px del borde superior.
            .with_constraint(Constraint::new().x(SizeExpr::px(8.0)).y(SizeExpr::px(48.0)));

        // Un botón por cada tipo de widget con su color de preview como label.
        for kind in &kinds {
            builder = builder.add_button_with_radius(kind.display_name(), 6.0);
        }

        // Acciones globales con colores de label diferentes (se tintarán en draw).
        builder = builder
            .add_button_with_radius("Generar Codigo", 8.0)
            .add_button_with_radius("Limpiar Todo", 6.0)
            .add_button_with_radius("Cerrar Codigo", 6.0);

        // Personalizar colores de los botones de acción tras build.
        let panel = builder.build();

        // Colorear labels de widgets según preview_color.
        for (i, kind) in kinds.iter().enumerate() {
            let color = kind.preview_color();
            panel.buttons[i].borrow_mut().label_color = color;
        }
        // Colores de los botones de acción.
        panel.buttons[tool_count].borrow_mut().label_color = [0.3, 1.0, 0.5, 1.0];
        panel.buttons[tool_count + 1].borrow_mut().label_color = [1.0, 0.5, 0.4, 1.0];
        panel.buttons[tool_count + 2].borrow_mut().label_color = [0.7, 0.7, 0.7, 1.0];

        // Conservar los handles antes de que el panel se mueva a Ui.
        let buttons = panel.buttons.clone();

        // Guardamos el panel para moverlo en register().
        // Lo envolvemos en Option para poder moverlo luego.
        // Usamos una celda de un solo uso.
        Self {
            buttons,
            tool_count,
            _panel: Some(panel),
        }
    }

    /// Registra el panel (y todos sus hijos) en el sistema de UI.
    pub fn register(&mut self, ui: &mut Ui) {
        if let Some(panel) = self._panel.take() {
            ui.add(panel);
        }
    }

    /// Handle del botón "Generar Codigo".
    pub fn btn_generate(&self) -> &ButtonHandle {
        &self.buttons[self.tool_count]
    }

    /// Handle del botón "Limpiar Todo".
    pub fn btn_clear(&self) -> &ButtonHandle {
        &self.buttons[self.tool_count + 1]
    }

    /// Handle del botón "Cerrar Codigo".
    pub fn btn_close_code(&self) -> &ButtonHandle {
        &self.buttons[self.tool_count + 2]
    }

    /// Devuelve el WidgetKind del primer botón de herramienta pulsado
    /// y consume el evento.
    pub fn consume_pressed(&self) -> Option<WidgetKind> {
        let kinds = WidgetKind::all();
        for (i, kind) in kinds.into_iter().enumerate() {
            let mut btn = self.buttons[i].borrow_mut();
            if btn.pressed {
                btn.pressed = false;
                return Some(kind);
            }
        }
        None
    }

    /// Dibuja el fondo + cabecera del panel y los botones con sus labels.
    pub fn draw(&self, gui: &mut GuiBatch, text: &mut TextBatch, font: &ferrous_assets::Font) {
        // Derivamos la posición actual desde el primer botón (ya resuelta por el constraint).
        let first_rect = self.buttons[0].borrow().rect;
        let px = first_rect[0] - 8.0;
        let header_y = first_rect[1] - 40.0; // 40px sobre el primer botón para el header

        // Fondo.
        gui.rect(px - 8.0, 0.0, 162.0, 9999.0, [0.12, 0.12, 0.15, 0.95]);

        text.draw_text(font, "WIDGETS", [px, header_y], 14.0, [0.9, 0.75, 0.3, 1.0]);
        text.draw_text(
            font,
            "(click para añadir)",
            [px, header_y + 18.0],
            10.0,
            [0.5, 0.5, 0.5, 1.0],
        );

        // Botones de widgets con draw_with_text.
        for i in 0..self.tool_count {
            let btn = self.buttons[i].borrow();
            btn.draw_with_text(gui, text, Some(font));
        }

        // Separador.
        let sep_y = if let Some(last) = self.buttons.get(self.tool_count - 1) {
            let r = last.borrow();
            r.rect[1] + r.rect[3] + 8.0
        } else {
            header_y + 50.0
        };
        gui.rect(px - 8.0, sep_y, 162.0, 2.0, [0.3, 0.3, 0.35, 1.0]);

        // Botones de acción.
        for i in self.tool_count..self.buttons.len() {
            let btn = self.buttons[i].borrow();
            btn.draw_with_text(gui, text, Some(font));
        }
    }
}
