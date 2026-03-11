use ferrous_app::{AppContext, Color as AppColor, DrawContext, MouseButton};
use ferrous_ui_core::Rect;

use crate::{
    c_border,
    scene::{SceneState, PALETTE},
    TOP_H,
    // hierarchy-specific logic has been moved into its own module
    panels::hierarchy_panel::{draw_hierarchy, update_hierarchy, hierarchy_total_height},
};

// ── Layout constants ──────────────────────────────────────────────────────────
const ITEM_H: f32 = 26.0;
const ITEM_PAD_X: f32 = 10.0;
const CAT_H: f32 = 22.0;
const SCROLL_SPEED: f32 = 30.0;
const TAB_H: f32 = 30.0; // height of the Palette / Hierarchy tab bar

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LeftTab {
    Palette,
    Hierarchy,
}

/// State for the left panel (palette + hierarchy).
pub struct PaletteState {
    pub collapsed: [bool; 7],
    pub scroll_y: f32,
    pub active_tab: LeftTab,
    pub hierarchy_scroll_y: f32,
}

impl Default for PaletteState {
    fn default() -> Self {
        Self {
            collapsed: [false; 7],
            scroll_y: 0.0,
            active_tab: LeftTab::Palette,
            hierarchy_scroll_y: 0.0,
        }
    }
}

// ── Update ────────────────────────────────────────────────────────────────────

pub fn update(
    ctx: &mut AppContext,
    left_w: f32,
    scene: &mut SceneState,
    palette_state: &mut PaletteState,
) {
    let (_, win_h) = ctx.window_size;
    let panel_h = win_h as f32 - TOP_H;
    let (mx, my) = ctx.input.mouse_pos_f32();

    let in_panel = mx >= 0.0 && mx < left_w && my >= TOP_H && my < TOP_H + panel_h;

    let lmb_pressed = ctx.input.button_just_pressed(MouseButton::Left);
    let lmb_released = ctx.input.button_just_released(MouseButton::Left);

    // ── Tab bar click ──────────────────────────────────────────────────────────
    if lmb_pressed && in_panel && my >= TOP_H && my < TOP_H + TAB_H {
        let tab_w = (left_w - 2.0) / 2.0;
        if mx < tab_w {
            palette_state.active_tab = LeftTab::Palette;
        } else {
            palette_state.active_tab = LeftTab::Hierarchy;
        }
        return;
    }

    // ── Scroll wheel ──────────────────────────────────────────────────────────
    if in_panel {
        let (_, sy) = ctx.input.scroll_delta();
        if sy != 0.0 {
            match palette_state.active_tab {
                LeftTab::Palette => {
                    let total_h = palette_total_height(palette_state);
                    let content_h = panel_h - TAB_H;
                    let max_scroll = (total_h - content_h + 16.0).max(0.0);
                    palette_state.scroll_y =
                        (palette_state.scroll_y - sy * SCROLL_SPEED).clamp(0.0, max_scroll);
                }
                LeftTab::Hierarchy => {
                    let total_h = hierarchy_total_height(scene);
                    let content_h = panel_h - TAB_H;
                    let max_scroll = (total_h - content_h + 16.0).max(0.0);
                    palette_state.hierarchy_scroll_y = (palette_state.hierarchy_scroll_y
                        - sy * SCROLL_SPEED)
                        .clamp(0.0, max_scroll);
                }
            }
        }
    }

    // Release: if drag ends outside panel, canvas.rs will handle the drop.
    if lmb_released && in_panel {
        scene.palette_drag = None;
    }

    if lmb_pressed && in_panel {
        // below the tab bar
        let content_top = TOP_H + TAB_H;
        if my < content_top {
            return;
        }

        match palette_state.active_tab {
            LeftTab::Palette => {
                let local_y = my - content_top + palette_state.scroll_y;
                let mut cursor_y = 0.0_f32;

                'outer: for (cat_idx, cat) in PALETTE.iter().enumerate() {
                    if local_y >= cursor_y && local_y < cursor_y + CAT_H {
                        palette_state.collapsed[cat_idx] = !palette_state.collapsed[cat_idx];
                        break 'outer;
                    }
                    cursor_y += CAT_H + 2.0;

                    if palette_state.collapsed[cat_idx] {
                        continue;
                    }

                    for &kind in cat.widgets {
                        if local_y >= cursor_y && local_y < cursor_y + ITEM_H {
                            scene.palette_drag = Some(kind);
                            break 'outer;
                        }
                        cursor_y += ITEM_H + 2.0;
                    }
                    cursor_y += 4.0;
                }
            }

            LeftTab::Hierarchy => {
                // the full hierarchy-handling logic lives in the new module now
                update_hierarchy(ctx, left_w, scene, palette_state);
            }
        }
    }
}

