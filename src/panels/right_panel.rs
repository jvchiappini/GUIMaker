use ferrous_app::{AppContext, Color as AppColor, DrawContext};
use ferrous_gui::{Color, NodeId, Rect, UiTree};
use ferrous_ui_core::{Button, EventContext, Label, Panel, StyleBuilder};

use crate::{
    c_border,
    scene::{SceneState, WidgetKind},
    GUIMakerApp, TOP_H,
};

// ── Palette ───────────────────────────────────────────────────────────────────
const C_TEXT: [f32; 4] = [0.867, 0.867, 0.867, 1.0];
const C_MUTED: [f32; 4] = [0.55, 0.55, 0.55, 1.0];
const C_FIELD_BG: [f32; 4] = [0.098, 0.098, 0.098, 1.0];
const C_FIELD_BORDER: [f32; 4] = [0.267, 0.267, 0.267, 1.0];
const C_SECTION: [f32; 4] = [0.18, 0.18, 0.184, 1.0];
const C_ACCENT: [f32; 4] = [0.0, 0.478, 0.800, 1.0];
const C_DELETE: [f32; 4] = [0.75, 0.18, 0.18, 1.0];
const C_DELETE_H: [f32; 4] = [0.90, 0.22, 0.22, 1.0];

// ── Layout constants ──────────────────────────────────────────────────────────
const PAD: f32 = 10.0;
const ROW_H: f32 = 28.0;
const FIELD_H: f32 = 22.0;
const LABEL_W: f32 = 54.0;

fn hit(mx: f32, my: f32, x: f32, y: f32, w: f32, h: f32) -> bool {
    mx >= x && mx <= x + w && my >= y && my <= y + h
}

// ── Update ────────────────────────────────────────────────────────────────────

pub fn update(_ctx: &mut AppContext, _right_w: f32, _scene: &mut SceneState) {
    // No-op. Previously contained manual hit testing.
}

// ── Draw ──────────────────────────────────────────────────────────────────────

