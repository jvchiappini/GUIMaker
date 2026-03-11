// ── Shared draw helpers ────────────────────────────────────────────────────────
//
// Layout constants, colour utilities, and primitive row renderers used by
// every per-widget module and by right_panel / canvas.

use ferrous_app::{Color as AppColor, DrawContext};
use ferrous_gui::GuiBatch;
use ferrous_ui_core::Rect;

use crate::scene::{HAlign, PlacedWidget, VAlign};

// ── Layout constants ──────────────────────────────────────────────────────────
pub const HEADER_H: f32 = 34.0;
pub const SECTION_H: f32 = 20.0;
pub const ROW_H: f32 = 26.0;
pub const ROW_PAD_X: f32 = 10.0;
pub const BTN_W: f32 = 22.0;
pub const LABEL_W: f32 = 54.0;

// ── Colour helpers ────────────────────────────────────────────────────────────

/// Returns (text, bg, border, selection) colours for a text-row.
/// Text goes white only when `focused` AND `label` is in the whitelist.
pub fn default_row_colors(
    label: &str,
    focused: bool,
) -> ([f32; 4], [f32; 4], Option<[f32; 4]>, [f32; 4]) {
    const WHITE_ON_FOCUS: &[&str] = &["X", "Y", "W", "H", "Label", "Color"];
    let text_col = if focused && WHITE_ON_FOCUS.contains(&label) {
        [1.0; 4]
    } else {
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
    (text_col, bg_col, border_col, [0.0, 0.47, 0.83, 0.35])
}

/// Parse `"#RRGGBB[AA]"` → linear `[f32; 4]`, falling back to `default`.
pub fn parse_hex_or(hex: &str, default: [f32; 4]) -> [f32; 4] {
    let s = hex.trim().trim_start_matches('#');
    if s.len() < 6 {
        return default;
    }
    let p = |i: usize| u8::from_str_radix(&s[i..i + 2], 16).ok();
    match (p(0), p(2), p(4)) {
        (Some(r), Some(g), Some(b)) => {
            let a = if s.len() >= 8 {
                p(6).unwrap_or(255)
            } else {
                255
            };
            AppColor::from_rgba8(r, g, b, a).to_linear_f32()
        }
        _ => default,
    }
}

// ── Primitive row renderers ───────────────────────────────────────────────────

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
        let right_w_for_text = val_w + ROW_PAD_X * 2.0 + LABEL_W;
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
}

pub fn draw_info_row(
    dc: &mut DrawContext<'_, '_>,
    panel_x: f32,
    y: f32,
    right_w: f32,
    label: &str,
    value: &str,
) {
    let (text_col, bg_col, border_col, sel_col) = default_row_colors(label, false);
    draw_text_row(
        dc, panel_x, y, right_w, label, value, false, false, 0, None, false, text_col, bg_col,
        border_col, sel_col,
    );
}

// ── Canvas overlays ───────────────────────────────────────────────────────────

/// Eight corner+midpoint resize handles around the selection rect.
pub fn draw_selection_handles(dc: &mut DrawContext<'_, '_>, sx: f32, sy: f32, sw: f32, sh: f32) {
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

/// Pivot/anchor crosshair and diamond overlay for custom-aligned text.
pub fn draw_pivot_overlay(
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

    let line_col: [f32; 4] = [0.98, 0.78, 0.12, 0.70];
    let dot_col: [f32; 4] = [1.00, 0.92, 0.23, 1.00];
    let lw = 1.0_f32;

    if h_custom {
        dc.gui.rect(anchor_x - lw * 0.5, sy, lw, sh, line_col);
    }
    if v_custom {
        dc.gui.rect(sx, anchor_y - lw * 0.5, sw, lw, line_col);
    }

    let d = 5.0_f32;
    dc.gui
        .rect(anchor_x - d, anchor_y - lw, d * 2.0, lw * 2.0, dot_col);
    dc.gui
        .rect(anchor_x - lw, anchor_y - d, lw * 2.0, d * 2.0, dot_col);
    let r = 3.0_f32;
    dc.gui
        .rect(anchor_x - r, anchor_y - r, r * 2.0, r * 2.0, dot_col);

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
            },
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

/// Measures `text` using GuiBatch metrics.
pub fn measure_text(dc: &DrawContext<'_, '_>, text: &str, size: f32) -> f32 {
    GuiBatch::measure_text(dc.font, text, size)
}

/// Draw a simple clipped rect background (used by Label/generic).
pub fn draw_clip_rect(dc: &mut DrawContext<'_, '_>, x: f32, y: f32, w: f32, h: f32) {
    dc.gui.push_clip(Rect::new(x, y, w, h));
}

pub fn pop_clip(dc: &mut DrawContext<'_, '_>) {
    dc.gui.pop_clip();
}
