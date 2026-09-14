// ── Widget Editor ─────────────────────────────────────────────────────────────
//
// Thin dispatcher: routes draw/update/commit calls to the per-kind modules
// inside `crate::panels::widgets`.  To add a new widget kind, create a new
// file in widgets/ and add its match arms here.

use ferrous_engine::{AppContext, Color as AppColor, DrawContext, MouseButton};

use crate::scene::{SceneState, WidgetKind, PlacedWidget};
use crate::panels::widgets;

// Re-export everything right_panel.rs imports from widget_editor.
pub use widgets::{HEADER_H, SECTION_H, ROW_H, ROW_PAD_X, BTN_W, LABEL_W};
pub use widgets::{
    draw_section, draw_row_label, draw_text_row, draw_number_row, draw_info_row,
    default_row_colors, parse_hex_or, radius_handle_pos,
    draw_selection_handles,
};

// ── Canvas: draw one placed widget ───────────────────────────────────────────

pub fn draw_widget(
    dc: &mut DrawContext<'_, '_>,
    w: &PlacedWidget,
    sx: f32,
    sy: f32,
    sw: f32,
    sh: f32,
    is_selected: bool,
    active_radius_corner: Option<usize>,
    theme: &crate::theme::Theme,
) {
    match w.kind {
        WidgetKind::Label  => widgets::label::draw_canvas(dc, w, sx, sy, sw, sh, is_selected, theme),
        WidgetKind::Button => widgets::button::draw_canvas(dc, w, sx, sy, sw, sh, is_selected, active_radius_corner, theme),
        _                  => widgets::generic::draw_canvas(dc, w, sx, sy, sw, sh, is_selected, theme),
    }
    if is_selected {
        draw_selection_handles(dc, sx, sy, sw, sh, theme);
    }
}

// ── Inspector: draw property rows ────────────────────────────────────────────

pub fn draw_properties(
    dc: &mut DrawContext<'_, '_>,
    scene: &mut SceneState,
    panel_x: f32,
    right_w: f32,
    cursor_y: f32,
    theme: &crate::theme::Theme,
    cp: &crate::color_picker::ColorPickerState,
) -> f32 {
    let mut cy = cursor_y;

    let Some(selected) = scene.selected() else { return cy; };
    let selected_id = selected.id;
    let kind = selected.kind;

    draw_section(dc, panel_x, cy, right_w, "Properties", theme);
    cy += SECTION_H;

    let (mx, my) = dc.ctx.input.mouse_pos_f32();
    let clicked = dc.ctx.input.button_just_pressed(MouseButton::Left);

    // Visible toggle
    widgets::draw_row_label(dc, panel_x, cy, "Visible", theme);
    let toggle_x = panel_x + widgets::ROW_PAD_X + widgets::LABEL_W + 4.0;
    let toggle_label = if scene.selected().map_or(false, |w| w.props.visible) { "On" } else { "Off" };
    if dc.gui.button(dc.font, toggle_x, cy + 3.0, 60.0, ROW_H - 6.0, toggle_label, mx, my, clicked) {
        if let Some(w) = scene.widgets.iter_mut().find(|w| w.id == selected_id) {
            w.props.visible = !w.props.visible;
        }
    }
    cy += ROW_H;

    // Per-kind property rows
    match kind {
        WidgetKind::Label  => widgets::label::draw_props(dc, scene, panel_x, right_w, &mut cy, selected_id, mx, my, theme),
        WidgetKind::Button => widgets::button::draw_props(dc, scene, panel_x, right_w, &mut cy, selected_id, mx, my, theme),
        _                  => widgets::generic::draw_props(dc, scene, panel_x, right_w, &mut cy, mx, my, theme),
    }

    // Layout Constraints Section
    widgets::layout_props::draw_layout_section(dc, scene, panel_x, right_w, &mut cy, mx, my, theme);

    // Delete button
    let del_y = cy + 8.0;
    if dc.gui.button_colored(
        dc.font, panel_x + 10.0, del_y, right_w - 20.0, 28.0, "Delete Widget",
        mx, my, clicked,
        AppColor::hex("#3C3C3C").to_linear_f32(),
        AppColor::hex("#C0392B").to_linear_f32(),
    ) {
        scene.delete_selected();
    }

    let has_parent = scene.selected().map(|w| w.parent_id.is_some()).unwrap_or(false);
    if has_parent {
        let parent_y = del_y + 36.0;
        cy = parent_y + 28.0;
        if dc.gui.button_colored(
            dc.font, panel_x + 10.0, parent_y, right_w - 20.0, 28.0, "Unparent",
            mx, my, clicked,
            AppColor::hex("#3C3C3C").to_linear_f32(),
            AppColor::hex("#6C63FF").to_linear_f32(),
        ) {
            if let Some(id) = scene.selected_id {
                scene.set_parent(id, None);
            }
        }
    } else {
        cy = del_y + 28.0;
    }

    cy
}

// ── Inspector: update (click / focus) ────────────────────────────────────────

pub fn update_properties(
    ctx: &mut AppContext,
    scene: &mut SceneState,
    _panel_x: f32,
    _right_w: f32,
    _mx: f32,
    my: f32,
    raw_my: f32,
    hit_x: f32,
    cursor_y: &mut f32,
    cp: &mut crate::color_picker::ColorPickerState,
) {
    *cursor_y += SECTION_H; // skip "Properties" header

    let clicked = ctx.input.button_just_pressed(MouseButton::Left);
    let Some(selected_id) = scene.selected_id else { return; };
    let kind = scene.widgets.iter().find(|w| w.id == selected_id).map(|w| w.kind);
    let Some(kind) = kind else { return; };

    *cursor_y += ROW_H; // skip Visible toggle row

    if !clicked { return; }

    match kind {
        WidgetKind::Label  => widgets::label::update_props(ctx, scene, _panel_x, _right_w, _mx, my, hit_x, cursor_y, cp),
        WidgetKind::Button => widgets::button::update_props(ctx, scene, _panel_x, _right_w, _mx, my, hit_x, cursor_y, cp),
        _                  => widgets::generic::update_props(ctx, scene, _panel_x, _right_w, _mx, my, raw_my, hit_x, cursor_y, cp),
    }

    widgets::layout_props::update_layout_props(ctx, scene, _panel_x, _right_w, _mx, my, hit_x, cursor_y);
}

// ── Commit active text-field edit ─────────────────────────────────────────────

pub fn commit_field(scene: &mut SceneState, field: &str, buf: String) {
    // Transform fields (shared, handled directly)
    if let Some(w) = scene.selected_mut() {
        match field {
            "tx" => { if let Ok(v) = buf.trim().parse::<f32>() { w.x = v; } return; }
            "ty" => { if let Ok(v) = buf.trim().parse::<f32>() { w.y = v; } return; }
            "tw" => { if let Ok(v) = buf.trim().parse::<f32>() { w.w = v.max(8.0); } return; }
            "th" => { if let Ok(v) = buf.trim().parse::<f32>() { w.h = v.max(8.0); } return; }
            _ => {}
        }
    }

    // Alignment fields (shared between Label + Button)
    if widgets::label_align::commit_field(scene, field, buf.clone()) { return; }

    // Per-kind fields
    if widgets::layout_props::commit_layout_field(scene, field, buf.clone()) { return; }

    let kind = scene.selected().map(|w| w.kind);
    match kind {
        Some(WidgetKind::Label)  => { widgets::label::commit_field(scene, field, buf); }
        Some(WidgetKind::Button) => { widgets::button::commit_field(scene, field, buf); }
        _                        => { widgets::generic::commit_field(scene, field, buf); }
    }
}

