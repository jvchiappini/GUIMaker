pub mod panels;
pub mod scene;
pub mod project;
pub mod file_dialog;
pub mod codegen;
pub mod plugin;
pub mod template;
pub mod theme;
pub mod color_picker;
pub mod widget_categories;

use std::sync::Arc;
use ferrous_engine::Color as AppColor;
use ferrous_engine::{
    AppContext, CursorIcon, DrawContext, FerrousApp, KeyCode, WindowResizeDirection,
};
use ferrous_assets::Texture2d;

use crate::panels::left_panel::PaletteState;
use crate::scene::SceneState;

// ── Layout constants ──────────────────────────────────────────────────────────
pub const TOP_H: f32 = 40.0;
pub const LEFT_W: f32 = 200.0;
pub const RIGHT_W: f32 = 240.0;

pub const LEFT_W_MIN: f32 = 150.0;
pub const LEFT_W_MAX: f32 = 400.0;
pub const RIGHT_W_MIN: f32 = 180.0;
pub const RIGHT_W_MAX: f32 = 500.0;
pub const PANEL_EDGE_HIT: f32 = 6.0;

// ── Color palette ─────────────────────────────────────────────────────────────
pub fn c_top(theme: &theme::Theme) -> [f32; 4] {
    theme::ThemeRegistry::hex_to_f32(&theme.colors.top_bar)
}
pub fn c_canvas(theme: &theme::Theme) -> [f32; 4] {
    theme::ThemeRegistry::hex_to_f32(&theme.colors.background)
}
pub fn c_border(theme: &theme::Theme) -> [f32; 4] {
    theme::ThemeRegistry::hex_to_f32(&theme.colors.border)
}
pub fn c_grid(theme: &theme::Theme) -> [f32; 4] {
    theme::ThemeRegistry::hex_to_f32(&theme.colors.grid)
}

// ── Preview resize handle ─────────────────────────────────────────────────────
#[derive(Clone, Copy, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub enum PreviewDrag {
    Left,
    Right,
    Top,
    Bottom,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

// ── Application state ─────────────────────────────────────────────────────────
pub struct GUIMakerApp {
    // Canvas navigation
    pub zoom: f32,
    pub pan_x: f32,
    pub pan_y: f32,
    pub last_mx: f32,
    pub last_my: f32,
    // Window control
    pub drag_offset: Option<(i32, i32)>,
    pub is_maximized: bool,
    // Preview canvas
    pub show_settings_modal: bool,
    pub preview_width: f32,
    pub preview_height: f32,
    pub preview_drag: Option<PreviewDrag>,
    // Panel resize
    pub left_w: f32,
    pub right_w: f32,
    pub resizing_left: bool,
    pub resizing_right: bool,
    // Title-bar icons
    pub icon_close: Option<Arc<Texture2d>>,
    pub icon_minimize: Option<Arc<Texture2d>>,
    pub icon_restore: Option<Arc<Texture2d>>,
    // Scene builder
    pub scene: SceneState,
    pub palette_state: PaletteState,
    // Project state
    pub current_project_path: Option<std::path::PathBuf>,
    pub project: project::Project,
    pub project_buttons: panels::top_bar::ProjectButtons,
    
    // Plugin system
    pub plugin_manager: plugin::PluginManager,
    pub template_registry: template::TemplateRegistryRef,
    
    // Code generation
    pub codegen: codegen::modular::ModularCodegen,
    pub codegen_config: plugin::CodegenConfig,
    
    // Theming system
    pub theme_registry: theme::ThemeRegistry,

    pub window_close_requested: bool,
    pub window_maximize_requested: bool,
    pub window_minimize_requested: bool,
    pub settings_requested: bool,

    // Color picker
    pub color_picker: color_picker::ColorPickerState,

    // Scroll for right panel
    pub right_scroll_y: f32,
    pub window_width: f32,
}

