use std::sync::Arc;

use ferrous_app::{AppContext, DrawContext, MouseButton};
use ferrous_assets::Texture2d;

use crate::{c_top, TOP_H};

// ── Update ────────────────────────────────────────────────────────────────────

/// Gestiona los botones de control de ventana (cerrar, maximizar, minimizar)
/// y el drag de la barra superior.
/// Devuelve `true` si se solicitó salir de la aplicación.
pub fn update(
    ctx: &mut AppContext,
    drag_offset: &mut Option<(i32, i32)>,
    is_maximized: &mut bool,
    open_settings: &mut bool,
) -> bool {
    let (win_w, win_h) = ctx.window_size;
    let ww = win_w as f32;
    let (mx, my) = ctx.input.mouse_pos_f32();

    // ── Botón cerrar [×] ─────────────────────────────────────────────────────
    let close_x = ww - TOP_H;
    let over_close = mx >= close_x && mx <= ww && my >= 0.0 && my <= TOP_H;
    if over_close && ctx.input.button_just_pressed(MouseButton::Left) {
        return true;
    }

    // ── Botón maximizar [□] ──────────────────────────────────────────────────
    let max_x = ww - TOP_H * 2.0;
    let over_max = mx >= max_x && mx < close_x && my >= 0.0 && my <= TOP_H;
    if over_max && ctx.input.button_just_pressed(MouseButton::Left) {
        *is_maximized = !*is_maximized;
        ctx.window.set_maximized(*is_maximized);
    }

    // ── Botón minimizar [–] ──────────────────────────────────────────────────
    let min_x = ww - TOP_H * 3.0;
    let over_min = mx >= min_x && mx < max_x && my >= 0.0 && my <= TOP_H;
    if over_min && ctx.input.button_just_pressed(MouseButton::Left) {
        ctx.window.set_minimized(true);
    }

    // ── Botón Config ─────────────────────────────────────────────────────────
    // 4° slot; se evalúa después de los otros para no robar el evento
    let gear_slot_x = ww - TOP_H * 4.0;
    let over_gear = mx >= gear_slot_x && mx < min_x && my >= 0.0 && my <= TOP_H;
    if over_gear && ctx.input.button_just_pressed(MouseButton::Left) {
        *open_settings = !*open_settings;
    }

    // ── Drag de la barra ─────────────────────────────────────────────────────
    // excluye los 4 slots de botones (cerrar+maximizar+minimizar+gear)
    let drag_area_w = ww - TOP_H * 4.0;
    let over_drag = mx >= 0.0
        && mx < drag_area_w
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

// ── Draw ──────────────────────────────────────────────────────────────────────

pub fn draw(
    dc: &mut DrawContext<'_, '_>,
    zoom: f32,
    is_maximized: bool,
    icon_close: Option<Arc<Texture2d>>,
    icon_minimize: Option<Arc<Texture2d>>,
    icon_restore: Option<Arc<Texture2d>>,
    _show_settings: bool,
) {
    let (win_w, _) = dc.ctx.window_size;
    let ww = win_w as f32;
    let canvas_x = crate::LEFT_W;
    let (mx, my) = dc.ctx.input.mouse_pos_f32();

    dc.gui.rect(0.0, 0.0, ww, TOP_H, c_top());
    // Línea de acento inferior: azul VSCode #007ACC, 2px
    dc.gui.rect(
        0.0,
        TOP_H - 2.0,
        ww,
        2.0,
        ferrous_app::Color::hex("#007ACC").to_linear_f32(),
    );

    // texto / iconos usan este color por defecto
    let text_color = ferrous_app::Color::hex("#CCCCCC").to_linear_f32();
    // Logo / título — blanco VSCode
    dc.gui
        .draw_text(dc.font, "GUIMaker", [14.0, 12.0], 15.0, text_color);

    // Indicador de zoom — azul acento VSCode
    let zoom_label = format!("zoom  {:.0}%", zoom * 100.0);
    dc.gui.draw_text(
        dc.font,
        &zoom_label,
        [canvas_x + 8.0, 13.0],
        11.0,
        ferrous_app::Color::hex("#3DC9B0").to_linear_f32(),
    );

    // ── Botón Config ───────────────────────────────────────────
    // 4° slot desde la derecha, igual que los botones de ventana
    let min_x = ww - TOP_H * 3.0;
    let gear_slot_x = ww - TOP_H * 4.0;
    let over_gear = mx >= gear_slot_x && mx < min_x && my >= 0.0 && my <= TOP_H;
    let c_gear = if over_gear {
        ferrous_app::Color::hex("#505050").to_linear_f32()
    } else {
        c_top()
    };
    dc.gui.rect(gear_slot_x, 0.0, TOP_H, TOP_H, c_gear);
    let gear_color = if over_gear {
        ferrous_app::Color::hex("#3DC9B0").to_linear_f32()
    } else {
        text_color
    };
    dc.gui.draw_text(
        dc.font,
        "Config",
        [gear_slot_x + 4.0, 13.0],
        10.0,
        gear_color,
    );

    // ── Cerrar [×] ───────────────────────────────────────────────────────────
    let close_x = ww - TOP_H;
    let hover_close = mx >= close_x && mx <= ww && my >= 0.0 && my <= TOP_H;
    // botón de cerrar: fondo rojo en hover, normal es mismo color que la barra
    let c_close = if hover_close {
        ferrous_app::Color::hex("#F14C4C").to_linear_f32()
    } else {
        c_top()
    };
    dc.gui.rect_r(close_x, 0.0, TOP_H, TOP_H, c_close, 0.0);
    let icon_sz = TOP_H * 0.45;
    let icon_off = (TOP_H - icon_sz) * 0.5;
    if let Some(tex) = icon_close {
        // icono se tinea con texto normal, excepto cuando estamos en hover
        let icon_color = if hover_close {
            // conservamos la tonalidad rosada anterior
            [1.0, 0.70, 0.78, 1.0]
        } else {
            text_color
        };
        dc.gui.image(
            close_x + icon_off,
            icon_off,
            icon_sz,
            icon_sz,
            tex,
            [0.0, 0.0],
            [1.0, 1.0],
            icon_color,
        );
    } else {
        dc.gui.draw_text(
            dc.font,
            "×",
            [close_x + TOP_H * 0.28, 9.0],
            18.0,
            [1.0, 0.70, 0.78, 1.0],
        );
    }

    // ── Maximizar [□ / ❐] ────────────────────────────────────────────────────
    let max_x = ww - TOP_H * 2.0;
    let hover_max = mx >= max_x && mx < close_x && my >= 0.0 && my <= TOP_H;
    let c_max = if hover_max {
        ferrous_app::Color::hex("#505050").to_linear_f32()
    } else {
        c_top()
    };
    dc.gui.rect(max_x, 0.0, TOP_H, TOP_H, c_max);
    // restore.svg sirve tanto para restaurar (maximizado) como para maximizar
    if let Some(tex) = icon_restore {
        // dibujo normal, sin hover especial (solo cambia el símbolo textual)
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
            ferrous_app::Color::hex("#CCCCCC").to_linear_f32(),
        );
    }

    // ── Minimizar [–] ────────────────────────────────────────────────────────
    let min_x = ww - TOP_H * 3.0;
    let hover_min = mx >= min_x && mx < max_x && my >= 0.0 && my <= TOP_H;
    let c_min = if hover_min {
        ferrous_app::Color::hex("#505050").to_linear_f32()
    } else {
        c_top()
    };
    dc.gui.rect(min_x, 0.0, TOP_H, TOP_H, c_min);
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
            ferrous_app::Color::hex("#CCCCCC").to_linear_f32(),
        );
    }
}
