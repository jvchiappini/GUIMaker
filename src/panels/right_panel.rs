use ferrous_app::{AppContext, Color as AppColor, DrawContext, KeyCode, MouseButton};
use ferrous_gui::Rect;

use crate::{
    c_border,
    scene::{SceneState, WidgetKind},
    TOP_H,
};

// ── Layout constants ──────────────────────────────────────────────────────────
const HEADER_H: f32 = 34.0;
const SECTION_H: f32 = 20.0;
const ROW_H: f32 = 26.0;
const ROW_PAD_X: f32 = 10.0;
const BTN_W: f32 = 22.0;
const LABEL_W: f32 = 54.0;

// ── Update (interaction) ──────────────────────────────────────────────────────

pub fn update(ctx: &mut AppContext, right_w: f32, scene: &mut SceneState) {
    let (win_w, win_h) = ctx.window_size;
    let panel_x = win_w as f32 - right_w;
    let panel_h = win_h as f32 - TOP_H;

    let (mx, my) = ctx.input.mouse_pos_f32();
    let mouse_in_panel = mx >= panel_x && mx < win_w as f32 && my >= TOP_H && my < TOP_H + panel_h;
    let clicked = ctx.input.button_just_pressed(MouseButton::Left);

    // ── Keyboard handling for focused text field ───────────────────────────
    if scene.editing_field.is_some() {
        // Backspace
        if ctx.input.just_pressed(KeyCode::Backspace) {
            scene.edit_buffer.pop();
        }
        // Printable chars
        for &c in ctx.input.typed_chars() {
            // Skip control characters (backspace is already handled via KeyCode)
            if !c.is_control() {
                scene.edit_buffer.push(c);
            }
        }
        // Commit on Enter, cancel on Escape
        if ctx.input.just_pressed(KeyCode::Enter) || ctx.input.just_pressed(KeyCode::Escape) {
            let commit = ctx.input.just_pressed(KeyCode::Enter);
            if commit {
                if let Some(field) = scene.editing_field.clone() {
                    let buf = scene.edit_buffer.clone();
                    if let Some(w) = scene.selected_mut() {
                        match field.as_str() {
                            "label" => w.props.label = buf,
                            "color" => w.props.color_hex = buf,
                            _ => {}
                        }
                    }
                }
            }
            scene.editing_field = None;
            scene.edit_buffer.clear();
        }
        // Clicking outside the panel also commits
        if clicked && !mouse_in_panel {
            if let Some(field) = scene.editing_field.clone() {
                let buf = scene.edit_buffer.clone();
                if let Some(w) = scene.selected_mut() {
                    match field.as_str() {
                        "label" => w.props.label = buf,
                        "color" => w.props.color_hex = buf,
                        _ => {}
                    }
                }
            }
            scene.editing_field = None;
            scene.edit_buffer.clear();
        }
        // If a text field is focused, swallow ALL input so camera/canvas don't react
        return;
    }

    if !mouse_in_panel || !clicked {
        return;
    }

    let Some(selected_id) = scene.selected_id else {
        return;
    };

    let hit_x = mx - panel_x;
    let minus_x = right_w - ROW_PAD_X - BTN_W * 2.0 - 4.0;
    let plus_x = right_w - ROW_PAD_X - BTN_W;

    let mut cursor_y = TOP_H + HEADER_H;
    cursor_y += 22.0 + 10.0; // kind badge

    // skip hidden hint row
    if scene
        .widgets
        .iter()
        .any(|w| w.id == selected_id && !w.props.visible)
    {
        cursor_y += ROW_H;
    }

    // -- Transform section ----------------------------------------------------
    cursor_y += SECTION_H;
    for field in 0u8..4 {
        if my >= cursor_y && my < cursor_y + ROW_H {
            if hit_x >= minus_x && hit_x < minus_x + BTN_W {
                if let Some(w) = scene.widgets.iter_mut().find(|w| w.id == selected_id) {
                    match field {
                        0 => w.x -= 1.0,
                        1 => w.y -= 1.0,
                        2 => w.w = (w.w - 1.0).max(8.0),
                        3 => w.h = (w.h - 1.0).max(8.0),
                        _ => {}
                    }
                }
            } else if hit_x >= plus_x && hit_x < plus_x + BTN_W {
                if let Some(w) = scene.widgets.iter_mut().find(|w| w.id == selected_id) {
                    match field {
                        0 => w.x += 1.0,
                        1 => w.y += 1.0,
                        2 => w.w += 1.0,
                        3 => w.h += 1.0,
                        _ => {}
                    }
                }
            }
        }
        cursor_y += ROW_H;
    }

    // -- Properties section ---------------------------------------------------
    cursor_y += SECTION_H;

    // Visible toggle
    if my >= cursor_y && my < cursor_y + ROW_H {
        let toggle_x = LABEL_W + ROW_PAD_X + 4.0;
        if hit_x >= toggle_x && hit_x < toggle_x + 60.0 {
            if let Some(w) = scene.widgets.iter_mut().find(|w| w.id == selected_id) {
                w.props.visible = !w.props.visible;
            }
        }
    }
    cursor_y += ROW_H;

    // Editable text rows
    let kind = scene
        .widgets
        .iter()
        .find(|w| w.id == selected_id)
        .map(|w| w.kind);
    if let Some(kind) = kind {
        // Label / Hint row
        match kind {
            WidgetKind::Label
            | WidgetKind::Button
            | WidgetKind::Checkbox
            | WidgetKind::ToggleSwitch
            | WidgetKind::Tooltip
            | WidgetKind::Toast
            | WidgetKind::TextInput
            | WidgetKind::NumberInput => {
                if my >= cursor_y && my < cursor_y + ROW_H {
                    let val_x = LABEL_W + ROW_PAD_X;
                    if hit_x >= val_x {
                        // Focus the label/hint field
                        let current = scene
                            .widgets
                            .iter()
                            .find(|w| w.id == selected_id)
                            .map(|w| w.props.label.clone())
                            .unwrap_or_default();
                        scene.editing_field = Some("label".to_string());
                        scene.edit_buffer = current;
                        return;
                    }
                }
                cursor_y += ROW_H;
            }
            _ => {}
        }

        // Font size row (read-only)
        match kind {
            WidgetKind::Label | WidgetKind::Button | WidgetKind::TextInput => {
                cursor_y += ROW_H;
            }
            _ => {}
        }

        // Range rows (read-only)
        match kind {
            WidgetKind::Slider | WidgetKind::NumberInput | WidgetKind::ProgressBar => {
                cursor_y += ROW_H * 3.0;
            }
            _ => {}
        }

        // Color row
        match kind {
            WidgetKind::Panel | WidgetKind::Label | WidgetKind::Button => {
                if my >= cursor_y && my < cursor_y + ROW_H {
                    let val_x = LABEL_W + ROW_PAD_X;
                    if hit_x >= val_x {
                        let current = scene
                            .widgets
                            .iter()
                            .find(|w| w.id == selected_id)
                            .map(|w| w.props.color_hex.clone())
                            .unwrap_or_default();
                        scene.editing_field = Some("color".to_string());
                        scene.edit_buffer = current;
                        return;
                    }
                }
                cursor_y += ROW_H;
            }
            _ => {}
        }
    }

    // -- Delete button --------------------------------------------------------
    let del_y = cursor_y + 8.0;
    if my >= del_y && my < del_y + 28.0 && hit_x >= 10.0 && hit_x < right_w - 10.0 {
        scene.delete_selected();
    }
}