impl Default for GUIMakerApp {
    fn default() -> Self {
        // Initialize plugin manager and template registry
        let mut plugin_manager = plugin::PluginManager::new();
        let template_registry = Arc::new(std::sync::RwLock::new(template::TemplateRegistry::new()));

        // Register built-in plugins
        let built_in_widgets = Arc::new(plugin::BuiltInWidgetsPlugin);
        if let Err(e) = plugin_manager.register_plugin(built_in_widgets) {
            eprintln!("Failed to register built-in widgets plugin: {}", e);
        }
        
        // Register built-in templates
        {
            let mut registry = template_registry.write().unwrap();
            template::BuiltInTemplates::register_all(&mut registry);
        }
        
        // Initialize modular codegen with built-in generators
        let mut codegen = codegen::modular::ModularCodegen::new(template_registry.clone());
        
        // Register built-in code generators
        let ferrous_engine_gen = Box::new(codegen::plugin_generator::built_in::FerrousAppGenerator);
        let app_builder_gen = Box::new(codegen::plugin_generator::built_in::AppBuilderGenerator);
        
        if let Err(e) = codegen.register_generator(plugin::OutputFormat::FerrousApp, ferrous_engine_gen) {
            eprintln!("Failed to register FerrousApp generator: {}", e);
        }
        if let Err(e) = codegen.register_generator(plugin::OutputFormat::AppBuilder, app_builder_gen) {
            eprintln!("Failed to register AppBuilder generator: {}", e);
        }
        
        // Default codegen configuration
        let codegen_config = plugin::CodegenConfig {
            output_format: plugin::OutputFormat::FerrousApp,
            include_comments: true,
            include_examples: false,
            state_management: plugin::StateManagementStrategy::StructFields,
            layout_strategy: plugin::LayoutStrategy::Absolute,
            event_handling: plugin::EventHandlingStrategy::Callbacks,
        };
        
        Self {
            zoom: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
            last_mx: 0.0,
            last_my: 0.0,
            drag_offset: None,
            is_maximized: false,
            left_w: LEFT_W,
            right_w: RIGHT_W,
            resizing_left: false,
            resizing_right: false,
            icon_close: None,
            icon_minimize: None,
            icon_restore: None,
            show_settings_modal: false,
            preview_width: 800.0,
            preview_height: 600.0,
            preview_drag: None,
            scene: SceneState::default(),
            palette_state: PaletteState::default(),
            current_project_path: None,
            project: project::Project::new("Untitled"),
            project_buttons: panels::top_bar::ProjectButtons {
                save_pressed: false,
                load_pressed: false,
                copy_pressed: false,
            },
            plugin_manager,
            template_registry,
            codegen,
            codegen_config,
            // Theming system
            theme_registry: theme::ThemeRegistry::new(),
            color_picker: color_picker::ColorPickerState::default(),
            right_scroll_y: 0.0,
            window_width: 1280.0,
            settings_requested: false,
            window_close_requested: false,
            window_maximize_requested: false,
            window_minimize_requested: false,
        }
    }
}

impl FerrousApp for GUIMakerApp {
    fn setup(&mut self, ctx: &mut AppContext) {
        let renderer = ctx.render.renderer_mut();
        let device = &renderer.context.device;
        let queue = &renderer.context.queue;
        let size = 64_u32;

        self.icon_close = match Texture2d::from_svg_bytes(
            device,
            queue,
            include_bytes!("../assets/svgs/close.svg"),
            size,
            size,
        ) {
            Ok(tex) => Some(Arc::new(tex)),
            Err(e) => {
                eprintln!("[top_bar] Failed to load close.svg: {e}");
                None
            }
        };

        self.icon_minimize = match Texture2d::from_svg_bytes(
            device,
            queue,
            include_bytes!("../assets/svgs/minimize.svg"),
            size,
            size,
        ) {
            Ok(tex) => Some(Arc::new(tex)),
            Err(e) => {
                eprintln!("[top_bar] Failed to load minimize.svg: {e}");
                None
            }
        };

        self.icon_restore = match Texture2d::from_svg_bytes(
            device,
            queue,
            include_bytes!("../assets/svgs/restore.svg"),
            size,
            size,
        ) {
            Ok(tex) => Some(Arc::new(tex)),
            Err(e) => {
                eprintln!("[top_bar] Failed to load restore.svg: {e}");
                None
            }
        };
    }

