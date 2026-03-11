// ── Widget Editor ─────────────────────────────────────────────────────────────
//
// Centralizes all per-widget rendering (canvas preview) and per-widget
// property editing (right-panel inspector rows).
//
// Add new widget kinds here — canvas.rs and right_panel.rs stay generic.

use ferrous_app::{AppContext, Color as AppColor, DrawContext, MouseButton};
use ferrous_gui::GuiBatch;
use ferrous_ui_core::Rect;

use crate::scene::{HAlign, PlacedWidget, SceneState, VAlign, WidgetKind};

// ── Layout constants (mirrored from right_panel so both stay in sync) ─────────
pub(crate) const HEADER_H: f32 = 34.0;
pub(crate) const SECTION_H: f32 = 20.0;
pub(crate) const ROW_H: f32 = 26.0;
pub(crate) const ROW_PAD_X: f32 = 10.0;
pub(crate) const BTN_W: f32 = 22.0;
pub(crate) const LABEL_W: f32 = 54.0;

/// Return default RGBA colors for a text-row based on focus state.
/// Return default RGBA colors for a text-row based on focus state and the row's
/// label.  Historically every text row would simply turn its text white when
/// focused, but the user asked for the ability to only apply that treatment to
/// *certain* labelled rows (e.g. the transform fields).  To support that we
/// test the provided `label` against a small whitelist and only produce a white
/// text color when the row is both focused *and* its label is one of the
/// special ones.  Callers who don't care about the restriction can simply pass
/// a label that always matches (or modify the list below).
fn default_row_colors(
    label: &str,
    focused: bool,
) -> ([f32; 4], [f32; 4], Option<[f32; 4]>, [f32; 4]) {
    // AppColor is already imported above from ferrous_app
    // labels for which we want the text to go white when focused
    const WHITE_ON_FOCUS: &[&str] = &[
        "X", "Y", "W", "H",     // transform fields
        "Label", // text widget label property
        "Color", // colour hex fields
                 // add more strings here as needed
    ];

    let highlight_text = focused && WHITE_ON_FOCUS.contains(&label);
    let text_col = if highlight_text {
        [1.0; 4]
    } else {
        // un-focused or not in whitelist: keep the standard grey
        AppColor::hex("#DDDDDD").to_linear_f32()
    };
    let bg_col = if focused {
        AppColor::hex("#1E3A5F").to_linear_f32()
    } else {
        AppColor::hex("#191919").to_linear_f32()
    };
    let border_col = if focused {
        Some(AppColor::hex("#0078D4").to_linear_f32())
    } else {
        None
    };
    let sel_col = [0.0f32, 0.47, 0.83, 0.35];
    (text_col, bg_col, border_col, sel_col)
}

// ─────────────────────────────────────────────────────────────────────────────
// Canvas: draw one placed widget
// ─────────────────────────────────────────────────────────────────────────────

/// Draws a single placed widget at its screen-space position.
/// `sx/sy/sw/sh` are already in screen pixels (world→screen already applied).
/// The outer canvas clip is assumed to already be active.
/// `active_radius_corner`: if this widget has a radius drag in progress, the corner index (0-3).
pub fn draw_widget(
    dc: &mut DrawContext<'_, '_>,
    w: &PlacedWidget,
    sx: f32,
    sy: f32,
    sw: f32,
    sh: f32,
    is_selected: bool,
    active_radius_corner: Option<usize>,
) {
    match w.kind {
        WidgetKind::Label => draw_label(dc, w, sx, sy, sw, sh, is_selected),
        WidgetKind::Button => draw_button(dc, w, sx, sy, sw, sh, is_selected, active_radius_corner),
        _ => draw_generic(dc, w, sx, sy, sw, sh, is_selected),
    }

    // Selection handles (corners + midpoints) — drawn outside any inner clip
    if is_selected {
        draw_selection_handles(dc, sx, sy, sw, sh);
    }
}

// ── Per-kind canvas renderers ─────────────────────────────────────────────────

fn draw_label(
    dc: &mut DrawContext<'_, '_>,
    w: &PlacedWidget,
    sx: f32,
    sy: f32,
    sw: f32,
    sh: f32,
    is_selected: bool,
) {
    let zoom = sw / w.w; // recover zoom from screen vs world size
    let label_text = if w.props.label.is_empty() {
        "Label"
    } else {
        &w.props.label
    };
    let font_size = (w.props.font_size * zoom).max(4.0);

    // Resolve alignment using real font metrics
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
        [0.1, 0.1, 0.1, 1.0],
    );
    dc.gui.pop_clip();

    // Dashed selection outline (no fill for labels)
    if is_selected {
        let bw = 1.5_f32;
        let sel_col = [1.0_f32, 0.6, 0.0, 0.85];
        dc.gui.rect(sx, sy, sw, bw, sel_col);
        dc.gui.rect(sx, sy + sh - bw, sw, bw, sel_col);
        dc.gui.rect(sx, sy, bw, sh, sel_col);
        dc.gui.rect(sx + sw - bw, sy, bw, sh, sel_col);
        draw_pivot_overlay(dc, w, sx, sy, sw, sh);
    }
}