pub fn draw(dc: &mut DrawContext<'_, '_>, right_w: f32, scene: &SceneState) {
    let (win_w, win_h) = dc.ctx.window_size;
    let ww = win_w as f32;
    let panel_h = win_h as f32 - TOP_H;
    let panel_x = ww - right_w;
    let (mx, my) = dc.ctx.input.mouse_pos_f32();

    dc.gui
        .push_clip(Rect::new(panel_x, TOP_H, right_w, panel_h));

    // ── Header ────────────────────────────────────────────────────────────────
    dc.gui.rect(
        panel_x,
        TOP_H,
        right_w,
        34.0,
        AppColor::hex("#2D2D30").to_linear_f32(),
    );
    dc.gui.draw_text(
        dc.font,
        "PROPERTIES",
        [panel_x + PAD, TOP_H + 10.0],
        10.0,
        AppColor::hex("#888888").to_linear_f32(),
    );

    let Some(selected) = scene.selected() else {
        // Nothing selected hint
        dc.gui.draw_text(
            dc.font,
            "No widget selected",
            [panel_x + PAD, TOP_H + 60.0],
            11.0,
            C_MUTED,
        );
        dc.gui.pop_clip();
        return;
    };

    // if the currently selected widget is hidden, show a reminder so the user
    // doesn't assume the inspector has cleared
    if !selected.props.visible {
        dc.gui.draw_text(
            dc.font,
            "(hidden widget)",
            [panel_x + PAD, TOP_H + 60.0],
            11.0,
            C_MUTED,
        );
    }

    let mut cy = TOP_H + 38.0;

    // Kind badge
    dc.gui.rect(
        panel_x + PAD,
        cy,
        right_w - PAD * 2.0,
        22.0,
        selected.kind.color(),
    );
    dc.gui.draw_text(
        dc.font,
        selected.kind.name(),
        [panel_x + PAD + 6.0, cy + 5.0],
        11.0,
        C_TEXT,
    );
    cy += 26.0;

    // ── Transform section ────────────────────────────────────────────────────
    section_header(dc, panel_x, cy, right_w, "Transform");
    cy += 20.0;

    let field_vals = [selected.x, selected.y, selected.w, selected.h];
    let field_names = ["X", "Y", "W", "H"];
    for (i, (&val, &name)) in field_vals.iter().zip(field_names.iter()).enumerate() {
        prop_row_number(dc, panel_x, cy, right_w, mx, my, name, val, i < 2);
        cy += ROW_H;
    }
    cy += 6.0;

    // ── Properties section ────────────────────────────────────────────────────
    section_header(dc, panel_x, cy, right_w, "Properties");
    cy += 20.0;

    // Visible
    prop_row_toggle(
        dc,
        panel_x,
        cy,
        right_w,
        mx,
        my,
        "Visible",
        selected.props.visible,
    );
    cy += ROW_H;

    // Label (if applicable)
    match selected.kind {
        WidgetKind::Label
        | WidgetKind::Button
        | WidgetKind::Checkbox
        | WidgetKind::ToggleSwitch
        | WidgetKind::Tooltip
        | WidgetKind::Toast => {
            prop_row_text(dc, panel_x, cy, right_w, "Label", &selected.props.label);
            cy += ROW_H;
        }
        WidgetKind::TextInput | WidgetKind::NumberInput => {
            prop_row_text(dc, panel_x, cy, right_w, "Hint", &selected.props.label);
            cy += ROW_H;
        }
        _ => {}
    }

    // Font size (for text-bearing widgets)
    match selected.kind {
        WidgetKind::Label | WidgetKind::Button | WidgetKind::TextInput => {
            prop_row_number(
                dc,
                panel_x,
                cy,
                right_w,
                mx,
                my,
                "Size",
                selected.props.font_size,
                false,
            );
            cy += ROW_H;
        }
        _ => {}
    }

    // Slider / NumberInput range
    match selected.kind {
        WidgetKind::Slider | WidgetKind::NumberInput | WidgetKind::ProgressBar => {
            prop_row_number(
                dc,
                panel_x,
                cy,
                right_w,
                mx,
                my,
                "Min",
                selected.props.min,
                false,
            );
            cy += ROW_H;
            prop_row_number(
                dc,
                panel_x,
                cy,
                right_w,
                mx,
                my,
                "Max",
                selected.props.max,
                false,
            );
            cy += ROW_H;
            prop_row_number(
                dc,
                panel_x,
                cy,
                right_w,
                mx,
                my,
                "Val",
                selected.props.value,
                false,
            );
            cy += ROW_H;
        }
        _ => {}
    }

    // Color hex
    match selected.kind {
        WidgetKind::Panel | WidgetKind::Label | WidgetKind::Button => {
            prop_row_text(dc, panel_x, cy, right_w, "Color", &selected.props.color_hex);
            cy += ROW_H;
        }
        _ => {}
    }

    cy += 8.0;

    // ── Delete button ─────────────────────────────────────────────────────────
    let del_x = panel_x + PAD;
    let del_w = right_w - PAD * 2.0;
    let del_y = cy;
    let hov_del = hit(mx, my, del_x, del_y, del_w, 28.0);
    dc.gui.rect(
        del_x,
        del_y,
        del_w,
        28.0,
        if hov_del { C_DELETE_H } else { C_DELETE },
    );
    dc.gui.draw_text(
        dc.font,
        "Delete Widget",
        [del_x + 8.0, del_y + 8.0],
        11.0,
        C_TEXT,
    );

    dc.gui.pop_clip();
}

// ── Sub-draw helpers ──────────────────────────────────────────────────────────

fn section_header(dc: &mut DrawContext<'_, '_>, px: f32, cy: f32, rw: f32, title: &str) {
    dc.gui.rect(px, cy, rw, 18.0, C_SECTION);
    // accent line
    dc.gui.rect(px, cy + 16.0, rw, 2.0, C_ACCENT);
    dc.gui
        .draw_text(dc.font, title, [px + PAD, cy + 3.0], 10.0, C_MUTED);
}

