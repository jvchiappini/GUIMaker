// ── Button widget ─────────────────────────────────────────────────────────────

use ferrous_engine::{AppContext, DrawContext, MouseButton};
use ferrous_gui::GuiBatch;
use ferrous_ui_core::Rect;

use super::shared::{
    default_row_colors, draw_gradient_rect, draw_pivot_overlay, draw_text_row, parse_hex_or,
    LABEL_W, ROW_H, ROW_PAD_X,
};
use crate::scene::{GradientKind, PlacedWidget, SceneState};
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
    active_radius_corner: Option<usize>,
    theme: &Theme,
) {
    let mut bg_color = parse_hex_or(&w.props.bg_color_hex, [0.424, 0.388, 1.0, 1.0]);
    bg_color[3] *= w.props.opacity;
    let text_color = parse_hex_or(&w.props.text_color_hex, [1.0, 1.0, 1.0, 1.0]);

    let zoom = sw / w.w;
    let half = (sw.min(sh)) * 0.5;
    let radii = [
        (w.props.border_radii[0] * zoom).min(half),
        (w.props.border_radii[1] * zoom).min(half),
        (w.props.border_radii[2] * zoom).min(half),
        (w.props.border_radii[3] * zoom).min(half),
    ];

    // Draw solid base
    dc.gui.rect_radii(sx, sy, sw, sh, bg_color, radii);

    // Draw gradient on top (clipped to button bounds)
    let bg = w.props.to_background();
    if !matches!(bg, ferrous_ui_core::Background::None) {
        draw_gradient_rect(dc, &bg, sx, sy, sw, sh, radii, 128);
    }

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
        let mut sel_col = ThemeRegistry::hex_to_f32(&theme.colors.accent);
        sel_col[3] = 1.0;
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
        if !matches!(bg, ferrous_ui_core::Background::None) {
            draw_gradient_rect(dc, &bg, sx, sy, sw, sh, radii, 128);
        }
        dc.gui
            .push_clip(Rect::new(sx + 2.0, sy + 2.0, sw - 4.0, sh - 4.0));
        dc.gui
            .draw_text(dc.font, label, [text_x, text_y], font_size, text_color);
        dc.gui.pop_clip();

        // Radius drag handles
        let rh_size = 8.0_f32;
        let rh_inner = 4.0_f32;
        let rh_col = ThemeRegistry::hex_to_f32(&theme.colors.accent);
        let rh_col_active = ThemeRegistry::hex_to_f32(&theme.colors.text);
        let rh_fill = ThemeRegistry::hex_to_f32(&theme.colors.surface);
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

    // Border radii (4 corners)
    let radii_snap = selected.props.border_radii;
    let corner_labels = ["R: TL", "R: TR", "R: BL", "R: BR"];
    let corner_keys = ["r_tl", "r_tr", "r_bl", "r_br"];
    for i in 0..4 {
        let focused = scene.editing_field.as_deref() == Some(corner_keys[i]);
        let buf = format!("{:.0}", radii_snap[i]);
        let val: &str = if focused { &scene.edit_buffer } else { &buf };
        let (tc, bc, brc, sc) = default_row_colors(corner_labels[i], focused, theme);
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
    let bg_hex = &selected.props.bg_color_hex;
    super::shared::draw_color_row(dc, panel_x, *cy, right_w, "Bg Color", bg_hex, theme);
    *cy += ROW_H;

    // Text Color
    let tc_hex = &selected.props.text_color_hex;
    super::shared::draw_color_row(dc, panel_x, *cy, right_w, "Txt Color", tc_hex, theme);
    *cy += ROW_H;

    // ── Gradient section ──────────────────────────────────────────────────
    super::shared::draw_section(dc, panel_x, *cy, right_w, "GRADIENT", theme);
    *cy += super::shared::SECTION_H;

    // Gradient type button (cycles through Solid / Linear / Radial / Conic)
    let grad_label = selected.props.gradient_type.label();
    let btn_col = ThemeRegistry::hex_to_f32(&theme.colors.surface);
    let txt_col = ThemeRegistry::hex_to_f32(&theme.colors.text_dim);
    dc.gui.rect(
        panel_x + ROW_PAD_X,
        *cy + 3.0,
        right_w - ROW_PAD_X * 2.0,
        ROW_H - 6.0,
        btn_col,
    );
    dc.gui.draw_text(
        dc.font,
        &format!("Type: {}", grad_label),
        [panel_x + ROW_PAD_X + 4.0, *cy + 7.0],
        10.0,
        txt_col,
    );
    *cy += ROW_H;

    // Only show stop / angle rows when a gradient type is active
    let has_grad = selected.props.gradient_type != GradientKind::None;

    // Stops
    let stops_snap: Vec<(f32, String)> = selected.props.gradient_stops.clone();
    for (i, (pos, hex)) in stops_snap.iter().enumerate() {
        let pos_key = format!("stop_pos_{i}");
        let col_key = format!("stop_col_{i}");

        // Position
        let pos_focused = scene.editing_field.as_deref() == Some(&pos_key);
        let pos_buf = format!("{:.2}", pos);
        let pos_val: &str = if pos_focused {
            &scene.edit_buffer
        } else {
            &pos_buf
        };
        let pos_lbl = format!("S{i} Pos");
        let (tc, bc, brc, sc) = default_row_colors(&pos_lbl, pos_focused, theme);
        draw_text_row(
            dc,
            panel_x,
            *cy,
            right_w,
            &pos_lbl,
            pos_val,
            pos_focused,
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

        // Colour hex
        let col_focused = scene.editing_field.as_deref() == Some(&col_key);
        let col_val: &str = if col_focused { &scene.edit_buffer } else { hex };
        let col_lbl = format!("S{i} Col");
        let (tc, bc, brc, sc) = default_row_colors(&col_lbl, col_focused, theme);
        draw_text_row(
            dc,
            panel_x,
            *cy,
            right_w,
            &col_lbl,
            col_val,
            col_focused,
            scene.edit_state.cursor_visible,
            scene.edit_state.cursor_pos,
            scene.edit_state.selection(),
            scene.edit_state.all_selected,
            tc,
            bc,
            brc,
            sc,
        );
        // Colour swatch
        let sw_x = panel_x + right_w - ROW_PAD_X - (ROW_H - 8.0);
        dc.gui.rect(
            sw_x,
            *cy + 4.0,
            ROW_H - 8.0,
            ROW_H - 8.0,
            parse_hex_or(hex, [1.0, 0.0, 1.0, 1.0]),
        );
        *cy += ROW_H;
    }

    // Add / Remove stop buttons
    {
        let half_w = (right_w - ROW_PAD_X * 2.0) * 0.5 - 2.0;
        let add_col = ThemeRegistry::hex_to_f32(&theme.colors.success);
        let rem_col = ThemeRegistry::hex_to_f32(&theme.colors.error);
        let lbl_col = ThemeRegistry::hex_to_f32(&theme.colors.text);
        dc.gui
            .rect(panel_x + ROW_PAD_X, *cy + 3.0, half_w, ROW_H - 6.0, add_col);
        dc.gui.draw_text(
            dc.font,
            "+ Stop",
            [panel_x + ROW_PAD_X + 4.0, *cy + 7.0],
            10.0,
            lbl_col,
        );
        dc.gui.rect(
            panel_x + ROW_PAD_X + half_w + 4.0,
            *cy + 3.0,
            half_w,
            ROW_H - 6.0,
            rem_col,
        );
        dc.gui.draw_text(
            dc.font,
            "- Stop",
            [panel_x + ROW_PAD_X + half_w + 8.0, *cy + 7.0],
            10.0,
            lbl_col,
        );
    }
    *cy += ROW_H;

    // Angle (linear only)
    if has_grad && selected.props.gradient_type == GradientKind::Linear {
        let ang_focused = scene.editing_field.as_deref() == Some("grad_angle");
        let ang_buf = format!("{:.0}", selected.props.gradient_angle);
        let ang_val: &str = if ang_focused {
            &scene.edit_buffer
        } else {
            &ang_buf
        };
        let (tc, bc, brc, sc) = default_row_colors("Angle", ang_focused, theme);
        draw_text_row(
            dc,
            panel_x,
            *cy,
            right_w,
            "Angle",
            ang_val,
            ang_focused,
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

    // Center X / Y (radial & conic)
    if has_grad
        && matches!(
            selected.props.gradient_type,
            GradientKind::Radial | GradientKind::Conic
        )
    {
        for (lbl, key) in [("Ctr X", "grad_cx"), ("Ctr Y", "grad_cy")] {
            let focused = scene.editing_field.as_deref() == Some(key);
            let idx = if key == "grad_cx" { 0 } else { 1 };
            let buf = format!("{:.2}", selected.props.gradient_center[idx]);
            let val: &str = if focused { &scene.edit_buffer } else { &buf };
            let (tc, bc, brc, sc) = default_row_colors(lbl, focused, theme);
            draw_text_row(
                dc,
                panel_x,
                *cy,
                right_w,
                lbl,
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
    }

    // Radius (radial only)
    if has_grad && selected.props.gradient_type == GradientKind::Radial {
        let rad_focused = scene.editing_field.as_deref() == Some("grad_radius");
        let rad_buf = format!("{:.2}", selected.props.gradient_radius);
        let rad_val: &str = if rad_focused {
            &scene.edit_buffer
        } else {
            &rad_buf
        };
        let (tc, bc, brc, sc) = default_row_colors("Radius", rad_focused, theme);
        draw_text_row(
            dc,
            panel_x,
            *cy,
            right_w,
            "Radius",
            rad_val,
            rad_focused,
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

    // Start angle (conic only)
    if has_grad && selected.props.gradient_type == GradientKind::Conic {
        let sa_focused = scene.editing_field.as_deref() == Some("grad_start_angle");
        let sa_buf = format!("{:.0}", selected.props.gradient_start_angle);
        let sa_val: &str = if sa_focused {
            &scene.edit_buffer
        } else {
            &sa_buf
        };
        let (tc, bc, brc, sc) = default_row_colors("Start Ang", sa_focused, theme);
        draw_text_row(
            dc,
            panel_x,
            *cy,
            right_w,
            "Start Ang",
            sa_val,
            sa_focused,
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

    // Label alignment
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
    if my >= *cy && my < *cy + ROW_H && hit_x >= val_x {
        let current = scene.selected().map(|w| w.props.bg_color_hex.clone()).unwrap_or_default();
        cp.active = true;
        cp.target_field = "bg_color".to_string();
        cp.widget_id = Some(selected_id);
        cp.screen_pos = (mx, my);
        cp.current_color = parse_hex_or(&current, [1.0, 1.0, 1.0, 1.0]);
            cp.alpha = scene.selected().map(|w| w.props.opacity).unwrap_or(1.0);
        return;
    }
    *cy += ROW_H;

    // Text Color
    if my >= *cy && my < *cy + ROW_H && hit_x >= val_x {
        let current = scene.selected().map(|w| w.props.text_color_hex.clone()).unwrap_or_default();
        cp.active = true;
        cp.target_field = "text_color".to_string();
        cp.widget_id = Some(selected_id);
        cp.screen_pos = (mx, my);
        cp.current_color = parse_hex_or(&current, [1.0, 1.0, 1.0, 1.0]);
            cp.alpha = 1.0;
        return;
    }
    *cy += ROW_H;

    // ── Gradient section ──────────────────────────────────────────────────
    *cy += super::shared::SECTION_H; // section header

    // Type toggle button
    if my >= *cy && my < *cy + ROW_H {
        if let Some(w) = scene.selected_mut() {
            w.props.gradient_type = w.props.gradient_type.cycle_next();
        }
        return;
    }
    *cy += ROW_H;

    let stops_len = scene
        .selected()
        .map(|w| w.props.gradient_stops.len())
        .unwrap_or(0);
    let grad_type = scene
        .selected()
        .map(|w| w.props.gradient_type.clone())
        .unwrap_or(GradientKind::None);

    // Stop rows (pos + col per stop)
    for i in 0..stops_len {
        // Pos
        if my >= *cy && my < *cy + ROW_H && hit_x >= val_x {
            let key = format!("stop_pos_{i}");
            focus_text(scene, selected_id, &key, move |w| {
                format!(
                    "{:.2}",
                    w.props.gradient_stops.get(i).map(|s| s.0).unwrap_or(0.0)
                )
            });
            return;
        }
        *cy += ROW_H;
        // Col
        if my >= *cy && my < *cy + ROW_H && hit_x >= val_x {
            let current = scene.selected()
                .map(|w| w.props.gradient_stops.get(i).map(|s| s.1.clone()).unwrap_or_default())
                .unwrap_or_default();
            cp.active = true;
            cp.target_field = format!("stop_col_{i}");
            cp.widget_id = Some(selected_id);
            cp.screen_pos = (mx, my);
            cp.current_color = parse_hex_or(&current, [1.0, 1.0, 1.0, 1.0]);
            return;
        }
        *cy += ROW_H;
    }

    // Add / Remove stop buttons
    if my >= *cy && my < *cy + ROW_H {
        let half_w = (right_w - ROW_PAD_X * 2.0) * 0.5 - 2.0;
        let add_btn_x = panel_x + ROW_PAD_X;
        let rem_btn_x = panel_x + ROW_PAD_X + half_w + 4.0;
        if hit_x >= add_btn_x && hit_x < add_btn_x + half_w {
            if let Some(w) = scene.selected_mut() {
                let new_pos = if w.props.gradient_stops.len() > 1 {
                    let last = w.props.gradient_stops.last().unwrap().0;
                    (last + (1.0 - last) * 0.5).min(1.0)
                } else {
                    1.0
                };
                w.props
                    .gradient_stops
                    .push((new_pos, "#FFFFFF".to_string()));
            }
            return;
        } else if hit_x >= rem_btn_x {
            if let Some(w) = scene.selected_mut() {
                if w.props.gradient_stops.len() > 2 {
                    w.props.gradient_stops.pop();
                }
            }
            return;
        }
    }
    *cy += ROW_H;

    // Angle (linear)
    if grad_type == GradientKind::Linear {
        if my >= *cy && my < *cy + ROW_H && hit_x >= val_x {
            focus_text(scene, selected_id, "grad_angle", |w| {
                format!("{:.0}", w.props.gradient_angle)
            });
            return;
        }
        *cy += ROW_H;
    }

    // Center + Radius / Start angle (radial / conic)
    if matches!(grad_type, GradientKind::Radial | GradientKind::Conic) {
        for key in ["grad_cx", "grad_cy"] {
            if my >= *cy && my < *cy + ROW_H && hit_x >= val_x {
                let idx = if key == "grad_cx" { 0usize } else { 1 };
                focus_text(scene, selected_id, key, move |w| {
                    format!("{:.2}", w.props.gradient_center[idx])
                });
                return;
            }
            *cy += ROW_H;
        }
    }
    if grad_type == GradientKind::Radial {
        if my >= *cy && my < *cy + ROW_H && hit_x >= val_x {
            focus_text(scene, selected_id, "grad_radius", |w| {
                format!("{:.2}", w.props.gradient_radius)
            });
            return;
        }
        *cy += ROW_H;
    }
    if grad_type == GradientKind::Conic {
        if my >= *cy && my < *cy + ROW_H && hit_x >= val_x {
            focus_text(scene, selected_id, "grad_start_angle", |w| {
                format!("{:.0}", w.props.gradient_start_angle)
            });
            return;
        }
        *cy += ROW_H;
    }

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
            "grad_angle" => {
                if let Ok(v) = buf.trim().parse::<f32>() {
                    w.props.gradient_angle = v;
                }
                return true;
            }
            "grad_cx" => {
                if let Ok(v) = buf.trim().parse::<f32>() {
                    w.props.gradient_center[0] = v.clamp(0.0, 1.0);
                }
                return true;
            }
            "grad_cy" => {
                if let Ok(v) = buf.trim().parse::<f32>() {
                    w.props.gradient_center[1] = v.clamp(0.0, 1.0);
                }
                return true;
            }
            "grad_radius" => {
                if let Ok(v) = buf.trim().parse::<f32>() {
                    w.props.gradient_radius = v.clamp(0.01, 2.0);
                }
                return true;
            }
            "grad_start_angle" => {
                if let Ok(v) = buf.trim().parse::<f32>() {
                    w.props.gradient_start_angle = v;
                }
                return true;
            }
            _ => {}
        }
        // Dynamic stop keys: "stop_pos_{i}" / "stop_col_{i}"
        if let Some(rest) = field.strip_prefix("stop_pos_") {
            if let Ok(i) = rest.parse::<usize>() {
                if let Ok(v) = buf.trim().parse::<f32>() {
                    if let Some(stop) = w.props.gradient_stops.get_mut(i) {
                        stop.0 = v.clamp(0.0, 1.0);
                    }
                }
                return true;
            }
        }
        if let Some(rest) = field.strip_prefix("stop_col_") {
            if let Ok(i) = rest.parse::<usize>() {
                if let Some(stop) = w.props.gradient_stops.get_mut(i) {
                    stop.1 = buf;
                }
                return true;
            }
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
