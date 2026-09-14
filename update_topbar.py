import re

with open("/opt/softwaremaker/workspaces/d3a95f70-92a6-48b3-a540-f319263da71e/src/panels/top_bar.rs", "w") as f:
    f.write("""use std::sync::Arc;
use ferrous_app::{AppContext, DrawContext, MouseButton};
use ferrous_assets::Texture2d;
use ferrous_gui::UiSystem;

use crate::{GUIMakerApp, TOP_H};

// ── Estados para los botones de proyecto ──────────────────────────────────────────
pub struct ProjectButtons {
    pub save_pressed: bool,
    pub load_pressed: bool,
    pub copy_pressed: bool,
}

// ── FUI Builder ──────────────────────────────────────────────────────────────────

pub fn build_top_bar(ui: &mut UiSystem<GUIMakerApp>, theme: &crate::theme::Theme) {
    let top_col = crate::theme::ThemeRegistry::hex_to_f32(&theme.colors.secondary);
    let dimmed_col = crate::theme::ThemeRegistry::hex_to_f32(&theme.colors.text_dim);
    let text_col = crate::theme::ThemeRegistry::hex_to_f32(&theme.colors.text);

    ui.panel()
        .row()
        .fill_width()
        .height(TOP_H)
        .color(top_col)
        .id("top_bar_container")
        .spawn_with(ui, |ui| {
            // Label de la app
            ui.label()
                .text("GUIMaker")
                .color(text_col)
                .margin_left(14.0)
                .margin_top(12.0)
                .margin_right(20.0)
                .spawn(ui);

            // Botón Guardar
            ui.button()
                .text("💾 Guardar")
                .text_color(dimmed_col)
                .width(80.0)
                .height(TOP_H)
                .on_click(|app| {
                    app.project_buttons.save_pressed = true;
                })
                .spawn(ui);

            // Botón Cargar
            ui.button()
                .text("📂 Cargar")
                .text_color(dimmed_col)
                .width(80.0)
                .height(TOP_H)
                .on_click(|app| {
                    app.project_buttons.load_pressed = true;
                })
                .spawn(ui);

            // Botón Exportar
            ui.button()
                .text("📋 Export FUI")
                .text_color(dimmed_col)
                .width(100.0)
                .height(TOP_H)
                .on_click(|app| {
                    app.project_buttons.copy_pressed = true;
                })
                .spawn(ui);

            // Spacer flexible
            ui.panel()
                .fill_width()
                .height(TOP_H)
                .color([0.0, 0.0, 0.0, 0.0])
                .id("top_bar_drag_area")
                .spawn(ui);

            // Config
            ui.button()
                .text("Config")
                .text_color(dimmed_col)
                .width(60.0)
                .height(TOP_H)
                .on_click(|app| {
                    app.settings_requested = true;
                })
                .spawn(ui);

            // Botón minimizar
            ui.button()
                .text("–")
                .text_color(text_col)
                .width(TOP_H)
                .height(TOP_H)
                .on_click(|app| {
                    app.window_minimize_requested = true;
                })
                .spawn(ui);

            // Botón maximizar
            ui.button()
                .text("□")
                .text_color(text_col)
                .width(TOP_H)
                .height(TOP_H)
                .on_click(|app| {
                    app.window_maximize_requested = true;
                })
                .spawn(ui);

            // Botón cerrar
            ui.button()
                .text("×")
                .text_color(text_col)
                .width(TOP_H)
                .height(TOP_H)
                // Usamos on_click normal
                .on_click(|app| {
                    app.window_close_requested = true;
                })
                .spawn(ui);
        });
}

// ── Update ────────────────────────────────────────────────────────────────────

pub fn update(
    ctx: &mut AppContext,
    drag_offset: &mut Option<(i32, i32)>,
    is_maximized: &mut bool,
) -> bool {
    let (win_w, win_h) = ctx.window_size;
    let (mx, my) = ctx.input.mouse_pos_f32();
    let ww = win_w as f32;

    // Solo verificamos el dragging porque FUI ya hace el hit-testing de botones
    let over_drag = mx > 350.0 
        && mx < ww - (TOP_H * 4.0) 
        && my >= 0.0 
        && my <= TOP_H
        && crate::resize_direction(mx, my, win_w, win_h).is_none();

    if ctx.input.button_just_pressed(MouseButton::Left) && over_drag {
        let win_pos = ctx.window_position().unwrap_or((0, 0));
        let screen_mx = win_pos.0 + mx as i32;
        let screen_my = win_pos.1 + my as i32;
        *drag_offset = Some((screen_mx - win_pos.0, screen_my - win_pos.1));
    }
    if ctx.input.button_just_released(MouseButton::Left) {
        *drag_offset = None;
    }
    if ctx.input.is_button_down(MouseButton::Left) {
        if let Some(offset) = *drag_offset {
            let win_pos = ctx.window_position().unwrap_or((0, 0));
            let screen_mx = win_pos.0 + mx as i32;
            let screen_my = win_pos.1 + my as i32;
            ctx.set_window_position(screen_mx - offset.0, screen_my - offset.1);
        }
    }

    false // Ya no retornamos true por cerrar, ahora la app procesa `window_close_requested`
}

// La UI es dibujada por FUI! 
// Pero si queremos repintar encima iconos y linea azul:
pub fn draw(
    dc: &mut DrawContext<'_, '_>,
    _zoom: f32,
    _is_maximized: bool,
    _icon_close: Option<Arc<Texture2d>>,
    _icon_minimize: Option<Arc<Texture2d>>,
    _icon_restore: Option<Arc<Texture2d>>,
    _show_settings: bool,
    theme: &crate::theme::Theme,
) {
    let (win_w, _) = dc.ctx.window_size;
    let ww = win_w as f32;
    // Sombra para el top_bar
    dc.gui.shadow(0.0, TOP_H - 1.0, ww, 2.0, [0.0, 0.0, 0.0, 0.4], 12.0, 2.0, [0.0, 3.0], [0.0; 4]);
    
    // Línea de acento inferior
    dc.gui.rect(
        0.0,
        TOP_H - 1.0,
        ww,
        1.0,
        crate::theme::ThemeRegistry::hex_to_f32(&theme.colors.accent),
    );
}
""")

print("top_bar rewritten")
