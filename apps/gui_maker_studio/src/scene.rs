// ── Scene state ────────────────────────────────────────────────────────────────
//
// Holds all placed widgets and the selection / drag state for the scene builder.
//
// `WidgetKind`, `WidgetCategory`, `PaletteCategory`, and `WIDGET_REGISTRY` live
// in `ferrous_ui_core` — the single source of truth. We re-export them here so
// the rest of GUIMaker can keep importing from `scene` unchanged.
use ferrous_ui_core::{Background, GradientAngle, GradientStop, TextFieldState};

pub use ferrous_ui_core::WidgetKind;
/// Alias kept for backward compat within this binary.
pub use ferrous_ui_core::WIDGET_REGISTRY as PALETTE;
// Re-export alignment types so the rest of GUIMaker can import from scene
pub use ferrous_ui_core::{HAlign, TextAlign, VAlign};

// ─────────────────────────────────────────────────────────────────────────────
// (All WidgetKind/impl WidgetKind/PaletteCategory/PALETTE definitions have been
//  moved to ferrous_ui_core::widgets::widget_meta)
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AutoLayoutDirection {
    Horizontal,
    Vertical,
}

impl Default for AutoLayoutDirection {
    fn default() -> Self {
        AutoLayoutDirection::Horizontal
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum LayoutValue {
    Absolute(f32),
    Percent(f32),
    Fraction(f32),
    Auto,
    MinMax { min: Box<LayoutValue>, max: Box<LayoutValue> },
}

impl Default for LayoutValue {
    fn default() -> Self {
        Self::Absolute(0.0)
    }
}

impl LayoutValue {
    pub fn resolve(&self, parent_size: f32) -> f32 {
        match self {
            Self::Absolute(v) => *v,
            Self::Percent(p) => parent_size * (p / 100.0),
            Self::Fraction(f) => parent_size * *f,
            Self::Auto => 0.0,
            Self::MinMax { min, max } => {
                let min_val = min.resolve(parent_size);
                let max_val = max.resolve(parent_size);
                min_val.max(0.0).min(max_val)
            }
        }
    }

    pub fn parse(input: &str) -> Option<LayoutValue> {
        let s = input.trim();
        if s.is_empty() || s == "---" { return None; }
        if s.eq_ignore_ascii_case("auto") { return Some(LayoutValue::Auto); }
        if let Some(stripped) = s.strip_suffix("fr") {
            if let Ok(v) = stripped.trim().parse::<f32>() {
                return Some(LayoutValue::Fraction(v));
            }
        }
        if s.to_lowercase().starts_with("minmax(") && s.ends_with(')') {
            let inner = &s[7..s.len() - 1];
            let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();
            if parts.len() == 2 {
                let min_val = LayoutValue::parse(parts[0])?;
                let max_val = LayoutValue::parse(parts[1])?;
                return Some(LayoutValue::MinMax { min: Box::new(min_val), max: Box::new(max_val) });
            }
        }
        if s.ends_with('%') {
            if let Ok(v) = s[..s.len()-1].parse::<f32>() {
                return Some(LayoutValue::Percent(v));
            }
        }
        if s.ends_with("px") {
            if let Ok(v) = s[..s.len()-2].parse::<f32>() {
                return Some(LayoutValue::Absolute(v));
            }
        }
        if let Ok(v) = s.parse::<f32>() {
            return Some(LayoutValue::Absolute(v));
        }
        None
    }

    pub fn to_string_clean(&self) -> String {
        match self {
            Self::Absolute(v) => format!("{:.0}px", v),
            Self::Percent(p) => format!("{:.0}%", p),
            Self::Fraction(f) => format!("{:.2}fr", f),
            Self::Auto => "auto".to_string(),
            Self::MinMax { min, max } => format!("minmax({},{})", min.to_string_clean(), max.to_string_clean()),
        }
    }

    pub fn value(&self) -> f32 {
        match self {
            Self::Absolute(v) => *v,
            Self::Percent(p) => *p,
            Self::Fraction(f) => *f,
            Self::Auto => 0.0,
            Self::MinMax { min, .. } => min.value(),
        }
    }
}

/// Gradient type used by widgets (for inspector controls).
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum GradientKind {
    None,
    Linear,
    Radial,
    Conic,
}

impl GradientKind {
    pub fn label(&self) -> &'static str {
        match self {
            GradientKind::None => "None",
            GradientKind::Linear => "Linear",
            GradientKind::Radial => "Radial",
            GradientKind::Conic => "Conic",
        }
    }

    pub fn cycle_next(&self) -> Self {
        match self {
            GradientKind::None => GradientKind::Linear,
            GradientKind::Linear => GradientKind::Radial,
            GradientKind::Radial => GradientKind::Conic,
            GradientKind::Conic => GradientKind::None,
        }
    }
}

/// Properties editable in the inspector panel.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
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
    /// Fixed size constraints in pixels.
    pub min_width: Option<f32>,
    pub pref_width: Option<f32>,
    pub max_width: Option<f32>,
    pub min_height: Option<f32>,
    pub pref_height: Option<f32>,
    pub max_height: Option<f32>,
    /// Layout anchoring constraints.
    pub anchor_top: Option<LayoutValue>,
    pub anchor_right: Option<LayoutValue>,
    pub anchor_bottom: Option<LayoutValue>,
    pub anchor_left: Option<LayoutValue>,
    /// Padding [top, right, bottom, left].
    pub padding: [LayoutValue; 4],
    /// Auto-layout controls.
    pub auto_layout: bool,
    pub auto_layout_direction: AutoLayoutDirection,
    pub auto_layout_gap: f32,
    /// Gradient styling
    pub gradient_type: GradientKind,
    pub gradient_stops: Vec<(f32, String)>,
    pub gradient_angle: f32,
    pub gradient_center: [f32; 2],
    pub gradient_radius: f32,
    pub gradient_start_angle: f32,
    /// Global widget opacity (0.0..1.0) applied on top of color alpha.
    pub opacity: f32,
}

