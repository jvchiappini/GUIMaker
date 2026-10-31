// ── Label widget ──────────────────────────────────────────────────────────────

use ferrous_engine::{AppContext, DrawContext, MouseButton};
use ferrous_gui::GuiBatch;
use ferrous_ui_core::Rect;

use super::shared::{
    default_row_colors, draw_pivot_overlay, draw_text_row, LABEL_W, ROW_H, ROW_PAD_X,
};
use crate::scene::{PlacedWidget, SceneState};
use crate::theme::{Theme, ThemeRegistry};

// ── Canvas preview ────────────────────────────────────────────────────────────

pub fn draw_canvas(
    dc: &mut DrawContext<'_, '_>,
    w: &PlacedWidget,
    sx: f32,
    sy: f32,
    sw: f32,
    sh: f32,
    is_selected: bool,
    theme: &Theme,
) {
    let zoom = sw / w.w;
    let label_text = if w.props.label.is_empty() {
        "Label"
    } else {
        &w.props.label
    };
    let font_size = (w.props.font_size * zoom).max(4.0);

    let text_w = GuiBatch::measure_text(dc.font, label_text, font_size);
    let align = w.props.text_align();
    let text_x = align.resolve_x(sx, sw, text_w, 4.0);
    let text_y = align.resolve_y(sy, sh, font_size, 4.0);

    dc.gui.push_clip(Rect::new(sx, sy, sw, sh));
    dc.gui.draw_text(
        dc.font,
        label_text,
        [text_x, text_y],
        font_size,
        super::shared::parse_hex_or(&w.props.color_hex, [1.0, 1.0, 1.0, 1.0]),
    );
    dc.gui.pop_clip();

    if is_selected {
        let bw = 1.5_f32;
        let bh = bw * 2.0;
        let mut sel_col = ThemeRegistry::hex_to_f32(&theme.colors.accent);
        sel_col[3] = 0.85;
        dc.gui.rect(sx, sy, sw, bw, sel_col);
        dc.gui.rect(sx, sy + sh - bw, sw, bw, sel_col);
        dc.gui.rect(sx, sy, bw, sh, sel_col);
        dc.gui.rect(sx + sw - bw, sy, bw, sh, sel_col);
        draw_pivot_overlay(dc, w, sx, sy, sw, sh);
    }
}

// ── Inspector: draw ───────────────────────────────────────────────────────────

pub fn draw_props(
    dc: &mut DrawContext<'_, '_>,
    scene: &mut SceneState,
    panel_x: f32,
    right_w: f32,
    cy: &mut f32,
    selected_id: u32,
    mx: f32,
    my: f32,
    theme: &Theme,
) {
    let selected = match scene.selected() {
        Some(s) => s,
        None => return,
    };

    // Label
    let label_focused = scene.editing_field.as_deref() == Some("label");
    let label_value: &str = if label_focused {
        &scene.edit_buffer
    } else {
        &selected.props.label
    };
    let (tc, bc, brc, sc) = default_row_colors("Label", label_focused, theme);
    draw_text_row(
        dc,
        panel_x,
        *cy,
        right_w,
        "Label",
        label_value,
        label_focused,
        scene.edit_state.cursor_visible,
        scene.edit_state.cursor_pos,
        scene.edit_state.selection(),
        scene.edit_state.all_selected,
        tc,
        bc,
        brc,
        sc,
    );
    *cy += ROW_H;

    // Font size
    let size_focused = scene.editing_field.as_deref() == Some("font_size");
    let size_buf = format!("{:.0}", selected.props.font_size);
    let size_value: &str = if size_focused {
        &scene.edit_buffer
    } else {
        &size_buf
    };
    let (tc, bc, brc, sc) = default_row_colors("Size", size_focused, theme);
    draw_text_row(
        dc,
        panel_x,
        *cy,
        right_w,
        "Size",
        size_value,
        size_focused,
        scene.edit_state.cursor_visible,
        scene.edit_state.cursor_pos,
        scene.edit_state.selection(),
        scene.edit_state.all_selected,
        tc,
        bc,
        brc,
        sc,
    );
    *cy += ROW_H;

    // Color
    let hex = &selected.props.color_hex;
    super::shared::draw_color_row(dc, panel_x, *cy, right_w, "Color", hex, theme);
    *cy += ROW_H;
    
    // Quick Color Palette
    let colors = [
        "#FFFFFF", "#000000", "#FF0000", "#00FF00", "#0000FF", 
        "#FFFF00", "#FF00FF", "#00FFFF", "#FF8C00", "#6C63FF"
    ];
    let chip_w = (right_w - 20.0) / colors.len() as f32;
    for (i, hex) in colors.iter().enumerate() {
        let chip_x = panel_x + 10.0 + i as f32 * chip_w;
        let is_over = mx >= chip_x && mx < chip_x + chip_w - 2.0 && my >= *cy && my < *cy + 16.0;
        let color = super::shared::parse_hex_or(hex, [1.0, 1.0, 1.0, 1.0]);
        dc.gui.rect(chip_x, *cy, chip_w - 2.0, 16.0, color);
        if is_over {
            dc.gui.rect(chip_x, *cy, chip_w - 2.0, 1.0, [1.0, 1.0, 1.0, 1.0]);
            dc.gui.rect(chip_x, *cy + 15.0, chip_w - 2.0, 1.0, [1.0, 1.0, 1.0, 1.0]);
        }
    }
    *cy += 20.0;

    // Label alignment section
    super::label_align::draw(dc, scene, panel_x, right_w, cy, selected_id, theme);
}

