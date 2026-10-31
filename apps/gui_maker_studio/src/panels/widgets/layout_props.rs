// ── Layout Properties ────────────────────────────────────────────────────────
//
// Specialized inspector rows for AnchorPane constraints, padding, and size limits.

use ferrous_engine::{DrawContext, MouseButton};
use crate::scene::{PlacedWidget, SceneState, LayoutValue, AutoLayoutDirection};
use crate::theme::Theme;
use super::shared::{
    default_row_colors, draw_text_row,
    LABEL_W, ROW_H, ROW_PAD_X, SECTION_H, draw_section
};

pub fn draw_layout_section(
    dc: &mut DrawContext<'_, '_>,
    scene: &mut SceneState,
    panel_x: f32,
    right_w: f32,
    cy: &mut f32,
    mx: f32,
    my: f32,
    theme: &Theme,
) {
    let selected_id = match scene.selected_id {
        Some(id) => id,
        None => return,
    };

    draw_section(dc, panel_x, *cy, right_w, "Layout { Anchor Pane }", theme);
    *cy += SECTION_H;

    // Anchor Diagram
    let sel_widget = scene.widgets.iter().find(|w| w.id == selected_id).unwrap();
    draw_anchor_diagram(dc, panel_x, *cy, right_w, sel_widget);
    *cy += 100.0;

    // Anchor Top
    draw_anchor_row(dc, scene, panel_x, *cy, right_w, "Top", "anchor_top", selected_id, theme);
    *cy += ROW_H;
    
    // Anchor Right
    draw_anchor_row(dc, scene, panel_x, *cy, right_w, "Right", "anchor_right", selected_id, theme);
    *cy += ROW_H;
    
    // Anchor Bottom
    draw_anchor_row(dc, scene, panel_x, *cy, right_w, "Bottom", "anchor_bottom", selected_id, theme);
    *cy += ROW_H;
    
    // Anchor Left
    draw_anchor_row(dc, scene, panel_x, *cy, right_w, "Left", "anchor_left", selected_id, theme);
    *cy += ROW_H;

    // Padding
    draw_padding_rows(dc, scene, panel_x, *cy, right_w, selected_id, theme);
    *cy += ROW_H * 4.0;

    // Min/Pref/Max Sizes
    draw_size_rows(dc, scene, panel_x, *cy, right_w, selected_id, theme);
    *cy += ROW_H * 2.0; 
}

fn draw_anchor_diagram(
    dc: &mut DrawContext<'_, '_>,
    panel_x: f32,
    y: f32,
    right_w: f32,
    w: &PlacedWidget,
) {
    let size = 80.0;
    let dx = panel_x + (right_w - size) * 0.5;
    let dy = y + 5.0;

    let bg_col = [0.12, 0.12, 0.12, 1.0];
    let active_col = [1.0, 0.45, 0.0, 1.0]; // Vibrant orange
    let inactive_col = [0.25, 0.25, 0.25, 1.0];

    // Outter rect (Container)
    dc.gui.rect(dx, dy, size, size, bg_col);
    
    // Inner rect (The widget)
    let iw = 34.0;
    let idx = dx + (size - iw) * 0.5;
    let idy = dy + (size - iw) * 0.5;
    dc.gui.rect(idx, idy, iw, iw, [0.2, 0.2, 0.2, 1.0]);
    dc.gui.rect(idx + 1.0, idy + 1.0, iw - 2.0, iw - 2.0, [0.25, 0.25, 0.25, 1.0]);

    // Top anchor line
    let t_col = if w.props.anchor_top.is_some() { active_col } else { inactive_col };
    dc.gui.rect(dx + size*0.5 - 1.0, dy + 4.0, 2.0, (idy - dy) - 6.0, t_col);

    // Bottom
    let b_col = if w.props.anchor_bottom.is_some() { active_col } else { inactive_col };
    dc.gui.rect(dx + size*0.5 - 1.0, idy + iw + 2.0, 2.0, (dy + size - (idy + iw)) - 6.0, b_col);

    // Left
    let l_col = if w.props.anchor_left.is_some() { active_col } else { inactive_col };
    dc.gui.rect(dx + 4.0, dy + size*0.5 - 1.0, (idx - dx) - 6.0, 2.0, l_col);

    // Right
    let r_col = if w.props.anchor_right.is_some() { active_col } else { inactive_col };
    dc.gui.rect(idx + iw + 2.0, dy + size*0.5 - 1.0, (dx + size - (idx + iw)) - 6.0, 2.0, r_col);
}

