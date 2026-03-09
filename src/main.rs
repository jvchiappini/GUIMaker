mod panels;

use std::sync::Arc;

use ferrous_app::{
    App, AppContext, AppMode, CursorIcon, DrawContext, FerrousApp, KeyCode,
    WindowResizeDirection,
};
use ferrous_app::Color as AppColor;
use ferrous_assets::Texture2d;
use ferrous_gui::{Button, Color, NodeId, UiTree};
use ferrous_ui_core::{Label, Panel, Separator, StyleBuilder};

// ── Constantes de layout ──────────────────────────────────────────────────────
pub(crate) const TOP_H: f32 = 40.0;
pub(crate) const LEFT_W: f32 = 200.0;
pub(crate) const RIGHT_W: f32 = 220.0;

// ── Límites de resize de paneles laterales ────────────────────────────────────
pub(crate) const LEFT_W_MIN: f32 = 120.0;
pub(crate) const LEFT_W_MAX: f32 = 400.0;
pub(crate) const RIGHT_W_MIN: f32 = 150.0;
pub(crate) const RIGHT_W_MAX: f32 = 480.0;
const PANEL_EDGE_HIT: f32 = 6.0;

// ── Paleta de colores — Dark+ de VS Code ──────────────────────────────────────
pub(crate) fn c_top() -> [f32; 4] {
    AppColor::hex("#3C3C3C").to_linear_f32()
}
pub(crate) fn c_canvas() -> [f32; 4] {
    AppColor::hex("#1E1E1E").to_linear_f32()
}
pub(crate) fn c_border() -> [f32; 4] {
    AppColor::hex("#333333").to_linear_f32()
}
pub(crate) fn c_grid() -> [f32; 4] {
    AppColor::hex("#2D2D2D").to_linear_f32()
}

// ── Estado de la aplicación ───────────────────────────────────────────────────
struct GUIMakerApp {
    // Canvas
    zoom: f32,
    pan_x: f32,
    pan_y: f32,
    last_mx: f32,
    last_my: f32,
    // Ventana personalizada
    drag_offset: Option<(i32, i32)>,
    is_maximized: bool,
    // Configuración de la previsualización
    show_settings_modal: bool,
    preview_width: f32,
    preview_height: f32,
    preview_responsive: bool,
    // Paneles redimensionables
    left_w: f32,
    right_w: f32,
    resizing_left: bool,
    resizing_right: bool,
    // Iconos de la barra de título
    icon_close: Option<Arc<Texture2d>>,
    icon_minimize: Option<Arc<Texture2d>>,
    icon_restore: Option<Arc<Texture2d>>,
    // NodeIds del nuevo árbol de UI
    left_panel_id: Option<NodeId>,
    right_panel_id: Option<NodeId>,
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
            preview_responsive: true,
            left_panel_id: None,
            right_panel_id: None,
        }
    }
}

impl FerrousApp for GUIMakerApp {
    fn setup(&mut self, ctx: &mut AppContext) {
        let renderer = ctx.render.renderer_mut();
        let device = &renderer.context.device;
        let queue = &renderer.context.queue;
        let size = 20_u32;

        self.icon_close =
            Texture2d::from_svg_file(device, queue, "assets/svgs/close.svg", size, size)
                .ok()
                .map(Arc::new);
        self.icon_minimize =
            Texture2d::from_svg_file(device, queue, "assets/svgs/minimize.svg", size, size)
                .ok()
                .map(Arc::new);
        self.icon_restore =
            Texture2d::from_svg_file(device, queue, "assets/svgs/restore.svg", size, size)
                .ok()
                .map(Arc::new);
    }