// ── Draw (immediate mode) ─────────────────────────────────────────────────────

pub fn draw(dc: &mut DrawContext<'_, '_>, right_w: f32, scene: &SceneState) {
    let (win_w, win_h) = dc.ctx.window_size;
    let panel_x = win_w as f32 - right_w;
    let (mx, my) = dc.ctx.input.mouse_pos_f32();

    dc.gui
        .push_clip(Rect::new(panel_x, TOP_H, right_w, win_h as f32 - TOP_H));

    // -- Header bar -----------------------------------------------------------
    dc.gui.rect(
        panel_x,
        TOP_H,
        right_w,
        HEADER_H,
        AppColor::hex("#2D2D30").to_linear_f32(),
    );
    dc.gui.draw_text(
        dc.font,
        "PROPERTIES",
        [panel_x + ROW_PAD_X, TOP_H + 10.0],
        10.0,
        AppColor::hex("#888888").to_linear_f32(),
    );

    let mut cursor_y = TOP_H + HEADER_H;

    let Some(selected) = scene.selected() else {
        dc.gui.draw_text(
            dc.font,
            "No widget selected",
            [panel_x + ROW_PAD_X, cursor_y + 10.0],
            10.0,
            AppColor::hex("#666666").to_linear_f32(),
        );
        dc.gui.pop_clip();
        return;
    };

    // -- Kind badge -----------------------------------------------------------
    dc.gui.rect(
        panel_x + ROW_PAD_X,
        cursor_y + 5.0,
        right_w - ROW_PAD_X * 2.0,
        22.0,
        selected.kind.color(),
    );
    dc.gui.draw_text(
        dc.font,
        selected.kind.name(),
        [panel_x + ROW_PAD_X + 6.0, cursor_y + 9.0],
        10.0,
        [1.0, 1.0, 1.0, 1.0],
    );
    cursor_y += 22.0 + 10.0;

    // -- Hidden hint ----------------------------------------------------------
    if !selected.props.visible {
        dc.gui.draw_text(
            dc.font,
            "(hidden widget)",
            [panel_x + ROW_PAD_X, cursor_y + 6.0],
            10.0,
            AppColor::hex("#888888").to_linear_f32(),
        );
        cursor_y += ROW_H;
    }

    // -- Transform section ----------------------------------------------------
    draw_section(dc, panel_x, cursor_y, right_w, "Transform");
    cursor_y += SECTION_H;
    draw_number_row(
        dc,
        panel_x,
        cursor_y,
        right_w,
        mx,
        my,
        "X",
        &format!("{:.0}", selected.x),
    );
    cursor_y += ROW_H;
    draw_number_row(
        dc,
        panel_x,
        cursor_y,
        right_w,
        mx,
        my,
        "Y",
        &format!("{:.0}", selected.y),
    );
    cursor_y += ROW_H;
    draw_number_row(
        dc,
        panel_x,
        cursor_y,
        right_w,
        mx,
        my,
        "W",
        &format!("{:.0}", selected.w),
    );
    cursor_y += ROW_H;
    draw_number_row(
        dc,
        panel_x,
        cursor_y,
        right_w,
        mx,
        my,
        "H",
        &format!("{:.0}", selected.h),
    );
    cursor_y += ROW_H;

    // -- Properties section ---------------------------------------------------
    draw_section(dc, panel_x, cursor_y, right_w, "Properties");
    cursor_y += SECTION_H;

    // Visible toggle
    draw_row_label(dc, panel_x, cursor_y, "Visible");
    let toggle_x = panel_x + ROW_PAD_X + LABEL_W + 4.0;
    let hov_toggle =
        mx >= toggle_x && mx < toggle_x + 60.0 && my >= cursor_y && my < cursor_y + ROW_H;
    draw_button(
        dc,
        toggle_x,
        cursor_y + 3.0,
        60.0,
        ROW_H - 6.0,
        if selected.props.visible { "On" } else { "Off" },
        hov_toggle,
    );
    cursor_y += ROW_H;

    // Label / Hint
    let label_focused = scene.editing_field.as_deref() == Some("label");
    let label_value = if label_focused {
        scene.edit_buffer.as_str()
    } else {
        selected.props.label.as_str()
    };
    match selected.kind {
        WidgetKind::Label
        | WidgetKind::Button
        | WidgetKind::Checkbox
        | WidgetKind::ToggleSwitch
        | WidgetKind::Tooltip
        | WidgetKind::Toast => {
            draw_text_row(
                dc,
                panel_x,
                cursor_y,
                right_w,
                "Label",
                label_value,
                label_focused,
            );
            cursor_y += ROW_H;
        }
        WidgetKind::TextInput | WidgetKind::NumberInput => {
            draw_text_row(
                dc,
                panel_x,
                cursor_y,
                right_w,
                "Hint",
                label_value,
                label_focused,
            );
            cursor_y += ROW_H;
        }
        _ => {}
    }

    // Font size
    match selected.kind {
        WidgetKind::Label | WidgetKind::Button | WidgetKind::TextInput => {
            draw_info_row(
                dc,
                panel_x,
                cursor_y,
                right_w,
                "Size",
                &format!("{:.0}", selected.props.font_size),
            );
            cursor_y += ROW_H;
        }
        _ => {}
    }

    // Range
    match selected.kind {
        WidgetKind::Slider | WidgetKind::NumberInput | WidgetKind::ProgressBar => {
            draw_info_row(
                dc,
                panel_x,
                cursor_y,
                right_w,
                "Min",
                &format!("{:.0}", selected.props.min),
            );
            cursor_y += ROW_H;
            draw_info_row(
                dc,
                panel_x,
                cursor_y,
                right_w,
                "Max",
                &format!("{:.0}", selected.props.max),
            );
            cursor_y += ROW_H;
            draw_info_row(
                dc,
                panel_x,
                cursor_y,
                right_w,
                "Val",
                &format!("{:.0}", selected.props.value),
            );
            cursor_y += ROW_H;
        }
        _ => {}
    }

    // Color hex
    let color_focused = scene.editing_field.as_deref() == Some("color");
    let color_value = if color_focused {
        scene.edit_buffer.as_str()
    } else {
        selected.props.color_hex.as_str()
    };
    match selected.kind {
        WidgetKind::Panel | WidgetKind::Label | WidgetKind::Button => {
            draw_text_row(
                dc,
                panel_x,
                cursor_y,
                right_w,
                "Color",
                color_value,
                color_focused,
            );
            cursor_y += ROW_H;
        }
        _ => {}
    }

    // -- Delete button --------------------------------------------------------
    let del_y = cursor_y + 8.0;
    let del_x = panel_x + 10.0;
    let del_w = right_w - 20.0;
    let hov_del = mx >= del_x && mx < del_x + del_w && my >= del_y && my < del_y + 28.0;
    draw_button(dc, del_x, del_y, del_w, 28.0, "Delete Widget", hov_del);

    dc.gui.pop_clip();
}

