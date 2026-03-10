use ferrous_app::{AppContext, CursorIcon, DrawContext, MouseButton};
use ferrous_ui_core::Rect;

use crate::{c_canvas, c_grid, scene::SceneState, PreviewDrag, TOP_H};

const HANDLE_HIT: f32 = 8.0;

//test

// ── Update ────────────────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
pub fn update(
    ctx: &mut AppContext,
    zoom: &mut f32,
    pan_x: &mut f32,
    pan_y: &mut f32,
    last_mx: &mut f32,
    last_my: &mut f32,
    left_w: f32,
    right_w: f32,
    preview_w: &mut f32,
    preview_h: &mut f32,
    preview_drag: &mut Option<PreviewDrag>,
    scene: &mut SceneState,
) {
    let (win_w, win_h) = ctx.window_size;
    let ww = win_w as f32;
    let wh = win_h as f32;
    let (mx, my) = ctx.input.mouse_pos_f32();

    let canvas_x = left_w;
    let canvas_y = TOP_H;
    let canvas_w = ww - left_w - right_w;
    let canvas_h = wh - TOP_H;

    let over_canvas =
        mx >= canvas_x && mx < canvas_x + canvas_w && my >= canvas_y && my < canvas_y + canvas_h;

    // ── Preview rect handles ──────────────────────────────────────────────────
    let (px, py, pw, ph) = preview_screen_rect(
        canvas_x, canvas_y, canvas_w, canvas_h, *pan_x, *pan_y, *zoom, *preview_w, *preview_h,
    );

    let hovered_handle = if over_canvas {
        detect_handle(mx, my, px, py, pw, ph)
    } else {
        None
    };

    if let Some(drag) = preview_drag.or(hovered_handle) {
        ctx.window.set_cursor(match drag {
            PreviewDrag::Left | PreviewDrag::Right => CursorIcon::EwResize,
            PreviewDrag::Top | PreviewDrag::Bottom => CursorIcon::NsResize,
            PreviewDrag::TopLeft | PreviewDrag::BottomRight => CursorIcon::NwseResize,
            PreviewDrag::TopRight | PreviewDrag::BottomLeft => CursorIcon::NeswResize,
        });
    }

    let lmb_pressed = ctx.input.button_just_pressed(MouseButton::Left);
    let lmb_down = ctx.input.is_button_down(MouseButton::Left);
    let lmb_released = ctx.input.button_just_released(MouseButton::Left);

    // ── Drop widget from palette ──────────────────────────────────────────────
    if lmb_released && over_canvas {
        if let Some(kind) = scene.palette_drag.take() {
            // Convert screen → world coords
            let origin_sx = canvas_x + canvas_w * 0.5 + *pan_x;
            let origin_sy = canvas_y + canvas_h * 0.5 + *pan_y;
            let world_x = (mx - origin_sx) / zoom.max(0.001);
            let world_y = (my - origin_sy) / zoom.max(0.001);
            let id = scene.add_widget(kind, world_x, world_y);
            scene.selected_id = Some(id);
            scene.sync_prop_strings();
            *last_mx = mx;
            *last_my = my;
            return;
        }
    }

    // ── Start preview-rect drag ───────────────────────────────────────────────
    if lmb_pressed && preview_drag.is_none() {
        if let Some(h) = hovered_handle {
            *preview_drag = Some(h);
        }
    }

    // ── Apply preview-rect drag ───────────────────────────────────────────────
    if lmb_down {
        if let Some(drag) = *preview_drag {
            let dx = (mx - *last_mx) / zoom.max(0.001);
            let dy = (my - *last_my) / zoom.max(0.001);
            match drag {
                PreviewDrag::Left => *preview_w = (*preview_w - dx).max(50.0),
                PreviewDrag::Right => *preview_w = (*preview_w + dx).max(50.0),
                PreviewDrag::Top => *preview_h = (*preview_h - dy).max(50.0),
                PreviewDrag::Bottom => *preview_h = (*preview_h + dy).max(50.0),
                PreviewDrag::TopLeft => {
                    *preview_w = (*preview_w - dx).max(50.0);
                    *preview_h = (*preview_h - dy).max(50.0);
                }
                PreviewDrag::TopRight => {
                    *preview_w = (*preview_w + dx).max(50.0);
                    *preview_h = (*preview_h - dy).max(50.0);
                }
                PreviewDrag::BottomLeft => {
                    *preview_w = (*preview_w - dx).max(50.0);
                    *preview_h = (*preview_h + dy).max(50.0);
                }
                PreviewDrag::BottomRight => {
                    *preview_w = (*preview_w + dx).max(50.0);
                    *preview_h = (*preview_h + dy).max(50.0);
                }
            }
        }
    }

    if lmb_released {
        *preview_drag = None;
    }

    // ── Widget selection & dragging on canvas ─────────────────────────────────
    if scene.palette_drag.is_none() && preview_drag.is_none() {
        // World coords of mouse
        let origin_sx = canvas_x + canvas_w * 0.5 + *pan_x;
        let origin_sy = canvas_y + canvas_h * 0.5 + *pan_y;
        let world_mx = (mx - origin_sx) / zoom.max(0.001);
        let world_my = (my - origin_sy) / zoom.max(0.001);

        // Start drag or select
        if lmb_pressed && over_canvas && hovered_handle.is_none() {
            // Check if we hit a placed widget (top-most first)
            let hit_id = scene
                .widgets
                .iter()
                .rev()
                .find(|w| {
                    world_mx >= w.x - w.w * 0.5
                        && world_mx <= w.x + w.w * 0.5
                        && world_my >= w.y - w.h * 0.5
                        && world_my <= w.y + w.h * 0.5
                })
                .map(|w| w.id);

            if let Some(id) = hit_id {
                // Select and start canvas drag
                let prev = scene.selected_id;
                scene.selected_id = Some(id);
                if prev != Some(id) {
                    scene.sync_prop_strings();
                }
                let off_x = world_mx
                    - scene
                        .widgets
                        .iter()
                        .find(|w| w.id == id)
                        .map(|w| w.x)
                        .unwrap_or(0.0);
                let off_y = world_my
                    - scene
                        .widgets
                        .iter()
                        .find(|w| w.id == id)
                        .map(|w| w.y)
                        .unwrap_or(0.0);
                scene.drag_canvas = Some((id, off_x, off_y));
            } else {
                // Clicked empty space → deselect
                scene.selected_id = None;
                scene.drag_canvas = None;
            }
        }

        // Drag widget
        if lmb_down {
            if let Some((drag_id, off_x, off_y)) = scene.drag_canvas {
                if let Some(w) = scene.widgets.iter_mut().find(|w| w.id == drag_id) {
                    w.x = world_mx - off_x;
                    w.y = world_my - off_y;
                }
                scene.sync_prop_strings();
            }
        }

        if lmb_released {
            scene.drag_canvas = None;
        }
    }

    // ── Zoom ──────────────────────────────────────────────────────────────────
    if preview_drag.is_none() && scene.drag_canvas.is_none() && over_canvas {
        let sd = ctx.input.scroll_delta();
        let scroll = if sd.1 != 0.0 { sd.1 } else { sd.0 };
        if scroll != 0.0 {
            let old_zoom = *zoom;
            let factor = if scroll > 0.0 { 1.1_f32 } else { 1.0 / 1.1 };
            let new_zoom = (old_zoom * factor).clamp(0.05, 20.0);

            let origin_sx = canvas_x + canvas_w * 0.5 + *pan_x;
            let origin_sy = canvas_y + canvas_h * 0.5 + *pan_y;
            let world_x = (mx - origin_sx) / old_zoom;
            let world_y = (my - origin_sy) / old_zoom;
            *pan_x = mx - (canvas_x + canvas_w * 0.5) - world_x * new_zoom;
            *pan_y = my - (canvas_y + canvas_h * 0.5) - world_y * new_zoom;
            *zoom = new_zoom;
        }
    }

    // ── Pan ───────────────────────────────────────────────────────────────────
    let panning = ctx.input.is_button_down(MouseButton::Middle)
        || ctx.input.is_button_down(MouseButton::Right);
    if panning && over_canvas && preview_drag.is_none() && scene.drag_canvas.is_none() {
        *pan_x += mx - *last_mx;
        *pan_y += my - *last_my;
    }

    *last_mx = mx;
    *last_my = my;
}