fn prop_row_number(
    dc: &mut DrawContext<'_, '_>,
    px: f32,
    cy: f32,
    rw: f32,
    mx: f32,
    my: f32,
    label: &str,
    val: f32,
    _can_be_neg: bool,
) {
    dc.gui
        .draw_text(dc.font, label, [px + PAD, cy + 7.0], 10.0, C_MUTED);

    let field_x = px + PAD + LABEL_W;
    let field_w = rw - PAD * 2.0 - LABEL_W - 50.0;
    let btn_m_x = field_x + field_w + 2.0;
    let btn_p_x = btn_m_x + 24.0;

    // Field bg
    dc.gui.rect(field_x, cy + 3.0, field_w, FIELD_H, C_FIELD_BG);
    dc.gui.rect(field_x, cy + 3.0, 1.0, FIELD_H, C_FIELD_BORDER);
    dc.gui.rect(field_x, cy + 3.0, field_w, 1.0, C_FIELD_BORDER);
    dc.gui.rect(
        field_x,
        cy + 3.0 + FIELD_H - 1.0,
        field_w,
        1.0,
        C_FIELD_BORDER,
    );
    dc.gui.rect(
        field_x + field_w - 1.0,
        cy + 3.0,
        1.0,
        FIELD_H,
        C_FIELD_BORDER,
    );

    dc.gui.draw_text(
        dc.font,
        &format!("{:.0}", val),
        [field_x + 4.0, cy + 6.0],
        10.0,
        C_TEXT,
    );

    // – button
    let hov_m = hit(mx, my, btn_m_x, cy + 3.0, 22.0, FIELD_H);
    dc.gui.rect(
        btn_m_x,
        cy + 3.0,
        22.0,
        FIELD_H,
        if hov_m {
            [0.267, 0.267, 0.267, 1.0]
        } else {
            [0.18, 0.18, 0.18, 1.0]
        },
    );
    dc.gui
        .draw_text(dc.font, "−", [btn_m_x + 6.0, cy + 4.0], 12.0, C_TEXT);

    // + button
    let hov_p = hit(mx, my, btn_p_x, cy + 3.0, 22.0, FIELD_H);
    dc.gui.rect(
        btn_p_x,
        cy + 3.0,
        22.0,
        FIELD_H,
        if hov_p {
            [0.267, 0.267, 0.267, 1.0]
        } else {
            [0.18, 0.18, 0.18, 1.0]
        },
    );
    dc.gui
        .draw_text(dc.font, "+", [btn_p_x + 6.0, cy + 4.0], 12.0, C_TEXT);
}

fn prop_row_text(
    dc: &mut DrawContext<'_, '_>,
    px: f32,
    cy: f32,
    rw: f32,
    label: &str,
    value: &str,
) {
    dc.gui
        .draw_text(dc.font, label, [px + PAD, cy + 7.0], 10.0, C_MUTED);

    let field_x = px + PAD + LABEL_W;
    let field_w = rw - PAD * 2.0 - LABEL_W;
    dc.gui.rect(field_x, cy + 3.0, field_w, FIELD_H, C_FIELD_BG);
    dc.gui.rect(field_x, cy + 3.0, 1.0, FIELD_H, C_FIELD_BORDER);
    dc.gui.rect(field_x, cy + 3.0, field_w, 1.0, C_FIELD_BORDER);
    dc.gui.rect(
        field_x,
        cy + 3.0 + FIELD_H - 1.0,
        field_w,
        1.0,
        C_FIELD_BORDER,
    );
    dc.gui.rect(
        field_x + field_w - 1.0,
        cy + 3.0,
        1.0,
        FIELD_H,
        C_FIELD_BORDER,
    );

    // Truncate text to fit
    let display = if value.len() > 20 {
        &value[..20]
    } else {
        value
    };
    dc.gui
        .draw_text(dc.font, display, [field_x + 4.0, cy + 6.0], 10.0, C_TEXT);
}

