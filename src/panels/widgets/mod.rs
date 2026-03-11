// ── widgets/ ──────────────────────────────────────────────────────────────────
//
// One module per widget kind + shared helpers.
// All public items that widget_editor.rs needs are re-exported here.

pub mod shared;
pub mod label_align;
pub mod label;
pub mod button;
pub mod generic;

// Re-export the things widget_editor.rs and right_panel.rs import directly.
pub use shared::{
    HEADER_H, SECTION_H, ROW_H, ROW_PAD_X, BTN_W, LABEL_W,
    default_row_colors, parse_hex_or,
    draw_section, draw_row_label,
    draw_text_row, draw_number_row, draw_info_row,
    draw_selection_handles,
    draw_pivot_overlay,
};
pub use button::radius_handle_pos;
