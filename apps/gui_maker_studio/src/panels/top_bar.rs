use std::sync::Arc;
use ferrous_engine::{AppContext, DrawContext, MouseButton};
use ferrous_assets::Texture2d;
use ferrous_gui::UiSystem;
use ferrous_ui_core::Background;

use crate::{c_top, GUIMakerApp, TOP_H};
const TOP_BAR_LEFT_FIXED_W: f32 = 100.0 + 80.0 + 80.0 + 100.0;
const TOP_BAR_RIGHT_FIXED_W: f32 = 60.0 + (TOP_H * 3.0);

pub struct ProjectButtons {
    pub save_pressed: bool,
    pub load_pressed: bool,
    pub copy_pressed: bool,
}

pub fn build_top_bar(ui: &mut UiSystem<GUIMakerApp>, theme: &crate::theme::Theme, bar_width: f32) {
    let top_col = crate::theme::ThemeRegistry::hex_to_f32(&theme.colors.top_bar);
    let text_col = crate::theme::ThemeRegistry::hex_to_f32(&theme.colors.text);

    let flat_button_bg = Background::Solid([top_col[0], top_col[1], top_col[2], 1.0]);
    ui.panel()
        .row()
        .width(bar_width)
        .height(TOP_H)
        .color([0.0, 0.0, 0.0, 0.0].into())
        .id("top_bar_container")
        .spawn_with(ui, |ui: &mut UiSystem<GUIMakerApp>, _| {
            ui.label("GUIMaker").color(text_col.into()).width(100.0).spawn(ui);
                    ui.button("Guardar")
                        .chrome(false)
                        .border_radius(0.0)
                        .background(flat_button_bg.clone())
                        .width(80.0)
                        .height(TOP_H)
                        .on_click(|app| {
                            app.app.project_buttons.save_pressed = true;
                        })
                        .spawn(ui);

                    ui.button("Cargar")
                        .chrome(false)
                        .border_radius(0.0)
                        .background(flat_button_bg.clone())
                        .width(80.0)
                        .height(TOP_H)
                        .on_click(|app| {
                            app.app.project_buttons.load_pressed = true;
                        })
                        .spawn(ui);

                    ui.button("Export FUI")
                        .chrome(false)
                        .border_radius(0.0)
                        .background(flat_button_bg.clone())
                        .width(100.0)
                        .height(TOP_H)
                        .on_click(|app| {
                            app.app.project_buttons.copy_pressed = true;
                        })
                        .spawn(ui);

            ui.panel()
                .fill()
                .height(TOP_H)
                .color([0.0, 0.0, 0.0, 0.0].into())
                .id("top_bar_drag_area")
                .spawn(ui);

            ui.panel()
                .row()
                .width(TOP_BAR_RIGHT_FIXED_W)
                .height(TOP_H)
                .color([0.0, 0.0, 0.0, 0.0].into())
                .spawn_with(ui, |ui, _| {
                    ui.button("Config")
                        .chrome(false)
                        .border_radius(0.0)
                        .background(flat_button_bg.clone())
                        .width(60.0)
                        .height(TOP_H)
                        .on_click(|app| {
                            app.app.settings_requested = true;
                        })
                        .spawn(ui);

                    ui.button("–")
                        .chrome(false)
                        .border_radius(0.0)
                        .background(flat_button_bg.clone())
                        .id("top_btn_min")
                        .width(TOP_H)
                        .height(TOP_H)
                        .on_click(|app| {
                            app.app.window_minimize_requested = true;
                        })
                        .spawn(ui);

                    ui.button("□")
                        .chrome(false)
                        .border_radius(0.0)
                        .background(flat_button_bg.clone())
                        .id("top_btn_max")
                        .width(TOP_H)
                        .height(TOP_H)
                        .on_click(|app| {
                            app.app.window_maximize_requested = true;
                        })
                        .spawn(ui);

                    ui.button("×")
                        .chrome(false)
                        .border_radius(0.0)
                        .background(flat_button_bg)
                        .id("top_btn_close")
                        .width(TOP_H)
                        .height(TOP_H)
                        .on_click(|app| {
                            app.app.window_close_requested = true;
                        })
                        .spawn(ui);
                });
        });
}

pub fn update(
    ctx: &mut AppContext,
    drag_offset: &mut Option<(i32, i32)>,
    _is_maximized: &mut bool,
) -> bool {
    let (win_w, win_h) = ctx.window_size;
    let (mx, my) = ctx.input.mouse_pos_f32();
    let ww = win_w as f32;

    let over_drag = mx >= TOP_BAR_LEFT_FIXED_W
        && mx < ww - TOP_BAR_RIGHT_FIXED_W
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

    false
}

pub fn draw(
    dc: &mut DrawContext<'_, '_>,
    zoom: f32,
    is_maximized: bool,
    icon_close: Option<Arc<Texture2d>>,
    icon_minimize: Option<Arc<Texture2d>>,
    icon_restore: Option<Arc<Texture2d>>,
    _show_settings: bool,
    theme: &crate::theme::Theme,
) {
    let (win_w, _) = dc.ctx.window_size;
    let ww = win_w as f32;

    dc.gui.rect(0.0, 0.0, ww, TOP_H, c_top(theme));
    dc.gui.rect(
        0.0,
        TOP_H - 2.0,
        ww,
        2.0,
        ferrous_engine::Color::hex("#007ACC").to_linear_f32(),
    );

    let zoom_label = format!("zoom  {:.0}%", zoom * 100.0);
    dc.gui.draw_text(
        dc.font,
        &zoom_label,
        [crate::LEFT_W + 8.0, 13.0],
        11.0,
        ferrous_engine::Color::hex("#3DC9B0").to_linear_f32(),
    );

    let text_color = ferrous_engine::Color::hex("#CCCCCC").to_linear_f32();
    let icon_sz = TOP_H * 0.45;
    let icon_off = (TOP_H - icon_sz) * 0.5;

    let close_x = ww - TOP_H;
    if let Some(tex) = icon_close {
        dc.gui.image(
            close_x + icon_off,
            icon_off,
            icon_sz,
            icon_sz,
            tex,
            [0.0, 0.0],
            [1.0, 1.0],
            text_color,
        );
    }

    let max_x = ww - TOP_H * 2.0;
    if let Some(tex) = icon_restore {
        dc.gui.image(
            max_x + icon_off,
            icon_off,
            icon_sz,
            icon_sz,
            tex,
            [0.0, 0.0],
            [1.0, 1.0],
            text_color,
        );
    } else {
        let max_icon = if is_maximized { "❐" } else { "□" };
        dc.gui.draw_text(
            dc.font,
            max_icon,
            [max_x + TOP_H * 0.22, 8.0],
            16.0,
            text_color,
        );
    }

    let min_x = ww - TOP_H * 3.0;
    if let Some(tex) = icon_minimize {
        dc.gui.image(
            min_x + icon_off,
            icon_off,
            icon_sz,
            icon_sz,
            tex,
            [0.0, 0.0],
            [1.0, 1.0],
            text_color,
        );
    } else {
        dc.gui.draw_text(
            dc.font,
            "–",
            [min_x + TOP_H * 0.28, 9.0],
            18.0,
            text_color,
        );
    }
}
