/// Tipos de widget que el usuario puede colocar en el canvas.
#[derive(Debug, Clone, PartialEq)]
pub enum WidgetKind {
    Button,
    Label,
    Slider,
    Checkbox,
    TextInput,
    Panel,
    Separator,
    Image,
    ProgressBar,
    Dropdown,
}

impl WidgetKind {
    /// Nombre legible para mostrar en la toolbox.
    pub fn display_name(&self) -> &'static str {
        match self {
            WidgetKind::Button      => "Button",
            WidgetKind::Label       => "Label",
            WidgetKind::Slider      => "Slider",
            WidgetKind::Checkbox    => "Checkbox",
            WidgetKind::TextInput   => "TextInput",
            WidgetKind::Panel       => "Panel",
            WidgetKind::Separator   => "Separator",
            WidgetKind::Image       => "Image",
            WidgetKind::ProgressBar => "ProgressBar",
            WidgetKind::Dropdown    => "Dropdown",
        }
    }

    /// Color de relleno para previsualizar el widget en el canvas.
    pub fn preview_color(&self) -> [f32; 4] {
        match self {
            WidgetKind::Button      => [0.25, 0.52, 0.96, 1.0],
            WidgetKind::Label       => [0.60, 0.60, 0.60, 1.0],
            WidgetKind::Slider      => [0.30, 0.78, 0.54, 1.0],
            WidgetKind::Checkbox    => [0.96, 0.78, 0.25, 1.0],
            WidgetKind::TextInput   => [0.80, 0.80, 0.80, 1.0],
            WidgetKind::Panel       => [0.20, 0.20, 0.25, 0.85],
            WidgetKind::Separator   => [0.45, 0.45, 0.50, 1.0],
            WidgetKind::Image       => [0.40, 0.65, 0.90, 1.0],
            WidgetKind::ProgressBar => [0.30, 0.70, 0.30, 1.0],
            WidgetKind::Dropdown    => [0.80, 0.50, 0.20, 1.0],
        }
    }

    /// Dimensiones predeterminadas (ancho, alto) en píxeles lógicos.
    pub fn default_size(&self) -> (f32, f32) {
        match self {
            WidgetKind::Button      => (160.0, 40.0),
            WidgetKind::Label       => (180.0, 24.0),
            WidgetKind::Slider      => (200.0, 24.0),
            WidgetKind::Checkbox    => (140.0, 28.0),
            WidgetKind::TextInput   => (200.0, 32.0),
            WidgetKind::Panel       => (240.0, 160.0),
            WidgetKind::Separator   => (200.0, 6.0),
            WidgetKind::Image       => (120.0, 120.0),
            WidgetKind::ProgressBar => (200.0, 20.0),
            WidgetKind::Dropdown    => (180.0, 32.0),
        }
    }

    pub fn all() -> Vec<WidgetKind> {
        vec![
            WidgetKind::Button,
            WidgetKind::Label,
            WidgetKind::Slider,
            WidgetKind::Checkbox,
            WidgetKind::TextInput,
            WidgetKind::Panel,
            WidgetKind::Separator,
            WidgetKind::Image,
            WidgetKind::ProgressBar,
            WidgetKind::Dropdown,
        ]
    }
}

/// Un widget concreto colocado en el canvas por el usuario.
#[derive(Debug, Clone)]
pub struct WidgetNode {
    pub id:     usize,
    pub kind:   WidgetKind,
    /// Posición relativa al origen del canvas (píxeles lógicos).
    pub x:      f32,
    pub y:      f32,
    pub width:  f32,
    pub height: f32,
    /// Texto/etiqueta visible en el widget (si aplica).
    pub label:  String,
    /// Nombre de variable que se usará en el código generado.
    pub var_name: String,
    /// Valor numérico inicial (p.ej. valor del slider, progreso).
    pub value:  f32,
    /// Esquinas redondeadas en píxeles.
    pub radius: f32,
}

impl WidgetNode {
    pub fn new(id: usize, kind: WidgetKind, x: f32, y: f32) -> Self {
        let (w, h) = kind.default_size();
        let label   = format!("{} {}", kind.display_name(), id);
        let var_name = format!("{}_{}", kind.display_name().to_lowercase(), id);
        Self {
            id,
            kind,
            x, y,
            width: w, height: h,
            label,
            var_name,
            value: 0.5,
            radius: 4.0,
        }
    }
}

/// Estado global del canvas/editor.
pub struct CanvasState {
    pub widgets:      Vec<WidgetNode>,
    pub next_id:      usize,
    pub selected_id:  Option<usize>,
    /// Desplazamiento del canvas (scroll/pan).
    pub offset_x:     f32,
    pub offset_y:     f32,
    /// Nombre del proyecto (se usa en el código generado).
    pub project_name: String,
    /// Código Rust generado (se actualiza al pulsar "Generar código").
    pub generated_code: String,
    /// Mostrar el panel de código generado.
    pub show_code_panel: bool,
    /// Último widget siendo arrastrado.
    pub drag_widget_id:  Option<usize>,
    /// Offset dentro del widget donde se inició el arrastre.
    pub drag_offset:     (f32, f32),
}

impl Default for CanvasState {
    fn default() -> Self {
        Self {
            widgets:        Vec::new(),
            next_id:        1,
            selected_id:    None,
            offset_x:       0.0,
            offset_y:       0.0,
            project_name:   "my_gui_app".to_string(),
            generated_code: String::new(),
            show_code_panel: false,
            drag_widget_id:  None,
            drag_offset:     (0.0, 0.0),
        }
    }
}

impl CanvasState {
    /// Añade un widget al canvas y devuelve su id.
    pub fn add_widget(&mut self, kind: WidgetKind, canvas_x: f32, canvas_y: f32) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        self.widgets.push(WidgetNode::new(id, kind, canvas_x, canvas_y));
        id
    }

    /// Elimina el widget seleccionado.
    pub fn delete_selected(&mut self) {
        if let Some(sel) = self.selected_id {
            self.widgets.retain(|w| w.id != sel);
            self.selected_id = None;
        }
    }

    /// Devuelve una referencia al widget con el id dado.
    pub fn get(&self, id: usize) -> Option<&WidgetNode> {
        self.widgets.iter().find(|w| w.id == id)
    }

    /// Devuelve una referencia mutable al widget con el id dado.
    pub fn get_mut(&mut self, id: usize) -> Option<&mut WidgetNode> {
        self.widgets.iter_mut().find(|w| w.id == id)
    }

    /// Hit-test: devuelve el id del widget situado en (px, py) en coordenadas canvas.
    /// Iterar en orden inverso para dar prioridad al widget de más arriba.
    pub fn hit_test(&self, px: f32, py: f32) -> Option<usize> {
        for w in self.widgets.iter().rev() {
            if px >= w.x && px <= w.x + w.width
            && py >= w.y && py <= w.y + w.height
            {
                return Some(w.id);
            }
        }
        None
    }
}
