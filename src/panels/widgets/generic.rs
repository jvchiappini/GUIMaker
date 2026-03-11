// ── Generic widget ────────────────────────────────────────────────────────────
//
// Canvas preview and inspector rows for all widget kinds that do not have their
// own dedicated module (Slider, ProgressBar, Panel, Checkbox, etc.).

use ferrous_app::{AppContext, DrawContext, MouseButton};
use ferrous_ui_core::Rect;

use crate::scene::{PlacedWidget, SceneState, WidgetKind};
use super::shared::{
    default_row_colors, draw_info_row, draw_text_row,
    LABEL_W, ROW_H, ROW_PAD_X,
};

// ── Canvas preview ────────────────────────────────────────────────────────────

pub fn draw_canvas(
    dc: &mut DrawContext<'_, '_>,
    w: &PlacedWidget,
    sx: f32,
    sy: f32,
    sw: f32,
    sh: f32,
    is_selected: bool,
) {
    let zoom = sw / w.w;

    let mut col = w.kind.color();
    col[3] = if is_selected { 0.85 } else { 0.65 };
    dc.gui.rect(sx, sy, sw, sh, col);

    dc.gui.push_clip(Rect::new(sx, sy, sw, sh));
    if sw > 20.0 && sh > 12.0 {
        let label = if w.props.label.is_empty() { w.kind.display_name() } else { &w.props.label };
        let font_size = (w.props.font_size * zoom).clamp(8.0, 32.0);
        let text_x = sx + 4.0;
        let text_y = sy + (sh - font_size) * 0.5;
        dc.gui.draw_text(dc.font, label, [text_x, text_y], font_size, [1.0, 1.0, 1.0, 0.9]);
    }
    dc.gui.pop_clip();

    let border_col = if is_selected {
        [1.0_f32, 0.8, 0.0, 1.0]
    } else {
        [col[0] * 1.4, col[1] * 1.4, col[2] * 1.4, 0.9]
    };
    let bw = if is_selected { 2.0 } else { 1.0 };
    dc.gui.rect(sx, sy, sw, bw, border_col);
    dc.gui.rect(sx, sy + sh - bw, sw, bw, border_col);
    dc.gui.rect(sx, sy, bw, sh, border_col);
    dc.gui.rect(sx + sw - bw, sy, bw, sh, border_col);
}

// ── Inspector: draw ───────────────────────────────────────────────────────────

pub fn draw_props(
    dc: &mut DrawContext<'_, '_>,
    scene: &mut SceneState,
    panel_x: f32,
    right_w: f32,
    cy: &mut f32,
) {
    let selected = match scene.selected() {
        Some(s) => s,
        None => return,
    };
    let kind = selected.kind;

    // Hint / Label for text-input widgets
    match kind {
        WidgetKind::Checkbox | WidgetKind::ToggleSwitch | WidgetKind::Tooltip | WidgetKind::Toast => {
            let focused = scene.editing_field.as_deref() == Some("label");
            let value: &str = if focused { &scene.edit_buffer } else { &selected.props.label };
            let (tc, bc, brc, sc) = default_row_colors("Label", focused);
            draw_text_row(dc, panel_x, *cy, right_w, "Label", value, focused,
                scene.edit_state.cursor_visible, scene.edit_state.cursor_pos,
                scene.edit_state.selection(), scene.edit_state.all_selected, tc, bc, brc, sc);
            *cy += ROW_H;
        }
        WidgetKind::TextInput | WidgetKind::NumberInput => {
            let focused = scene.editing_field.as_deref() == Some("label");
            let value: &str = if focused { &scene.edit_buffer } else { &selected.props.label };
            let (tc, bc, brc, sc) = default_row_colors("Hint", focused);
            draw_text_row(dc, panel_x, *cy, right_w, "Hint", value, focused,
                scene.edit_state.cursor_visible, scene.edit_state.cursor_pos,
                scene.edit_state.selection(), scene.edit_state.all_selected, tc, bc, brc, sc);
            *cy += ROW_H;
        }
        _ => {}
    }

    // Range (read-only) for Slider / NumberInput / ProgressBar
    match kind {
        WidgetKind::Slider | WidgetKind::NumberInput | WidgetKind::ProgressBar => {
            draw_info_row(dc, panel_x, *cy, right_w, "Min", &format!("{:.0}", selected.props.min));
            *cy += ROW_H;
            draw_info_row(dc, panel_x, *cy, right_w, "Max", &format!("{:.0}", selected.props.max));
            *cy += ROW_H;
            draw_info_row(dc, panel_x, *cy, right_w, "Val", &format!("{:.0}", selected.props.value));
            *cy += ROW_H;
        }
        _ => {}
    }

    // Color hex for Panel
    match kind {
        WidgetKind::Panel => {
            let focused = scene.editing_field.as_deref() == Some("color");
            let value: &str = if focused { &scene.edit_buffer } else { &selected.props.color_hex };
            let (tc, bc, brc, sc) = default_row_colors("Color", focused);
            draw_text_row(dc, panel_x, *cy, right_w, "Color", value, focused,
                scene.edit_state.cursor_visible, scene.edit_state.cursor_pos,
                scene.edit_state.selection(), scene.edit_state.all_selected, tc, bc, brc, sc);
            *cy += ROW_H;
        }
        _ => {}
    }
}

