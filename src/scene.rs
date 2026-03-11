// ── Scene state ────────────────────────────────────────────────────────────────
//
// Holds all placed widgets and the selection / drag state for the scene builder.
//
// `WidgetKind`, `WidgetCategory`, `PaletteCategory`, and `WIDGET_REGISTRY` live
// in `ferrous_ui_core` — the single source of truth. We re-export them here so
// the rest of GUIMaker can keep importing from `scene` unchanged.
use ferrous_ui_core::TextFieldState;

pub use ferrous_ui_core::WidgetKind;
/// Alias kept for backward compat within this binary.
pub use ferrous_ui_core::WIDGET_REGISTRY as PALETTE;
// Re-export alignment types so the rest of GUIMaker can import from scene
pub use ferrous_ui_core::{HAlign, TextAlign, VAlign};

// ─────────────────────────────────────────────────────────────────────────────
// (All WidgetKind/impl WidgetKind/PaletteCategory/PALETTE definitions have been
//  moved to ferrous_ui_core::widgets::widget_meta)
// ─────────────────────────────────────────────────────────────────────────────

// Placeholder so we can reuse the old enum-start marker for the replacement
// block below.  Immediately removed.
/// Properties editable in the inspector panel.
#[derive(Clone, Debug)]
pub struct WidgetProps {
    pub label: String,
    pub color_hex: String,
    /// Button background color (hex). Defaults to theme primary "#6C63FF".
    pub bg_color_hex: String,
    /// Button text color (hex). Defaults to theme on_primary "#FFFFFF".
    pub text_color_hex: String,
    /// Button border radii [top-left, top-right, bottom-right, bottom-left] in px.
    /// Each corner defaults to 6.0 (theme dark default).
    pub border_radii: [f32; 4],
    pub visible: bool,
    pub font_size: f32,
    // For sliders / number inputs
    pub min: f32,
    pub max: f32,
    pub value: f32,
    // For checkboxes / toggles
    pub checked: bool,
    /// Horizontal alignment of the label text within the widget bounds.
    pub label_h_align: HAlign,
    /// Vertical alignment of the label text within the widget bounds.
    pub label_v_align: VAlign,
    /// Custom offset value (px or %) used when h_align == Custom.
    pub label_h_custom: f32,
    /// Whether `label_h_custom` is a percentage (true) or pixel offset (false).
    pub label_h_custom_pct: bool,
    /// Custom offset value (px or %) used when v_align == Custom.
    pub label_v_custom: f32,
    /// Whether `label_v_custom` is a percentage (true) or pixel offset (false).
    pub label_v_custom_pct: bool,
    /// Pivot for horizontal custom alignment (0.0 = left edge of text, 0.5 = center, 1.0 = right edge).
    pub label_h_pivot: f32,
    /// Pivot for vertical custom alignment (0.0 = top edge of text, 0.5 = center, 1.0 = bottom edge).
    pub label_v_pivot: f32,
}

impl WidgetProps {
    /// Builds a `TextAlign` from the stored alignment properties.
    pub fn text_align(&self) -> TextAlign {
        let h = match self.label_h_align {
            HAlign::Custom { .. } => HAlign::Custom {
                value: self.label_h_custom,
                percent: self.label_h_custom_pct,
                pivot: self.label_h_pivot,
            },
            other => other,
        };
        let v = match self.label_v_align {
            VAlign::Custom { .. } => VAlign::Custom {
                value: self.label_v_custom,
                percent: self.label_v_custom_pct,
                pivot: self.label_v_pivot,
            },
            other => other,
        };
        TextAlign::new(h, v)
    }
}

impl Default for WidgetProps {
    fn default() -> Self {
        Self {
            label: String::new(),
            color_hex: "#FFFFFF".to_string(),
            bg_color_hex: "#6C63FF".to_string(),
            text_color_hex: "#FFFFFF".to_string(),
            border_radii: [6.0; 4],
            visible: true,
            font_size: 14.0,
            min: 0.0,
            max: 100.0,
            value: 50.0,
            checked: false,
            label_h_align: HAlign::Center,
            label_v_align: VAlign::Center,
            label_h_custom: 0.0,
            label_h_custom_pct: true,
            label_v_custom: 50.0,
            label_v_custom_pct: true,
            label_h_pivot: 0.5,
            label_v_pivot: 0.5,
        }
    }
}

