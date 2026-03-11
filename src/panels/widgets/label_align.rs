// ── Label alignment section ───────────────────────────────────────────────────
//
// Shared by Label and Button widgets.  Handles the "Label Align" inspector
// section: four H-align buttons, four V-align buttons, optional custom-value
// sliders + text fields + pivot sliders.

use ferrous_app::{Color as AppColor, DrawContext, MouseButton};

use super::shared::{draw_row_label, draw_section, LABEL_W, ROW_H, ROW_PAD_X, SECTION_H};
use crate::scene::{HAlign, SceneState, VAlign};

// ── Draw ──────────────────────────────────────────────────────────────────────

pub fn draw(
    dc: &mut DrawContext<'_, '_>,
    scene: &mut SceneState,
    panel_x: f32,
    right_w: f32,
    cy: &mut f32,
    selected_id: u32,
) {
    draw_section(dc, panel_x, *cy, right_w, "Label Align");
    *cy += SECTION_H;

    let selected = match scene.selected() {
        Some(s) => s,
        None => return,
    };

    let h_align = selected.props.label_h_align;
    let v_align = selected.props.label_v_align;
    let h_custom_pct = selected.props.label_h_custom_pct;
    let v_custom_pct = selected.props.label_v_custom_pct;
    let h_custom_val = selected.props.label_h_custom;
    let v_custom_val = selected.props.label_v_custom;
    let h_pivot_val = selected.props.label_h_pivot;
    let v_pivot_val = selected.props.label_v_pivot;
    let widget_w = selected.w;
    let widget_h = selected.h;
    let is_h_custom = matches!(h_align, HAlign::Custom { .. });
    let is_v_custom = matches!(v_align, VAlign::Custom { .. });

    let (mx, my) = dc.ctx.input.mouse_pos_f32();
    let clicked = dc.ctx.input.button_just_pressed(MouseButton::Left);

    // ── Align X row ───────────────────────────────────────────────────────────
    draw_align_buttons(
        dc,
        panel_x,
        right_w,
        *cy,
        mx,
        my,
        clicked,
        &["L", "C", "R", "•"],
        match h_align {
            HAlign::Left => 0,
            HAlign::Center => 1,
            HAlign::Right => 2,
            HAlign::Custom { .. } => 3,
        },
        |i| match i {
            0 => HAlign::Left,
            1 => HAlign::Center,
            2 => HAlign::Right,
            _ => HAlign::Custom {
                value: h_custom_val,
                percent: h_custom_pct,
                pivot: h_pivot_val,
            },
        },
        |new| {
            if let Some(w) = scene.widgets.iter_mut().find(|w| w.id == selected_id) {
                w.props.label_h_align = new;
            }
        },
    );
    *cy += ROW_H;

    // Custom X rows
    if is_h_custom {
        draw_custom_val_row(
            dc,
            scene,
            panel_x,
            right_w,
            cy,
            selected_id,
            mx,
            my,
            clicked,
            "  X val",
            "align_h_custom",
            h_custom_val,
            h_custom_pct,
            widget_w,
            |w: &mut crate::scene::PlacedWidget, v| {
                w.props.label_h_custom = v;
                w.props.label_h_align = HAlign::Custom {
                    value: v,
                    percent: w.props.label_h_custom_pct,
                    pivot: w.props.label_h_pivot,
                };
            },
            |w: &mut crate::scene::PlacedWidget| {
                w.props.label_h_custom_pct = !w.props.label_h_custom_pct;
            },
        );
        draw_pivot_row(
            dc,
            scene,
            panel_x,
            right_w,
            cy,
            selected_id,
            mx,
            my,
            "  X pivot",
            "align_h_pivot",
            h_pivot_val,
            |w, v| {
                w.props.label_h_pivot = v;
                if let HAlign::Custom { value, percent, .. } = w.props.label_h_align {
                    w.props.label_h_align = HAlign::Custom {
                        value,
                        percent,
                        pivot: v,
                    };
                }
            },
        );
    }

    // ── Align Y row ───────────────────────────────────────────────────────────
    draw_align_buttons(
        dc,
        panel_x,
        right_w,
        *cy,
        mx,
        my,
        clicked,
        &["T", "C", "B", "•"],
        match v_align {
            VAlign::Top => 0,
            VAlign::Center => 1,
            VAlign::Bottom => 2,
            VAlign::Custom { .. } => 3,
        },
        |i| match i {
            0 => VAlign::Top,
            1 => VAlign::Center,
            2 => VAlign::Bottom,
            _ => VAlign::Custom {
                value: v_custom_val,
                percent: v_custom_pct,
                pivot: v_pivot_val,
            },
        },
        |new| {
            if let Some(w) = scene.widgets.iter_mut().find(|w| w.id == selected_id) {
                w.props.label_v_align = new;
            }
        },
    );
    *cy += ROW_H;

    // Custom Y rows
    if is_v_custom {
        draw_custom_val_row(
            dc,
            scene,
            panel_x,
            right_w,
            cy,
            selected_id,
            mx,
            my,
            clicked,
            "  Y val",
            "align_v_custom",
            v_custom_val,
            v_custom_pct,
            widget_h,
            |w: &mut crate::scene::PlacedWidget, v| {
                w.props.label_v_custom = v;
                w.props.label_v_align = VAlign::Custom {
                    value: v,
                    percent: w.props.label_v_custom_pct,
                    pivot: w.props.label_v_pivot,
                };
            },
            |w: &mut crate::scene::PlacedWidget| {
                w.props.label_v_custom_pct = !w.props.label_v_custom_pct;
            },
        );
        draw_pivot_row(
            dc,
            scene,
            panel_x,
            right_w,
            cy,
            selected_id,
            mx,
            my,
            "  Y pivot",
            "align_v_pivot",
            v_pivot_val,
            |w, v| {
                w.props.label_v_pivot = v;
                if let VAlign::Custom { value, percent, .. } = w.props.label_v_align {
                    w.props.label_v_align = VAlign::Custom {
                        value,
                        percent,
                        pivot: v,
                    };
                }
            },
        );
    }
}