    fn configure_ui(&mut self, ui: &mut ferrous_gui::UiSystem<Self>) {
        crate::panels::top_bar::build_top_bar(ui, &self.theme_registry.get_current(), self.window_width);
    }

    fn update(&mut self, ctx: &mut AppContext) {
        let (win_w, win_h) = ctx.window_size;
        self.window_width = win_w as f32;

        if ctx.input.just_pressed(KeyCode::Escape) && !self.show_settings_modal {
            ctx.request_exit();
        }

        let ctrl = ctx.input.is_key_down(KeyCode::ControlLeft) || ctx.input.is_key_down(KeyCode::ControlRight);
        if ctrl && ctx.input.just_pressed(KeyCode::KeyC) {
            self.scene.copy_selected();
        }
        if ctrl && ctx.input.just_pressed(KeyCode::KeyV) {
            self.scene.paste_start();
        }
        if ctx.input.just_pressed(KeyCode::Delete) || ctx.input.just_pressed(KeyCode::Backspace) {
            if self.scene.editing_field.is_none() {
                self.scene.delete_selected();
            }
        }

        if self.show_settings_modal {
            crate::panels::settings::update(
                ctx,
                &mut self.preview_width,
                &mut self.preview_height,
                &mut self.show_settings_modal,
                &mut self.theme_registry,
            );
        }

        if crate::panels::top_bar::update(
            ctx,
            &mut self.drag_offset,
            &mut self.is_maximized,
        ) {
            ctx.request_exit();
            return;
        }
        
        if self.window_close_requested {
            ctx.request_exit();
            return;
        }
        if self.window_maximize_requested {
            self.window_maximize_requested = false;
            self.is_maximized = !self.is_maximized;
            ctx.window.set_maximized(self.is_maximized);
        }
        if self.window_minimize_requested {
            self.window_minimize_requested = false;
            ctx.window.set_minimized(true);
        }
        if self.settings_requested {
            self.settings_requested = false;
            self.show_settings_modal = !self.show_settings_modal;
        }

        if self.project_buttons.save_pressed {
            self.project_buttons.save_pressed = false;
            self.save_project();
        }
        if self.project_buttons.load_pressed {
            self.project_buttons.load_pressed = false;
            self.load_project();
        }
        if self.project_buttons.copy_pressed {
            self.project_buttons.copy_pressed = false;
            self.copy_to_clipboard();
        }

        let (mx, my) = ctx.input.mouse_pos_f32();
        let mut color_picker_hovered = self.color_picker.active;
        if self.color_picker.active {
            let cx = self.color_picker.screen_pos.0;
            let cy = self.color_picker.screen_pos.1;
            let w = 280.0 * (0.85 + 0.15 * self.color_picker.open_anim.min(1.0).powi(3));
            let h = 520.0 * (0.85 + 0.15 * self.color_picker.open_anim.min(1.0).powi(3));
            if mx >= cx && mx <= cx + w && my >= cy && my <= cy + h {
                color_picker_hovered = true;
            }
        }
        let modal_blocked = self.show_settings_modal || color_picker_hovered;

        if !modal_blocked {
            if !self.is_maximized {
                let resize_dir = resize_direction(mx, my, win_w, win_h);
                let icon = match resize_dir {
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
                if ctx.input.button_just_pressed(ferrous_engine::MouseButton::Left) {
                    if let Some(dir) = resize_dir {
                        ctx.start_window_resize(dir);
                    }
                }
            } else {
                ctx.window.set_cursor(CursorIcon::Default);
            }
        } else {
            ctx.window.set_cursor(CursorIcon::Default);
        }

        let ww = win_w as f32;
        let over_left_edge = (mx - self.left_w).abs() < PANEL_EDGE_HIT && my > TOP_H;
        let over_right_edge = (mx - (ww - self.right_w)).abs() < PANEL_EDGE_HIT && my > TOP_H;

        if ctx.input.button_just_pressed(ferrous_engine::MouseButton::Left) {
            if !modal_blocked && over_left_edge {
                self.resizing_left = true;
            }
            if !modal_blocked && over_right_edge {
                self.resizing_right = true;
            }
        }
        if ctx.input.button_just_released(ferrous_engine::MouseButton::Left) {
            self.resizing_left = false;
            self.resizing_right = false;
        }
        if ctx.input.is_button_down(ferrous_engine::MouseButton::Left) {
            if !modal_blocked && self.resizing_left {
                self.left_w = mx.clamp(LEFT_W_MIN, LEFT_W_MAX);
            }
            if !modal_blocked && self.resizing_right {
                self.right_w = (ww - mx).clamp(RIGHT_W_MIN, RIGHT_W_MAX);
            }
        }
        if !modal_blocked && (over_left_edge || self.resizing_left || over_right_edge || self.resizing_right)
            && resize_direction(mx, my, win_w, win_h).is_none()
        {
            ctx.window.set_cursor(CursorIcon::EwResize);
        }

        if !self.show_settings_modal && !self.resizing_left && !self.resizing_right && !color_picker_hovered {
            crate::panels::left_panel::update(ctx, self.left_w, &mut self.scene, &mut self.palette_state);
            crate::panels::right_panel::update(ctx, self.right_w, &mut self.scene, &mut self.right_scroll_y, &mut self.color_picker);
        }

        crate::panels::canvas::update(
            ctx,
            &mut self.zoom,
            &mut self.pan_x,
            &mut self.pan_y,
            &mut self.last_mx,
            &mut self.last_my,
            self.left_w,
            self.right_w,
            &mut self.preview_width,
            &mut self.preview_height,
            &mut self.preview_drag,
            &mut self.scene,
            color_picker_hovered,
        );

        self.scene.resolve_all_layouts(self.preview_width, self.preview_height);
    }

    fn draw_ui(&mut self, dc: &mut DrawContext<'_, '_>) {
        let theme = self.theme_registry.get_current().clone();
        
        crate::panels::canvas::draw(
            dc,
            self.zoom,
            self.pan_x,
            self.pan_y,
            self.left_w,
            self.right_w,
            self.preview_width,
            self.preview_height,
            &self.scene,
            &theme,
        );

        crate::panels::left_panel::draw_background(dc, self.left_w, &theme);
        crate::panels::right_panel::draw_background(dc, self.right_w, &theme);
        crate::panels::left_panel::draw_border(dc, self.left_w, &theme);
        crate::panels::right_panel::draw_border(dc, self.right_w, &theme);
        crate::panels::left_panel::draw(dc, self.left_w, &self.palette_state, &self.scene, &theme);
        crate::panels::right_panel::draw(dc, self.right_w, &mut self.scene, &theme, self.right_scroll_y, &self.color_picker);

        crate::panels::top_bar::draw(
            dc,
            self.zoom,
            self.is_maximized,
            self.icon_close.clone(),
            self.icon_minimize.clone(),
            self.icon_restore.clone(),
            self.show_settings_modal,
            &theme,
        );

        if self.show_settings_modal {
            crate::panels::settings::draw(dc, self.preview_width, self.preview_height, &theme, &self.theme_registry);
        }

        let cur_hex = if let Some(wid) = self.color_picker.widget_id {
            self.scene.widgets.iter().find(|w| w.id == wid).map(|w| {
                match self.color_picker.target_field.as_str() {
                    "bg_color" => w.props.bg_color_hex.clone(),
                    "text_color" => w.props.text_color_hex.clone(),
                    "color_hex" => w.props.color_hex.clone(),
                    f if f.starts_with("stop_col_") => {
                       let idx = f["stop_col_".len()..].parse::<usize>().unwrap_or(0);
                       w.props.gradient_stops.get(idx).map(|s| s.1.clone()).unwrap_or_else(|| "#FFFFFF".to_string())
                    }
                    _ => "#FFFFFF".to_string(),
                }
            }).unwrap_or_else(|| "#FFFFFF".to_string())
        } else {
            "#FFFFFF".to_string()
        };

        let (new_hex, should_close) = color_picker::draw_picker_modal(dc, &mut self.color_picker, cur_hex, &theme);
        if let Some(new_hex) = new_hex {
            if let Some(wid) = self.color_picker.widget_id {
                if let Some(w) = self.scene.widgets.iter_mut().find(|w| w.id == wid) {
                    use crate::panels::widgets::shared::parse_hex_or;
                    match self.color_picker.target_field.as_str() {
                        "bg_color" => {
                            let parsed = parse_hex_or(&new_hex, [1.0, 1.0, 1.0, 1.0]);
                            let r = (parsed[0] * 255.0).round() as u8;
                            let g = (parsed[1] * 255.0).round() as u8;
                            let b = (parsed[2] * 255.0).round() as u8;
                            w.props.bg_color_hex = format!("#{:02X}{:02X}{:02X}", r, g, b);
                            w.props.opacity = self.color_picker.alpha.clamp(0.0, 1.0);
                        }
                        "text_color" => w.props.text_color_hex = new_hex,
                        "color_hex" => w.props.color_hex = new_hex,
                        f if f.starts_with("stop_col_") => {
                           let idx = f["stop_col_".len()..].parse::<usize>().unwrap_or(0);
                           if let Some(stop) = w.props.gradient_stops.get_mut(idx) {
                               stop.1 = new_hex;
                           }
                        }
                        _ => {}
                    }
                }
            }
        }
        if should_close {
            self.color_picker.active = false;
        }
    }
}

pub fn resize_direction(
    mx: f32,
    my: f32,
    win_w: u32,
    win_h: u32,
) -> Option<WindowResizeDirection> {
    const E: f32 = 6.0;
    let (w, h) = (win_w as f32, win_h as f32);
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

impl GUIMakerApp {
    pub fn save_project(&mut self) {
        let path = match &self.current_project_path {
            Some(p) => p.clone(),
            None => {
                if let Some(path) = file_dialog::save_project_dialog(&self.project.name) {
                    self.current_project_path = Some(path.clone());
                    path
                } else { return; }
            }
        };
        if let Some(page) = self.project.current_page_mut() {
            page.widgets = self.scene.widgets.clone();
            page.canvas_config = project::CanvasConfig {
                zoom: self.zoom, pan_x: self.pan_x, pan_y: self.pan_y,
                preview_width: self.preview_width, preview_height: self.preview_height,
            };
        }
        let _ = self.project.save_to_file(&path);
    }

    pub fn load_project(&mut self) {
        if let Some(path) = file_dialog::open_project_dialog() {
            if let Ok(project) = project::Project::load_from_file(&path) {
                project.apply_to_scene(&mut self.scene);
                self.project = project;
                self.current_project_path = Some(path.clone());
                if let Some(page) = self.project.current_page() {
                    self.zoom = page.canvas_config.zoom;
                    self.pan_x = page.canvas_config.pan_x;
                    self.pan_y = page.canvas_config.pan_y;
                }
            }
        }
    }

    pub fn copy_to_clipboard(&mut self) {
        let fui_json = crate::codegen::scene_builder::generate_fui_json(&self.scene);
        let controller_code = crate::codegen::scene_builder::generate_controller_macro_skeleton(&self.scene);
        let combined = format!("FUI:\n{}\n\nController:\n{}", fui_json, controller_code);
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                let _ = clipboard.set_text(&combined);
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            // Web clipboard API is async and needs JS glue, for now just log
            eprintln!("Clipboard copy requested on WASM (not yet implemented)");
        }
    }

    pub fn generate_code(&self) -> String {
        self.codegen.generate(&self.scene, &self.codegen_config, &self.project.name)
    }
}