/// A widget instance placed on the canvas.
#[derive(Clone, Debug)]
pub struct PlacedWidget {
    pub id: u32,
    pub kind: WidgetKind,
    /// Position in world (canvas) coordinates.
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub props: WidgetProps,
}

impl PlacedWidget {
    pub fn new(id: u32, kind: WidgetKind, x: f32, y: f32) -> Self {
        let (w, h) = kind.default_size();
        let mut props = WidgetProps::default();
        props.label = kind.display_name().to_string();
        Self {
            id,
            kind,
            x,
            y,
            w,
            h,
            props,
        }
    }
}

/// Top-level scene state: the list of placed widgets and selection/drag info.
pub struct SceneState {
    pub widgets: Vec<PlacedWidget>,
    next_id: u32,

    /// Widget being dragged on the canvas (id, offset from widget origin).
    pub drag_canvas: Option<(u32, f32, f32)>,

    /// Widget being resized from one of its 8 handles (id, handle direction).
    pub resize_widget: Option<(u32, crate::PreviewDrag)>,

    /// Button corner being dragged to adjust its border radius (id, corner index 0=TL 1=TR 2=BR 3=BL).
    pub radius_drag: Option<(u32, usize)>,

    /// Widget kind being dragged from the palette (ghost follows the cursor).
    pub palette_drag: Option<WidgetKind>,

    /// Currently selected widget id.
    pub selected_id: Option<u32>,

    /// Property field currently being edited (field_key) and its string buffer.
    pub editing_field: Option<String>,
    pub edit_buffer: String,

    // ── Input animation / interaction state — managed by TextFieldState ─────
    pub edit_state: TextFieldState,

    /// Prop delta widgets for +/- buttons in the inspector.
    pub prop_x_str: String,
    pub prop_y_str: String,
    pub prop_w_str: String,
    pub prop_h_str: String,
}

impl Default for SceneState {
    fn default() -> Self {
        Self {
            widgets: Vec::new(),
            next_id: 1,
            drag_canvas: None,
            resize_widget: None,
            radius_drag: None,
            palette_drag: None,
            selected_id: None,
            editing_field: None,
            edit_buffer: String::new(),
            edit_state: TextFieldState::new(),
            prop_x_str: String::new(),
            prop_y_str: String::new(),
            prop_w_str: String::new(),
            prop_h_str: String::new(),
        }
    }
}

impl SceneState {
    pub fn add_widget(&mut self, kind: WidgetKind, x: f32, y: f32) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.widgets.push(PlacedWidget::new(id, kind, x, y));
        id
    }

    pub fn selected(&self) -> Option<&PlacedWidget> {
        let id = self.selected_id?;
        self.widgets.iter().find(|w| w.id == id)
    }

    pub fn selected_mut(&mut self) -> Option<&mut PlacedWidget> {
        let id = self.selected_id?;
        self.widgets.iter_mut().find(|w| w.id == id)
    }

    /// Sync the inspector string buffers from the currently selected widget.
    pub fn sync_prop_strings(&mut self) {
        let id = match self.selected_id {
            Some(id) => id,
            None => return,
        };
        if let Some(w) = self.widgets.iter().find(|w| w.id == id) {
            self.prop_x_str = format!("{:.0}", w.x);
            self.prop_y_str = format!("{:.0}", w.y);
            self.prop_w_str = format!("{:.0}", w.w);
            self.prop_h_str = format!("{:.0}", w.h);
        }
    }

    /// Delete the selected widget.
    pub fn delete_selected(&mut self) {
        if let Some(id) = self.selected_id {
            self.widgets.retain(|w| w.id != id);
            self.selected_id = None;
        }
    }
}
