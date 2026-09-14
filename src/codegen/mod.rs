//! Code generation module for GUIMaker
//!
//! This module provides code generation functionality for converting
//! GUI designs to Rust code.

// Re-export the main codegen functions
pub use self::codegen::*;

// Sub-modules
pub mod modular;
pub mod template_generator;
pub mod plugin_generator;
pub mod event_system;
pub mod scene_builder;

// Main codegen implementation
mod codegen {
    use crate::scene::{PlacedWidget, SceneState, WidgetKind, WidgetProps};

    /// Generates complete Rust code from the scene state.
    pub fn generate(scene: &SceneState, project_name: &str) -> String {
        let mut code = String::new();
        
        // 1. Header with comments
        code.push_str(&format!("// Código generado por GUIMaker\n"));
        code.push_str(&format!("// Proyecto: {}\n", project_name));
        code.push_str(&format!("// Generado: {}\n\n", chrono::Utc::now().to_rfc3339()));
        
        // 2. Imports
        code.push_str(&generate_imports(scene));
        
        // 3. Struct de la aplicación
        code.push_str(&generate_struct(scene));
        
        // 4. impl Default
        code.push_str(&generate_default(scene));
        
        // 5. impl FerrousApp
        code.push_str(&generate_ferrous_engine(scene));
        
        // 6. fn main()
        code.push_str(&generate_main(project_name));
        
        code
    }

    /// Generates the necessary imports based on the widgets present.
    fn generate_imports(scene: &SceneState) -> String {
        let mut imports = String::new();
        
        // Imports de ferrous_engine
        imports.push_str("use ferrous_engine::{App, AppContext, AppMode, Color, FerrousApp, KeyCode};\n");
        
        // Imports de ferrous_assets
        imports.push_str("use ferrous_assets::Font;\n");
        
        // Imports de ferrous_gui - solo los tipos usados
        let mut gui_types = vec!["GuiBatch", "TextBatch", "Ui"];
        
        let has_button = scene.widgets.iter().any(|w| w.kind == WidgetKind::Button);
        let has_slider = scene.widgets.iter().any(|w| w.kind == WidgetKind::Slider);
        let has_checkbox = scene.widgets.iter().any(|w| w.kind == WidgetKind::Checkbox);
        let has_text_input = scene.widgets.iter().any(|w| w.kind == WidgetKind::TextInput);
        let has_progress_bar = scene.widgets.iter().any(|w| w.kind == WidgetKind::ProgressBar);
        let has_dropdown = scene.widgets.iter().any(|w| w.kind == WidgetKind::DropDown);
        
        if has_button { gui_types.push("Button"); }
        if has_slider { gui_types.push("Slider"); }
        if has_checkbox { gui_types.push("Checkbox"); }
        if has_text_input { gui_types.push("TextInput"); }
        if has_progress_bar { gui_types.push("ProgressBar"); }
        if has_dropdown { gui_types.push("Dropdown"); }
        
        imports.push_str(&format!("use ferrous_gui::{{{}}};\n", gui_types.join(", ")));
        imports.push_str("\n");
        
        imports
    }

    /// Generates the struct definition for the application.
    fn generate_struct(scene: &SceneState) -> String {
        let mut struct_def = String::new();
        
        let struct_name = to_pascal_case(&scene.widgets.first().map(|w| w.kind.display_name()).unwrap_or("App"));
        
        struct_def.push_str(&format!("struct {} {{\n", struct_name));
        
        for widget in &scene.widgets {
            let var_name = generate_var_name(widget);
            let type_name = widget_type_name(widget.kind);
            
            // Algunos widgets necesitan campos adicionales
            match widget.kind {
                WidgetKind::Checkbox => {
                    struct_def.push_str(&format!("    {}: {}_widget,\n", var_name, type_name));
                    struct_def.push_str(&format!("    {}_checked: bool,\n", var_name));
                }
                WidgetKind::TextInput => {
                    struct_def.push_str(&format!("    {}: {}_widget,\n", var_name, type_name));
                    struct_def.push_str(&format!("    {}_text: String,\n", var_name));
                }
                WidgetKind::DropDown => {
                    struct_def.push_str(&format!("    {}: {}_widget,\n", var_name, type_name));
                    struct_def.push_str(&format!("    {}_index: usize,\n", var_name));
                }
                _ => {
                    struct_def.push_str(&format!("    {}: {},\n", var_name, type_name));
                }
            }
        }
        
        struct_def.push_str("}\n\n");
        
        struct_def
    }

