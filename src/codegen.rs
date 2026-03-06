use crate::model::{CanvasState, WidgetKind};

/// Genera un archivo `main.rs` completo de FerrousEngine con el esqueleto
/// de la GUI diseñada en el canvas.
pub fn generate(state: &CanvasState) -> String {
    let mut out = String::new();

    // ── Cabecera ─────────────────────────────────────────────────────────────
    out.push_str(&format!(
        "// Código generado por GUIMaker\n\
         // Proyecto: {}\n\
         //\n\
         // Este archivo contiene el esqueleto de tu GUI creada visualmente.\n\
         // Implementa la lógica de negocio en los bloques marcados con TODO.\n\n",
        state.project_name
    ));

    // ── Imports ───────────────────────────────────────────────────────────────
    let has_slider      = state.widgets.iter().any(|w| w.kind == WidgetKind::Slider);
    let has_checkbox    = state.widgets.iter().any(|w| w.kind == WidgetKind::Checkbox);
    let has_text_input  = state.widgets.iter().any(|w| w.kind == WidgetKind::TextInput);
    let has_button      = state.widgets.iter().any(|w| w.kind == WidgetKind::Button);
    let has_progress    = state.widgets.iter().any(|w| w.kind == WidgetKind::ProgressBar);
    let has_dropdown    = state.widgets.iter().any(|w| w.kind == WidgetKind::Dropdown);

    out.push_str("use ferrous_app::{App, AppContext, AppMode, Color, FerrousApp, KeyCode};\n");
    out.push_str("use ferrous_assets::Font;\n");

    // Construir lista de widgets importados de ferrous_gui
    let mut gui_imports: Vec<&str> = vec!["GuiBatch", "TextBatch", "Ui"];
    if has_button     { gui_imports.push("Button"); }
    if has_slider     { gui_imports.push("Slider"); }
    if has_checkbox   { gui_imports.push("Checkbox"); }
    if has_text_input { gui_imports.push("TextInput"); }
    if has_progress   { gui_imports.push("ProgressBar"); }
    if has_dropdown   { gui_imports.push("Dropdown"); }

    gui_imports.sort_unstable();
    gui_imports.dedup();
    out.push_str(&format!(
        "use ferrous_gui::{{{}}};\n\n",
        gui_imports.join(", ")
    ));

    // ── Struct de la aplicación ───────────────────────────────────────────────
    let struct_name = to_pascal_case(&state.project_name);
    out.push_str("// ── Application state ────────────────────────────────────────────────────────\n\n");
    out.push_str(&format!("struct {} {{\n", struct_name));

    for w in &state.widgets {
        let type_name = widget_type_name(&w.kind);
        if let Some(t) = type_name {
            out.push_str(&format!("    {}: {},\n", w.var_name, t));
        }
    }

    // Campos de estado extra para los widgets que guardan valor
    for w in &state.widgets {
        match w.kind {
            WidgetKind::Checkbox => {
                out.push_str(&format!("    {}_checked: bool,\n", w.var_name));
            }
            WidgetKind::TextInput => {
                out.push_str(&format!("    {}_text: String,\n", w.var_name));
            }
            WidgetKind::Dropdown => {
                out.push_str(&format!("    {}_index: usize,\n", w.var_name));
            }
            _ => {}
        }
    }

    out.push_str("}\n\n");

    // ── Default ───────────────────────────────────────────────────────────────
    out.push_str(&format!("impl Default for {} {{\n", struct_name));
    out.push_str("    fn default() -> Self {\n");
    out.push_str("        Self {\n");

    for w in &state.widgets {
        match w.kind {
            WidgetKind::Button => {
                out.push_str(&format!(
                    "            {}: Button::new({:.1}, {:.1}, {:.1}, {:.1}).with_radius({:.1}),\n",
                    w.var_name, w.x, w.y, w.width, w.height, w.radius
                ));
            }
            WidgetKind::Slider => {
                out.push_str(&format!(
                    "            {}: Slider::new({:.1}, {:.1}, {:.1}, {:.1}, {:.2}),\n",
                    w.var_name, w.x, w.y, w.width, w.height, w.value
                ));
            }
            WidgetKind::Checkbox => {
                out.push_str(&format!(
                    "            {}: Checkbox::new({:.1}, {:.1}, {:.1}, {:.1}),\n",
                    w.var_name, w.x, w.y, w.width, w.height
                ));
                out.push_str(&format!(
                    "            {}_checked: false,\n", w.var_name
                ));
            }
            WidgetKind::TextInput => {
                out.push_str(&format!(
                    "            {}: TextInput::new({:.1}, {:.1}, {:.1}, {:.1}),\n",
                    w.var_name, w.x, w.y, w.width, w.height
                ));
                out.push_str(&format!(
                    "            {}_text: String::new(),\n", w.var_name
                ));
            }
            WidgetKind::ProgressBar => {
                out.push_str(&format!(
                    "            {}: ProgressBar::new({:.1}, {:.1}, {:.1}, {:.1}, {:.2}),\n",
                    w.var_name, w.x, w.y, w.width, w.height, w.value
                ));
            }
            WidgetKind::Dropdown => {
                out.push_str(&format!(
                    "            {}: Dropdown::new({:.1}, {:.1}, {:.1}, {:.1}, vec![\"Option 1\", \"Option 2\", \"Option 3\"]),\n",
                    w.var_name, w.x, w.y, w.width, w.height
                ));
                out.push_str(&format!(
                    "            {}_index: 0,\n", w.var_name
                ));
            }
            // Label, Panel, Separator, Image son puramente visuales — sin tipo ferrous_gui.
            _ => {}
        }
    }

    out.push_str("        }\n    }\n}\n\n");

    // ── FerrousApp impl ───────────────────────────────────────────────────────
    out.push_str("// ── FerrousApp implementation ─────────────────────────────────────────────────\n\n");
    out.push_str(&format!("impl FerrousApp for {} {{\n", struct_name));

    // configure_ui
    out.push_str("    fn configure_ui(&mut self, ui: &mut Ui) {\n");
    for w in &state.widgets {
        if widget_type_name(&w.kind).is_some() {
            out.push_str(&format!("        ui.add(self.{}.clone());\n", w.var_name));
        }
    }
    out.push_str("    }\n\n");

    // update
    out.push_str("    fn update(&mut self, ctx: &mut AppContext) {\n");
    out.push_str("        if ctx.input.just_pressed(KeyCode::Escape) {\n");
    out.push_str("            ctx.request_exit();\n");
    out.push_str("        }\n\n");

    for w in &state.widgets {
        match w.kind {
            WidgetKind::Button => {
                out.push_str(&format!(
                    "        if self.{}.pressed {{\n\n            // TODO: lógica al presionar \"{}\"\n            self.{}.pressed = false; // consumir evento\n        }}\n\n",
                    w.var_name, w.label, w.var_name
                ));
            }
            WidgetKind::Checkbox => {
                out.push_str(&format!(
                    "        self.{}_checked = self.{}.checked;\n",
                    w.var_name, w.var_name
                ));
            }
            WidgetKind::TextInput => {
                out.push_str(&format!(
                    "        self.{}_text = self.{}.text.clone();\n",
                    w.var_name, w.var_name
                ));
            }
            WidgetKind::Dropdown => {
                out.push_str(&format!(
                    "        self.{}_index = self.{}.selected_index;\n",
                    w.var_name, w.var_name
                ));
            }
            _ => {}
        }
    }

    out.push_str("\n        // TODO: lógica de actualización del frame\n");
    out.push_str("    }\n\n");

    // draw_ui
    out.push_str("    fn draw_ui(\n");
    out.push_str("        &mut self,\n");
    out.push_str("        gui:  &mut GuiBatch,\n");
    out.push_str("        text: &mut TextBatch,\n");
    out.push_str("        font: Option<&Font>,\n");
    out.push_str("        _ctx: &mut AppContext,\n");
    out.push_str("    ) {\n");

    // Dibujar widgets con tipo ferrous_gui
    for w in &state.widgets {
        if widget_type_name(&w.kind).is_some() {
            out.push_str(&format!("        self.{}.draw(gui);\n", w.var_name));
        }
    }

    // Dibujar Labels como texto
    for w in &state.widgets {
        if w.kind == WidgetKind::Label {
            out.push_str("\n        if let Some(f) = font {\n");
            out.push_str(&format!(
                "            text.push_str(\"{}\", {:.1}, {:.1}, 16.0, [0.9, 0.9, 0.9, 1.0], f);\n",
                w.label, w.x, w.y
            ));
            out.push_str("        }\n");
        }
    }

    out.push_str("    }\n");
    out.push_str("}\n\n");

    // ── main ──────────────────────────────────────────────────────────────────
    out.push_str("// ── Entry point ───────────────────────────────────────────────────────────────\n\n");
    out.push_str("fn main() {\n");
    out.push_str(&format!(
        "    App::new({}::default())\n", struct_name
    ));
    out.push_str(&format!(
        "        .with_title(\"{}\")\n", state.project_name
    ));
    out.push_str("        .with_size(1280, 720)\n");
    out.push_str("        .with_mode(AppMode::Desktop2D)\n");
    out.push_str("        .with_background_color(Color::rgb(0.10, 0.10, 0.12))\n");
    out.push_str("        .with_font(\"assets/fonts/Roboto-Regular.ttf\")\n");
    out.push_str("        .run();\n");
    out.push_str("}\n");

    out
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn widget_type_name(kind: &WidgetKind) -> Option<&'static str> {
    match kind {
        WidgetKind::Button      => Some("Button"),
        WidgetKind::Slider      => Some("Slider"),
        WidgetKind::Checkbox    => Some("Checkbox"),
        WidgetKind::TextInput   => Some("TextInput"),
        WidgetKind::ProgressBar => Some("ProgressBar"),
        WidgetKind::Dropdown    => Some("Dropdown"),
        // Label, Panel, Separator, Image: no tienen tipo ferrous_gui propio.
        _ => None,
    }
}

/// "my_gui_app" → "MyGuiApp"
pub fn to_pascal_case(s: &str) -> String {
    s.split(|c: char| c == '_' || c == '-' || c.is_whitespace())
        .filter(|p| !p.is_empty())
        .map(|p| {
            let mut chars = p.chars();
            match chars.next() {
                None    => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}