fn draw_anchor_row(
    dc: &mut DrawContext<'_, '_>,
    scene: &mut SceneState,
    panel_x: f32,
    y: f32,
    right_w: f32,
    label: &str,
    field_key: &str,
    widget_id: u32,
    theme: &Theme,
) {
    let val = {
        let w = scene.widgets.iter().find(|w| w.id == widget_id).unwrap();
        match field_key {
            "anchor_top" => w.props.anchor_top.clone(),
            "anchor_right" => w.props.anchor_right.clone(),
            "anchor_bottom" => w.props.anchor_bottom.clone(),
            "anchor_left" => w.props.anchor_left.clone(),
            _ => None,
        }
    };

    let focused = scene.editing_field.as_deref() == Some(field_key);
    let display_val = if focused {
        scene.edit_buffer.clone()
    } else {
        val.as_ref().map(|v| v.to_string_clean()).unwrap_or_else(|| "---".to_string())
    };

    let (tc, bc, brc, sc) = default_row_colors(label, focused, theme);
    
    // Color it differently if enabled
    let mut actual_tc = tc;
    if val.is_some() && !focused {
        actual_tc = [0.4, 0.8, 1.0, 1.0]; // Light blue for enabled anchors
    }

    draw_text_row(dc, panel_x, y, right_w, label, &display_val, focused,
        scene.edit_state.cursor_visible, scene.edit_state.cursor_pos,
        scene.edit_state.selection(), scene.edit_state.all_selected, actual_tc, bc, brc, sc);
}

fn draw_padding_rows(
    dc: &mut DrawContext<'_, '_>,
    scene: &mut SceneState,
    panel_x: f32,
    y: f32,
    right_w: f32,
    widget_id: u32,
    theme: &Theme,
) {
    let w = scene.widgets.iter().find(|w| w.id == widget_id).unwrap();
    let p = {
        let w = scene.widgets.iter().find(|w| w.id == widget_id).unwrap();
        w.props.padding.clone()
    };
    let mut cy = y;
    
    draw_padding_field(dc, scene, panel_x, cy, right_w, "Pad Top", "pad_top", p[0].clone(), theme);
    cy += ROW_H;
    draw_padding_field(dc, scene, panel_x, cy, right_w, "Pad Right", "pad_right", p[1].clone(), theme);
    cy += ROW_H;
    draw_padding_field(dc, scene, panel_x, cy, right_w, "Pad Bottom", "pad_bottom", p[2].clone(), theme);
    cy += ROW_H;
    draw_padding_field(dc, scene, panel_x, cy, right_w, "Pad Left", "pad_left", p[3].clone(), theme);
}

fn draw_padding_field(dc: &mut DrawContext, scene: &mut SceneState, x: f32, y: f32, w: f32, label: &str, field_key: &str, val: LayoutValue, theme: &Theme) {
    let focused = scene.editing_field.as_deref() == Some(field_key);
    let display_val = if focused { scene.edit_buffer.clone() } else { val.to_string_clean() };
    let (tc, bc, brc, sc) = default_row_colors(label, focused, theme);
    
    draw_text_row(dc, x, y, w, label, &display_val, focused,
        scene.edit_state.cursor_visible, scene.edit_state.cursor_pos,
        scene.edit_state.selection(), scene.edit_state.all_selected, tc, bc, brc, sc);
}

