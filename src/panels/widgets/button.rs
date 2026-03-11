// ── Button widget ─────────────────────────────────────────────────────────────

use ferrous_app::{AppContext, DrawContext, MouseButton};
use ferrous_gui::GuiBatch;
use ferrous_ui_core::Rect;

use super::shared::{
    default_row_colors, draw_pivot_overlay, draw_text_row, parse_hex_or, LABEL_W, ROW_H, ROW_PAD_X,
};
use crate::scene::{PlacedWidget, SceneState};

// ── Canvas preview ────────────────────────────────────────────────────────────

pub fn draw_canvas(
    dc: &mut DrawContext<'_, '_>,
    w: &PlacedWidget,
    sx: f32,
    sy: f32,
    sw: f32,
    sh: f32,
    is_selected: bool,
    active_radius_corner: Option<usize>,
) {
    let bg_color = parse_hex_or(&w.props.bg_color_hex, [0.424, 0.388, 1.0, 1.0]);
    let text_color = parse_hex_or(&w.props.text_color_hex, [1.0, 1.0, 1.0, 1.0]);

    let zoom = sw / w.w;
    let half = (sw.min(sh)) * 0.5;
    let radii = [
        (w.props.border_radii[0] * zoom).min(half),
        (w.props.border_radii[1] * zoom).min(half),
        (w.props.border_radii[2] * zoom).min(half),
        (w.props.border_radii[3] * zoom).min(half),
    ];

    dc.gui.rect_radii(sx, sy, sw, sh, bg_color, radii);

    let label = if w.props.label.is_empty() {
        "Button"
    } else {
        &w.props.label
    };
    let font_size = (w.props.font_size * zoom).clamp(8.0, sh * 0.7);
    dc.gui
        .push_clip(Rect::new(sx + 2.0, sy + 2.0, sw - 4.0, sh - 4.0));
    let text_w = GuiBatch::measure_text(dc.font, label, font_size);
    let align = w.props.text_align();
    let text_x = align.resolve_x(sx, sw, text_w, 8.0);
    let text_y = align.resolve_y(sy, sh, font_size, 4.0);
    dc.gui
        .draw_text(dc.font, label, [text_x, text_y], font_size, text_color);
    dc.gui.pop_clip();

    if is_selected {
        let bw = 2.0_f32;
        let sel_col = [1.0_f32, 0.8, 0.0, 1.0];
        dc.gui.rect_radii(
            sx - bw,
            sy - bw,
            sw + bw * 2.0,
            sh + bw * 2.0,
            sel_col,
            [radii[0] + bw, radii[1] + bw, radii[2] + bw, radii[3] + bw],
        );
        // Re-draw fill on top
        dc.gui.rect_radii(sx, sy, sw, sh, bg_color, radii);
        dc.gui
            .push_clip(Rect::new(sx + 2.0, sy + 2.0, sw - 4.0, sh - 4.0));
        dc.gui
            .draw_text(dc.font, label, [text_x, text_y], font_size, text_color);
        dc.gui.pop_clip();

        // Radius drag handles
        let rh_size = 8.0_f32;
        let rh_inner = 4.0_f32;
        let rh_col: [f32; 4] = [0.0, 0.85, 0.85, 1.0];
        let rh_col_active: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
        let rh_fill: [f32; 4] = [0.08, 0.08, 0.12, 1.0];
        let corner_names = ["TL", "TR", "BL", "BR"];
        for corner in 0..4_usize {
            let (hx, hy) = radius_handle_pos(sx, sy, sw, sh, &radii, corner);
            let is_active = active_radius_corner == Some(corner);
            let outer = if is_active { rh_size + 3.0 } else { rh_size };
            let col = if is_active { rh_col_active } else { rh_col };
            dc.gui
                .rect(hx - outer * 0.5, hy - outer * 0.5, outer, outer, col);
            dc.gui.rect(
                hx - rh_inner * 0.5,
                hy - rh_inner * 0.5,
                rh_inner,
                rh_inner,
                rh_fill,
            );
            if is_active {
                let lbl = format!(
                    "{}: {:.0}px",
                    corner_names[corner], w.props.border_radii[corner]
                );
                dc.gui
                    .draw_text(dc.font, &lbl, [hx + 7.0, hy - 7.0], 10.0, col);
            }
        }
        draw_pivot_overlay(dc, w, sx, sy, sw, sh);
    }
}

