use ferrous_app::{AppContext, Color as AppColor, DrawContext, MouseButton};

use crate::{scene::SceneState, TOP_H};

// constants are duplicated from left_panel; keeping them local avoids
// making the original ones public and keeps the module self-contained.
const ITEM_H: f32 = 26.0;
const ITEM_PAD_X: f32 = 10.0;
const SCROLL_SPEED: f32 = 30.0;
const TAB_H: f32 = 30.0; // height of the Palette / Hierarchy tab bar

/// Helper used by the left panel state
pub fn hierarchy_total_height(scene: &SceneState) -> f32 {
    // every row is ITEM_H high with 2px spacing, plus 8px padding
    scene.widgets.len() as f32 * (ITEM_H + 2.0) + 8.0
}

/// Update logic for the hierarchy tab. This used to live inside `left_panel.rs` but
/// it was moved here to keep the two panels separate.
pub fn update_hierarchy(
    ctx: &mut AppContext,
    left_w: f32,
    scene: &mut SceneState,
    palette_state: &mut crate::panels::left_panel::PaletteState,
) {
    let (_, win_h) = ctx.window_size;
    let panel_h = win_h as f32 - TOP_H;
    let (mx, my) = ctx.input.mouse_pos_f32();

    let in_panel = mx >= 0.0 && mx < left_w && my >= TOP_H && my < TOP_H + panel_h;
    let lmb_pressed = ctx.input.button_just_pressed(MouseButton::Left);

    // scroll wheel (only when the cursor is over the left panel)
    if in_panel {
        let (_, sy) = ctx.input.scroll_delta();
        if sy != 0.0 {
            let total_h = hierarchy_total_height(scene);
            let content_h = panel_h - TAB_H;
            let max_scroll = (total_h - content_h + 16.0).max(0.0);
            palette_state.hierarchy_scroll_y =
                (palette_state.hierarchy_scroll_y - sy * SCROLL_SPEED).clamp(0.0, max_scroll);
        }
    }

    if lmb_pressed && in_panel {
        // pointer must be below the tab bar
        let content_top = TOP_H + TAB_H;
        if my < content_top {
            return;
        }

        let local_y = my - content_top + palette_state.hierarchy_scroll_y;
        let mut cursor_y = 4.0_f32;

        // collect ids in reverse order (same as draw_hierarchy) so that the
        // click Y positions match the rendered row positions
        let ids: Vec<u32> = scene.widgets.iter().rev().map(|w| w.id).collect();
        for id in ids {
            if local_y >= cursor_y && local_y < cursor_y + ITEM_H {
                // eye icon region on the right
                let eye_x = left_w - 2.0 - 26.0;
                if mx >= eye_x && mx < left_w - 4.0 {
                    if let Some(w) = scene.widgets.iter_mut().find(|w| w.id == id) {
                        w.props.visible = !w.props.visible;
                    }
                } else {
                    let prev = scene.selected_id;
                    scene.selected_id = Some(id);
                    if prev != Some(id) {
                        scene.sync_prop_strings();
                    }
                }
                break;
            }
            cursor_y += ITEM_H + 2.0;
        }
    }
}

/// Draw the contents of the hierarchy tab.  Invisible widgets are shown with a
/// dimmer text colour and a hollow eye symbol so they can't be "lost".
pub fn draw_hierarchy(
    dc: &mut DrawContext<'_, '_>,
    left_w: f32,
    palette_state: &crate::panels::left_panel::PaletteState,
    scene: &SceneState,
    content_top: f32,
    content_h: f32,
    mx: f32,
    my: f32,
) {
    let scroll = palette_state.hierarchy_scroll_y;
    let mut cursor_y = content_top + 4.0 - scroll;
    let eye_x = left_w - 2.0 - 26.0;

    if scene.widgets.is_empty() {
        dc.gui.draw_text(
            dc.font,
            "No widgets in scene",
            [ITEM_PAD_X, content_top + 16.0],
            11.0,
            AppColor::hex("#555555").to_linear_f32(),
        );
        return;
    }

    // Draw in reverse so the "top" widget in Z-order appears first
    for w in scene.widgets.iter().rev() {
        if cursor_y + ITEM_H < content_top || cursor_y > content_top + content_h {
            cursor_y += ITEM_H + 2.0;
            continue;
        }

        let is_selected = scene.selected_id == Some(w.id);
        let row_hovered =
            mx >= 0.0 && mx < left_w - 2.0 && my >= cursor_y && my < cursor_y + ITEM_H;

        // Row background
        let bg = if is_selected {
            [0.0_f32, 0.267, 0.467, 0.6] // blue tint
        } else if row_hovered {
            AppColor::hex("#37373D").to_linear_f32()
        } else {
            AppColor::hex("#252526").to_linear_f32()
        };
        dc.gui.rect(0.0, cursor_y, left_w - 2.0, ITEM_H, bg);

        // Left colour bar
        dc.gui
            .rect(0.0, cursor_y + 3.0, 3.0, ITEM_H - 6.0, w.kind.color());

        // Widget name (kind + label)
        let label = if !w.props.label.is_empty() && w.props.label != w.kind.name() {
            format!("{} — {}", w.kind.name(), &w.props.label)
        } else {
            w.kind.name().to_string()
        };

        // visible widgets are bright; hidden ones use the same colour but
        // we'll also draw a strikethrough so they cannot be mistaken for gone
        let text_col = AppColor::hex("#CCCCCC").to_linear_f32();
        dc.gui.draw_text(
            dc.font,
            &label,
            [ITEM_PAD_X + 8.0, cursor_y + 6.0],
            11.0,
            text_col,
        );

        // Eye icon (● = visible, ○ = hidden)
        let eye_hovered =
            mx >= eye_x && mx < left_w - 4.0 && my >= cursor_y && my < cursor_y + ITEM_H;
        let eye_col = if eye_hovered {
            AppColor::hex("#FFFFFF").to_linear_f32()
        } else if w.props.visible {
            AppColor::hex("#888888").to_linear_f32()
        } else {
            AppColor::hex("#555555").to_linear_f32()
        };
        let eye_sym = if w.props.visible { "●" } else { "○" };
        dc.gui.draw_text(
            dc.font,
            eye_sym,
            [eye_x + 4.0, cursor_y + 5.0],
            11.0,
            eye_col,
        );

        // indicate hidden state with a strike-through line
        if !w.props.visible {
            let line_y = cursor_y + ITEM_H * 0.5;
            dc.gui.rect(
                ITEM_PAD_X,
                line_y,
                left_w - 2.0 - ITEM_PAD_X * 2.0,
                1.0,
                AppColor::hex("#888888").to_linear_f32(),
            );
        }

        cursor_y += ITEM_H + 2.0;
    }
}
