use ferrous_engine::{AppContext, Color as AppColor, DrawContext, KeyCode, MouseButton};
use ferrous_gui::Rect;
use ferrous_ui_core::{FieldKey, FieldKeyResult};

use crate::{
    c_border,
    panels::widget_editor::{self, BTN_W, HEADER_H, LABEL_W, ROW_H, ROW_PAD_X, SECTION_H},
    scene::SceneState,
    TOP_H,
    theme::{Theme, ThemeRegistry},
};

// ── Update (interaction) ──────────────────────────────────────────────────────

pub fn update(ctx: &mut AppContext, right_w: f32, scene: &mut SceneState, scroll_y: &mut f32, cp: &mut crate::color_picker::ColorPickerState) {
    let (win_w, win_h) = ctx.window_size;
    let panel_x = win_w as f32 - right_w;
    let panel_h = win_h as f32 - TOP_H;

    let (mx, my) = ctx.input.mouse_pos_f32();
    let mouse_in_panel = mx >= panel_x && mx < win_w as f32 && my >= TOP_H && my < TOP_H + panel_h;
    
    // Scroll handling
    if mouse_in_panel {
        let (_, sy) = ctx.input.scroll_delta();
        if sy != 0.0 {
            const SCROLL_SPEED: f32 = 30.0;
            // Limit scroll based on a rough estimate or fixed large value for now
            // Better: just let it scroll and draw logic will handle it
            *scroll_y = (*scroll_y - sy * SCROLL_SPEED).max(0.0);
        }
    }

    let clicked = ctx.input.button_just_pressed(MouseButton::Left);

    // ── Keyboard handling for focused text field ─────────────────────────
    if scene.editing_field.is_some() {
        let dt = ctx.time.delta;
        let ctrl = ctx.input.is_key_down(KeyCode::ControlLeft)
            || ctx.input.is_key_down(KeyCode::ControlRight);

        // Blink timer
        scene.edit_state.tick(dt);

        // Hold-backspace via polling
        let bs_just = ctx.input.just_pressed(KeyCode::Backspace);
        let bs_down = ctx.input.is_key_down(KeyCode::Backspace);
        if bs_just {
            scene
                .edit_state
                .on_key(FieldKey::Backspace, &mut scene.edit_buffer);
        } else if scene.edit_state.poll_backspace_repeat(dt, bs_down) {
            scene
                .edit_state
                .on_key(FieldKey::Backspace, &mut scene.edit_buffer);
        }

        // Special keys
        let check = |kc: KeyCode| ctx.input.just_pressed(kc);
        let shift =
            ctx.input.is_key_down(KeyCode::ShiftLeft) || ctx.input.is_key_down(KeyCode::ShiftRight);
        let fkey: Option<FieldKey> = if ctrl && check(KeyCode::KeyA) {
            Some(FieldKey::SelectAll)
        } else if ctrl && shift && check(KeyCode::ArrowLeft) {
            Some(FieldKey::CtrlShiftArrowLeft)
        } else if ctrl && shift && check(KeyCode::ArrowRight) {
            Some(FieldKey::CtrlShiftArrowRight)
        } else if ctrl && check(KeyCode::ArrowLeft) {
            Some(FieldKey::CtrlArrowLeft)
        } else if ctrl && check(KeyCode::ArrowRight) {
            Some(FieldKey::CtrlArrowRight)
        } else if shift && check(KeyCode::ArrowLeft) {
            Some(FieldKey::ShiftArrowLeft)
        } else if shift && check(KeyCode::ArrowRight) {
            Some(FieldKey::ShiftArrowRight)
        } else if shift && check(KeyCode::Home) {
            Some(FieldKey::ShiftHome)
        } else if shift && check(KeyCode::End) {
            Some(FieldKey::ShiftEnd)
        } else if check(KeyCode::ArrowLeft) {
            Some(FieldKey::ArrowLeft)
        } else if check(KeyCode::ArrowRight) {
            Some(FieldKey::ArrowRight)
        } else if check(KeyCode::Home) {
            Some(FieldKey::Home)
        } else if check(KeyCode::End) {
            Some(FieldKey::End)
        } else if check(KeyCode::Delete) {
            Some(FieldKey::Delete)
        } else if check(KeyCode::Enter) {
            Some(FieldKey::Enter)
        } else if check(KeyCode::Escape) {
            Some(FieldKey::Escape)
        } else {
            None
        };

        let mut commit_or_cancel: Option<bool> = None;
        if let Some(key) = fkey {
            match scene.edit_state.on_key(key, &mut scene.edit_buffer) {
                FieldKeyResult::Submit => commit_or_cancel = Some(true),
                FieldKeyResult::Cancel => commit_or_cancel = Some(false),
                _ => {}
            }
        }

        // Typed chars (not when ctrl is held)
        if !ctrl {
            let chars: Vec<char> = ctx.input.typed_chars().to_vec();
            for c in chars {
                scene.edit_state.on_char(c, &mut scene.edit_buffer);
            }
        }

        // Click outside panel also commits
        if clicked && !mouse_in_panel {
            commit_or_cancel = Some(true);
        }

        // Click inside panel while editing: commit and let the click fall through
        // so the new field can be focused in the same frame.
        let click_in_panel_while_editing = clicked && mouse_in_panel;
        if click_in_panel_while_editing {
            commit_or_cancel = Some(true);
        }

        // Apply commit / cancel
        if let Some(commit) = commit_or_cancel {
            if commit {
                if let Some(field) = scene.editing_field.clone() {
                    let buf = scene.edit_buffer.clone();
                    widget_editor::commit_field(scene, &field, buf);
                }
            }
            scene.editing_field = None;
            scene.edit_buffer.clear();
            scene.edit_state.blur();
        }

        // If the click was inside the panel, fall through to process the new click.
        // Otherwise swallow input so the canvas doesn't react.
        if !click_in_panel_while_editing {
            return;
        }
    }

    if !mouse_in_panel || !clicked {
        return;
    }

    let Some(_selected_id) = scene.selected_id else {
        return;
    };
    // prepare a small snapshot of the selected widget so we don't have to
    // borrow `scene` multiple times later.  this mirrors the logic in draw().
    let snapshot = scene
        .selected()
        .map(|s| (s.id, s.kind, s.x, s.y, s.w, s.h, s.props.visible));
    let Some((_, _selected_kind, sel_x, sel_y, sel_w, sel_h, sel_visible)) = snapshot else {
        // the selected id no longer exists?  nothing to do.
        return;
    };

    let hit_x = mx - panel_x;

    let mut cursor_y = TOP_H + HEADER_H;
    cursor_y += 22.0 + 10.0; // kind badge

    // skip hidden hint row if the widget is invisible
    if !sel_visible {
        cursor_y += ROW_H;
    }

    // Transform +/- buttons and Properties buttons are handled in draw().
    // update() only needs to detect clicks on the value portion of the
    // numeric fields and start editing.  we abstract that logic so future
    // rows (or other panels) can reuse the same behaviour instead of
    // duplicating the hit‑testing code.

    // compute constants used by draw_number_row below as well
    let val_x = ROW_PAD_X + LABEL_W;
    let val_w = right_w - ROW_PAD_X * 2.0 - LABEL_W - BTN_W * 2.0 - 8.0;

    // small helper, returns true if focus was claimed and we should bail out
    // early (so the click can still fall through to +/- buttons)
    fn try_focus_field(
        clicked: bool,
        my: f32,
        y: f32,
        hit_x: f32,
        val_x: f32,
        val_w: f32,
        current: String,
        key: &str,
        scene: &mut SceneState,
    ) -> bool {
        if clicked && my >= y && my < y + ROW_H && hit_x >= val_x && hit_x < val_x + val_w {
            scene.editing_field = Some(key.to_string());
            scene.edit_buffer = current;
            scene.edit_state.focus();
            scene.edit_state.cursor_pos = scene.edit_buffer.len();
            return true;
        }
        false
    }

    // row Y positions start after the "Transform" header
    let mut ty = cursor_y + SECTION_H;

    // Apply scroll offset to mouse Y for hit detection
    let my_scrolled = my + *scroll_y;

    if clicked {
        if try_focus_field(
            clicked,
            my_scrolled,
            ty,
            hit_x,
            val_x,
            val_w,
            format!("{:.0}", sel_x),
            "tx",
            scene,
        ) {
            return;
        }
        ty += ROW_H;
        if try_focus_field(
            clicked,
            my_scrolled,
            ty,
            hit_x,
            val_x,
            val_w,
            format!("{:.0}", sel_y),
            "ty",
            scene,
        ) {
            return;
        }
        ty += ROW_H;
        if try_focus_field(
            clicked,
            my_scrolled,
            ty,
            hit_x,
            val_x,
            val_w,
            format!("{:.0}", sel_w),
            "tw",
            scene,
        ) {
            return;
        }
        ty += ROW_H;
        if try_focus_field(
            clicked,
            my_scrolled,
            ty,
            hit_x,
            val_x,
            val_w,
            format!("{:.0}", sel_h),
            "th",
            scene,
        ) {
            return;
        }
    }

    // advance cursor_y so property rows are positioned correctly after the
    // transform block (same as before)
    cursor_y += SECTION_H + ROW_H * 4.0;

    // -- Properties section: text fields only --------------------------------
    widget_editor::update_properties(ctx, scene, panel_x, right_w, mx, my_scrolled, my, hit_x, &mut cursor_y, cp);
}