    fn configure_ui(&mut self, ui: &mut UiTree<Self>) {
        // ── Panel izquierdo — Herramientas ────────────────────────────────────
        let left_panel = Panel::new()
            .with_color(Color::hex("#252526"));
        let left_id = ui.add_node(Box::new(left_panel), None);
        ui.set_node_style(
            left_id,
            StyleBuilder::new()
                .absolute()
                .left(0.0)
                .top(TOP_H)
                .width_px(LEFT_W)
                .fill_height()
                .column()
                .padding_all(8.0)
                .build(),
        );
        self.left_panel_id = Some(left_id);

        // Título del panel izquierdo
        let title_left = Label::new("Herramientas")
            .with_color(Color::hex("#BBBBBB"))
            .with_size(11.0);
        let title_left_id = ui.add_node(Box::new(title_left), Some(left_id));
        ui.set_node_style(
            title_left_id,
            StyleBuilder::new().fill_width().height_px(20.0).margin_xy(0.0, 4.0).build(),
        );

        // Separador
        let sep = Separator::new();
        let sep_id = ui.add_node(Box::new(sep), Some(left_id));
        ui.set_node_style(
            sep_id,
            StyleBuilder::new().fill_width().height_px(1.0).margin_xy(0.0, 4.0).build(),
        );

        // Botones de herramientas
        for label in &["Selector", "Texto", "Imagen", "Botón", "Contenedor"] {
            let btn = Button::<GUIMakerApp>::new(*label).on_click(|_ctx| {});
            let btn_id = ui.add_node(Box::new(btn), Some(left_id));
            ui.set_node_style(
                btn_id,
                StyleBuilder::new().fill_width().height_px(30.0).margin_xy(0.0, 3.0).build(),
            );
        }

        // ── Panel derecho — Propiedades ───────────────────────────────────────
        let right_panel = Panel::new()
            .with_color(Color::hex("#252526"));
        let right_id = ui.add_node(Box::new(right_panel), None);
        ui.set_node_style(
            right_id,
            StyleBuilder::new()
                .absolute()
                .right(0.0)
                .top(TOP_H)
                .width_px(RIGHT_W)
                .fill_height()
                .column()
                .padding_all(8.0)
                .build(),
        );
        self.right_panel_id = Some(right_id);

        // Título del panel derecho
        let title_right = Label::new("Propiedades")
            .with_color(Color::hex("#BBBBBB"))
            .with_size(11.0);
        let title_right_id = ui.add_node(Box::new(title_right), Some(right_id));
        ui.set_node_style(
            title_right_id,
            StyleBuilder::new().fill_width().height_px(20.0).margin_xy(0.0, 4.0).build(),
        );

        // Separador
        let sep2 = Separator::new();
        let sep2_id = ui.add_node(Box::new(sep2), Some(right_id));
        ui.set_node_style(
            sep2_id,
            StyleBuilder::new().fill_width().height_px(1.0).margin_xy(0.0, 4.0).build(),
        );

        // Filas de propiedades
        for prop in &["X:", "Y:", "Ancho:", "Alto:"] {
            let lbl = Label::new(*prop)
                .with_color(Color::hex("#CCCCCC"))
                .with_size(11.0);
            let lbl_id = ui.add_node(Box::new(lbl), Some(right_id));
            ui.set_node_style(
                lbl_id,
                StyleBuilder::new().fill_width().height_px(22.0).margin_xy(0.0, 2.0).build(),
            );
        }
    }