/// Screen-space position of the radius drag handle for corner `i`
/// (0=TL, 1=TR, 2=BL, 3=BR).
pub fn radius_handle_pos(
    sx: f32,
    sy: f32,
    sw: f32,
    sh: f32,
    radii: &[f32; 4],
    corner: usize,
) -> (f32, f32) {
    let frac = std::f32::consts::FRAC_1_SQRT_2;
    let r = radii[corner];
    match corner {
        0 => (sx + r * frac, sy + r * frac),
        1 => (sx + sw - r * frac, sy + r * frac),
        2 => (sx + r * frac, sy + sh - r * frac),
        _ => (sx + sw - r * frac, sy + sh - r * frac),
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
    let (tc, bc, brc, sc) = default_row_colors("Label", label_focused);
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
    let (tc, bc, brc, sc) = default_row_colors("Size", size_focused);
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

    // Border radii (4 corners)
    let radii_snap = selected.props.border_radii;
    let corner_labels = ["R: TL", "R: TR", "R: BL", "R: BR"];
    let corner_keys = ["r_tl", "r_tr", "r_bl", "r_br"];
    for i in 0..4 {
        let focused = scene.editing_field.as_deref() == Some(corner_keys[i]);
        let buf = format!("{:.0}", radii_snap[i]);
        let val: &str = if focused { &scene.edit_buffer } else { &buf };
        let (tc, bc, brc, sc) = default_row_colors(corner_labels[i], focused);
        draw_text_row(
            dc,
            panel_x,
            *cy,
            right_w,
            corner_labels[i],
            val,
            focused,
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
    }

    // Re-borrow to get fresh data (mutable gui ops above)
    let selected = match scene.selected() {
        Some(s) => s,
        None => return,
    };
    let (mx, my) = dc.ctx.input.mouse_pos_f32();
    let clicked = dc.ctx.input.button_just_pressed(MouseButton::Left);

    // Bg Color
    let bg_focused = scene.editing_field.as_deref() == Some("bg_color");
    let bg_buf = selected.props.bg_color_hex.clone();
    let bg_value: &str = if bg_focused {
        &scene.edit_buffer
    } else {
        &bg_buf
    };
    let (tc, bc, brc, sc) = default_row_colors("Bg Color", bg_focused);
    draw_text_row(
        dc,
        panel_x,
        *cy,
        right_w,
        "Bg Color",
        bg_value,
        bg_focused,
        scene.edit_state.cursor_visible,
        scene.edit_state.cursor_pos,
        scene.edit_state.selection(),
        scene.edit_state.all_selected,
        tc,
        bc,
        brc,
        sc,
    );
    let swatch_x = panel_x + right_w - ROW_PAD_X - (ROW_H - 8.0);
    dc.gui.rect(
        swatch_x,
        *cy + 4.0,
        ROW_H - 8.0,
        ROW_H - 8.0,
        parse_hex_or(&selected.props.bg_color_hex, [0.424, 0.388, 1.0, 1.0]),
    );
    *cy += ROW_H;

    // Text Color
    let tc_focused = scene.editing_field.as_deref() == Some("text_color");
    let tc_buf = selected.props.text_color_hex.clone();
    let tc_value: &str = if tc_focused {
        &scene.edit_buffer
    } else {
        &tc_buf
    };
    let (tcc, bcc, brcc, scc) = default_row_colors("Txt Color", tc_focused);
    draw_text_row(
        dc,
        panel_x,
        *cy,
        right_w,
        "Txt Color",
        tc_value,
        tc_focused,
        scene.edit_state.cursor_visible,
        scene.edit_state.cursor_pos,
        scene.edit_state.selection(),
        scene.edit_state.all_selected,
        tcc,
        bcc,
        brcc,
        scc,
    );
    let swatch_x2 = panel_x + right_w - ROW_PAD_X - (ROW_H - 8.0);
    dc.gui.rect(
        swatch_x2,
        *cy + 4.0,
        ROW_H - 8.0,
        ROW_H - 8.0,
        parse_hex_or(&selected.props.text_color_hex, [1.0, 1.0, 1.0, 1.0]),
    );
    *cy += ROW_H;

    // Label alignment
    super::label_align::draw(dc, scene, panel_x, right_w, cy, selected_id);
}

// ── Inspector: update ─────────────────────────────────────────────────────────

pub fn update_props(
    ctx: &mut AppContext,
    scene: &mut SceneState,
    _panel_x: f32,
    right_w: f32,
    _mx: f32,
    my: f32,
    hit_x: f32,
    cy: &mut f32,
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
        focus_text(scene, selected_id, "label", |w| w.props.label.clone());
        return;
    }
    *cy += ROW_H;

    // Font size
    if my >= *cy && my < *cy + ROW_H && hit_x >= val_x {
        focus_text(scene, selected_id, "font_size", |w| {
            format!("{:.0}", w.props.font_size)
        });
        return;
    }
    *cy += ROW_H;

    // 4 corner radii
    let corner_keys = ["r_tl", "r_tr", "r_bl", "r_br"];
    let corner_idx = [0usize, 1, 2, 3];
    for (key, idx) in corner_keys.iter().zip(corner_idx.iter()) {
        if my >= *cy && my < *cy + ROW_H && hit_x >= val_x {
            let i = *idx;
            focus_text(scene, selected_id, key, move |w| {
                format!("{:.0}", w.props.border_radii[i])
            });
            return;
        }
        *cy += ROW_H;
    }

    // Bg Color
    if my >= *cy && my < *cy + ROW_H && hit_x >= val_x {
        focus_text(scene, selected_id, "bg_color", |w| {
            w.props.bg_color_hex.clone()
        });
        return;
    }
    *cy += ROW_H;

    // Text Color
    if my >= *cy && my < *cy + ROW_H && hit_x >= val_x {
        focus_text(scene, selected_id, "text_color", |w| {
            w.props.text_color_hex.clone()
        });
        return;
    }
    *cy += ROW_H;

    super::label_align::update(scene, right_w, _mx, my, hit_x, cy, selected_id);
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
            "bg_color" => {
                w.props.bg_color_hex = buf;
                return true;
            }
            "text_color" => {
                w.props.text_color_hex = buf;
                return true;
            }
            "r_tl" => {
                if let Ok(v) = buf.trim().parse::<f32>() {
                    w.props.border_radii[0] = v.max(0.0);
                }
                return true;
            }
            "r_tr" => {
                if let Ok(v) = buf.trim().parse::<f32>() {
                    w.props.border_radii[1] = v.max(0.0);
                }
                return true;
            }
            "r_bl" => {
                if let Ok(v) = buf.trim().parse::<f32>() {
                    w.props.border_radii[2] = v.max(0.0);
                }
                return true;
            }
            "r_br" => {
                if let Ok(v) = buf.trim().parse::<f32>() {
                    w.props.border_radii[3] = v.max(0.0);
                }
                return true;
            }
            _ => {}
        }
    }
    false
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn focus_text(
    scene: &mut SceneState,
    selected_id: u32,
    key: &str,
    value_fn: impl Fn(&PlacedWidget) -> String,
) {
    let current = scene
        .widgets
        .iter()
        .find(|w| w.id == selected_id)
        .map(value_fn)
        .unwrap_or_default();
    scene.editing_field = Some(key.to_string());
    scene.edit_buffer = current;
    scene.edit_state.focus();
    scene.edit_state.cursor_pos = scene.edit_buffer.len();
}
