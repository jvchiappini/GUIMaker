mod panels;
mod scene;

use std::sync::Arc;

use ferrous_app::Color as AppColor;
use ferrous_app::{
    App, AppContext, AppMode, CursorIcon, DrawContext, FerrousApp, KeyCode, WindowResizeDirection,
};
use ferrous_assets::Texture2d;
use ferrous_gui::{Color, NodeId, UiTree};

use panels::left_panel::PaletteState;
use scene::SceneState;

// ── Layout constants ──────────────────────────────────────────────────────────
pub(crate) const TOP_H: f32 = 40.0;
pub(crate) const LEFT_W: f32 = 200.0;
pub(crate) const RIGHT_W: f32 = 240.0;

pub(crate) const LEFT_W_MIN: f32 = 150.0;
pub(crate) const LEFT_W_MAX: f32 = 400.0;
pub(crate) const RIGHT_W_MIN: f32 = 180.0;
pub(crate) const RIGHT_W_MAX: f32 = 500.0;
const PANEL_EDGE_HIT: f32 = 6.0;

// ── Color palette ─────────────────────────────────────────────────────────────
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

// ── Preview resize handle ─────────────────────────────────────────────────────
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum PreviewDrag {
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
struct GUIMakerApp {
    // Canvas navigation
    zoom: f32,
    pan_x: f32,
    pan_y: f32,
    last_mx: f32,
    last_my: f32,
    // Window control
    drag_offset: Option<(i32, i32)>,
    is_maximized: bool,
    // Preview canvas
    show_settings_modal: bool,
    preview_width: f32,
    preview_height: f32,
    preview_drag: Option<PreviewDrag>,
    // Panel resize
    left_w: f32,
    right_w: f32,
    resizing_left: bool,
    resizing_right: bool,
    // Title-bar icons
    icon_close: Option<Arc<Texture2d>>,
    icon_minimize: Option<Arc<Texture2d>>,
    icon_restore: Option<Arc<Texture2d>>,
    // UI node ids (unused after removing old UI-tree widget building)
    left_panel_id: Option<NodeId>,
    right_panel_id: Option<NodeId>,
    last_selected_id: Option<u32>,
    // ── Scene builder ─────────────────────────────────────────────────────────
    scene: SceneState,
    palette_state: PaletteState,
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
            preview_drag: None,
            left_panel_id: None,
            right_panel_id: None,
            last_selected_id: None,
            scene: SceneState::default(),
            palette_state: PaletteState::default(),
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
        // We only keep the invisible root panels for node-id bookkeeping.
        // All visible content is rendered manually in draw_ui.
        use ferrous_ui_core::{Panel, StyleBuilder};

        let left_panel = Panel::new().with_color(Color::hex("#00000000"));
        let left_id = ui.add_node(Box::new(left_panel), None);
        ui.set_node_style(left_id, StyleBuilder::new().absolute().build());
        self.left_panel_id = Some(left_id);

        let right_panel = Panel::new().with_color(Color::hex("#00000000"));
        let right_id = ui.add_node(Box::new(right_panel), None);
        ui.set_node_style(right_id, StyleBuilder::new().absolute().build());
        self.right_panel_id = Some(right_id);

        // Build initial right panel state
        panels::right_panel::configure_ui(ui, right_id, &self.scene);
    }

    fn update(&mut self, ctx: &mut AppContext) {
        // ── Input: handle UI events first ───────────────────────────────────
        if !self.show_settings_modal {
            let win_w = ctx.window_size.0 as f32;
            let right_x = win_w - self.right_w;
            // Only handle events for the right panel if mouse is over it.
            // (Simple optimization, and prevents canvas drag through panel).
            let (mx, my) = ctx.input.mouse_pos_f32();
            if mx >= right_x && my >= TOP_H {
                ctx.gui.handle_event(ctx);
            }
        }

        // Escape to exit (unless a modal is open)
        if ctx.input.just_pressed(KeyCode::Escape) && !self.show_settings_modal {
            ctx.request_exit();
        }

        // Settings modal
        if self.show_settings_modal {
            panels::settings::update(
                ctx,
                &mut self.preview_width,
                &mut self.preview_height,
                &mut self.show_settings_modal,
            );
        }

        // Top bar
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

        // Window-edge resize
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

        // Panel side-resize
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

        // ── Left panel: widget palette ────────────────────────────────────────
        // Only handle palette interaction when the settings modal is not open
        if !self.show_settings_modal && !self.resizing_left && !self.resizing_right {
            panels::left_panel::update(ctx, self.left_w, &mut self.scene, &mut self.palette_state);
        }

        // ── Right panel: properties inspector ────────────────────────────────
        if !self.show_settings_modal && !self.resizing_left && !self.resizing_right {
            // Check if selection changed to rebuild UI
            if self.scene.selected_id != self.last_selected_id {
                if let Some(right_id) = self.right_panel_id {
                    ctx.gui.clear_node_children(right_id);
                    panels::right_panel::configure_ui(ctx.gui, right_id, &self.scene);
                }
                self.last_selected_id = self.scene.selected_id;
            }
            panels::right_panel::update(ctx, self.right_w, &mut self.scene);
        }

        // ── Canvas: zoom, pan, widget select/drag, palette drop ───────────────
        panels::canvas::update(
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
        );
    }

    fn draw_ui(&mut self, dc: &mut DrawContext<'_, '_>) {
        // 1. Canvas (below everything)
        panels::canvas::draw(
            dc,
            self.zoom,
            self.pan_x,
            self.pan_y,
            self.left_w,
            self.right_w,
            self.preview_width,
            self.preview_height,
            &self.scene,
        );

        // 2. Panel backgrounds
        panels::left_panel::draw_background(dc, self.left_w);
        panels::right_panel::draw_background(dc, self.right_w);

        // 3. Panel borders
        panels::left_panel::draw_border(dc, self.left_w);
        panels::right_panel::draw_border(dc, self.right_w);

        // 4. Left panel content: widget palette
        panels::left_panel::draw(dc, self.left_w, &self.palette_state, &self.scene);

        // 5. Right panel content: properties inspector
        panels::right_panel::draw(dc, self.right_w, &self.scene);

        // 5.1 Right panel UI tree (interactive elements)
        let (win_w, win_h) = dc.ctx.window_size;
        let right_x = win_w as f32 - self.right_w;
        let panel_h = win_h as f32 - TOP_H;
        dc.gui.push_clip(ferrous_gui::Rect::new(right_x, TOP_H, self.right_w, panel_h));
        if let Some(rid) = self.right_panel_id {
            dc.gui.set_node_position(rid, right_x, TOP_H);
            dc.gui.set_node_size(rid, self.right_w, panel_h);
            dc.gui.draw_node(rid, dc);
        }
        dc.gui.pop_clip();

        // 6. Top bar (always on top)
        panels::top_bar::draw(
            dc,
            self.zoom,
            self.is_maximized,
            self.icon_close.clone(),
            self.icon_minimize.clone(),
            self.icon_restore.clone(),
            self.show_settings_modal,
        );

        // 7. Settings modal (on top of everything when visible)
        if self.show_settings_modal {
            panels::settings::draw(dc, self.preview_width, self.preview_height);
        }
    }
}

// ── Helper: window edge resize direction ──────────────────────────────────────
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
        .with_vsync(false)
        .with_target_fps(Some(240))
        .with_background_color(AppColor::hex("#1E1E1E"))
        .with_font("assets/fonts/Roboto-Regular.ttf")
        .run();
}