    fn update(&mut self, ctx: &mut AppContext) {
        if ctx.input.just_pressed(KeyCode::Escape) && !self.show_settings_modal {
            ctx.request_exit();
        }

        // Si el modal está abierto, procesarlo primero para que capture los eventos
        if self.show_settings_modal {
            panels::settings::update(
                ctx,
                &mut self.preview_width,
                &mut self.preview_height,
                &mut self.preview_responsive,
                &mut self.show_settings_modal,
            );
        }

        // Barra superior: botones de ventana + drag
        let should_exit = panels::top_bar::update(
            ctx,
            &mut self.drag_offset,
            &mut self.is_maximized,
            &mut self.show_settings_modal,
        );
        if should_exit {
            ctx.request_exit();
            return;
        }

        // Resize desde los bordes de la ventana
        let (win_w, win_h) = ctx.window_size;
        let (mx, my) = ctx.input.mouse_pos_f32();
        if !self.is_maximized {
            let resize_dir = resize_direction(mx, my, win_w, win_h);
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
            if ctx
                .input
                .button_just_pressed(ferrous_app::MouseButton::Left)
            {
                if let Some(dir) = resize_dir {
                    ctx.start_window_resize(dir);
                }
            }
        } else {
            ctx.window.set_cursor(CursorIcon::Default);
        }

        // ── Resize de paneles laterales ───────────────────────────────────────
        let ww = win_w as f32;
        let over_left_edge = (mx - self.left_w).abs() < PANEL_EDGE_HIT && my > TOP_H;
        let over_right_edge = (mx - (ww - self.right_w)).abs() < PANEL_EDGE_HIT && my > TOP_H;

        if ctx
            .input
            .button_just_pressed(ferrous_app::MouseButton::Left)
        {
            if over_left_edge {
                self.resizing_left = true;
            }
            if over_right_edge {
                self.resizing_right = true;
            }
        }
        if ctx
            .input
            .button_just_released(ferrous_app::MouseButton::Left)
        {
            self.resizing_left = false;
            self.resizing_right = false;
        }
        if ctx.input.is_button_down(ferrous_app::MouseButton::Left) {
            if self.resizing_left {
                self.left_w = mx.clamp(LEFT_W_MIN, LEFT_W_MAX);
            }
            if self.resizing_right {
                self.right_w = (ww - mx).clamp(RIGHT_W_MIN, RIGHT_W_MAX);
            }
        }
        if (over_left_edge || self.resizing_left || over_right_edge || self.resizing_right)
            && resize_direction(mx, my, win_w, win_h).is_none()
        {
            ctx.window.set_cursor(CursorIcon::EwResize);
        }

        // Canvas: zoom y paneo
        panels::canvas::update(
            ctx,
            &mut self.zoom,
            &mut self.pan_x,
            &mut self.pan_y,
            &mut self.last_mx,
            &mut self.last_my,
            self.left_w,
            self.right_w,
        );
    }

    fn draw_ui(&mut self, dc: &mut DrawContext<'_, '_>) {
        // 1. Canvas / previsualizador (primero, queda debajo de los paneles)
        panels::canvas::draw(
            dc,
            self.zoom,
            self.pan_x,
            self.pan_y,
            self.left_w,
            self.right_w,
            self.preview_width,
            self.preview_height,
            self.preview_responsive,
        );
        // 2. Borde izquierdo (separador visual entre canvas y panel)
        panels::left_panel::draw_border(dc, self.left_w);
        // 3. Borde derecho (separador visual entre canvas y panel)
        panels::right_panel::draw_border(dc, self.right_w);
        // 4. Barra superior (al final, siempre encima de todo)
        panels::top_bar::draw(
            dc,
            self.zoom,
            self.is_maximized,
            self.icon_close.clone(),
            self.icon_minimize.clone(),
            self.icon_restore.clone(),
            self.show_settings_modal,
        );
        // Modal de ajustes (superpone todo cuando está activo)
        if self.show_settings_modal {
            panels::settings::draw(
                dc,
                self.preview_width,
                self.preview_height,
                self.preview_responsive,
            );
        }
    }
}

// ── Helper: detectar zona de resize en los bordes de la ventana ──────────────
pub(crate) fn resize_direction(
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

fn main() {
    App::new(GUIMakerApp::default())
        .with_title("GUIMaker")
        .with_size(1280, 720)
        .with_mode(AppMode::Desktop2D)
        .with_decorations(false)
        .with_resizable(true)
        .with_background_color(AppColor::hex("#1E1E1E"))
        .with_font("assets/fonts/Roboto-Regular.ttf")
        .run();
}