// ── Background / border ───────────────────────────────────────────────────────

pub fn draw_background(dc: &mut DrawContext<'_, '_>, right_w: f32) {
    let (win_w, win_h) = dc.ctx.window_size;
    let right_x = win_w as f32 - right_w;
    dc.gui.rect(
        right_x,
        TOP_H,
        right_w,
        win_h as f32 - TOP_H,
        AppColor::hex("#252526").to_linear_f32(),
    );
}

pub fn draw_border(dc: &mut DrawContext<'_, '_>, right_w: f32) {
    let (win_w, win_h) = dc.ctx.window_size;
    let right_x = win_w as f32 - right_w;
    dc.gui
        .rect(right_x, TOP_H, 2.0, win_h as f32 - TOP_H, c_border());
}

// ── Draw helpers ─────────────────────────────────────────────────────────────

fn draw_section(dc: &mut DrawContext<'_, '_>, panel_x: f32, y: f32, right_w: f32, title: &str) {
    dc.gui.rect(
        panel_x,
        y,
        right_w,
        SECTION_H,
        AppColor::hex("#2D2D30").to_linear_f32(),
    );
    dc.gui.draw_text(
        dc.font,
        title,
        [panel_x + ROW_PAD_X, y + 4.0],
        10.0,
        AppColor::hex("#AAAAAA").to_linear_f32(),
    );
}