// ── Draw ──────────────────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
pub fn draw(
    dc: &mut DrawContext<'_, '_>,
    zoom: f32,
    pan_x: f32,
    pan_y: f32,
    left_w: f32,
    right_w: f32,
    preview_w: f32,
    preview_h: f32,
    scene: &SceneState,
) {
    let (win_w, win_h) = dc.ctx.window_size;
    let ww = win_w as f32;
    let wh = win_h as f32;

    let canvas_x = left_w;
    let canvas_y = TOP_H;
    let canvas_w = ww - left_w - right_w;
    let canvas_h = wh - TOP_H;

    // Clip to canvas area so widget drawings don't bleed into panels
    dc.gui
        .push_clip(Rect::new(canvas_x, canvas_y, canvas_w, canvas_h));

    // 1. Background
    dc.gui
        .rect(canvas_x, canvas_y, canvas_w, canvas_h, c_canvas());

    // 2. Grid
    let grid = 20.0 * zoom;
    if grid > 2.0 {
        let offset_x = pan_x % grid;
        let offset_y = pan_y % grid;
        let color_grid = c_grid();
        let mut gx = canvas_x + offset_x;
        while gx < canvas_x + canvas_w {
            dc.gui.rect(gx, canvas_y, 1.0, canvas_h, color_grid);
            gx += grid;
        }
        let mut gy = canvas_y + offset_y;
        while gy < canvas_y + canvas_h {
            dc.gui.rect(canvas_x, gy, canvas_w, 1.0, color_grid);
            gy += grid;
        }
    }

    // 3. Origin axes
    let origin_x = canvas_x + canvas_w * 0.5 + pan_x;
    let origin_y = canvas_y + canvas_h * 0.5 + pan_y;
    let axis_col = ferrous_app::Color::hex("#007ACC50").to_linear_f32();
    dc.gui
        .rect(origin_x - 0.5, canvas_y, 1.0, canvas_h, axis_col);
    dc.gui
        .rect(canvas_x, origin_y - 0.5, canvas_w, 1.0, axis_col);

    // 4. Preview rect
    let (px, py, pw, ph) = preview_screen_rect(
        canvas_x, canvas_y, canvas_w, canvas_h, pan_x, pan_y, zoom, preview_w, preview_h,
    );
    let right = px + pw;
    let bottom = py + ph;

    let visible = right > canvas_x
        && px < canvas_x + canvas_w
        && bottom > canvas_y
        && py < canvas_y + canvas_h;

    if visible {
        // Shadow
        dc.gui
            .rect(px + 4.0, py + 4.0, pw, ph, [0.0, 0.0, 0.0, 0.35]);
        // White fill
        dc.gui.rect(px, py, pw, ph, [1.0, 1.0, 1.0, 1.0]);

        // Border
        let bc = [0.0_f32, 0.478, 0.8, 0.9];
        dc.gui.rect(px, py, pw, 1.0, bc);
        dc.gui.rect(px, bottom - 1.0, pw, 1.0, bc);
        dc.gui.rect(px, py, 1.0, ph, bc);
        dc.gui.rect(right - 1.0, py, 1.0, ph, bc);

        // Handles
        let hs = 7.0_f32;
        let hh = hs * 0.5;
        let mid_x = px + pw * 0.5;
        let mid_y = py + ph * 0.5;
        let hc = [0.0_f32, 0.478, 0.8, 1.0];
        let hbg = [0.12_f32, 0.12, 0.12, 1.0];
        for (hx, hy) in [
            (px - hh, py - hh),
            (mid_x - hh, py - hh),
            (right - hh, py - hh),
            (right - hh, mid_y - hh),
            (right - hh, bottom - hh),
            (mid_x - hh, bottom - hh),
            (px - hh, bottom - hh),
            (px - hh, mid_y - hh),
        ] {
            dc.gui.rect(hx, hy, hs, hs, hc);
            dc.gui.rect(hx + 1.0, hy + 1.0, hs - 2.0, hs - 2.0, hbg);
        }

        // Dimension label
        let label = format!("{:.0} x {:.0}", preview_w, preview_h);
        let label_y = (bottom + 5.0).min(canvas_y + canvas_h - 14.0);
        let label_x = px.max(canvas_x + 4.0);
        if label_y > canvas_y && label_y < canvas_y + canvas_h && label_x < canvas_x + canvas_w {
            dc.gui.draw_text(
                dc.font,
                &label,
                [label_x, label_y],
                10.0,
                ferrous_app::Color::hex("#888888").to_linear_f32(),
            );
        }
    }

    // 5. Placed widgets
    for w in &scene.widgets {
        if !w.props.visible {
            continue;
        }

        // World → screen
        let sw = w.w * zoom;
        let sh = w.h * zoom;
        let sx = origin_x + w.x * zoom - sw * 0.5;
        let sy = origin_y + w.y * zoom - sh * 0.5;

        // Skip if out of canvas
        if sx + sw < canvas_x
            || sx > canvas_x + canvas_w
            || sy + sh < canvas_y
            || sy > canvas_y + canvas_h
        {
            continue;
        }

        let is_selected = scene.selected_id == Some(w.id);

        // Fill
        let mut col = w.kind.color();
        col[3] = if is_selected { 0.85 } else { 0.65 };
        dc.gui.rect(sx, sy, sw, sh, col);

        // Border
        let border_col = if is_selected {
            [1.0_f32, 0.8, 0.0, 1.0] // golden selection border
        } else {
            [col[0] * 1.4, col[1] * 1.4, col[2] * 1.4, 0.9]
        };
        let bw = if is_selected { 2.0 } else { 1.0 };
        dc.gui.rect(sx, sy, sw, bw, border_col);
        dc.gui.rect(sx, sy + sh - bw, sw, bw, border_col);
        dc.gui.rect(sx, sy, bw, sh, border_col);
        dc.gui.rect(sx + sw - bw, sy, bw, sh, border_col);

        // Label
        if sw > 20.0 && sh > 12.0 {
            let label = if !w.props.label.is_empty() {
                w.props.label.as_str()
            } else {
                w.kind.name()
            };
            let font_size = (11.0 * zoom).clamp(8.0, 16.0);
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

        // Selection handles (corners + midpoints)
        if is_selected {
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
    }

    // 6. Controls hint
    dc.gui.draw_text(
        dc.font,
        "Rueda: zoom  |  Btn.medio / Btn.derecho: pan  |  Click: seleccionar  |  Arrastrar widget desde paleta",
        [canvas_x + 8.0, canvas_y + canvas_h - 18.0],
        10.0,
        ferrous_app::Color::hex("#444444").to_linear_f32(),
    );

    dc.gui.pop_clip();
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn preview_screen_rect(
    canvas_x: f32,
    canvas_y: f32,
    canvas_w: f32,
    canvas_h: f32,
    pan_x: f32,
    pan_y: f32,
    zoom: f32,
    preview_w: f32,
    preview_h: f32,
) -> (f32, f32, f32, f32) {
    let origin_sx = canvas_x + canvas_w * 0.5 + pan_x;
    let origin_sy = canvas_y + canvas_h * 0.5 + pan_y;
    let pw = preview_w * zoom;
    let ph = preview_h * zoom;
    (origin_sx - pw * 0.5, origin_sy - ph * 0.5, pw, ph)
}

fn detect_handle(mx: f32, my: f32, px: f32, py: f32, pw: f32, ph: f32) -> Option<PreviewDrag> {
    let h = HANDLE_HIT;
    let right = px + pw;
    let bottom = py + ph;

    let on_tl = (mx - px).abs() < h && (my - py).abs() < h;
    let on_tr = (mx - right).abs() < h && (my - py).abs() < h;
    let on_bl = (mx - px).abs() < h && (my - bottom).abs() < h;
    let on_br = (mx - right).abs() < h && (my - bottom).abs() < h;

    let on_left = (mx - px).abs() < h && my > py + h && my < bottom - h;
    let on_right = (mx - right).abs() < h && my > py + h && my < bottom - h;
    let on_top = (my - py).abs() < h && mx > px + h && mx < right - h;
    let on_bottom = (my - bottom).abs() < h && mx > px + h && mx < right - h;

    if on_tl {
        Some(PreviewDrag::TopLeft)
    } else if on_tr {
        Some(PreviewDrag::TopRight)
    } else if on_bl {
        Some(PreviewDrag::BottomLeft)
    } else if on_br {
        Some(PreviewDrag::BottomRight)
    } else if on_left {
        Some(PreviewDrag::Left)
    } else if on_right {
        Some(PreviewDrag::Right)
    } else if on_top {
        Some(PreviewDrag::Top)
    } else if on_bottom {
        Some(PreviewDrag::Bottom)
    } else {
        None
    }
}