// ── Draw (immediate mode) ─────────────────────────────────────────────────────
pub fn draw(dc: &mut DrawContext<'_, '_>, right_w: f32, scene: &mut SceneState, theme: &Theme, scroll_y: f32, cp: &crate::color_picker::ColorPickerState) {
    let (win_w, win_h) = dc.ctx.window_size;
    let panel_x = win_w as f32 - right_w;

    // 1. Background for the entire panel area
    draw_background(dc, right_w, theme);
    draw_border(dc, right_w, theme);

    // 2. Fixed Header bar
    dc.gui.rect(
        panel_x,
        TOP_H,
        right_w,
        HEADER_H,
        ThemeRegistry::hex_to_f32(&theme.colors.secondary),
    );
    dc.gui.draw_text(
        dc.font,
        "PROPERTIES",
        [panel_x + ROW_PAD_X, TOP_H + 10.0],
        10.0,
        ThemeRegistry::hex_to_f32(&theme.colors.text_dim),
    );

    // 3. Scrollable content area
    let content_h = win_h as f32 - TOP_H - HEADER_H;
    dc.gui.push_clip(Rect::new(panel_x, TOP_H + HEADER_H, right_w, content_h));

    // Extract snapshot data from selected widget
    let snapshot = scene
        .selected()
        .map(|s| (s.id, s.kind, s.x, s.y, s.w, s.h, s.props.visible));

    let Some((selected_id, selected_kind, sel_x, sel_y, sel_w, sel_h, sel_visible)) = snapshot
    else {
        dc.gui.draw_text(
            dc.font,
            "No widget selected",
            [panel_x + ROW_PAD_X, TOP_H + HEADER_H + 10.0 - scroll_y],
            10.0,
            ThemeRegistry::hex_to_f32(&theme.colors.text_dim),
        );
        dc.gui.pop_clip();
        return;
    };

    // All content inside the clip should be offset by scroll_y
    let mut cursor_y = TOP_H + HEADER_H - scroll_y;

    // -- Kind badge -----------------------------------------------------------
    dc.gui.rect(
        panel_x + ROW_PAD_X,
        cursor_y + 5.0,
        right_w - ROW_PAD_X * 2.0,
        22.0,
        selected_kind.color(),
    );
    dc.gui.draw_text(
        dc.font,
        selected_kind.display_name(),
        [panel_x + ROW_PAD_X + 6.0, cursor_y + 9.0],
        10.0,
        ThemeRegistry::hex_to_f32(&theme.colors.text),
    );
    cursor_y += 22.0 + 10.0;

    // -- Hidden hint ----------------------------------------------------------
    if !sel_visible {
        dc.gui.draw_text(
            dc.font,
            "(hidden widget)",
            [panel_x + ROW_PAD_X, cursor_y + 6.0],
            10.0,
            ThemeRegistry::hex_to_f32(&theme.colors.text_dim),
        );
        cursor_y += ROW_H;
    }

    // -- Transform section ----------------------------------------------------
    widget_editor::draw_section(dc, panel_x, cursor_y, right_w, "Transform", theme);
    cursor_y += SECTION_H;

    let (mx, my) = dc.ctx.input.mouse_pos_f32();
    let clicked = dc.ctx.input.button_just_pressed(MouseButton::Left);
    let minus_x = panel_x + right_w - ROW_PAD_X - BTN_W * 2.0 - 4.0;
    let plus_x = panel_x + right_w - ROW_PAD_X - BTN_W;

    // X
    // if the user is actively editing one of the transform fields, show the
    // contents of the edit buffer instead of the live value so typing is
    // visible.
    let x_buf = if scene.editing_field.as_deref() == Some("tx") {
        scene.edit_buffer.clone()
    } else {
        format!("{:.0}", sel_x)
    };
    let x_focused = scene.editing_field.as_deref() == Some("tx");
    widget_editor::draw_number_row(
        dc,
        panel_x,
        cursor_y,
        right_w,
        mx,
        my,
        "X",
        &x_buf,
        x_focused,
        x_focused,
        scene.edit_state.cursor_pos,
        scene.edit_state.selection(),
        scene.edit_state.all_selected,
        theme,
    );
    if dc.gui.button(
        dc.font,
        minus_x,
        cursor_y + 3.0,
        BTN_W,
        ROW_H - 6.0,
        "-",
        mx,
        my,
        clicked,
    ) {
        if let Some(w) = scene.widgets.iter_mut().find(|w| w.id == selected_id) {
            w.x -= 1.0;
        }
    }
    if dc.gui.button(
        dc.font,
        plus_x,
        cursor_y + 3.0,
        BTN_W,
        ROW_H - 6.0,
        "+",
        mx,
        my,
        clicked,
    ) {
        if let Some(w) = scene.widgets.iter_mut().find(|w| w.id == selected_id) {
            w.x += 1.0;
        }
    }
    cursor_y += ROW_H;

    // Y
    let y_buf = if scene.editing_field.as_deref() == Some("ty") {
        scene.edit_buffer.clone()
    } else {
        format!("{:.0}", sel_y)
    };
    let y_focused = scene.editing_field.as_deref() == Some("ty");
    widget_editor::draw_number_row(
        dc,
        panel_x,
        cursor_y,
        right_w,
        mx,
        my,
        "Y",
        &y_buf,
        y_focused,
        y_focused,
        scene.edit_state.cursor_pos,
        scene.edit_state.selection(),
        scene.edit_state.all_selected,
        theme,
    );
    if dc.gui.button(
        dc.font,
        minus_x,
        cursor_y + 3.0,
        BTN_W,
        ROW_H - 6.0,
        "-",
        mx,
        my,
        clicked,
    ) {
        if let Some(w) = scene.widgets.iter_mut().find(|w| w.id == selected_id) {
            w.y -= 1.0;
        }
    }
    if dc.gui.button(
        dc.font,
        plus_x,
        cursor_y + 3.0,
        BTN_W,
        ROW_H - 6.0,
        "+",
        mx,
        my,
        clicked,
    ) {
        if let Some(w) = scene.widgets.iter_mut().find(|w| w.id == selected_id) {
            w.y += 1.0;
        }
    }
    cursor_y += ROW_H;

    // W
    let w_buf = if scene.editing_field.as_deref() == Some("tw") {
        scene.edit_buffer.clone()
    } else {
        format!("{:.0}", sel_w)
    };
    let w_focused = scene.editing_field.as_deref() == Some("tw");
    widget_editor::draw_number_row(
        dc,
        panel_x,
        cursor_y,
        right_w,
        mx,
        my,
        "W",
        &w_buf,
        w_focused,
        w_focused, // simplified for now
        scene.edit_state.cursor_pos,
        scene.edit_state.selection(),
        scene.edit_state.all_selected,
        theme,
    );
    if dc.gui.button(
        dc.font,
        minus_x,
        cursor_y + 3.0,
        BTN_W,
        ROW_H - 6.0,
        "-",
        mx,
        my,
        clicked,
    ) {
        if let Some(w) = scene.widgets.iter_mut().find(|w| w.id == selected_id) {
            w.w = (w.w - 1.0).max(8.0);
        }
    }
    if dc.gui.button(
        dc.font,
        plus_x,
        cursor_y + 3.0,
        BTN_W,
        ROW_H - 6.0,
        "+",
        mx,
        my,
        clicked,
    ) {
        if let Some(w) = scene.widgets.iter_mut().find(|w| w.id == selected_id) {
            w.w += 1.0;
        }
    }
    cursor_y += ROW_H;

    // H
    let h_buf = if scene.editing_field.as_deref() == Some("th") {
        scene.edit_buffer.clone()
    } else {
        format!("{:.0}", sel_h)
    };
    let h_focused = scene.editing_field.as_deref() == Some("th");
    widget_editor::draw_number_row(
        dc,
        panel_x,
        cursor_y,
        right_w,
        mx,
        my,
        "H",
        &h_buf,
        h_focused,
        h_focused,
        scene.edit_state.cursor_pos,
        scene.edit_state.selection(),
        scene.edit_state.all_selected,
        theme,
    );
    if dc.gui.button(
        dc.font,
        minus_x,
        cursor_y + 3.0,
        BTN_W,
        ROW_H - 6.0,
        "-",
        mx,
        my,
        clicked,
    ) {
        if let Some(w) = scene.widgets.iter_mut().find(|w| w.id == selected_id) {
            w.h = (w.h - 1.0).max(8.0);
        }
    }
    if dc.gui.button(
        dc.font,
        plus_x,
        cursor_y + 3.0,
        BTN_W,
        ROW_H - 6.0,
        "+",
        mx,
        my,
        clicked,
    ) {
        if let Some(w) = scene.widgets.iter_mut().find(|w| w.id == selected_id) {
            w.h += 1.0;
        }
    }
    cursor_y += ROW_H;

    // -- Properties section ---------------------------------------------------
    widget_editor::draw_properties(dc, scene, panel_x, right_w, cursor_y, theme, cp);

    dc.gui.pop_clip();
}

// ── Background / border ───────────────────────────────────────────────────────

pub fn draw_background(dc: &mut DrawContext<'_, '_>, right_w: f32, theme: &Theme) {
    let (win_w, win_h) = dc.ctx.window_size;
    let right_x = win_w as f32 - right_w;
    dc.gui.rect(
        right_x,
        TOP_H,
        right_w,
        win_h as f32 - TOP_H,
        ThemeRegistry::hex_to_f32(&theme.colors.surface),
    );
}

pub fn draw_border(dc: &mut DrawContext<'_, '_>, right_w: f32, theme: &Theme) {
    let (win_w, win_h) = dc.ctx.window_size;
    let right_x = win_w as f32 - right_w;
    dc.gui
        .rect(right_x, TOP_H, 2.0, win_h as f32 - TOP_H, c_border(theme));
}