fn draw_row_label(dc: &mut DrawContext<'_, '_>, panel_x: f32, y: f32, label: &str) {
    dc.gui.draw_text(
        dc.font,
        label,
        [panel_x + ROW_PAD_X, y + 7.0],
        10.0,
        AppColor::hex("#CCCCCC").to_linear_f32(),
    );
}

fn draw_button(
    dc: &mut DrawContext<'_, '_>,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    label: &str,
    hovered: bool,
) {
    let bg = if hovered {
        AppColor::hex("#0078D4").to_linear_f32()
    } else {
        AppColor::hex("#3C3C3C").to_linear_f32()
    };
    dc.gui.rect(x, y, w, h, bg);
    dc.gui.draw_text(
        dc.font,
        label,
        [x + 4.0, y + (h - 10.0) * 0.5],
        10.0,
        [1.0, 1.0, 1.0, 1.0],
    );
}

fn draw_number_row(
    dc: &mut DrawContext<'_, '_>,
    panel_x: f32,
    y: f32,
    right_w: f32,
    mx: f32,
    my: f32,
    label: &str,
    value: &str,
) {
    draw_row_label(dc, panel_x, y, label);
    let val_x = panel_x + ROW_PAD_X + LABEL_W;
    let val_w = right_w - ROW_PAD_X * 2.0 - LABEL_W - BTN_W * 2.0 - 8.0;
    dc.gui.rect(
        val_x,
        y + 3.0,
        val_w,
        ROW_H - 6.0,
        AppColor::hex("#191919").to_linear_f32(),
    );
    dc.gui.draw_text(
        dc.font,
        value,
        [val_x + 4.0, y + 7.0],
        10.0,
        AppColor::hex("#DDDDDD").to_linear_f32(),
    );
    let minus_x = panel_x + right_w - ROW_PAD_X - BTN_W * 2.0 - 4.0;
    let plus_x = panel_x + right_w - ROW_PAD_X - BTN_W;
    let hov_m = mx >= minus_x && mx < minus_x + BTN_W && my >= y && my < y + ROW_H;
    let hov_p = mx >= plus_x && mx < plus_x + BTN_W && my >= y && my < y + ROW_H;
    draw_button(dc, minus_x, y + 3.0, BTN_W, ROW_H - 6.0, "-", hov_m);
    draw_button(dc, plus_x, y + 3.0, BTN_W, ROW_H - 6.0, "+", hov_p);
}

