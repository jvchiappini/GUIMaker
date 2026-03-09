// ── Scene state ────────────────────────────────────────────────────────────────
//
// Holds all placed widgets and the selection / drag state for the scene builder.

/// Every widget kind available in the palette.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WidgetKind {
    // Basic
    Label,
    Button,
    Panel,
    Separator,
    Spacer,
    Placeholder,
    // Input
    TextInput,
    Checkbox,
    Slider,
    NumberInput,
    ToggleSwitch,
    DropDown,
    ColorPicker,
    // Layout
    ScrollView,
    SplitPane,
    DockLayout,
    AspectRatio,
    // Display
    Image,
    Svg,
    ProgressBar,
    // Navigation / Containers
    Tabs,
    Accordion,
    TreeView,
    Modal,
    Tooltip,
    // Data
    DataTable,
    VirtualList,
    VirtualGrid,
    // Feedback
    Toast,
}

impl WidgetKind {
    pub fn name(self) -> &'static str {
        match self {
            WidgetKind::Label => "Label",
            WidgetKind::Button => "Button",
            WidgetKind::Panel => "Panel",
            WidgetKind::Separator => "Separator",
            WidgetKind::Spacer => "Spacer",
            WidgetKind::Placeholder => "Placeholder",
            WidgetKind::TextInput => "Text Input",
            WidgetKind::Checkbox => "Checkbox",
            WidgetKind::Slider => "Slider",
            WidgetKind::NumberInput => "Number Input",
            WidgetKind::ToggleSwitch => "Toggle Switch",
            WidgetKind::DropDown => "Drop Down",
            WidgetKind::ColorPicker => "Color Picker",
            WidgetKind::ScrollView => "Scroll View",
            WidgetKind::SplitPane => "Split Pane",
            WidgetKind::DockLayout => "Dock Layout",
            WidgetKind::AspectRatio => "Aspect Ratio",
            WidgetKind::Image => "Image",
            WidgetKind::Svg => "Svg",
            WidgetKind::ProgressBar => "Progress Bar",
            WidgetKind::Tabs => "Tabs",
            WidgetKind::Accordion => "Accordion",
            WidgetKind::TreeView => "Tree View",
            WidgetKind::Modal => "Modal",
            WidgetKind::Tooltip => "Tooltip",
            WidgetKind::DataTable => "Data Table",
            WidgetKind::VirtualList => "Virtual List",
            WidgetKind::VirtualGrid => "Virtual Grid",
            WidgetKind::Toast => "Toast",
        }
    }

    /// Accent / tint color per category, as linear RGBA.
    pub fn color(self) -> [f32; 4] {
        match self {
            // Basic — blue
            WidgetKind::Label
            | WidgetKind::Button
            | WidgetKind::Panel
            | WidgetKind::Separator
            | WidgetKind::Spacer
            | WidgetKind::Placeholder => [0.18, 0.46, 0.71, 0.85],
            // Input — green
            WidgetKind::TextInput
            | WidgetKind::Checkbox
            | WidgetKind::Slider
            | WidgetKind::NumberInput
            | WidgetKind::ToggleSwitch
            | WidgetKind::DropDown
            | WidgetKind::ColorPicker => [0.20, 0.63, 0.35, 0.85],
            // Layout — purple
            WidgetKind::ScrollView
            | WidgetKind::SplitPane
            | WidgetKind::DockLayout
            | WidgetKind::AspectRatio => [0.55, 0.30, 0.80, 0.85],
            // Display — orange
            WidgetKind::Image | WidgetKind::Svg | WidgetKind::ProgressBar => {
                [0.85, 0.50, 0.10, 0.85]
            }
            // Navigation — teal
            WidgetKind::Tabs
            | WidgetKind::Accordion
            | WidgetKind::TreeView
            | WidgetKind::Modal
            | WidgetKind::Tooltip => [0.10, 0.65, 0.65, 0.85],
            // Data — pink
            WidgetKind::DataTable | WidgetKind::VirtualList | WidgetKind::VirtualGrid => {
                [0.75, 0.20, 0.50, 0.85]
            }
            // Feedback — yellow
            WidgetKind::Toast => [0.80, 0.70, 0.10, 0.85],
        }
    }

    /// Default size when dropped onto the canvas (world units).
    pub fn default_size(self) -> (f32, f32) {
        match self {
            WidgetKind::Separator => (200.0, 4.0),
            WidgetKind::Spacer => (80.0, 20.0),
            WidgetKind::Placeholder => (120.0, 60.0),
            WidgetKind::Panel => (200.0, 150.0),
            WidgetKind::ScrollView => (220.0, 180.0),
            WidgetKind::SplitPane => (300.0, 200.0),
            WidgetKind::DockLayout => (320.0, 220.0),
            WidgetKind::AspectRatio => (160.0, 90.0),
            WidgetKind::DataTable => (360.0, 200.0),
            WidgetKind::VirtualList => (200.0, 200.0),
            WidgetKind::VirtualGrid => (280.0, 200.0),
            WidgetKind::Tabs => (280.0, 160.0),
            WidgetKind::Accordion => (240.0, 120.0),
            WidgetKind::TreeView => (200.0, 180.0),
            WidgetKind::Modal => (320.0, 200.0),
            WidgetKind::Image => (120.0, 90.0),
            WidgetKind::Svg => (64.0, 64.0),
            WidgetKind::ProgressBar => (200.0, 20.0),
            WidgetKind::ColorPicker => (180.0, 220.0),
            WidgetKind::Slider => (160.0, 28.0),
            WidgetKind::ToggleSwitch => (80.0, 28.0),
            WidgetKind::DropDown => (160.0, 32.0),
            WidgetKind::Checkbox => (140.0, 28.0),
            WidgetKind::NumberInput => (120.0, 32.0),
            WidgetKind::TextInput => (160.0, 32.0),
            WidgetKind::Toast => (260.0, 48.0),
            WidgetKind::Tooltip => (140.0, 36.0),
            WidgetKind::Button => (100.0, 34.0),
            WidgetKind::Label => (100.0, 22.0),
        }
    }
}