impl WidgetProps {
    pub fn text_align(&self) -> TextAlign {
        let h = match self.label_h_align {
            HAlign::Custom { value, percent, .. } => HAlign::Custom { value, percent, pivot: self.label_h_pivot },
            other => other,
        };
        let v = match self.label_v_align {
            VAlign::Custom { value, percent, .. } => VAlign::Custom { value, percent, pivot: self.label_v_pivot },
            other => other,
        };
        TextAlign::new(h, v)
    }

    pub fn to_background(&self) -> Background {
        if self.gradient_type != GradientKind::None && !self.gradient_stops.is_empty() {
            let stops: Vec<GradientStop> = self
                .gradient_stops
                .iter()
                .map(|(pos, color)| GradientStop::from_hex(*pos, color))
                .collect();
            match self.gradient_type {
                GradientKind::Linear => Background::LinearGradient {
                    stops,
                    angle: GradientAngle(self.gradient_angle),
                },
                GradientKind::Radial => Background::RadialGradient {
                    stops,
                    center: self.gradient_center,
                    radius: self.gradient_radius,
                },
                GradientKind::Conic => Background::ConicGradient {
                    stops,
                    center: self.gradient_center,
                    start_angle: self.gradient_start_angle,
                },
                GradientKind::None => Background::None,
            }
        } else {
            Background::None
        }
    }
}