fn draw_text_row(
    dc: &mut DrawContext<'_, '_>,
    panel_x: f32,
    y: f32,
    right_w: f32,
    label: &str,
    value: &str,
    focused: bool,
) {
    draw_row_label(dc, panel_x, y, label);
    let val_x = panel_x + ROW_PAD_X + LABEL_W;
    let val_w = right_w - ROW_PAD_X * 2.0 - LABEL_W;
    let bg = if focused {
        AppColor::hex("#1E3A5F").to_linear_f32()
    } else {
        AppColor::hex("#191919").to_linear_f32()
    };
    dc.gui.rect(val_x, y + 3.0, val_w, ROW_H - 6.0, bg);
    if focused {
        // Highlight border
        dc.gui.rect(
            val_x,
            y + 3.0,
            val_w,
            1.0,
            AppColor::hex("#0078D4").to_linear_f32(),
        );
        dc.gui.rect(
            val_x,
            y + ROW_H - 4.0,
            val_w,
            1.0,
            AppColor::hex("#0078D4").to_linear_f32(),
        );
        dc.gui.rect(
            val_x,
            y + 3.0,
            1.0,
            ROW_H - 6.0,
            AppColor::hex("#0078D4").to_linear_f32(),
        );
        dc.gui.rect(
            val_x + val_w - 1.0,
            y + 3.0,
            1.0,
            ROW_H - 6.0,
            AppColor::hex("#0078D4").to_linear_f32(),
        );
    }
    // Show value with cursor appended when focused
    let display_value = if focused {
        let s = if value.len() > 17 {
            &value[value.len() - 17..]
        } else {
            value
        };
        let mut s = s.to_string();
        s.push('|');
        s
    } else {
        let s = if value.len() > 18 {
            &value[..18]
        } else {
            value
        };
        s.to_string()
    };
    let text_color = if focused {
        [1.0f32, 1.0, 1.0, 1.0]
    } else {
        AppColor::hex("#DDDDDD").to_linear_f32()
    };
    dc.gui.draw_text(
        dc.font,
        &display_value,
        [val_x + 4.0, y + 7.0],
        10.0,
        text_color,
    );
}

fn draw_info_row(
    dc: &mut DrawContext<'_, '_>,
    panel_x: f32,
    y: f32,
    right_w: f32,
    label: &str,
    value: &str,
) {
    draw_text_row(dc, panel_x, y, right_w, label, value, false);
}