/// Properties editable in the inspector panel.
#[derive(Clone, Debug)]
pub struct WidgetProps {
    pub label: String,
    pub color_hex: String,
    pub visible: bool,
    pub font_size: f32,
    // For sliders / number inputs
    pub min: f32,
    pub max: f32,
    pub value: f32,
    // For checkboxes / toggles
    pub checked: bool,
}

impl Default for WidgetProps {
    fn default() -> Self {
        Self {
            label: String::new(),
            color_hex: "#FFFFFF".to_string(),
            visible: true,
            font_size: 14.0,
            min: 0.0,
            max: 100.0,
            value: 50.0,
            checked: false,
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
        props.label = kind.name().to_string();
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

/// All palette categories with their widgets.
pub struct PaletteCategory {
    pub name: &'static str,
    pub widgets: &'static [WidgetKind],
}

pub const PALETTE: &[PaletteCategory] = &[
    PaletteCategory {
        name: "Basic",
        widgets: &[
            WidgetKind::Label,
            WidgetKind::Button,
            WidgetKind::Panel,
            WidgetKind::Separator,
            WidgetKind::Spacer,
            WidgetKind::Placeholder,
        ],
    },
    PaletteCategory {
        name: "Input",
        widgets: &[
            WidgetKind::TextInput,
            WidgetKind::Checkbox,
            WidgetKind::Slider,
            WidgetKind::NumberInput,
            WidgetKind::ToggleSwitch,
            WidgetKind::DropDown,
            WidgetKind::ColorPicker,
        ],
    },
    PaletteCategory {
        name: "Layout",
        widgets: &[
            WidgetKind::ScrollView,
            WidgetKind::SplitPane,
            WidgetKind::DockLayout,
            WidgetKind::AspectRatio,
        ],
    },
    PaletteCategory {
        name: "Display",
        widgets: &[WidgetKind::Image, WidgetKind::Svg, WidgetKind::ProgressBar],
    },
    PaletteCategory {
        name: "Navigation",
        widgets: &[
            WidgetKind::Tabs,
            WidgetKind::Accordion,
            WidgetKind::TreeView,
            WidgetKind::Modal,
            WidgetKind::Tooltip,
        ],
    },
    PaletteCategory {
        name: "Data",
        widgets: &[
            WidgetKind::DataTable,
            WidgetKind::VirtualList,
            WidgetKind::VirtualGrid,
        ],
    },
    PaletteCategory {
        name: "Feedback",
        widgets: &[WidgetKind::Toast],
    },
];

/// Top-level scene state: the list of placed widgets and selection/drag info.
pub struct SceneState {
    pub widgets: Vec<PlacedWidget>,
    next_id: u32,

    /// Widget being dragged on the canvas (id, offset from widget origin).
    pub drag_canvas: Option<(u32, f32, f32)>,

    /// Widget kind being dragged from the palette (ghost follows the cursor).
    pub palette_drag: Option<WidgetKind>,

    /// Currently selected widget id.
    pub selected_id: Option<u32>,

    /// Property field currently being edited (field_key) and its string buffer.
    pub editing_field: Option<String>,
    pub edit_buffer: String,

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
            palette_drag: None,
            selected_id: None,
            editing_field: None,
            edit_buffer: String::new(),
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
