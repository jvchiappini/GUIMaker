// ── Generic widget ────────────────────────────────────────────────────────────
//
// Canvas preview and inspector rows for all widget kinds that do not have their
// own dedicated module (Slider, ProgressBar, Panel, Checkbox, etc.).

use ferrous_engine::{AppContext, DrawContext, MouseButton};
use ferrous_ui_core::Rect;

use crate::scene::{PlacedWidget, SceneState, WidgetKind};
use crate::theme::{Theme, ThemeRegistry};
use super::shared::{
    default_row_colors, draw_info_row, draw_text_row, parse_hex_or,
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
    theme: &Theme,
) {
    let zoom = sw / w.w;

    let mut col = if w.kind == WidgetKind::Panel {
        ThemeRegistry::hex_to_f32(&w.props.bg_color_hex)
    } else {
        w.kind.color()
    };
    if w.kind == WidgetKind::Panel {
        col[3] *= w.props.opacity; // apply explicit panel opacity
    } else {
        col[3] = if is_selected { 0.85 } else { 0.65 };
        col[3] *= w.props.opacity;
    }
    dc.gui.rect(sx, sy, sw, sh, col);

    dc.gui.push_clip(Rect::new(sx, sy, sw, sh));
    if sw > 20.0 && sh > 12.0 && w.kind != WidgetKind::Panel {
        let label = if w.props.label.is_empty() { w.kind.display_name() } else { &w.props.label };
        let font_size = (w.props.font_size * zoom).clamp(8.0, 32.0);
        let text_x = sx + 4.0;
        let text_y = sy + (sh - font_size) * 0.5;
        dc.gui.draw_text(dc.font, label, [text_x, text_y], font_size, ThemeRegistry::hex_to_f32(&theme.colors.text));
    }
    dc.gui.pop_clip();

    let border_col = if is_selected {
        ThemeRegistry::hex_to_f32(&theme.colors.accent)
    } else {
        let mut c = ThemeRegistry::hex_to_f32(&theme.colors.border);
        c[3] = 0.9;
        c
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
    mx: f32,
    my: f32,
    theme: &Theme,
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
            let (tc, bc, brc, sc) = default_row_colors("Label", focused, theme);
            draw_text_row(dc, panel_x, *cy, right_w, "Label", value, focused,
                scene.edit_state.cursor_visible, scene.edit_state.cursor_pos,
                scene.edit_state.selection(), scene.edit_state.all_selected, tc, bc, brc, sc);
            *cy += ROW_H;
        }
        WidgetKind::TextInput | WidgetKind::NumberInput => {
            let focused = scene.editing_field.as_deref() == Some("label");
            let value: &str = if focused { &scene.edit_buffer } else { &selected.props.label };
            let (tc, bc, brc, sc) = default_row_colors("Hint", focused, theme);
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
            draw_info_row(dc, panel_x, *cy, right_w, "Min", &format!("{:.0}", selected.props.min), theme);
            *cy += ROW_H;
            draw_info_row(dc, panel_x, *cy, right_w, "Max", &format!("{:.0}", selected.props.max), theme);
            *cy += ROW_H;
            draw_info_row(dc, panel_x, *cy, right_w, "Val", &format!("{:.0}", selected.props.value), theme);
            *cy += ROW_H;
        }
        _ => {}
    }

    // Per-kind property rows
    match kind {
        WidgetKind::Panel => {
            let hex = &selected.props.bg_color_hex;
            super::shared::draw_color_row(dc, panel_x, *cy, right_w, "Background", hex, theme);
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
    mx: f32,
    my: f32,
    raw_my: f32,
    hit_x: f32,
    cy: &mut f32,
    cp: &mut crate::color_picker::ColorPickerState,
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
                .map(|w| w.props.bg_color_hex.clone()).unwrap_or_default();
            cp.active = true;
            cp.just_opened = true;
            cp.open_anim = 0.0;
            cp.target_field = "bg_color".to_string();
            cp.widget_id = Some(selected_id);
            cp.screen_pos = (mx, raw_my);
            cp.current_color = parse_hex_or(&current, [1.0, 1.0, 1.0, 1.0]);
            cp.alpha = scene.widgets.iter().find(|w| w.id == selected_id).map(|w| w.props.opacity).unwrap_or(1.0);
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
            "background" => { w.props.bg_color_hex = buf; return true; }
            _ => {}
        }
    }
    false
}