/// Previews a `Button` widget exactly as `Button::draw()` would render it:
/// primary-color fill, on_primary text, simulated border-radius via inset.
fn draw_button(
    dc: &mut DrawContext<'_, '_>,
    w: &PlacedWidget,
    sx: f32,
    sy: f32,
    sw: f32,
    sh: f32,
    is_selected: bool,
    active_radius_corner: Option<usize>,
) {
    // Theme defaults (dark theme: primary=#6C63FF, on_primary=white, border_radius=6)
    let bg_color = parse_hex_or(&w.props.bg_color_hex, [0.424, 0.388, 1.0, 1.0]);
    let text_color = parse_hex_or(&w.props.text_color_hex, [1.0, 1.0, 1.0, 1.0]);

    // Simulated border-radius: draw a slightly inset rect in the corner color
    // to fake rounded corners (GUIMaker uses plain rects, not rounded quads).
    let zoom = sw / w.w;
    let rr = w.props.border_radii;
    // Scale each corner radius by zoom, clamped to half the smallest dimension
    let half = (sw.min(sh)) * 0.5;
    let radii = [
        (rr[0] * zoom).min(half),
        (rr[1] * zoom).min(half),
        (rr[2] * zoom).min(half),
        (rr[3] * zoom).min(half),
    ];

    // Use rect_radii — same path the real Button::draw() takes via RenderCommand::Quad
    dc.gui.rect_radii(sx, sy, sw, sh, bg_color, radii);

    // Centered label
    let label = if w.props.label.is_empty() {
        "Button"
    } else {
        &w.props.label
    };
    let font_size = (w.props.font_size * zoom).clamp(8.0, sh * 0.7);
    dc.gui.push_clip(ferrous_ui_core::Rect::new(
        sx + 2.0,
        sy + 2.0,
        sw - 4.0,
        sh - 4.0,
    ));
    // Use real font metrics + alignment
    let text_w = GuiBatch::measure_text(dc.font, label, font_size);
    let align = w.props.text_align();
    let text_x = align.resolve_x(sx, sw, text_w, 8.0);
    let text_y = align.resolve_y(sy, sh, font_size, 4.0);
    dc.gui
        .draw_text(dc.font, label, [text_x, text_y], font_size, text_color);
    dc.gui.pop_clip();

    // Selection outline: only when selected, drawn as a rounded rect slightly
    // outside the button so it doesn't overdraw the rounded corners.
    if is_selected {
        let bw = 2.0_f32;
        let sel_col = [1.0_f32, 0.8, 0.0, 1.0];
        // Expand by bw on each side so the outline wraps the rounded fill cleanly.
        dc.gui.rect_radii(
            sx - bw,
            sy - bw,
            sw + bw * 2.0,
            sh + bw * 2.0,
            sel_col,
            [radii[0] + bw, radii[1] + bw, radii[2] + bw, radii[3] + bw],
        );
        // Re-draw the fill on top so the interior stays clean.
        dc.gui.rect_radii(sx, sy, sw, sh, bg_color, radii);
        // Re-draw the label on top of the outline overdraw.
        dc.gui.push_clip(ferrous_ui_core::Rect::new(
            sx + 2.0,
            sy + 2.0,
            sw - 4.0,
            sh - 4.0,
        ));
        dc.gui
            .draw_text(dc.font, label, [text_x, text_y], font_size, text_color);
        dc.gui.pop_clip();

        // ── Radius drag handles — one per corner ──────────────────────────────
        // Cyan squares positioned on the 45° diagonal at the current radius.
        // The active (dragged) handle is shown larger and bright white.
        let rh_size = 8.0_f32; // normal outer size
        let rh_inner = 4.0_f32; // inner fill size
        let rh_col: [f32; 4] = [0.0, 0.85, 0.85, 1.0]; // cyan
        let rh_col_active: [f32; 4] = [1.0, 1.0, 1.0, 1.0]; // white when dragging
        let rh_fill: [f32; 4] = [0.08, 0.08, 0.12, 1.0]; // dark interior
        let corner_names = ["TL", "TR", "BL", "BR"];
        for corner in 0..4_usize {
            let (hx, hy) = radius_handle_pos(sx, sy, sw, sh, &radii, corner);
            let is_active = active_radius_corner == Some(corner);
            let outer = if is_active { rh_size + 3.0 } else { rh_size };
            let col = if is_active { rh_col_active } else { rh_col };
            // Outer square
            dc.gui
                .rect(hx - outer * 0.5, hy - outer * 0.5, outer, outer, col);
            // Inner dark fill
            dc.gui.rect(
                hx - rh_inner * 0.5,
                hy - rh_inner * 0.5,
                rh_inner,
                rh_inner,
                rh_fill,
            );
            // Live value label next to the active handle
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
    // Note: selection handles are drawn by draw_widget() after this call.
}

/// Returns the screen-space position of the radius drag handle for a given
/// corner index (0=TL, 1=TR, 2=BR, 3=BL).
/// The handle sits on the 45° diagonal at exactly `radius` pixels from the corner.
pub(crate) fn radius_handle_pos(
    sx: f32,
    sy: f32,
    sw: f32,
    sh: f32,
    radii: &[f32; 4],
    corner: usize,
) -> (f32, f32) {
    // Shader radii order: [0]=TL, [1]=TR, [2]=BL, [3]=BR
    // Each handle sits on the 45° inward diagonal at `r` pixels from its corner.
    let frac = std::f32::consts::FRAC_1_SQRT_2; // 1/√2 ≈ 0.707
    let r = radii[corner];
    match corner {
        0 => (sx + r * frac, sy + r * frac),           // TL: right+down
        1 => (sx + sw - r * frac, sy + r * frac),      // TR: left+down
        2 => (sx + r * frac, sy + sh - r * frac),      // BL: right+up
        _ => (sx + sw - r * frac, sy + sh - r * frac), // BR: left+up
    }
}

/// Parse a hex color string like "#6C63FF" into a linear [f32;4],
/// falling back to `default` if the string is empty or invalid.
fn parse_hex_or(hex: &str, default: [f32; 4]) -> [f32; 4] {
    let s = hex.trim();
    if s.is_empty() {
        return default;
    }
    let s = s.trim_start_matches('#');
    if s.len() < 6 {
        return default;
    }
    let parse = |i: usize| u8::from_str_radix(&s[i..i + 2], 16).ok();
    match (parse(0), parse(2), parse(4)) {
        (Some(r), Some(g), Some(b)) => {
            let a = if s.len() >= 8 {
                parse(6).unwrap_or(255)
            } else {
                255
            };
            ferrous_app::Color::from_rgba8(r, g, b, a).to_linear_f32()
        }
        _ => default,
    }
}

/// Draws the pivot/anchor point visualizer over a selected widget.
/// Shows crosshair lines at the anchor position and a diamond at the point.
/// Only draws the axes that are in `Custom` mode.
fn draw_pivot_overlay(
    dc: &mut DrawContext<'_, '_>,
    w: &PlacedWidget,
    sx: f32,
    sy: f32,
    sw: f32,
    sh: f32,
) {
    let h_custom = matches!(w.props.label_h_align, HAlign::Custom { .. });
    let v_custom = matches!(w.props.label_v_align, VAlign::Custom { .. });
    if !h_custom && !v_custom {
        return;
    }

    // Compute anchor position in screen space
    let anchor_x = if h_custom {
        if w.props.label_h_custom_pct {
            sx + sw * (w.props.label_h_custom / 100.0)
        } else {
            sx + w.props.label_h_custom * (sw / w.w)
        }
    } else {
        sx + sw * 0.5
    };
    let anchor_y = if v_custom {
        if w.props.label_v_custom_pct {
            sy + sh * (w.props.label_v_custom / 100.0)
        } else {
            sy + w.props.label_v_custom * (sh / w.h)
        }
    } else {
        sy + sh * 0.5
    };

    let line_col: [f32; 4] = [0.98, 0.78, 0.12, 0.70]; // amber
    let dot_col: [f32; 4] = [1.00, 0.92, 0.23, 1.00];
    let lw = 1.0_f32;

    // Vertical crosshair line
    if h_custom {
        dc.gui.rect(anchor_x - lw * 0.5, sy, lw, sh, line_col);
    }
    // Horizontal crosshair line
    if v_custom {
        dc.gui.rect(sx, anchor_y - lw * 0.5, sw, lw, line_col);
    }

    // Diamond at anchor (rotated square via two overlapping rects)
    let d = 5.0_f32;
    // Draw as a small cross-diamond: 4 triangles approximated with two rects
    dc.gui
        .rect(anchor_x - d, anchor_y - lw, d * 2.0, lw * 2.0, dot_col);
    dc.gui
        .rect(anchor_x - lw, anchor_y - d, lw * 2.0, d * 2.0, dot_col);
    // Solid center
    let r = 3.0_f32;
    dc.gui
        .rect(anchor_x - r, anchor_y - r, r * 2.0, r * 2.0, dot_col);

    // Small label showing value
    let lbl_col: [f32; 4] = [1.0, 0.95, 0.4, 1.0];
    if h_custom && v_custom {
        let lbl = format!(
            "({:.0}{}, {:.0}{})",
            w.props.label_h_custom,
            if w.props.label_h_custom_pct {
                "%"
            } else {
                "px"
            },
            w.props.label_v_custom,
            if w.props.label_v_custom_pct {
                "%"
            } else {
                "px"
            }
        );
        dc.gui.draw_text(
            dc.font,
            &lbl,
            [anchor_x + 6.0, anchor_y + 4.0],
            9.0,
            lbl_col,
        );
    } else if h_custom {
        let lbl = format!(
            "{:.0}{}",
            w.props.label_h_custom,
            if w.props.label_h_custom_pct {
                "%"
            } else {
                "px"
            }
        );
        dc.gui
            .draw_text(dc.font, &lbl, [anchor_x + 4.0, sy + 3.0], 9.0, lbl_col);
    } else {
        let lbl = format!(
            "{:.0}{}",
            w.props.label_v_custom,
            if w.props.label_v_custom_pct {
                "%"
            } else {
                "px"
            }
        );
        dc.gui
            .draw_text(dc.font, &lbl, [sx + 3.0, anchor_y + 3.0], 9.0, lbl_col);
    }
}

fn draw_generic(
    dc: &mut DrawContext<'_, '_>,
    w: &PlacedWidget,
    sx: f32,
    sy: f32,
    sw: f32,
    sh: f32,
    is_selected: bool,
) {
    let zoom = sw / w.w;

    // Fill
    let mut col = w.kind.color();
    col[3] = if is_selected { 0.85 } else { 0.65 };
    dc.gui.rect(sx, sy, sw, sh, col);

    // Clipped text content
    dc.gui.push_clip(Rect::new(sx, sy, sw, sh));
    if sw > 20.0 && sh > 12.0 {
        let label = if w.props.label.is_empty() {
            w.kind.display_name()
        } else {
            &w.props.label
        };
        let font_size = (w.props.font_size * zoom).clamp(8.0, 32.0);
        let text_x = sx + 4.0;
        let text_y = sy + (sh - font_size) * 0.5;
        dc.gui.draw_text(
            dc.font,
            label,
            [text_x, text_y],
            font_size,
            [1.0, 1.0, 1.0, 0.9],
        );
    }
    dc.gui.pop_clip();

    // Border (outside clip so it's never trimmed)
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

fn draw_selection_handles(dc: &mut DrawContext<'_, '_>, sx: f32, sy: f32, sw: f32, sh: f32) {
    let hs = 6.0_f32;
    let hh = hs * 0.5;
    let mid_sx = sx + sw * 0.5;
    let mid_sy = sy + sh * 0.5;
    let handle_col = [1.0_f32, 0.8, 0.0, 1.0];
    let handle_bg = [0.1_f32, 0.1, 0.1, 1.0];
    for (hx, hy) in [
        (sx - hh, sy - hh),
        (mid_sx - hh, sy - hh),
        (sx + sw - hh, sy - hh),
        (sx + sw - hh, mid_sy - hh),
        (sx + sw - hh, sy + sh - hh),
        (mid_sx - hh, sy + sh - hh),
        (sx - hh, sy + sh - hh),
        (sx - hh, mid_sy - hh),
    ] {
        dc.gui.rect(hx, hy, hs, hs, handle_col);
        dc.gui
            .rect(hx + 1.0, hy + 1.0, hs - 2.0, hs - 2.0, handle_bg);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Inspector panel: update (interaction) for widget properties
// ─────────────────────────────────────────────────────────────────────────────

/// Handles clicks on text-field rows in the Properties section (label, font
/// size, color).  Button interactions (visible toggle, delete) are handled
/// by `draw_properties()` via `GuiBatch::button()`.
///
/// `cursor_y` must start at the first row of the Properties section
/// (already past the section header added by `draw_properties`).
pub fn update_properties(
    ctx: &mut AppContext,
    scene: &mut SceneState,
    _panel_x: f32,
    _right_w: f32,
    _mx: f32,
    my: f32,
    hit_x: f32,
    cursor_y: &mut f32,
) {
    // Mirror draw_properties: skip the "Properties" section header first.
    *cursor_y += SECTION_H;

    let clicked = ctx.input.button_just_pressed(MouseButton::Left);

    let Some(selected_id) = scene.selected_id else {
        return;
    };

    let kind = scene
        .widgets
        .iter()
        .find(|w| w.id == selected_id)
        .map(|w| w.kind);

    let Some(kind) = kind else {
        return;
    };

    let val_x = LABEL_W + ROW_PAD_X;

    // Visible toggle row — interaction handled by GuiBatch::button() in draw_properties.
    *cursor_y += ROW_H;

    if !clicked {
        return;
    }

    // Label / Hint — click on text field to focus it
    match kind {
        WidgetKind::Label
        | WidgetKind::Button
        | WidgetKind::Checkbox
        | WidgetKind::ToggleSwitch
        | WidgetKind::Tooltip
        | WidgetKind::Toast
        | WidgetKind::TextInput
        | WidgetKind::NumberInput => {
            if my >= *cursor_y && my < *cursor_y + ROW_H && hit_x >= val_x {
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
            *cursor_y += ROW_H;
        }
        _ => {}
    }

    // Font size — click on text field to focus it
    match kind {
        WidgetKind::Label | WidgetKind::Button | WidgetKind::TextInput => {
            if my >= *cursor_y && my < *cursor_y + ROW_H && hit_x >= val_x {
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
            *cursor_y += ROW_H;
        }
        _ => {}
    }

    // Range (read-only, just skip)
    match kind {
        WidgetKind::Slider | WidgetKind::NumberInput | WidgetKind::ProgressBar => {
            *cursor_y += ROW_H * 3.0;
        }
        _ => {}
    }

    // Color — click on text field to focus it (Label / Panel only)
    match kind {
        WidgetKind::Panel | WidgetKind::Label => {
            if my >= *cursor_y && my < *cursor_y + ROW_H && hit_x >= val_x {
                let current = scene
                    .widgets
                    .iter()
                    .find(|w| w.id == selected_id)
                    .map(|w| w.props.color_hex.clone())
                    .unwrap_or_default();
                scene.editing_field = Some("color".to_string());
                scene.edit_buffer = current;
                scene.edit_state.focus();
                scene.edit_state.cursor_pos = scene.edit_buffer.len();
                return;
            }
            *cursor_y += ROW_H;
        }
        _ => {}
    }

    // Button: per-corner radii, bg_color + text_color fields
    if kind == WidgetKind::Button {
        // 4 corner radius rows
        let corner_keys = ["r_tl", "r_tr", "r_bl", "r_br"];
        let corner_idx = [0usize, 1, 2, 3];
        for (key, idx) in corner_keys.iter().zip(corner_idx.iter()) {
            if my >= *cursor_y && my < *cursor_y + ROW_H && hit_x >= val_x {
                let current = scene
                    .widgets
                    .iter()
                    .find(|w| w.id == selected_id)
                    .map(|w| format!("{:.0}", w.props.border_radii[*idx]))
                    .unwrap_or_default();
                scene.editing_field = Some((*key).to_string());
                scene.edit_buffer = current;
                scene.edit_state.focus();
                scene.edit_state.cursor_pos = scene.edit_buffer.len();
                return;
            }
            *cursor_y += ROW_H;
        }

        // Bg Color row
        if my >= *cursor_y && my < *cursor_y + ROW_H && hit_x >= val_x {
            let current = scene
                .widgets
                .iter()
                .find(|w| w.id == selected_id)
                .map(|w| w.props.bg_color_hex.clone())
                .unwrap_or_default();
            scene.editing_field = Some("bg_color".to_string());
            scene.edit_buffer = current;
            scene.edit_state.focus();
            scene.edit_state.cursor_pos = scene.edit_buffer.len();
            return;
        }
        *cursor_y += ROW_H;

        // Text Color row
        if my >= *cursor_y && my < *cursor_y + ROW_H && hit_x >= val_x {
            let current = scene
                .widgets
                .iter()
                .find(|w| w.id == selected_id)
                .map(|w| w.props.text_color_hex.clone())
                .unwrap_or_default();
            scene.editing_field = Some("text_color".to_string());
            scene.edit_buffer = current;
            scene.edit_state.focus();
            scene.edit_state.cursor_pos = scene.edit_buffer.len();
            return;
        }
        *cursor_y += ROW_H;
    }

    // Alignment section (Button + Label)
    match kind {
        WidgetKind::Button | WidgetKind::Label => {
            // Section header
            *cursor_y += SECTION_H;

            let is_h_custom = {
                scene
                    .widgets
                    .iter()
                    .find(|w| w.id == selected_id)
                    .map(|w| matches!(w.props.label_h_align, HAlign::Custom { .. }))
                    .unwrap_or(false)
            };
            let is_v_custom = {
                scene
                    .widgets
                    .iter()
                    .find(|w| w.id == selected_id)
                    .map(|w| matches!(w.props.label_v_align, VAlign::Custom { .. }))
                    .unwrap_or(false)
            };

            // Align X row — buttons handled by draw_properties, skip
            *cursor_y += ROW_H;

            // Layout constants matching draw_properties val/pivot rows.
            // hit_x is relative to panel_x, so we do NOT add _panel_x here.
            let cust_toggle_w = 28.0_f32;
            let cust_field_w = 46.0_f32;
            let cust_slider_w =
                _right_w - ROW_PAD_X * 2.0 - LABEL_W - cust_field_w - cust_toggle_w - 6.0;
            let cust_field_x = ROW_PAD_X + LABEL_W + cust_slider_w + 3.0;
            let piv_field_w = 46.0_f32;
            let piv_slider_w = _right_w - ROW_PAD_X * 2.0 - LABEL_W - piv_field_w - 3.0;
            let piv_field_x = ROW_PAD_X + LABEL_W + piv_slider_w + 3.0;

            // Custom X value row
            if is_h_custom {
                // X val row — only activate text field when clicking text field portion
                if my >= *cursor_y && my < *cursor_y + ROW_H && hit_x >= cust_field_x {
                    let current = scene
                        .widgets
                        .iter()
                        .find(|w| w.id == selected_id)
                        .map(|w| format!("{:.1}", w.props.label_h_custom))
                        .unwrap_or_default();
                    scene.editing_field = Some("align_h_custom".to_string());
                    scene.edit_buffer = current;
                    scene.edit_state.focus();
                    scene.edit_state.cursor_pos = scene.edit_buffer.len();
                    return;
                }
                *cursor_y += ROW_H; // X val

                // X pivot row — activate text field when clicking text field portion
                if my >= *cursor_y && my < *cursor_y + ROW_H && hit_x >= piv_field_x {
                    let current = scene
                        .widgets
                        .iter()
                        .find(|w| w.id == selected_id)
                        .map(|w| format!("{:.2}", w.props.label_h_pivot))
                        .unwrap_or_default();
                    scene.editing_field = Some("align_h_pivot".to_string());
                    scene.edit_buffer = current;
                    scene.edit_state.focus();
                    scene.edit_state.cursor_pos = scene.edit_buffer.len();
                    return;
                }
                *cursor_y += ROW_H; // X pivot
            }

            // Align Y row — buttons handled by draw_properties, skip
            *cursor_y += ROW_H;

            // Custom Y value row
            if is_v_custom {
                // Y val row — only activate text field when clicking text field portion
                if my >= *cursor_y && my < *cursor_y + ROW_H && hit_x >= cust_field_x {
                    let current = scene
                        .widgets
                        .iter()
                        .find(|w| w.id == selected_id)
                        .map(|w| format!("{:.1}", w.props.label_v_custom))
                        .unwrap_or_default();
                    scene.editing_field = Some("align_v_custom".to_string());
                    scene.edit_buffer = current;
                    scene.edit_state.focus();
                    scene.edit_state.cursor_pos = scene.edit_buffer.len();
                    return;
                }
                *cursor_y += ROW_H; // Y val

                // Y pivot row — activate text field when clicking text field portion
                if my >= *cursor_y && my < *cursor_y + ROW_H && hit_x >= piv_field_x {
                    let current = scene
                        .widgets
                        .iter()
                        .find(|w| w.id == selected_id)
                        .map(|w| format!("{:.2}", w.props.label_v_pivot))
                        .unwrap_or_default();
                    scene.editing_field = Some("align_v_pivot".to_string());
                    scene.edit_buffer = current;
                    scene.edit_state.focus();
                    scene.edit_state.cursor_pos = scene.edit_buffer.len();
                    return;
                }
                *cursor_y += ROW_H; // Y pivot
            }
        }
        _ => {}
    }
    // Delete button interaction handled by GuiBatch::button() in draw_properties.
}

/// Commits the active text-field edit for widget properties.
pub fn commit_field(scene: &mut SceneState, field: &str, buf: String) {
    if let Some(w) = scene.selected_mut() {
        match field {
            "label" => w.props.label = buf,
            "color" => w.props.color_hex = buf,
            "bg_color" => w.props.bg_color_hex = buf,
            "text_color" => w.props.text_color_hex = buf,
            "align_h_custom" => {
                if let Ok(v) = buf.trim().parse::<f32>() {
                    w.props.label_h_custom = v;
                    w.props.label_h_align = HAlign::Custom {
                        value: v,
                        percent: w.props.label_h_custom_pct,
                        pivot: w.props.label_h_pivot,
                    };
                }
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
            }
            "r_tl" => {
                if let Ok(v) = buf.trim().parse::<f32>() {
                    w.props.border_radii[0] = v.max(0.0);
                }
            }
            "r_tr" => {
                if let Ok(v) = buf.trim().parse::<f32>() {
                    w.props.border_radii[1] = v.max(0.0);
                }
            }
            "r_bl" => {
                if let Ok(v) = buf.trim().parse::<f32>() {
                    w.props.border_radii[2] = v.max(0.0);
                }
            }
            "r_br" => {
                if let Ok(v) = buf.trim().parse::<f32>() {
                    w.props.border_radii[3] = v.max(0.0);
                }
            }
            "font_size" => {
                if let Ok(v) = buf.trim().parse::<f32>() {
                    w.props.font_size = v.max(1.0);
                }
            }
            // transform helpers used by right_panel when the user edits the
            // numeric fields directly instead of using the +/- buttons.
            "tx" => {
                if let Ok(v) = buf.trim().parse::<f32>() {
                    w.x = v;
                }
            }
            "ty" => {
                if let Ok(v) = buf.trim().parse::<f32>() {
                    w.y = v;
                }
            }
            "tw" => {
                if let Ok(v) = buf.trim().parse::<f32>() {
                    w.w = v.max(8.0);
                }
            }
            "th" => {
                if let Ok(v) = buf.trim().parse::<f32>() {
                    w.h = v.max(8.0);
                }
            }
            _ => {}
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Inspector panel: draw widget-specific property rows
// ─────────────────────────────────────────────────────────────────────────────

/// Draws the Properties section header + all widget-specific property rows,
/// then the Delete button.  Also handles button clicks (toggle visible,
/// delete) so the caller does not need any manual hit-testing.
/// Returns the final `cursor_y` after all rows.
pub fn draw_properties(
    dc: &mut DrawContext<'_, '_>,
    scene: &mut SceneState,
    panel_x: f32,
    right_w: f32,
    cursor_y: f32,
) -> f32 {
    let mut cy = cursor_y;

    let Some(selected) = scene.selected() else {
        return cy;
    };
    let selected_id = selected.id;

    draw_section(dc, panel_x, cy, right_w, "Properties");
    cy += SECTION_H;

    let (mx, my) = dc.ctx.input.mouse_pos_f32();
    let clicked = dc.ctx.input.button_just_pressed(MouseButton::Left);

    // Visible toggle — GuiBatch::button() handles hover + click
    draw_row_label(dc, panel_x, cy, "Visible");
    let toggle_x = panel_x + ROW_PAD_X + LABEL_W + 4.0;
    let toggle_label = if scene.selected().map_or(false, |w| w.props.visible) {
        "On"
    } else {
        "Off"
    };
    if dc.gui.button(
        dc.font,
        toggle_x,
        cy + 3.0,
        60.0,
        ROW_H - 6.0,
        toggle_label,
        mx,
        my,
        clicked,
    ) {
        if let Some(w) = scene.widgets.iter_mut().find(|w| w.id == selected_id) {
            w.props.visible = !w.props.visible;
        }
    }
    cy += ROW_H;

    // Re-borrow after mutable operations above
    let selected = match scene.selected() {
        Some(s) => s,
        None => return cy,
    };

    // Label / Hint
    let label_focused = scene.editing_field.as_deref() == Some("label");
    let label_value: &str = if label_focused {
        &scene.edit_buffer
    } else {
        &selected.props.label
    };
    match selected.kind {
        WidgetKind::Label
        | WidgetKind::Button
        | WidgetKind::Checkbox
        | WidgetKind::ToggleSwitch
        | WidgetKind::Tooltip
        | WidgetKind::Toast => {
            // highlight the main "Label" property when selected
            let (text_col, bg_col, border_col, sel_col) =
                default_row_colors("Label", label_focused);
            draw_text_row(
                dc,
                panel_x,
                cy,
                right_w,
                "Label",
                label_value,
                label_focused,
                scene.edit_state.cursor_visible,
                scene.edit_state.cursor_pos,
                scene.edit_state.selection(),
                scene.edit_state.all_selected,
                text_col,
                bg_col,
                border_col,
                sel_col,
            );
            cy += ROW_H;
        }
        WidgetKind::TextInput | WidgetKind::NumberInput => {
            let (text_col, bg_col, border_col, sel_col) = default_row_colors("Hint", label_focused);
            draw_text_row(
                dc,
                panel_x,
                cy,
                right_w,
                "Hint",
                label_value,
                label_focused,
                scene.edit_state.cursor_visible,
                scene.edit_state.cursor_pos,
                scene.edit_state.selection(),
                scene.edit_state.all_selected,
                text_col,
                bg_col,
                border_col,
                sel_col,
            );
            cy += ROW_H;
        }
        _ => {}
    }

    // Font size
    let font_size_focused = scene.editing_field.as_deref() == Some("font_size");
    let font_size_buf = format!("{:.0}", selected.props.font_size);
    let font_size_value: &str = if font_size_focused {
        &scene.edit_buffer
    } else {
        &font_size_buf
    };
    match selected.kind {
        WidgetKind::Label | WidgetKind::Button | WidgetKind::TextInput => {
            let (text_col, bg_col, border_col, sel_col) =
                default_row_colors("Size", font_size_focused);
            draw_text_row(
                dc,
                panel_x,
                cy,
                right_w,
                "Size",
                font_size_value,
                font_size_focused,
                scene.edit_state.cursor_visible,
                scene.edit_state.cursor_pos,
                scene.edit_state.selection(),
                scene.edit_state.all_selected,
                text_col,
                bg_col,
                border_col,
                sel_col,
            );
            cy += ROW_H;
        }
        _ => {}
    }

    // Range (read-only)
    match selected.kind {
        WidgetKind::Slider | WidgetKind::NumberInput | WidgetKind::ProgressBar => {
            draw_info_row(
                dc,
                panel_x,
                cy,
                right_w,
                "Min",
                &format!("{:.0}", selected.props.min),
            );
            cy += ROW_H;
            draw_info_row(
                dc,
                panel_x,
                cy,
                right_w,
                "Max",
                &format!("{:.0}", selected.props.max),
            );
            cy += ROW_H;
            draw_info_row(
                dc,
                panel_x,
                cy,
                right_w,
                "Val",
                &format!("{:.0}", selected.props.value),
            );
            cy += ROW_H;
        }
        _ => {}
    }

    // Color hex (Label / Panel text color)
    let color_focused = scene.editing_field.as_deref() == Some("color");
    let color_value: &str = if color_focused {
        &scene.edit_buffer
    } else {
        &selected.props.color_hex
    };
    match selected.kind {
        WidgetKind::Panel | WidgetKind::Label => {
            let (text_col, bg_col, border_col, sel_col) =
                default_row_colors("Color", color_focused);
            draw_text_row(
                dc,
                panel_x,
                cy,
                right_w,
                "Color",
                color_value,
                color_focused,
                scene.edit_state.cursor_visible,
                scene.edit_state.cursor_pos,
                scene.edit_state.selection(),
                scene.edit_state.all_selected,
                text_col,
                bg_col,
                border_col,
                sel_col,
            );
            cy += ROW_H;
        }
        _ => {}
    }

    // Button: background color + text color
    let bg_focused = scene.editing_field.as_deref() == Some("bg_color");
    let bg_color_buf = selected.props.bg_color_hex.clone();
    let bg_color_value: &str = if bg_focused {
        &scene.edit_buffer
    } else {
        &bg_color_buf
    };

    let tc_focused = scene.editing_field.as_deref() == Some("text_color");
    let text_color_buf = selected.props.text_color_hex.clone();
    let text_color_value: &str = if tc_focused {
        &scene.edit_buffer
    } else {
        &text_color_buf
    };

    if selected.kind == WidgetKind::Button {
        // Helper: draw one radius row
        let radii_snap = selected.props.border_radii;
        let corner_labels = ["R: TL", "R: TR", "R: BL", "R: BR"];
        let corner_keys = ["r_tl", "r_tr", "r_bl", "r_br"];
        for i in 0..4 {
            let focused = scene.editing_field.as_deref() == Some(corner_keys[i]);
            let buf = format!("{:.0}", radii_snap[i]);
            let val: &str = if focused { &scene.edit_buffer } else { &buf };
            let (text_col, bg_col, border_col, sel_col) =
                default_row_colors(corner_labels[i], focused);
            draw_text_row(
                dc,
                panel_x,
                cy,
                right_w,
                corner_labels[i],
                val,
                focused,
                scene.edit_state.cursor_visible,
                scene.edit_state.cursor_pos,
                scene.edit_state.selection(),
                scene.edit_state.all_selected,
                text_col,
                bg_col,
                border_col,
                sel_col,
            );
            cy += ROW_H;
        }

        // Bg Color row
        let (text_col, bg_col, border_col, sel_col) = default_row_colors("Bg Color", bg_focused);
        draw_text_row(
            dc,
            panel_x,
            cy,
            right_w,
            "Bg Color",
            bg_color_value,
            bg_focused,
            scene.edit_state.cursor_visible,
            scene.edit_state.cursor_pos,
            scene.edit_state.selection(),
            scene.edit_state.all_selected,
            text_col,
            bg_col,
            border_col,
            sel_col,
        );
        // Swatch preview
        let swatch_x = panel_x + right_w - ROW_PAD_X - (ROW_H - 8.0);
        let parsed_bg = parse_hex_or(&selected.props.bg_color_hex, [0.424, 0.388, 1.0, 1.0]);
        dc.gui
            .rect(swatch_x, cy + 4.0, ROW_H - 8.0, ROW_H - 8.0, parsed_bg);
        cy += ROW_H;

        // Text Color row
        let (text_col, bg_col, border_col, sel_col) = default_row_colors("Text Color", tc_focused);
        draw_text_row(
            dc,
            panel_x,
            cy,
            right_w,
            "Txt Color",
            text_color_value,
            tc_focused,
            scene.edit_state.cursor_visible,
            scene.edit_state.cursor_pos,
            scene.edit_state.selection(),
            scene.edit_state.all_selected,
            text_col,
            bg_col,
            border_col,
            sel_col,
        );
        let swatch_x2 = panel_x + right_w - ROW_PAD_X - (ROW_H - 8.0);
        let parsed_tc = parse_hex_or(&selected.props.text_color_hex, [1.0, 1.0, 1.0, 1.0]);
        dc.gui
            .rect(swatch_x2, cy + 4.0, ROW_H - 8.0, ROW_H - 8.0, parsed_tc);
        cy += ROW_H;
    }

    // Label alignment (Button + Label)
    match selected.kind {
        WidgetKind::Button | WidgetKind::Label => {
            draw_section(dc, panel_x, cy, right_w, "Label Align");
            cy += SECTION_H;

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

            // ── Align X row ───────────────────────────────────────────────
            draw_row_label(dc, panel_x, cy, "Align X");
            {
                // 4 segmented buttons: L | C | R | •
                let btn_area_x = panel_x + ROW_PAD_X + LABEL_W + 2.0;
                let avail_w = right_w - ROW_PAD_X * 2.0 - LABEL_W - 2.0;
                let bw = (avail_w / 4.0).floor() - 1.0;
                let bh = ROW_H - 6.0;
                let by = cy + 3.0;
                let labels = ["L", "C", "R", "•"];
                let active_idx = match h_align {
                    HAlign::Left => 0,
                    HAlign::Center => 1,
                    HAlign::Right => 2,
                    HAlign::Custom { .. } => 3,
                };
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
                        let new_align = match i {
                            0 => HAlign::Left,
                            1 => HAlign::Center,
                            2 => HAlign::Right,
                            _ => HAlign::Custom {
                                value: h_custom_val,
                                percent: h_custom_pct,
                                pivot: h_pivot_val,
                            },
                        };
                        if let Some(w) = scene.widgets.iter_mut().find(|w| w.id == selected_id) {
                            w.props.label_h_align = new_align;
                        }
                    }
                }
            }
            cy += ROW_H;

            // Custom X value row (only when Custom is active)
            if is_h_custom {
                let h_cust_focused = scene.editing_field.as_deref() == Some("align_h_custom");
                let h_cust_buf = format!("{:.1}", h_custom_val);
                let h_cust_str: &str = if h_cust_focused {
                    &scene.edit_buffer
                } else {
                    &h_cust_buf
                };
                // Layout: [row_label][slider_________][text_field][%/px]
                let toggle_w = 28.0;
                let field_w = 46.0;
                let slider_w = right_w - ROW_PAD_X * 2.0 - LABEL_W - field_w - toggle_w - 6.0;
                let slider_x = panel_x + ROW_PAD_X + LABEL_W;
                let field_x = slider_x + slider_w + 3.0;
                let tog_x = field_x + field_w + 2.0;
                let field_h = ROW_H - 6.0;
                let by = cy + 3.0;

                draw_row_label(dc, panel_x, cy, "  X val");

                // Slider track + fill + thumb
                let h_max = if h_custom_pct {
                    100.0_f32
                } else {
                    widget_w.max(1.0)
                };
                let h_t = (h_custom_val / h_max).clamp(0.0, 1.0);
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
                    h_t * slider_w,
                    4.0,
                    AppColor::hex("#4A4580").to_linear_f32(),
                );
                let thumb_sx = slider_x + h_t * slider_w - 5.0;
                dc.gui.rect(
                    thumb_sx,
                    by,
                    10.0,
                    field_h,
                    AppColor::hex("#6C63FF").to_linear_f32(),
                );
                // Slider drag
                if dc.ctx.input.is_button_down(MouseButton::Left)
                    && mx >= slider_x - 4.0
                    && mx <= slider_x + slider_w + 4.0
                    && my >= by
                    && my <= by + field_h
                    && !h_cust_focused
                {
                    let new_val = ((mx - slider_x) / slider_w).clamp(0.0, 1.0) * h_max;
                    if let Some(w) = scene.widgets.iter_mut().find(|w| w.id == selected_id) {
                        w.props.label_h_custom = new_val;
                        w.props.label_h_align = HAlign::Custom {
                            value: new_val,
                            percent: h_custom_pct,
                            pivot: h_pivot_val,
                        };
                    }
                }

                // Text field
                let bg = if h_cust_focused {
                    AppColor::hex("#1E3A5F").to_linear_f32()
                } else {
                    AppColor::hex("#191919").to_linear_f32()
                };
                let border = if h_cust_focused {
                    Some(AppColor::hex("#0078D4").to_linear_f32())
                } else {
                    None
                };
                let fg = if h_cust_focused {
                    [1.0f32; 4]
                } else {
                    AppColor::hex("#DDDDDD").to_linear_f32()
                };
                let sel_col = [0.0f32, 0.47, 0.83, 0.35];
                let eff_sel = if scene.edit_state.all_selected && !h_cust_str.is_empty() {
                    Some((0usize, h_cust_str.len()))
                } else {
                    scene.edit_state.selection()
                };
                dc.gui.draw_text_field(
                    dc.font,
                    field_x,
                    by,
                    field_w,
                    field_h,
                    h_cust_str,
                    10.0,
                    h_cust_focused,
                    scene.edit_state.cursor_visible,
                    scene.edit_state.cursor_pos,
                    eff_sel,
                    fg,
                    bg,
                    border,
                    sel_col,
                    4.0,
                );

                // px/% toggle
                let pct_lbl = if h_custom_pct { "%" } else { "px" };
                let pct_col = AppColor::hex("#3E3E42").to_linear_f32();
                let pct_hover = AppColor::hex("#555559").to_linear_f32();
                if dc.gui.button_colored(
                    dc.font, tog_x, by, toggle_w, field_h, pct_lbl, mx, my, clicked, pct_col,
                    pct_hover,
                ) {
                    if let Some(w) = scene.widgets.iter_mut().find(|w| w.id == selected_id) {
                        w.props.label_h_custom_pct = !w.props.label_h_custom_pct;
                    }
                }
                cy += ROW_H;

                // Pivot X row
                draw_row_label(dc, panel_x, cy, "  X pivot");
                {
                    let piv_field_w = 46.0_f32;
                    let piv_slider_w = right_w - ROW_PAD_X * 2.0 - LABEL_W - piv_field_w - 3.0;
                    let piv_slider_x = panel_x + ROW_PAD_X + LABEL_W;
                    let piv_field_x = piv_slider_x + piv_slider_w + 3.0;
                    let slider_h = ROW_H - 6.0;
                    let field_h = ROW_H - 6.0;
                    let sy = cy + 3.0;
                    let h_piv_focused = scene.editing_field.as_deref() == Some("align_h_pivot");
                    // Track
                    dc.gui.rect(
                        piv_slider_x,
                        sy + slider_h * 0.5 - 2.0,
                        piv_slider_w,
                        4.0,
                        AppColor::hex("#3E3E42").to_linear_f32(),
                    );
                    // Fill
                    dc.gui.rect(
                        piv_slider_x,
                        sy + slider_h * 0.5 - 2.0,
                        h_pivot_val * piv_slider_w,
                        4.0,
                        AppColor::hex("#4A4580").to_linear_f32(),
                    );
                    // Thumb
                    let thumb_x = piv_slider_x + h_pivot_val * piv_slider_w - 5.0;
                    dc.gui.rect(
                        thumb_x,
                        sy,
                        10.0,
                        slider_h,
                        AppColor::hex("#6C63FF").to_linear_f32(),
                    );
                    // Drag (held, not just_pressed) — only when text field not focused
                    if dc.ctx.input.is_button_down(MouseButton::Left)
                        && !h_piv_focused
                        && mx >= piv_slider_x - 6.0
                        && mx <= piv_slider_x + piv_slider_w + 6.0
                        && my >= sy
                        && my <= sy + slider_h
                    {
                        let new_piv = ((mx - piv_slider_x) / piv_slider_w).clamp(0.0, 1.0);
                        if let Some(w) = scene.widgets.iter_mut().find(|w| w.id == selected_id) {
                            w.props.label_h_pivot = new_piv;
                            if let HAlign::Custom { value, percent, .. } = w.props.label_h_align {
                                w.props.label_h_align = HAlign::Custom {
                                    value,
                                    percent,
                                    pivot: new_piv,
                                };
                            }
                        }
                    }
                    // Text field
                    let h_piv_str = if h_piv_focused {
                        &scene.edit_buffer
                    } else {
                        &format!("{:.2}", h_pivot_val)
                    };
                    let h_piv_str = h_piv_str.clone();
                    let bg = if h_piv_focused {
                        AppColor::hex("#1E3A5F").to_linear_f32()
                    } else {
                        AppColor::hex("#191919").to_linear_f32()
                    };
                    let border = if h_piv_focused {
                        Some(AppColor::hex("#0078D4").to_linear_f32())
                    } else {
                        None
                    };
                    let fg = if h_piv_focused {
                        [1.0f32; 4]
                    } else {
                        AppColor::hex("#DDDDDD").to_linear_f32()
                    };
                    let sel_col = [0.0f32, 0.47, 0.83, 0.35];
                    let eff_sel = if scene.edit_state.all_selected && !h_piv_str.is_empty() {
                        Some((0usize, h_piv_str.len()))
                    } else {
                        scene.edit_state.selection()
                    };
                    dc.gui.draw_text_field(
                        dc.font,
                        piv_field_x,
                        sy,
                        piv_field_w,
                        field_h,
                        &h_piv_str,
                        10.0,
                        h_piv_focused,
                        scene.edit_state.cursor_visible,
                        scene.edit_state.cursor_pos,
                        eff_sel,
                        fg,
                        bg,
                        border,
                        sel_col,
                        4.0,
                    );
                }
                cy += ROW_H;
            }

            // ── Align Y row ───────────────────────────────────────────────
            draw_row_label(dc, panel_x, cy, "Align Y");
            {
                let btn_area_x = panel_x + ROW_PAD_X + LABEL_W + 2.0;
                let avail_w = right_w - ROW_PAD_X * 2.0 - LABEL_W - 2.0;
                let bw = (avail_w / 4.0).floor() - 1.0;
                let bh = ROW_H - 6.0;
                let by = cy + 3.0;
                let labels = ["T", "C", "B", "•"];
                let active_idx = match v_align {
                    VAlign::Top => 0,
                    VAlign::Center => 1,
                    VAlign::Bottom => 2,
                    VAlign::Custom { .. } => 3,
                };
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
                        let new_align = match i {
                            0 => VAlign::Top,
                            1 => VAlign::Center,
                            2 => VAlign::Bottom,
                            _ => VAlign::Custom {
                                value: v_custom_val,
                                percent: v_custom_pct,
                                pivot: v_pivot_val,
                            },
                        };
                        if let Some(w) = scene.widgets.iter_mut().find(|w| w.id == selected_id) {
                            w.props.label_v_align = new_align;
                        }
                    }
                }
            }
            cy += ROW_H;

            // Custom Y value row (only when Custom is active)
            if is_v_custom {
                let v_cust_focused = scene.editing_field.as_deref() == Some("align_v_custom");
                let v_cust_buf = format!("{:.1}", v_custom_val);
                let v_cust_str: &str = if v_cust_focused {
                    &scene.edit_buffer
                } else {
                    &v_cust_buf
                };
                // Layout: [row_label][slider_________][text_field][%/px]
                let toggle_w = 28.0;
                let field_w = 46.0;
                let slider_w = right_w - ROW_PAD_X * 2.0 - LABEL_W - field_w - toggle_w - 6.0;
                let slider_x = panel_x + ROW_PAD_X + LABEL_W;
                let field_x = slider_x + slider_w + 3.0;
                let tog_x = field_x + field_w + 2.0;
                let field_h = ROW_H - 6.0;
                let by = cy + 3.0;

                draw_row_label(dc, panel_x, cy, "  Y val");

                // Slider track + fill + thumb
                let v_max = if v_custom_pct {
                    100.0_f32
                } else {
                    widget_h.max(1.0)
                };
                let v_t = (v_custom_val / v_max).clamp(0.0, 1.0);
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
                    v_t * slider_w,
                    4.0,
                    AppColor::hex("#4A4580").to_linear_f32(),
                );
                let thumb_sx = slider_x + v_t * slider_w - 5.0;
                dc.gui.rect(
                    thumb_sx,
                    by,
                    10.0,
                    field_h,
                    AppColor::hex("#6C63FF").to_linear_f32(),
                );
                // Slider drag
                if dc.ctx.input.is_button_down(MouseButton::Left)
                    && mx >= slider_x - 4.0
                    && mx <= slider_x + slider_w + 4.0
                    && my >= by
                    && my <= by + field_h
                    && !v_cust_focused
                {
                    let new_val = ((mx - slider_x) / slider_w).clamp(0.0, 1.0) * v_max;
                    if let Some(w) = scene.widgets.iter_mut().find(|w| w.id == selected_id) {
                        w.props.label_v_custom = new_val;
                        w.props.label_v_align = VAlign::Custom {
                            value: new_val,
                            percent: v_custom_pct,
                            pivot: v_pivot_val,
                        };
                    }
                }

                // Text field
                let bg = if v_cust_focused {
                    AppColor::hex("#1E3A5F").to_linear_f32()
                } else {
                    AppColor::hex("#191919").to_linear_f32()
                };
                let border = if v_cust_focused {
                    Some(AppColor::hex("#0078D4").to_linear_f32())
                } else {
                    None
                };
                let fg = if v_cust_focused {
                    [1.0f32; 4]
                } else {
                    AppColor::hex("#DDDDDD").to_linear_f32()
                };
                let sel_col = [0.0f32, 0.47, 0.83, 0.35];
                let eff_sel = if scene.edit_state.all_selected && !v_cust_str.is_empty() {
                    Some((0usize, v_cust_str.len()))
                } else {
                    scene.edit_state.selection()
                };
                dc.gui.draw_text_field(
                    dc.font,
                    field_x,
                    by,
                    field_w,
                    field_h,
                    v_cust_str,
                    10.0,
                    v_cust_focused,
                    scene.edit_state.cursor_visible,
                    scene.edit_state.cursor_pos,
                    eff_sel,
                    fg,
                    bg,
                    border,
                    sel_col,
                    4.0,
                );

                // px/% toggle
                let pct_lbl = if v_custom_pct { "%" } else { "px" };
                let pct_col = AppColor::hex("#3E3E42").to_linear_f32();
                let pct_hover = AppColor::hex("#555559").to_linear_f32();
                if dc.gui.button_colored(
                    dc.font, tog_x, by, toggle_w, field_h, pct_lbl, mx, my, clicked, pct_col,
                    pct_hover,
                ) {
                    if let Some(w) = scene.widgets.iter_mut().find(|w| w.id == selected_id) {
                        w.props.label_v_custom_pct = !w.props.label_v_custom_pct;
                    }
                }
                cy += ROW_H;

                // Pivot Y row
                draw_row_label(dc, panel_x, cy, "  Y pivot");
                {
                    let piv_field_w = 46.0_f32;
                    let piv_slider_w = right_w - ROW_PAD_X * 2.0 - LABEL_W - piv_field_w - 3.0;
                    let piv_slider_x = panel_x + ROW_PAD_X + LABEL_W;
                    let piv_field_x = piv_slider_x + piv_slider_w + 3.0;
                    let slider_h = ROW_H - 6.0;
                    let field_h = ROW_H - 6.0;
                    let sy = cy + 3.0;
                    let v_piv_focused = scene.editing_field.as_deref() == Some("align_v_pivot");
                    // Track
                    dc.gui.rect(
                        piv_slider_x,
                        sy + slider_h * 0.5 - 2.0,
                        piv_slider_w,
                        4.0,
                        AppColor::hex("#3E3E42").to_linear_f32(),
                    );
                    // Fill
                    dc.gui.rect(
                        piv_slider_x,
                        sy + slider_h * 0.5 - 2.0,
                        v_pivot_val * piv_slider_w,
                        4.0,
                        AppColor::hex("#4A4580").to_linear_f32(),
                    );
                    // Thumb
                    let thumb_x = piv_slider_x + v_pivot_val * piv_slider_w - 5.0;
                    dc.gui.rect(
                        thumb_x,
                        sy,
                        10.0,
                        slider_h,
                        AppColor::hex("#6C63FF").to_linear_f32(),
                    );
                    // Drag (held, not just_pressed) — only when text field not focused
                    if dc.ctx.input.is_button_down(MouseButton::Left)
                        && !v_piv_focused
                        && mx >= piv_slider_x - 6.0
                        && mx <= piv_slider_x + piv_slider_w + 6.0
                        && my >= sy
                        && my <= sy + slider_h
                    {
                        let new_piv = ((mx - piv_slider_x) / piv_slider_w).clamp(0.0, 1.0);
                        if let Some(w) = scene.widgets.iter_mut().find(|w| w.id == selected_id) {
                            w.props.label_v_pivot = new_piv;
                            if let VAlign::Custom { value, percent, .. } = w.props.label_v_align {
                                w.props.label_v_align = VAlign::Custom {
                                    value,
                                    percent,
                                    pivot: new_piv,
                                };
                            }
                        }
                    }
                    // Text field
                    let v_piv_str = if v_piv_focused {
                        &scene.edit_buffer
                    } else {
                        &format!("{:.2}", v_pivot_val)
                    };
                    let v_piv_str = v_piv_str.clone();
                    let bg = if v_piv_focused {
                        AppColor::hex("#1E3A5F").to_linear_f32()
                    } else {
                        AppColor::hex("#191919").to_linear_f32()
                    };
                    let border = if v_piv_focused {
                        Some(AppColor::hex("#0078D4").to_linear_f32())
                    } else {
                        None
                    };
                    let fg = if v_piv_focused {
                        [1.0f32; 4]
                    } else {
                        AppColor::hex("#DDDDDD").to_linear_f32()
                    };
                    let sel_col = [0.0f32, 0.47, 0.83, 0.35];
                    let eff_sel = if scene.edit_state.all_selected && !v_piv_str.is_empty() {
                        Some((0usize, v_piv_str.len()))
                    } else {
                        scene.edit_state.selection()
                    };
                    dc.gui.draw_text_field(
                        dc.font,
                        piv_field_x,
                        sy,
                        piv_field_w,
                        field_h,
                        &v_piv_str,
                        10.0,
                        v_piv_focused,
                        scene.edit_state.cursor_visible,
                        scene.edit_state.cursor_pos,
                        eff_sel,
                        fg,
                        bg,
                        border,
                        sel_col,
                        4.0,
                    );
                }
                cy += ROW_H;
            }
        }
        _ => {}
    }

    // Delete button — GuiBatch::button() handles hover + click
    let del_y = cy + 8.0;
    let del_x = panel_x + 10.0;
    let del_w = right_w - 20.0;
    if dc.gui.button_colored(
        dc.font,
        del_x,
        del_y,
        del_w,
        28.0,
        "Delete Widget",
        mx,
        my,
        clicked,
        AppColor::hex("#3C3C3C").to_linear_f32(),
        AppColor::hex("#C0392B").to_linear_f32(),
    ) {
        scene.delete_selected();
    }

    cy
}

// ─────────────────────────────────────────────────────────────────────────────
// Shared draw helpers (also used by right_panel for Transform section)
// ─────────────────────────────────────────────────────────────────────────────

pub fn draw_section(dc: &mut DrawContext<'_, '_>, panel_x: f32, y: f32, right_w: f32, title: &str) {
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

pub fn draw_row_label(dc: &mut DrawContext<'_, '_>, panel_x: f32, y: f32, label: &str) {
    dc.gui.draw_text(
        dc.font,
        label,
        [panel_x + ROW_PAD_X, y + 7.0],
        10.0,
        AppColor::hex("#CCCCCC").to_linear_f32(),
    );
}

/// Draws a number field row: label on the left, value display in the
/// middle.  When `focused` is true we draw the same blue focus background
/// used by text rows so the user gets visual feedback that the field is
/// active.  The caller is responsible for drawing and handling the `-`/`+`
/// stepper buttons using `dc.gui.button()`.
pub fn draw_number_row(
    dc: &mut DrawContext<'_, '_>,
    panel_x: f32,
    y: f32,
    right_w: f32,
    _mx: f32,
    _my: f32,
    label: &str,
    value: &str,
    focused: bool,
    cursor_visible: bool,
    cursor_pos: usize,
    sel: Option<(usize, usize)>,
    all_selected: bool,
) {
    draw_row_label(dc, panel_x, y, label);
    let val_x = panel_x + ROW_PAD_X + LABEL_W;
    let val_w = right_w - ROW_PAD_X * 2.0 - LABEL_W - BTN_W * 2.0 - 8.0;
    if focused {
        // when editing we delegate to the richer text-row renderer but
        // restrict the available width so the +/- buttons still fit.
        // compute a fake `right_w` that yields `val_w` above inside
        // `draw_text_row`'s math: right_w_arg - ROW_PAD_X*2 - LABEL_W == val_w
        let right_w_for_text = val_w + ROW_PAD_X * 2.0 + LABEL_W;
        // use the row's own label when determining whether the text should
        // highlight to white on focus
        let (text_col, bg_col, border_col, sel_col) = default_row_colors(label, true);
        draw_text_row(
            dc,
            panel_x,
            y,
            right_w_for_text,
            label,
            value,
            true,
            cursor_visible,
            cursor_pos,
            sel,
            all_selected,
            text_col,
            bg_col,
            border_col,
            sel_col,
        );
    } else {
        let bg_col = AppColor::hex("#191919").to_linear_f32();
        dc.gui.rect(val_x, y + 3.0, val_w, ROW_H - 6.0, bg_col);
        dc.gui.draw_text(
            dc.font,
            value,
            [val_x + 4.0, y + 7.0],
            10.0,
            AppColor::hex("#DDDDDD").to_linear_f32(),
        );
    }
    // Note: +/- buttons are drawn by the caller via dc.gui.button() so they
    // can also capture the click return value directly.
}
// redraw helpers ----------------------------------------------------------

/// Draw a labelled single-line text field row.  Colours are provided explicitly
/// so callers can customize them; use `default_row_colors` if you just want
/// the standard palette.
pub fn draw_text_row(
    dc: &mut DrawContext<'_, '_>,
    panel_x: f32,
    y: f32,
    right_w: f32,
    label: &str,
    value: &str,
    focused: bool,
    cursor_visible: bool,
    cursor_pos: usize,
    sel: Option<(usize, usize)>,
    all_selected: bool,
    text_color: [f32; 4],
    bg_color: [f32; 4],
    border_color: Option<[f32; 4]>,
    sel_color: [f32; 4],
) {
    // delegate to shared helper on DrawContext for consistency
    dc.draw_text_input_row(
        panel_x,
        y,
        right_w,
        ROW_PAD_X,
        LABEL_W,
        ROW_H,
        label,
        value,
        focused,
        cursor_visible,
        cursor_pos,
        sel,
        all_selected,
        text_color,
        bg_color,
        border_color,
        sel_color,
    );
}

pub fn draw_info_row(
    dc: &mut DrawContext<'_, '_>,
    panel_x: f32,
    y: f32,
    right_w: f32,
    label: &str,
    value: &str,
) -> f32 {
    let (text_col, bg_col, border_col, sel_col) = default_row_colors(label, false);
    draw_text_row(
        dc, panel_x, y, right_w, label, value, false, false, 0, None, false, text_col, bg_col,
        border_col, sel_col,
    );
    y
}
