use ferrous_engine::{AppContext, Color as AppColor, DrawContext, MouseButton};

use crate::{scene::SceneState, TOP_H, theme::{Theme, ThemeRegistry}};

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
        let content_top = TOP_H + TAB_H;
        if my < content_top { return; }

        let local_y = my - content_top + palette_state.hierarchy_scroll_y;
        let mut cursor_y = 4.0_f32;

        let mut id_at_y = None;
        fn collect_click_ids(scene: &SceneState, parent_id: Option<u32>, cursor_y: &mut f32, local_y: f32, id_at_y: &mut Option<u32>) {
            // Iteramos en orden inverso de Z-order (rev) para que coincida con el dibujo
            let children: Vec<u32> = scene.widgets.iter().rev().filter(|w| w.parent_id == parent_id).map(|w| w.id).collect();
            for id in children {
                if local_y >= *cursor_y && local_y < *cursor_y + ITEM_H {
                    *id_at_y = Some(id);
                }
                *cursor_y += ITEM_H + 2.0;
                
                // Si este ya fue el clickeado, no seguimos buscando hijos (o sí, pero id_at_y ya está seteado)
                if id_at_y.is_some() { return; }

                collect_click_ids(scene, Some(id), cursor_y, local_y, id_at_y);
                if id_at_y.is_some() { return; }
            }
        }

        collect_click_ids(scene, None, &mut cursor_y, local_y, &mut id_at_y);

        if let Some(id) = id_at_y {
            // Eye button (visibility)
            let eye_x = left_w - 28.0;
            // Move button (up/down)
            let btn_w = 20.0;
            let up_x = left_w - 28.0 - btn_w - 4.0;
            let down_x = up_x - btn_w - 4.0;

            if mx >= eye_x && mx < left_w - 4.0 {
                if let Some(w) = scene.widgets.iter_mut().find(|w| w.id == id) {
                    w.props.visible = !w.props.visible;
                }
            } else if mx >= up_x && mx < up_x + btn_w {
                scene.selected_id = Some(id);
                scene.move_up();
            } else if mx >= down_x && mx < down_x + btn_w {
                scene.selected_id = Some(id);
                scene.move_down();
            } else {
                let prev = scene.selected_id;
                scene.selected_id = Some(id);
                if prev != Some(id) {
                    scene.sync_prop_strings();
                }
            }
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
    theme: &Theme,
) {
    let scroll = palette_state.hierarchy_scroll_y;
    let cursor_y = content_top + 4.0 - scroll;
    let eye_x = left_w - 2.0 - 26.0;

    if scene.widgets.is_empty() {
        dc.gui.draw_text(
            dc.font,
            "No widgets in scene",
            [ITEM_PAD_X, content_top + 16.0],
            11.0,
            ThemeRegistry::hex_to_f32(&theme.colors.text_dim),
        );
        return;
    }

    // Recursive draw function
    fn draw_tree_recursive(
        dc: &mut DrawContext<'_, '_>,
        scene: &SceneState,
        parent_id: Option<u32>,
        left_w: f32,
        cursor_y: &mut f32,
        content_top: f32,
        content_h: f32,
        mx: f32,
        my: f32,
        theme: &Theme,
        depth: usize,
    ) {
        // Obtenemos hijos y los dibujamos en Z-order invertido
        let children: Vec<&crate::scene::PlacedWidget> = scene.widgets.iter().rev().filter(|w| w.parent_id == parent_id).collect();
        
        for w in children {
            let item_y = *cursor_y;
            *cursor_y += ITEM_H + 2.0;

            // Skip if out of view
            if item_y + ITEM_H < content_top || item_y > content_top + content_h {
                // Sigue dibujando hijos aunque el padre sea invisible (para ver el scroll correcto)
                draw_tree_recursive(dc, scene, Some(w.id), left_w, cursor_y, content_top, content_h, mx, my, theme, depth + 1);
                continue;
            }

            let is_selected = scene.selected_id == Some(w.id);
            let row_hovered = mx >= 0.0 && mx < left_w - 2.0 && my >= item_y && my < item_y + ITEM_H;

            // Row background
            let bg = if is_selected {
                let mut c = ThemeRegistry::hex_to_f32(&theme.colors.accent);
                c[3] = 0.4;
                c
            } else if row_hovered {
                let mut c = ThemeRegistry::hex_to_f32(&theme.colors.primary);
                c[3] = 0.2;
                c
            } else {
                ThemeRegistry::hex_to_f32(&theme.colors.surface)
            };
            dc.gui.rect(0.0, item_y, left_w - 2.0, ITEM_H, bg);

            // Left colour bar
            dc.gui.rect(0.0, item_y + 3.0, 3.0, ITEM_H - 6.0, w.kind.color());

            // Indentation
            let indent = depth as f32 * 12.0;

            // Widget name
            let label = if !w.props.label.is_empty() && w.props.label != w.kind.display_name() {
                format!("{} — {}", w.kind.display_name(), &w.props.label)
            } else {
                w.kind.display_name().to_string()
            };

            let text_col = ThemeRegistry::hex_to_f32(&theme.colors.text);
            dc.gui.draw_text(dc.font, &label, [ITEM_PAD_X + 8.0 + indent, item_y + 7.0], 9.0, text_col);

            // Eye / Visibility
            let eye_x = left_w - 28.0;
            let eye_hovered = mx >= eye_x && mx < left_w - 4.0 && my >= item_y && my < item_y + ITEM_H;
            let eye_col = if eye_hovered {
                ThemeRegistry::hex_to_f32(&theme.colors.text)
            } else if w.props.visible {
                ThemeRegistry::hex_to_f32(&theme.colors.accent)
            } else {
                ThemeRegistry::hex_to_f32(&theme.colors.text_dim)
            };
            let effectively_visible = scene.is_effectively_visible(w.id);
            let eye_txt = if !w.props.visible { "HID" } else if !effectively_visible { "I-HI" } else { "VIS" };
            dc.gui.draw_text(dc.font, eye_txt, [eye_x, item_y + 7.0], 9.0, eye_col);

            // Move buttons
            let btn_w = 20.0;
            let up_x = eye_x - btn_w - 4.0;
            let down_x = up_x - btn_w - 4.0;
            let up_h = mx >= up_x && mx < up_x + btn_w && my >= item_y && my < item_y + ITEM_H;
            let down_h = mx >= down_x && mx < down_x + btn_w && my >= item_y && my < item_y + ITEM_H;

            dc.gui.draw_text(dc.font, "UP", [up_x + 2.0, item_y + 7.0], 9.0, if up_h { ThemeRegistry::hex_to_f32(&theme.colors.accent) } else { ThemeRegistry::hex_to_f32(&theme.colors.text_dim) });
            dc.gui.draw_text(dc.font, "DN", [down_x + 2.0, item_y + 7.0], 9.0, if down_h { ThemeRegistry::hex_to_f32(&theme.colors.accent) } else { ThemeRegistry::hex_to_f32(&theme.colors.text_dim) });

            if !effectively_visible {
                dc.gui.rect(ITEM_PAD_X + indent, item_y + ITEM_H * 0.5, left_w - 100.0 - indent, 1.0, ThemeRegistry::hex_to_f32(&theme.colors.text_dim));
            }

            // Draw children recursively
            draw_tree_recursive(dc, scene, Some(w.id), left_w, cursor_y, content_top, content_h, mx, my, theme, depth + 1);
        }
    }

    let mut cursor_y = content_top + 4.0 - scroll;
    draw_tree_recursive(dc, scene, None, left_w, &mut cursor_y, content_top, content_h, mx, my, theme, 0);
}