// ── Draw ──────────────────────────────────────────────────────────────────────

pub fn draw(
    dc: &mut DrawContext<'_, '_>,
    left_w: f32,
    palette_state: &PaletteState,
    scene: &SceneState,
) {
    let (_, win_h) = dc.ctx.window_size;
    let panel_h = win_h as f32 - TOP_H;
    let (mx, my) = dc.ctx.input.mouse_pos_f32();
    let tab_w = (left_w - 2.0) / 2.0;

    // ── Tab bar ───────────────────────────────────────────────────────────────
    let tab_bar_y = TOP_H;
    dc.gui.rect(
        0.0,
        tab_bar_y,
        left_w - 2.0,
        TAB_H,
        AppColor::hex("#2D2D30").to_linear_f32(),
    );

    for (i, label) in ["Palette", "Hierarchy"].iter().enumerate() {
        let tx = i as f32 * tab_w;
        let is_active = match i {
            0 => palette_state.active_tab == LeftTab::Palette,
            _ => palette_state.active_tab == LeftTab::Hierarchy,
        };
        let tab_bg = if is_active {
            AppColor::hex("#252526").to_linear_f32()
        } else {
            AppColor::hex("#2D2D30").to_linear_f32()
        };
        dc.gui.rect(tx, tab_bar_y, tab_w, TAB_H, tab_bg);
        // Active tab indicator line
        if is_active {
            dc.gui.rect(
                tx,
                tab_bar_y + TAB_H - 2.0,
                tab_w,
                2.0,
                [0.0, 0.478, 0.800, 1.0],
            );
        }
        // Separator between tabs
        if i == 0 {
            dc.gui.rect(
                tab_w - 1.0,
                tab_bar_y,
                1.0,
                TAB_H,
                AppColor::hex("#3C3C3C").to_linear_f32(),
            );
        }
        dc.gui.draw_text(
            dc.font,
            label,
            [tx + tab_w * 0.5 - label.len() as f32 * 3.2, tab_bar_y + 9.0],
            11.0,
            if is_active {
                AppColor::hex("#FFFFFF").to_linear_f32()
            } else {
                AppColor::hex("#888888").to_linear_f32()
            },
        );
    }

    // ── Content area (below tab bar) ──────────────────────────────────────────
    let content_top = TOP_H + TAB_H;
    let content_h = panel_h - TAB_H;
    dc.gui
        .push_clip(Rect::new(0.0, content_top, left_w - 2.0, content_h));

    match palette_state.active_tab {
        LeftTab::Palette => draw_palette(dc, left_w, palette_state, content_top, content_h, mx, my),
        LeftTab::Hierarchy => draw_hierarchy(
            dc,
            left_w,
            palette_state,
            scene,
            content_top,
            content_h,
            mx,
            my,
        ),
    }

    dc.gui.pop_clip();

    // Ghost tooltip: shows what widget is being dragged (rendered outside clip)
    if let Some(kind) = scene.palette_drag {
        let gx = mx + 12.0;
        let gy = my - 11.0;
        dc.gui.rect(gx, gy, 120.0, 22.0, [0.0, 0.478, 0.800, 0.9]);
        dc.gui.draw_text(
            dc.font,
            kind.display_name(),
            [gx + 6.0, gy + 5.0],
            10.0,
            AppColor::hex("#FFFFFF").to_linear_f32(),
        );
    }
}