// ── Update ────────────────────────────────────────────────────────────────────

pub fn update(
    scene: &mut SceneState,
    right_w: f32,
    _mx: f32,
    my: f32,
    hit_x: f32,
    cy: &mut f32,
    selected_id: u32,
) {
    let cust_field_w = 46.0_f32;
    let cust_toggle_w = 28.0_f32;
    let cust_slider_w = right_w - ROW_PAD_X * 2.0 - LABEL_W - cust_field_w - cust_toggle_w - 6.0;
    let cust_field_x = ROW_PAD_X + LABEL_W + cust_slider_w + 3.0;
    let piv_field_w = 46.0_f32;
    let piv_slider_w = right_w - ROW_PAD_X * 2.0 - LABEL_W - piv_field_w - 3.0;
    let piv_field_x = ROW_PAD_X + LABEL_W + piv_slider_w + 3.0;

    let is_h_custom = scene
        .selected()
        .map(|w| matches!(w.props.label_h_align, HAlign::Custom { .. }))
        .unwrap_or(false);
    let is_v_custom = scene
        .selected()
        .map(|w| matches!(w.props.label_v_align, VAlign::Custom { .. }))
        .unwrap_or(false);

    // Align X row — buttons handled by draw, skip
    *cy += ROW_H;

    if is_h_custom {
        if my >= *cy && my < *cy + ROW_H && hit_x >= cust_field_x {
            focus_field(
                scene,
                "align_h_custom",
                |w| format!("{:.1}", w.props.label_h_custom),
                selected_id,
            );
            return;
        }
        *cy += ROW_H;
        if my >= *cy && my < *cy + ROW_H && hit_x >= piv_field_x {
            focus_field(
                scene,
                "align_h_pivot",
                |w| format!("{:.2}", w.props.label_h_pivot),
                selected_id,
            );
            return;
        }
        *cy += ROW_H;
    }

    // Align Y row — buttons handled by draw, skip
    *cy += ROW_H;

    if is_v_custom {
        if my >= *cy && my < *cy + ROW_H && hit_x >= cust_field_x {
            focus_field(
                scene,
                "align_v_custom",
                |w| format!("{:.1}", w.props.label_v_custom),
                selected_id,
            );
            return;
        }
        *cy += ROW_H;
        if my >= *cy && my < *cy + ROW_H && hit_x >= piv_field_x {
            focus_field(
                scene,
                "align_v_pivot",
                |w| format!("{:.2}", w.props.label_v_pivot),
                selected_id,
            );
            return;
        }
        *cy += ROW_H;
    }
}