impl Default for WidgetProps {
    fn default() -> Self {
        Self {
            label: String::new(),
            color_hex: "#000000".into(),
            bg_color_hex: "#6C63FF".into(),
            text_color_hex: "#FFFFFF".into(),
            border_radii: [6.0, 6.0, 6.0, 6.0],
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
            min_width: None,
            pref_width: None,
            max_width: None,
            min_height: None,
            pref_height: None,
            max_height: None,
            anchor_top: None,
            anchor_right: None,
            anchor_bottom: None,
            anchor_left: None,
            padding: [LayoutValue::Absolute(0.0), LayoutValue::Absolute(0.0), LayoutValue::Absolute(0.0), LayoutValue::Absolute(0.0)],
            auto_layout: false,
            auto_layout_direction: AutoLayoutDirection::Horizontal,
            auto_layout_gap: 8.0,
            gradient_type: GradientKind::None,
            gradient_stops: Vec::new(),
            gradient_angle: 0.0,
            gradient_center: [0.5, 0.5],
            gradient_radius: 0.5,
            gradient_start_angle: 0.0,
            opacity: 1.0,
        }
    }
}

/// A widget instance placed on the canvas.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PlacedWidget {
    pub id: u32,
    pub kind: WidgetKind,
    /// Position in world (canvas) coordinates.
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub props: WidgetProps,
    pub parent_id: Option<u32>,
}

impl PlacedWidget {
    pub fn new(id: u32, kind: WidgetKind, x: f32, y: f32) -> Self {
        let (w, h) = kind.default_size();
        let mut props = WidgetProps::default();
        props.label = kind.display_name().to_string();
        Self { id, kind, x, y, w, h, props, parent_id: None }
    }
}

/// A snapping guide line drawn during drag operations.
#[derive(Clone, Debug)]
pub struct SnapLine {
    pub pos: f32,
    pub is_vertical: bool,
    pub length_start: f32,
    pub length_end: f32,
}

mod layout;
mod snap;