    /// Generates the Default implementation.
    fn generate_default(scene: &SceneState) -> String {
        let mut default_impl = String::new();
        
        let struct_name = to_pascal_case(&scene.widgets.first().map(|w| w.kind.display_name()).unwrap_or("App"));
        
        default_impl.push_str(&format!("impl Default for {} {{\n", struct_name));
        default_impl.push_str("    fn default() -> Self {\n");
        default_impl.push_str("        Self {\n");
        
        for widget in &scene.widgets {
            let var_name = generate_var_name(widget);
            
            match widget.kind {
                WidgetKind::Button => {
                    default_impl.push_str(&format!(
                        "            {}: Button::new({:.1}, {:.1}, {:.1}, {:.1}).with_radius({:.1}),\n",
                        var_name, widget.x, widget.y, widget.w, widget.h, widget.props.border_radii[0]
                    ));
                }
                WidgetKind::Slider => {
                    default_impl.push_str(&format!(
                        "            {}: Slider::new({:.1}, {:.1}, {:.1}, {:.1}, {:.2}),\n",
                        var_name, widget.x, widget.y, widget.w, widget.h, widget.props.value
                    ));
                }
                WidgetKind::Checkbox => {
                    default_impl.push_str(&format!(
                        "            {}: Checkbox::new({:.1}, {:.1}, {:.1}, {:.1}),\n",
                        var_name, widget.x, widget.y, widget.w, widget.h
                    ));
                    default_impl.push_str(&format!(
                        "            {}_checked: {},\n",
                        var_name, widget.props.checked
                    ));
                }
                WidgetKind::TextInput => {
                    default_impl.push_str(&format!(
                        "            {}: TextInput::new({:.1}, {:.1}, {:.1}, {:.1}),\n",
                        var_name, widget.x, widget.y, widget.w, widget.h
                    ));
                    default_impl.push_str(&format!(
                        "            {}_text: \"{}\".to_string(),\n",
                        var_name, widget.props.label
                    ));
                }
                WidgetKind::ProgressBar => {
                    default_impl.push_str(&format!(
                        "            {}: ProgressBar::new({:.1}, {:.1}, {:.1}, {:.1}, {:.2}),\n",
                        var_name, widget.x, widget.y, widget.w, widget.h, widget.props.value
                    ));
                }
                WidgetKind::DropDown => {
                    default_impl.push_str(&format!(
                        "            {}: Dropdown::new({:.1}, {:.1}, {:.1}, {:.1}),\n",
                        var_name, widget.x, widget.y, widget.w, widget.h
                    ));
                    default_impl.push_str(&format!(
                        "            {}_index: 0,\n",
                        var_name
                    ));
                }
                _ => {
                    // Widgets visuales sin campo en struct
                    continue;
                }
            }
        }
        
        default_impl.push_str("        }\n");
        default_impl.push_str("    }\n");
        default_impl.push_str("}\n\n");
        
        default_impl
    }