fn prop_row_toggle(
    dc: &mut DrawContext<'_, '_>,
    px: f32,
    cy: f32,
    rw: f32,
    mx: f32,
    my: f32,
    label: &str,
    checked: bool,
) {
    dc.gui
        .draw_text(dc.font, label, [px + PAD, cy + 7.0], 10.0, C_MUTED);

    let toggle_x = px + PAD + LABEL_W;
    let _hov = hit(mx, my, toggle_x, cy + 4.0, 36.0, 20.0);
    // Track
    let track_col = if checked { C_ACCENT } else { C_FIELD_BG };
    dc.gui.rect(toggle_x, cy + 4.0, 36.0, 20.0, track_col);
    // Knob
    let knob_x = if checked {
        toggle_x + 18.0
    } else {
        toggle_x + 2.0
    };
    dc.gui.rect(knob_x, cy + 6.0, 16.0, 16.0, C_TEXT);
    // Label hint
    let state_label = if checked { "On" } else { "Off" };
    dc.gui.draw_text(
        dc.font,
        state_label,
        [toggle_x + 42.0, cy + 7.0],
        10.0,
        C_MUTED,
    );

    let _ = rw; // suppress unused warning
}

// ── Background / border ───────────────────────────────────────────────────────

pub fn draw_background(dc: &mut DrawContext<'_, '_>, right_w: f32) {
    let (win_w, win_h) = dc.ctx.window_size;
    let ww = win_w as f32;
    let panel_h = win_h as f32 - TOP_H;
    let right_x = ww - right_w;
    dc.gui.rect(
        right_x,
        TOP_H,
        right_w,
        panel_h,
        AppColor::hex("#252526").to_linear_f32(),
    );
}

pub fn draw_border(dc: &mut DrawContext<'_, '_>, right_w: f32) {
    let (win_w, win_h) = dc.ctx.window_size;
    let ww = win_w as f32;
    let canvas_h = win_h as f32 - TOP_H;
    let right_x = ww - right_w;
    dc.gui.rect(right_x, TOP_H, 2.0, canvas_h, c_border());
}

pub fn configure_ui(ui: &mut UiTree<GUIMakerApp>, right_panel_id: NodeId, scene: &SceneState) {
    let Some(selected) = scene.selected() else {
        let label = Label::new("No widget selected").with_style(StyleBuilder::new().padding_all(20.0).build());
        ui.add_node(Box::new(label), Some(right_panel_id));
        return;
    };

    // Skip redundant rebuild check if needed, but for now we'll rely on the main.rs
    // clear_node_children logic.
    
    // Adjust right_panel_id layout
    ui.set_node_style(
        right_panel_id,
        StyleBuilder::new()
            .padding_all(0.0)
            .padding_top(38.0) // Space for header badge
            .build(),
    );

    // Header Badge
    let kind_name = selected.kind.name().to_string();
    let col_arr = selected.kind.color();
    let badge = Panel::new().with_color(Color::new(col_arr[0], col_arr[1], col_arr[2], col_arr[3]));
    let badge_id = ui.add_node(Box::new(badge), Some(right_panel_id));
    ui.set_node_style(
        badge_id,
        StyleBuilder::new()
            .height_px(22.0)
            .margin_xy(10.0, 5.0)
            .build(),
    );
    let badge_text = Label::new(&kind_name);
    ui.add_node(Box::new(badge_text), Some(badge_id));

    // Sections
    add_section(ui, right_panel_id, "Transform");
    add_transform_row(ui, right_panel_id, "X", selected.id, 0);
    add_transform_row(ui, right_panel_id, "Y", selected.id, 1);
    add_transform_row(ui, right_panel_id, "W", selected.id, 2);
    add_transform_row(ui, right_panel_id, "H", selected.id, 3);

    add_section(ui, right_panel_id, "Properties");
    add_toggle_row(ui, right_panel_id, "Visible", selected.id);

    // Spacer
    let spacer = Panel::new().with_color(Color::hex("#00000000"));
    let spacer_id = ui.add_node(Box::new(spacer), Some(right_panel_id));
    ui.set_node_style(spacer_id, StyleBuilder::new().flex(1.0).build());

    // Delete Button
    let del_btn = Button::new("Delete Widget").on_click(|ctx: &mut EventContext<GUIMakerApp>| {
        ctx.app.scene.delete_selected();
    });
    let del_id = ui.add_node(Box::new(del_btn), Some(right_panel_id));
    ui.set_node_style(
        del_id,
        StyleBuilder::new()
            .margin_all(10.0)
            .height_px(28.0)
            .build(),
    );
}