/// Top-level scene state: the list of placed widgets and selection/drag info.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SceneState {
    pub widgets: Vec<PlacedWidget>,
    pub next_id: u32,

    /// Widget being dragged on the canvas (id, offset from widget origin).
    #[serde(skip)]
    pub drag_canvas: Option<(u32, f32, f32)>,

    /// Widget being resized from one of its 8 handles (id, handle direction).
    #[serde(skip)]
    pub resize_widget: Option<(u32, crate::PreviewDrag)>,

    /// Button corner being dragged to adjust its border radius (id, corner index 0=TL 1=TR 2=BR 3=BL).
    pub radius_drag: Option<(u32, usize)>,

    /// Widget kind being dragged from the palette (ghost follows the cursor).
    #[serde(skip)]
    pub palette_drag: Option<WidgetKind>,

    /// Widget being duplicated/placed (follows cursor before click).
    #[serde(skip)]
    pub placement_drag: Option<PlacedWidget>,

    /// Internal clipboard for copy-paste within the editor.
    #[serde(skip)]
    pub copied_widget: Option<PlacedWidget>,

    /// Currently selected widget id.
    pub selected_id: Option<u32>,

    /// Property field currently being edited (field_key) and its string buffer.
    pub editing_field: Option<String>,
    pub edit_buffer: String,

    // ── Input animation / interaction state — managed by TextFieldState ─────
    #[serde(skip)]
    pub edit_state: TextFieldState,

    /// Prop delta widgets for +/- buttons in the inspector.
    pub prop_x_str: String,
    pub prop_y_str: String,
    pub prop_w_str: String,
    pub prop_h_str: String,

    /// Global grid snap active
    pub snap_to_grid: bool,
    /// Snap distance in world units
    pub snap_threshold: f32,
    /// Current snap lines to draw
    #[serde(skip)]
    pub active_snap_lines: Vec<SnapLine>,
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
            placement_drag: None,
            copied_widget: None,
            selected_id: None,
            editing_field: None,
            edit_buffer: String::new(),
            edit_state: TextFieldState::new(),
            prop_x_str: String::new(),
            prop_y_str: String::new(),
            prop_w_str: String::new(),
            prop_h_str: String::new(),
            snap_to_grid: true,
            snap_threshold: 4.0,
            active_snap_lines: Vec::new(),
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

    /// Checks if a widget is visible, considering its parents' visibility.
    pub fn is_effectively_visible(&self, widget_id: u32) -> bool {
        let mut curr_id = Some(widget_id);
        while let Some(id) = curr_id {
            if let Some(w) = self.widgets.iter().find(|w| w.id == id) {
                if !w.props.visible {
                    return false;
                }
                curr_id = w.parent_id;
            } else {
                break;
            }
        }
        true
    }

    /// Moves a widget and all its children by a delta.
    pub fn move_widget(&mut self, id: u32, dx: f32, dy: f32) {
        let mut to_move = vec![id];
        let mut i = 0;
        while i < to_move.len() {
            let pid = to_move[i];
            let children: Vec<u32> = self.widgets.iter().filter(|w| w.parent_id == Some(pid)).map(|w| w.id).collect();
            to_move.extend(children);
            i += 1;
        }

        for mid in to_move {
            if let Some(w) = self.widgets.iter_mut().find(|w| w.id == mid) {
                w.x += dx;
                w.y += dy;
            }
        }
    }

    pub fn delete_selected(&mut self) {
        if let Some(id) = self.selected_id {
            let mut to_delete = vec![id];
            let mut i = 0;
            while i < to_delete.len() {
                let pid = to_delete[i];
                let children: Vec<u32> = self.widgets.iter().filter(|w| w.parent_id == Some(pid)).map(|w| w.id).collect();
                to_delete.extend(children);
                i += 1;
            }

            self.widgets.retain(|w| !to_delete.contains(&w.id));
            if to_delete.contains(&id) {
                self.selected_id = None;
            }
        }
    }

    pub fn move_up(&mut self) {
        if let Some(id) = self.selected_id {
            if let Some(idx) = self.widgets.iter().position(|w| w.id == id) {
                if idx < self.widgets.len() - 1 {
                    self.widgets.swap(idx, idx + 1);
                }
            }
        }
    }

    pub fn move_down(&mut self) {
        if let Some(id) = self.selected_id {
            if let Some(idx) = self.widgets.iter().position(|w| w.id == id) {
                if idx > 0 {
                    self.widgets.swap(idx, idx - 1);
                }
            }
        }
    }

    pub fn set_parent(&mut self, widget_id: u32, parent_id: Option<u32>) {
        if let Some(pid) = parent_id {
            if pid == widget_id { return; }
            let mut curr = parent_id;
            while let Some(cpid) = curr {
                if cpid == widget_id { return; }
                curr = self.widgets.iter().find(|w| w.id == cpid).and_then(|w| w.parent_id);
            }
        }

        if let Some(w) = self.widgets.iter_mut().find(|w| w.id == widget_id) {
            w.parent_id = parent_id;
        }
    }

    /// Hit test against scene widgets using z-order (reverse list order).
    /// Returns `Some(id)` of topmost widget under point, or `None`.
    pub fn hit_test(&self, x: f32, y: f32) -> Option<u32> {
        self.widgets
            .iter()
            .rev()
            .find(|w| {
                if !self.is_effectively_visible(w.id) {
                    return false;
                }
                let half_w = w.w * 0.5;
                let half_h = w.h * 0.5;
                x >= w.x - half_w && x <= w.x + half_w && y >= w.y - half_h && y <= w.y + half_h
            })
            .map(|w| w.id)
    }

    pub fn copy_selected(&mut self) {
        if let Some(sel) = self.selected() {
            self.copied_widget = Some(sel.clone());
        }
    }

    pub fn paste_start(&mut self) {
        if let Some(copied) = &self.copied_widget {
            self.placement_drag = Some(copied.clone());
        }
    }

    pub fn finalize_placement(&mut self, x: f32, y: f32) -> u32 {
        if let Some(mut w) = self.placement_drag.take() {
            w.id = self.next_id;
            self.next_id += 1;
            w.x = x;
            w.y = y;
            let id = w.id;
            self.widgets.push(w);
            self.selected_id = Some(id);
            id
        } else {
            0
        }
    }
}