fn draw_size_rows(
    dc: &mut DrawContext<'_, '_>,
    scene: &mut SceneState,
    panel_x: f32,
    y: f32,
    right_w: f32,
    widget_id: u32,
    theme: &Theme,
) {
    let w = scene.widgets.iter().find(|w| w.id == widget_id).unwrap();
    
    let min_w = w.props.min_width.map(|v| format!("{:.0}", v)).unwrap_or_else(|| "Auto".to_string());
    let pref_w = w.props.pref_width.map(|v| format!("{:.0}", v)).unwrap_or_else(|| format!("{:.0}", w.w));
    let max_w = w.props.max_width.map(|v| format!("{:.0}", v)).unwrap_or_else(|| "∞".to_string());
    
    draw_info_row_custom(dc, panel_x, y, right_w, "Width", &format!("{} / {} / {}", min_w, pref_w, max_w), theme);
    
    let min_h = w.props.min_height.map(|v| format!("{:.0}", v)).unwrap_or_else(|| "Auto".to_string());
    let pref_h = w.props.pref_height.map(|v| format!("{:.0}", v)).unwrap_or_else(|| format!("{:.0}", w.h));
    let max_h = w.props.max_height.map(|v| format!("{:.0}", v)).unwrap_or_else(|| "∞".to_string());

    draw_info_row_custom(dc, panel_x, y + ROW_H, right_w, "Height", &format!("{} / {} / {}", min_h, pref_h, max_h), theme);

    // Auto Layout overlay
    draw_info_row_custom(dc, panel_x, y + ROW_H*2.0, right_w, "Auto Layout", if w.props.auto_layout { "on" } else { "off" }, theme);
    draw_info_row_custom(dc, panel_x, y + ROW_H*3.0, right_w, "Direction", match w.props.auto_layout_direction { AutoLayoutDirection::Horizontal => "horizontal", AutoLayoutDirection::Vertical => "vertical" }, theme);
    draw_info_row_custom(dc, panel_x, y + ROW_H*4.0, right_w, "Gap", &format!("{:.0}", w.props.auto_layout_gap), theme);
}

fn draw_info_row_custom(dc: &mut DrawContext<'_, '_>, panel_x: f32, y: f32, right_w: f32, label: &str, value: &str, theme: &Theme) {
    let (tc, bc, brc, sc) = default_row_colors(label, false, theme);
    draw_text_row(dc, panel_x, y, right_w, label, value, false, false, 0, None, false, tc, bc, brc, sc);
}