// ── Commit ────────────────────────────────────────────────────────────────────

pub fn commit_field(scene: &mut SceneState, field: &str, buf: String) -> bool {
    if let Some(w) = scene.selected_mut() {
        match field {
            "align_h_custom" => {
                if let Ok(v) = buf.trim().parse::<f32>() {
                    w.props.label_h_custom = v;
                    w.props.label_h_align = HAlign::Custom {
                        value: v,
                        percent: w.props.label_h_custom_pct,
                        pivot: w.props.label_h_pivot,
                    };
                }
                return true;
            }
            "align_v_custom" => {
                if let Ok(v) = buf.trim().parse::<f32>() {
                    w.props.label_v_custom = v;
                    w.props.label_v_align = VAlign::Custom {
                        value: v,
                        percent: w.props.label_v_custom_pct,
                        pivot: w.props.label_v_pivot,
                    };
                }
                return true;
            }
            "align_h_pivot" => {
                if let Ok(v) = buf.trim().parse::<f32>() {
                    w.props.label_h_pivot = v.clamp(0.0, 1.0);
                    if let HAlign::Custom { value, percent, .. } = w.props.label_h_align {
                        w.props.label_h_align = HAlign::Custom {
                            value,
                            percent,
                            pivot: w.props.label_h_pivot,
                        };
                    }
                }
                return true;
            }
            "align_v_pivot" => {
                if let Ok(v) = buf.trim().parse::<f32>() {
                    w.props.label_v_pivot = v.clamp(0.0, 1.0);
                    if let VAlign::Custom { value, percent, .. } = w.props.label_v_align {
                        w.props.label_v_align = VAlign::Custom {
                            value,
                            percent,
                            pivot: w.props.label_v_pivot,
                        };
                    }
                }
                return true;
            }
            _ => {}
        }
    }
    false
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn focus_field(
    scene: &mut SceneState,
    key: &str,
    value_fn: impl Fn(&crate::scene::PlacedWidget) -> String,
    selected_id: u32,
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

fn draw_align_buttons<A: Copy>(
    dc: &mut DrawContext<'_, '_>,
    panel_x: f32,
    right_w: f32,
    cy: f32,
    mx: f32,
    my: f32,
    clicked: bool,
    labels: &[&str],
    active_idx: usize,
    make_align: impl Fn(usize) -> A,
    mut apply: impl FnMut(A),
) {
    draw_row_label(dc, panel_x, cy, labels[0]); // reuse as axis label is already drawn by caller
    let btn_area_x = panel_x + ROW_PAD_X + LABEL_W + 2.0;
    let avail_w = right_w - ROW_PAD_X * 2.0 - LABEL_W - 2.0;
    let bw = (avail_w / labels.len() as f32).floor() - 1.0;
    let bh = ROW_H - 6.0;
    let by = cy + 3.0;
    for (i, lbl) in labels.iter().enumerate() {
        let bx = btn_area_x + i as f32 * (bw + 1.0);
        let is_active = i == active_idx;
        let base_col = if is_active {
            AppColor::hex("#6C63FF").to_linear_f32()
        } else {
            AppColor::hex("#2D2D30").to_linear_f32()
        };
        let hover_col = if is_active {
            AppColor::hex("#8880FF").to_linear_f32()
        } else {
            AppColor::hex("#3E3E42").to_linear_f32()
        };
        if dc.gui.button_colored(
            dc.font, bx, by, bw, bh, lbl, mx, my, clicked, base_col, hover_col,
        ) {
            apply(make_align(i));
        }
    }
}

fn draw_custom_val_row(
    dc: &mut DrawContext<'_, '_>,
    scene: &mut SceneState,
    panel_x: f32,
    right_w: f32,
    cy: &mut f32,
    selected_id: u32,
    mx: f32,
    my: f32,
    clicked: bool,
    row_label: &str,
    field_key: &str,
    current_val: f32,
    is_pct: bool,
    widget_dim: f32,
    mut apply_val: impl FnMut(&mut crate::scene::PlacedWidget, f32),
    mut toggle_pct: impl FnMut(&mut crate::scene::PlacedWidget),
) {
    let toggle_w = 28.0_f32;
    let field_w = 46.0_f32;
    let slider_w = right_w - ROW_PAD_X * 2.0 - LABEL_W - field_w - toggle_w - 6.0;
    let slider_x = panel_x + ROW_PAD_X + LABEL_W;
    let field_x = slider_x + slider_w + 3.0;
    let tog_x = field_x + field_w + 2.0;
    let field_h = ROW_H - 6.0;
    let by = *cy + 3.0;

    draw_row_label(dc, panel_x, *cy, row_label);

    let max_val = if is_pct {
        100.0_f32
    } else {
        widget_dim.max(1.0)
    };
    let t = (current_val / max_val).clamp(0.0, 1.0);
    let slider_mid_y = by + field_h * 0.5 - 2.0;

    dc.gui.rect(
        slider_x,
        slider_mid_y,
        slider_w,
        4.0,
        AppColor::hex("#3E3E42").to_linear_f32(),
    );
    dc.gui.rect(
        slider_x,
        slider_mid_y,
        t * slider_w,
        4.0,
        AppColor::hex("#4A4580").to_linear_f32(),
    );
    dc.gui.rect(
        slider_x + t * slider_w - 5.0,
        by,
        10.0,
        field_h,
        AppColor::hex("#6C63FF").to_linear_f32(),
    );

    let focused = scene.editing_field.as_deref() == Some(field_key);
    if dc.ctx.input.is_button_down(MouseButton::Left)
        && mx >= slider_x - 4.0
        && mx <= slider_x + slider_w + 4.0
        && my >= by
        && my <= by + field_h
        && !focused
    {
        let new_val = ((mx - slider_x) / slider_w).clamp(0.0, 1.0) * max_val;
        if let Some(w) = scene.widgets.iter_mut().find(|w| w.id == selected_id) {
            apply_val(w, new_val);
        }
    }

    let buf = format!("{:.1}", current_val);
    let val_str: &str = if focused { &scene.edit_buffer } else { &buf };
    let bg = if focused {
        AppColor::hex("#1E3A5F").to_linear_f32()
    } else {
        AppColor::hex("#191919").to_linear_f32()
    };
    let border = if focused {
        Some(AppColor::hex("#0078D4").to_linear_f32())
    } else {
        None
    };
    let fg = if focused {
        [1.0f32; 4]
    } else {
        AppColor::hex("#DDDDDD").to_linear_f32()
    };
    let sel_col = [0.0f32, 0.47, 0.83, 0.35];
    let eff_sel = if scene.edit_state.all_selected && !val_str.is_empty() {
        Some((0usize, val_str.len()))
    } else {
        scene.edit_state.selection()
    };
    dc.gui.draw_text_field(
        dc.font,
        field_x,
        by,
        field_w,
        field_h,
        val_str,
        10.0,
        focused,
        scene.edit_state.cursor_visible,
        scene.edit_state.cursor_pos,
        eff_sel,
        fg,
        bg,
        border,
        sel_col,
        4.0,
    );

    let pct_lbl = if is_pct { "%" } else { "px" };
    if dc.gui.button_colored(
        dc.font,
        tog_x,
        by,
        toggle_w,
        field_h,
        pct_lbl,
        mx,
        my,
        clicked,
        AppColor::hex("#3E3E42").to_linear_f32(),
        AppColor::hex("#555559").to_linear_f32(),
    ) {
        if let Some(w) = scene.widgets.iter_mut().find(|w| w.id == selected_id) {
            toggle_pct(w);
        }
    }
    *cy += ROW_H;
}

fn draw_pivot_row<F>(
    dc: &mut DrawContext<'_, '_>,
    scene: &mut SceneState,
    panel_x: f32,
    right_w: f32,
    cy: &mut f32,
    selected_id: u32,
    mx: f32,
    my: f32,
    row_label: &str,
    field_key: &str,
    pivot_val: f32,
    mut apply: F,
) where
    F: FnMut(&mut crate::scene::PlacedWidget, f32),
{
    let piv_field_w = 46.0_f32;
    let piv_slider_w = right_w - ROW_PAD_X * 2.0 - LABEL_W - piv_field_w - 3.0;
    let piv_slider_x = panel_x + ROW_PAD_X + LABEL_W;
    let piv_field_x = piv_slider_x + piv_slider_w + 3.0;
    let slider_h = ROW_H - 6.0;
    let sy = *cy + 3.0;

    draw_row_label(dc, panel_x, *cy, row_label);

    dc.gui.rect(
        piv_slider_x,
        sy + slider_h * 0.5 - 2.0,
        piv_slider_w,
        4.0,
        AppColor::hex("#3E3E42").to_linear_f32(),
    );
    dc.gui.rect(
        piv_slider_x,
        sy + slider_h * 0.5 - 2.0,
        pivot_val * piv_slider_w,
        4.0,
        AppColor::hex("#4A4580").to_linear_f32(),
    );
    dc.gui.rect(
        piv_slider_x + pivot_val * piv_slider_w - 5.0,
        sy,
        10.0,
        slider_h,
        AppColor::hex("#6C63FF").to_linear_f32(),
    );

    let focused = scene.editing_field.as_deref() == Some(field_key);
    if dc.ctx.input.is_button_down(MouseButton::Left)
        && !focused
        && mx >= piv_slider_x - 6.0
        && mx <= piv_slider_x + piv_slider_w + 6.0
        && my >= sy
        && my <= sy + slider_h
    {
        let new_piv = ((mx - piv_slider_x) / piv_slider_w).clamp(0.0, 1.0);
        if let Some(w) = scene.widgets.iter_mut().find(|w| w.id == selected_id) {
            apply(w, new_piv);
        }
    }

    let buf = format!("{:.2}", pivot_val);
    let piv_str: &str = if focused { &scene.edit_buffer } else { &buf };
    let bg = if focused {
        AppColor::hex("#1E3A5F").to_linear_f32()
    } else {
        AppColor::hex("#191919").to_linear_f32()
    };
    let border = if focused {
        Some(AppColor::hex("#0078D4").to_linear_f32())
    } else {
        None
    };
    let fg = if focused {
        [1.0f32; 4]
    } else {
        AppColor::hex("#DDDDDD").to_linear_f32()
    };
    let sel_col = [0.0f32, 0.47, 0.83, 0.35];
    let eff_sel = if scene.edit_state.all_selected && !piv_str.is_empty() {
        Some((0usize, piv_str.len()))
    } else {
        scene.edit_state.selection()
    };
    dc.gui.draw_text_field(
        dc.font,
        piv_field_x,
        sy,
        piv_field_w,
        slider_h,
        piv_str,
        10.0,
        focused,
        scene.edit_state.cursor_visible,
        scene.edit_state.cursor_pos,
        eff_sel,
        fg,
        bg,
        border,
        sel_col,
        4.0,
    );
    *cy += ROW_H;
}