fn draw_palette(
    dc: &mut DrawContext<'_, '_>,
    left_w: f32,
    palette_state: &PaletteState,
    content_top: f32,
    content_h: f32,
    mx: f32,
    my: f32,
) {
    let scroll = palette_state.scroll_y;
    let mut cursor_y = content_top - scroll;

    for (cat_idx, cat) in PALETTE.iter().enumerate() {
        let cat_y = cursor_y;
        if cat_y + CAT_H >= content_top && cat_y < content_top + content_h {
            let arrow = if palette_state.collapsed[cat_idx] {
                "▶"
            } else {
                "▼"
            };
            dc.gui.rect(
                0.0,
                cat_y,
                left_w - 2.0,
                CAT_H,
                AppColor::hex("#2D2D30").to_linear_f32(),
            );
            dc.gui.draw_text(
                dc.font,
                arrow,
                [ITEM_PAD_X, cat_y + 5.0],
                9.0,
                AppColor::hex("#888888").to_linear_f32(),
            );
            dc.gui.draw_text(
                dc.font,
                cat.name,
                [ITEM_PAD_X + 14.0, cat_y + 5.0],
                11.0,
                AppColor::hex("#CCCCCC").to_linear_f32(),
            );
        }
        cursor_y += CAT_H + 2.0;

        if palette_state.collapsed[cat_idx] {
            continue;
        }

        for &kind in cat.widgets {
            if cursor_y + ITEM_H >= content_top && cursor_y < content_top + content_h {
                let hovered =
                    mx >= 0.0 && mx < left_w - 2.0 && my >= cursor_y && my < cursor_y + ITEM_H;
                let bg = if hovered {
                    AppColor::hex("#37373D").to_linear_f32()
                } else {
                    AppColor::hex("#252526").to_linear_f32()
                };
                dc.gui.rect(0.0, cursor_y, left_w - 2.0, ITEM_H, bg);
                dc.gui
                    .rect(0.0, cursor_y + 3.0, 3.0, ITEM_H - 6.0, kind.color());
                dc.gui.draw_text(
                    dc.font,
                    kind.display_name(),
                    [ITEM_PAD_X + 8.0, cursor_y + 6.0],
                    11.0,
                    if hovered {
                        AppColor::hex("#FFFFFF").to_linear_f32()
                    } else {
                        AppColor::hex("#BBBBBB").to_linear_f32()
                    },
                );
            }
            cursor_y += ITEM_H + 2.0;
        }
        cursor_y += 4.0;
    }
}


// ── Height helper ─────────────────────────────────────────────────────────────

fn palette_total_height(state: &PaletteState) -> f32 {
    let mut h = 26.0;
    for (i, cat) in PALETTE.iter().enumerate() {
        h += CAT_H + 2.0;
        if !state.collapsed[i] {
            h += (ITEM_H + 2.0) * cat.widgets.len() as f32 + 4.0;
        }
    }
    h
}

// ── Background / border ───────────────────────────────────────────────────────

pub fn draw_background(dc: &mut DrawContext<'_, '_>, left_w: f32) {
    let (_, win_h) = dc.ctx.window_size;
    let panel_h = win_h as f32 - TOP_H;
    dc.gui.rect(
        0.0,
        TOP_H,
        left_w,
        panel_h,
        AppColor::hex("#252526").to_linear_f32(),
    );
}

pub fn draw_border(dc: &mut DrawContext<'_, '_>, left_w: f32) {
    let (_, win_h) = dc.ctx.window_size;
    let canvas_h = win_h as f32 - TOP_H;
    dc.gui.rect(left_w - 2.0, TOP_H, 2.0, canvas_h, c_border());
}