pub fn update_props(
    ctx: &mut AppContext,
    scene: &mut SceneState,
    panel_x: f32,
    right_w: f32,
    mx: f32,
    my: f32,
    hit_x: f32,
    cy: &mut f32,
    cp: &mut crate::color_picker::ColorPickerState,
) {
    let val_x = LABEL_W + ROW_PAD_X;
    let clicked = ctx.input.button_just_pressed(MouseButton::Left);
    if !clicked {
        return;
    }

    let Some(selected_id) = scene.selected_id else {
        return;
    };

    // Label
    if my >= *cy && my < *cy + ROW_H && hit_x >= val_x {
        let current = scene
            .widgets
            .iter()
            .find(|w| w.id == selected_id)
            .map(|w| w.props.label.clone())
            .unwrap_or_default();
        scene.editing_field = Some("label".to_string());
        scene.edit_buffer = current;
        scene.edit_state.focus();
        scene.edit_state.cursor_pos = scene.edit_buffer.len();
        return;
    }
    *cy += ROW_H;

    // Font size
    if my >= *cy && my < *cy + ROW_H && hit_x >= val_x {
        let current = scene
            .widgets
            .iter()
            .find(|w| w.id == selected_id)
            .map(|w| format!("{:.0}", w.props.font_size))
            .unwrap_or_default();
        scene.editing_field = Some("font_size".to_string());
        scene.edit_buffer = current;
        scene.edit_state.focus();
        scene.edit_state.cursor_pos = scene.edit_buffer.len();
        return;
    }
    *cy += ROW_H;

    // Color
    if my >= *cy && my < *cy + ROW_H && hit_x >= val_x {
        let current = scene.selected().map(|w| w.props.color_hex.clone()).unwrap_or_default();
        cp.active = true;
        cp.target_field = "color_hex".to_string(); // we'll map "color" to "color_hex" for Label
        cp.widget_id = Some(selected_id);
        cp.screen_pos = (mx, my);
        cp.current_color = super::shared::parse_hex_or(&current, [1.0, 1.0, 1.0, 1.0]);
        cp.alpha = 1.0;
        return;
    }
    *cy += ROW_H;

    // Color Palette clicks
    let colors = [
        "#FFFFFF", "#000000", "#FF0000", "#00FF00", "#000000", 
        "#FFFF00", "#FF00FF", "#00FFFF", "#FF8C00", "#6C63FF"
    ];
    let chip_w = (right_w - 20.0) / colors.len() as f32;
    for (i, hex) in colors.iter().enumerate() {
        let chip_x = panel_x + 10.0 + i as f32 * chip_w;
        if my >= *cy && my < *cy + 16.0 && mx >= chip_x && mx < chip_x + chip_w - 2.0 {
            if let Some(w) = scene.widgets.iter_mut().find(|w| w.id == selected_id) {
                w.props.color_hex = hex.to_string();
                scene.sync_prop_strings();
                return;
            }
        }
    }
    *cy += 20.0;
    *cy += ROW_H;

    super::label_align::update(scene, right_w, mx, my, hit_x, cy, selected_id);
}

// ── Commit ────────────────────────────────────────────────────────────────────

pub fn commit_field(scene: &mut SceneState, field: &str, buf: String) -> bool {
    if let Some(w) = scene.selected_mut() {
        match field {
            "label" => {
                w.props.label = buf;
                return true;
            }
            "font_size" => {
                if let Ok(v) = buf.trim().parse::<f32>() {
                    w.props.font_size = v.max(1.0);
                }
                return true;
            }
            "color" => {
                w.props.color_hex = buf;
                return true;
            }
            _ => {}
        }
    }
    false
}