    /// Generates the FerrousApp implementation.
    fn generate_ferrous_engine(scene: &SceneState) -> String {
        let mut impl_app = String::new();
        
        let struct_name = to_pascal_case(&scene.widgets.first().map(|w| w.kind.display_name()).unwrap_or("App"));
        
        impl_app.push_str(&format!("impl FerrousApp for {} {{\n", struct_name));
        
        // configure_ui
        impl_app.push_str("    fn configure_ui(&mut self, ui: &mut Ui) {\n");
        for widget in &scene.widgets {
            match widget.kind {
                WidgetKind::Label | WidgetKind::Panel | WidgetKind::Separator | WidgetKind::Image => {
                    // Widgets visuales sin UI
                    continue;
                }
                _ => {
                    let var_name = generate_var_name(widget);
                    impl_app.push_str(&format!("        ui.add(self.{}.clone());\n", var_name));
                }
            }
        }
        impl_app.push_str("    }\n\n");
        
        // update
        impl_app.push_str("    fn update(&mut self, ctx: &mut AppContext) {\n");
        impl_app.push_str("        if ctx.input.just_pressed(KeyCode::Escape) {\n");
        impl_app.push_str("            ctx.request_exit();\n");
        impl_app.push_str("        }\n\n");
        
        for widget in &scene.widgets {
            if widget.kind == WidgetKind::Button {
                let var_name = generate_var_name(widget);
                impl_app.push_str(&format!(
                    "        if self.{}.pressed {{\n",
                    var_name
                ));
                impl_app.push_str(&format!(
                    "            // TODO: lógica al presionar \"{}\"\n",
                    widget.props.label
                ));
                impl_app.push_str(&format!(
                    "            self.{}.pressed = false;\n",
                    var_name
                ));
                impl_app.push_str("        }\n\n");
            }
        }
        
        impl_app.push_str("        // TODO: lógica de actualización del frame\n");
        impl_app.push_str("    }\n\n");
        
        // draw_ui
        impl_app.push_str("    fn draw_ui(\n");
        impl_app.push_str("        &mut self,\n");
        impl_app.push_str("        gui: &mut GuiBatch,\n");
        impl_app.push_str("        text: &mut TextBatch,\n");
        impl_app.push_str("        font: Option<&Font>,\n");
        impl_app.push_str("        _ctx: &mut AppContext,\n");
        impl_app.push_str("    ) {\n");
        
        for widget in &scene.widgets {
            let var_name = generate_var_name(widget);
            
            match widget.kind {
                WidgetKind::Label => {
                    impl_app.push_str(&format!(
                        "        text.add_text(font, \"{}\", {:.1}, {:.1});\n",
                        widget.props.label, widget.x, widget.y
                    ));
                }
                WidgetKind::Button | WidgetKind::Slider | WidgetKind::Checkbox | 
                WidgetKind::TextInput | WidgetKind::ProgressBar | WidgetKind::DropDown => {
                    impl_app.push_str(&format!("        self.{}.draw(gui);\n", var_name));
                }
                _ => {
                    // Widgets visuales sin draw
                    continue;
                }
            }
        }
        
        impl_app.push_str("    }\n");
        impl_app.push_str("}\n\n");
        
        impl_app
    }

    /// Generates the main function.
    fn generate_main(project_name: &str) -> String {
        let struct_name = to_pascal_case(project_name);
        
        let mut main_fn = String::new();
        
        main_fn.push_str("fn main() {\n");
        main_fn.push_str(&format!("    App::new({}::default())\n", struct_name));
        main_fn.push_str(&format!("        .with_title(\"{}\")\n", project_name));
        main_fn.push_str("        .with_size(1280, 720)\n");
        main_fn.push_str("        .with_mode(AppMode::Flat2D)\n");
        main_fn.push_str("        .with_background_color(Color::rgb(0.10, 0.10, 0.12))\n");
        main_fn.push_str("        .with_font(\"assets/fonts/Roboto-Regular.ttf\")\n");
        main_fn.push_str("        .run();\n");
        main_fn.push_str("}\n");
        
        main_fn
    }

    /// Converts a name to PascalCase.
    fn to_pascal_case(text: &str) -> String {
        let mut result = String::new();
        let mut capitalize_next = true;
        
        for c in text.chars() {
            if c.is_alphanumeric() {
                if capitalize_next {
                    result.push(c.to_ascii_uppercase());
                    capitalize_next = false;
                } else {
                    result.push(c);
                }
            } else {
                capitalize_next = true;
            }
        }
        
        if result.is_empty() {
            result = "App".to_string();
        }
        
        result
    }

    /// Generates a unique variable name for a widget.
    fn generate_var_name(widget: &PlacedWidget) -> String {
        format!("{}_{}", widget.kind.display_name().to_lowercase(), widget.id)
    }

    /// Returns the Rust type name for a WidgetKind.
    fn widget_type_name(kind: WidgetKind) -> &'static str {
        match kind {
            WidgetKind::Button => "Button",
            WidgetKind::Slider => "Slider",
            WidgetKind::Checkbox => "Checkbox",
            WidgetKind::TextInput => "TextInput",
            WidgetKind::ProgressBar => "ProgressBar",
            WidgetKind::DropDown => "Dropdown",
            WidgetKind::Label => "Label",
            WidgetKind::Panel => "Panel",
            WidgetKind::Separator => "Separator",
            WidgetKind::Image => "Image",
            _ => "Widget",
        }
    }
}