fn add_section(ui: &mut UiTree<GUIMakerApp>, parent: NodeId, title: &str) {
    let section = Panel::new().with_color(Color::hex("#2D2D30"));
    let sec_id = ui.add_node(Box::new(section), Some(parent));
    ui.set_node_style(
        sec_id,
        StyleBuilder::new().height_px(20.0).padding_all(2.0).build(),
    );
    let lbl = Label::new(title);
    ui.add_node(Box::new(lbl), Some(sec_id));
}

fn add_transform_row(
    ui: &mut UiTree<GUIMakerApp>,
    parent: NodeId,
    label: &str,
    widget_id: u32,
    field: u8,
) {
    let row = Panel::new().with_color(Color::hex("#00000000"));
    let row_id = ui.add_node(Box::new(row), Some(parent));
    ui.set_node_style(
        row_id,
        StyleBuilder::new()
            .row()
            .height_px(28.0)
            .padding_xy(10.0, 3.0)
            .build(),
    );

    let lbl = Label::new(label);
    let lbl_id = ui.add_node(Box::new(lbl), Some(row_id));
    ui.set_node_style(lbl_id, StyleBuilder::new().width_px(54.0).build());

    // Value label (simplified, using Label instead of full TextInput for now)
    let val_box = Panel::new().with_color(Color::hex("#191919"));
    let val_id = ui.add_node(Box::new(val_box), Some(row_id));
    ui.set_node_style(
        val_id,
        StyleBuilder::new().flex(1.0).margin_xy(0.0, 0.0).build(),
    );

    // Buttons
    let m_btn = Button::new("-").on_click(move |ctx: &mut EventContext<GUIMakerApp>| {
        if let Some(w) = ctx.app.scene.widgets.iter_mut().find(|w| w.id == widget_id) {
            match field {
                0 => w.x -= 1.0,
                1 => w.y -= 1.0,
                2 => w.w = (w.w - 1.0).max(8.0),
                3 => w.h = (w.h - 1.0).max(8.0),
                _ => {}
            }
        }
    });
    let m_id = ui.add_node(Box::new(m_btn), Some(row_id));
    ui.set_node_style(
        m_id,
        StyleBuilder::new()
            .width_px(22.0)
            .margin_xy(2.0, 0.0)
            .build(),
    );

    let p_btn = Button::new("+").on_click(move |ctx: &mut EventContext<GUIMakerApp>| {
        if let Some(w) = ctx.app.scene.widgets.iter_mut().find(|w| w.id == widget_id) {
            match field {
                0 => w.x += 1.0,
                1 => w.y += 1.0,
                2 => w.w += 1.0,
                3 => w.h += 1.0,
                _ => {}
            }
        }
    });
    let p_id = ui.add_node(Box::new(p_btn), Some(row_id));
    ui.set_node_style(
        p_id,
        StyleBuilder::new()
            .width_px(22.0)
            .margin_xy(2.0, 0.0)
            .build(),
    );
}

fn add_toggle_row(ui: &mut UiTree<GUIMakerApp>, parent: NodeId, label: &str, widget_id: u32) {
    let row = Panel::new().with_color(Color::hex("#00000000"));
    let row_id = ui.add_node(Box::new(row), Some(parent));
    ui.set_node_style(
        row_id,
        StyleBuilder::new()
            .row()
            .height_px(28.0)
            .padding_xy(10.0, 4.0)
            .build(),
    );

    let lbl = Label::new(label);
    let lbl_id = ui.add_node(Box::new(lbl), Some(row_id));
    ui.set_node_style(lbl_id, StyleBuilder::new().width_px(54.0).build());

    let toggle_btn = Button::new("Toggle").on_click(move |ctx: &mut EventContext<GUIMakerApp>| {
        if let Some(w) = ctx.app.scene.widgets.iter_mut().find(|w| w.id == widget_id) {
            w.props.visible = !w.props.visible;
        }
    });
    let t_id = ui.add_node(Box::new(toggle_btn), Some(row_id));
    ui.set_node_style(t_id, StyleBuilder::new().width_px(60.0).build());
}