pub fn update_layout_props(
    ctx: &mut ferrous_engine::AppContext,
    scene: &mut SceneState,
    _panel_x: f32,
    right_w: f32,
    mx: f32,
    my: f32,
    hit_x: f32,
    cy: &mut f32,
) {
    let val_x = LABEL_W + ROW_PAD_X;
    let clicked = ctx.input.button_just_pressed(MouseButton::Left);
    if !clicked { return; }

    *cy += SECTION_H; // skip header

    let id = match scene.selected_id {
        Some(id) => id,
        None => { return; }
    };

    // Diagram hit detection
    let diag_size = 80.0;
    let dx = (right_w - diag_size) * 0.5; 
    let dy = *cy + 5.0;
    
    // Detect click on Diagram sides
    if hit_x >= dx - 10.0 && hit_x <= dx + diag_size + 10.0 && my >= dy - 5.0 && my <= dy + diag_size + 5.0 {
        let ix = hit_x - dx;
        let iy = my - dy;
        let w = scene.widgets.iter_mut().find(|w| w.id == id).unwrap();
        
        let margin = 20.0;
        
        // Quad-based hit detection
        if iy < margin && ix > margin && ix < diag_size - margin { // Top
            w.props.anchor_top = if w.props.anchor_top.is_some() { None } else { Some(LayoutValue::Absolute(0.0)) };
        } else if iy > diag_size - margin && ix > margin && ix < diag_size - margin { // Bottom
            w.props.anchor_bottom = if w.props.anchor_bottom.is_some() { None } else { Some(LayoutValue::Absolute(0.0)) };
        } else if ix < margin && iy > margin && iy < diag_size - margin { // Left
            w.props.anchor_left = if w.props.anchor_left.is_some() { None } else { Some(LayoutValue::Absolute(0.0)) };
        } else if ix > diag_size - margin && iy > margin && iy < diag_size - margin { // Right
            w.props.anchor_right = if w.props.anchor_right.is_some() { None } else { Some(LayoutValue::Absolute(0.0)) };
        }
        return;
    }
    *cy += 100.0;

    for field in &["anchor_top", "anchor_right", "anchor_bottom", "anchor_left"] {
        if my >= *cy && my < *cy + ROW_H && hit_x >= val_x {
            let w = scene.widgets.iter().find(|w| w.id == id).unwrap();
            let val = match *field {
                "anchor_top" => w.props.anchor_top.clone(),
                "anchor_right" => w.props.anchor_right.clone(),
                "anchor_bottom" => w.props.anchor_bottom.clone(),
                "anchor_left" => w.props.anchor_left.clone(),
                _ => None,
            };
            
            scene.editing_field = Some(field.to_string());
            scene.edit_buffer = val.map(|v| v.to_string_clean()).unwrap_or_default();
            scene.edit_state.focus();
            scene.edit_state.cursor_pos = scene.edit_buffer.len();
            return;
        }
        *cy += ROW_H;
    }
    
    // Padding hit detection
    for (f, idx) in vec![("pad_top", 0), ("pad_right", 1), ("pad_bottom", 2), ("pad_left", 3)] {
        if my >= *cy && my < *cy + ROW_H && hit_x >= val_x {
             let w = scene.widgets.iter().find(|w| w.id == id).unwrap();
             scene.editing_field = Some(f.to_string());
             scene.edit_buffer = w.props.padding[idx].to_string_clean();
             scene.edit_state.focus();
             scene.edit_state.cursor_pos = scene.edit_buffer.len();
             return;
        }
        *cy += ROW_H;
    }

    // Size hit detection (auto-layout controls)
    // Auto Layout toggle (click to enable/disable)
    if my >= *cy && my < *cy + ROW_H && hit_x >= val_x {
        let w = scene.widgets.iter_mut().find(|w| w.id == id).unwrap();
        w.props.auto_layout = !w.props.auto_layout;
        return;
    }
    *cy += ROW_H;

    // Direction toggle (horizontal/vertical)
    if my >= *cy && my < *cy + ROW_H && hit_x >= val_x {
        let w = scene.widgets.iter_mut().find(|w| w.id == id).unwrap();
        w.props.auto_layout_direction = match w.props.auto_layout_direction {
            AutoLayoutDirection::Horizontal => AutoLayoutDirection::Vertical,
            AutoLayoutDirection::Vertical => AutoLayoutDirection::Horizontal,
        };
        return;
    }
    *cy += ROW_H;

    // Gap edit
    if my >= *cy && my < *cy + ROW_H && hit_x >= val_x {
        let w = scene.widgets.iter().find(|w| w.id == id).unwrap();
        scene.editing_field = Some("auto_layout_gap".to_string());
        scene.edit_buffer = format!("{:.0}", w.props.auto_layout_gap);
        scene.edit_state.focus();
        scene.edit_state.cursor_pos = scene.edit_buffer.len();
        return;
    }
    *cy += ROW_H;
}

pub fn commit_layout_field(scene: &mut SceneState, field: &str, buf: String) -> bool {
    let w = match scene.selected_mut() {
        Some(w) => w,
        None => return false,
    };

    let parse_layout_value = |s: &str| -> Option<LayoutValue> {
        LayoutValue::parse(s)
    };

    match field {
        "anchor_top" | "anchor_right" | "anchor_bottom" | "anchor_left" => {
            let val = parse_layout_value(&buf);
            match field {
                "anchor_top" => w.props.anchor_top = val,
                "anchor_right" => w.props.anchor_right = val,
                "anchor_bottom" => w.props.anchor_bottom = val,
                "anchor_left" => w.props.anchor_left = val,
                _ => {}
            }
            return true;
        }
        "pad_top" | "pad_right" | "pad_bottom" | "pad_left" => {
            if let Some(v) = parse_layout_value(&buf) {
                match field {
                    "pad_top" => w.props.padding[0] = v,
                    "pad_right" => w.props.padding[1] = v,
                    "pad_bottom" => w.props.padding[2] = v,
                    "pad_left" => w.props.padding[3] = v,
                    _ => {}
                }
            }
            return true;
        }
        "auto_layout_gap" => {
            if let Ok(v) = buf.trim().parse::<f32>() {
                w.props.auto_layout_gap = v.max(0.0);
            }
            return true;
        }
        _ => false
    }
}