// ── Inspector: update ─────────────────────────────────────────────────────────

pub fn update_props(
    ctx: &mut AppContext,
    scene: &mut SceneState,
    _panel_x: f32,
    _right_w: f32,
    _mx: f32,
    my: f32,
    hit_x: f32,
    cy: &mut f32,
) {
    let val_x = LABEL_W + ROW_PAD_X;
    let clicked = ctx.input.button_just_pressed(MouseButton::Left);
    if !clicked { return; }

    let Some(selected_id) = scene.selected_id else { return; };
    let kind = scene.widgets.iter().find(|w| w.id == selected_id).map(|w| w.kind);
    let Some(kind) = kind else { return; };

    // Hint / Label
    match kind {
        WidgetKind::Checkbox | WidgetKind::ToggleSwitch | WidgetKind::Tooltip | WidgetKind::Toast
        | WidgetKind::TextInput | WidgetKind::NumberInput => {
            if my >= *cy && my < *cy + ROW_H && hit_x >= val_x {
                let current = scene.widgets.iter().find(|w| w.id == selected_id)
                    .map(|w| w.props.label.clone()).unwrap_or_default();
                scene.editing_field = Some("label".to_string());
                scene.edit_buffer = current;
                scene.edit_state.focus();
                scene.edit_state.cursor_pos = scene.edit_buffer.len();
                return;
            }
            *cy += ROW_H;
        }
        _ => {}
    }

    // Range rows — read-only, just skip
    match kind {
        WidgetKind::Slider | WidgetKind::NumberInput | WidgetKind::ProgressBar => {
            *cy += ROW_H * 3.0;
        }
        _ => {}
    }

    // Color (Panel)
    if kind == WidgetKind::Panel {
        if my >= *cy && my < *cy + ROW_H && hit_x >= val_x {
            let current = scene.widgets.iter().find(|w| w.id == selected_id)
                .map(|w| w.props.color_hex.clone()).unwrap_or_default();
            scene.editing_field = Some("color".to_string());
            scene.edit_buffer = current;
            scene.edit_state.focus();
            scene.edit_state.cursor_pos = scene.edit_buffer.len();
            return;
        }
        *cy += ROW_H;
    }
}

// ── Commit ────────────────────────────────────────────────────────────────────

pub fn commit_field(scene: &mut SceneState, field: &str, buf: String) -> bool {
    if let Some(w) = scene.selected_mut() {
        match field {
            "label" => { w.props.label = buf; return true; }
            "color" => { w.props.color_hex = buf; return true; }
            _ => {}
        }
    }
    false
}
