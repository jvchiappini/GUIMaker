//! Event system for generated code
//!
//! This module provides event handling system for generated code.

use crate::plugin::{EventHandlerDefinition, PluginError};
use crate::scene::{PlacedWidget, WidgetKind};

/// Event handler for generated code
#[derive(Clone, Debug)]
pub struct GeneratedEventHandler {
    pub widget_id: u32,
    pub widget_kind: WidgetKind,
    pub event_type: String,
    pub handler_code: String,
}

/// Event system for code generation
pub struct EventSystem {
    handlers: Vec<GeneratedEventHandler>,
}

impl EventSystem {
    pub fn new() -> Self {
        Self { handlers: Vec::new() }
    }
    
    pub fn add_handler(&mut self, handler: GeneratedEventHandler) {
        self.handlers.push(handler);
    }
    
    pub fn get_handlers_for_widget(&self, widget_id: u32) -> Vec<&GeneratedEventHandler> {
        self.handlers.iter().filter(|h| h.widget_id == widget_id).collect()
    }
    
    pub fn generate_event_code(&self, scene: &crate::scene::SceneState) -> String {
        let mut code = String::new();
        
        code.push_str("// ─────────────────────────────────────────────────────────────────────────────\n");
        code.push_str("// Event Handlers\n");
        code.push_str("// ─────────────────────────────────────────────────────────────────────────────\n\n");
        
        for widget in &scene.widgets {
            let handlers = self.get_handlers_for_widget(widget.id);
            if !handlers.is_empty() {
                code.push_str(&format!("// Widget: {} (ID: {})\n", widget.kind.display_name(), widget.id));
                
                for handler in handlers {
                    code.push_str(&format!("// Event: {}\n", handler.event_type));
                    code.push_str(&format!("{}\n\n", handler.handler_code));
                }
            }
        }
        
        code
    }
}

/// Event type definitions
#[derive(Clone, Debug, PartialEq)]
pub enum EventType {
    /// Button click
    Click,
    /// Value changed (slider, input, etc.)
    ValueChanged,
    /// Text changed (input)
    TextChanged,
    /// Check state changed (checkbox, toggle)
    CheckChanged,
    /// Selection changed (dropdown, list, etc.)
    SelectionChanged,
    /// Focus gained
    FocusGained,
    /// Focus lost
    FocusLost,
    /// Mouse enter
    MouseEnter,
    /// Mouse leave
    MouseLeave,
    /// Drag start
    DragStart,
    /// Drag end
    DragEnd,
    /// Custom event
    Custom(String),
}

impl EventType {
    pub fn as_str(&self) -> &str {
        match self {
            EventType::Click => "click",
            EventType::ValueChanged => "value_changed",
            EventType::TextChanged => "text_changed",
            EventType::CheckChanged => "check_changed",
            EventType::SelectionChanged => "selection_changed",
            EventType::FocusGained => "focus_gained",
            EventType::FocusLost => "focus_lost",
            EventType::MouseEnter => "mouse_enter",
            EventType::MouseLeave => "mouse_leave",
            EventType::DragStart => "drag_start",
            EventType::DragEnd => "drag_end",
            EventType::Custom(name) => name,
        }
    }
    
    pub fn for_widget(kind: &WidgetKind) -> Vec<EventType> {
        match kind {
            WidgetKind::Button => vec![EventType::Click],
            WidgetKind::Slider => vec![EventType::ValueChanged],
            WidgetKind::Checkbox => vec![EventType::CheckChanged],
            WidgetKind::TextInput => vec![EventType::TextChanged, EventType::FocusGained, EventType::FocusLost],
            WidgetKind::DropDown => vec![EventType::SelectionChanged],
            WidgetKind::ToggleSwitch => vec![EventType::CheckChanged],
            WidgetKind::ColorPicker => vec![EventType::ValueChanged],
            WidgetKind::VirtualList => vec![EventType::SelectionChanged],
            WidgetKind::DataTable => vec![EventType::SelectionChanged],
            WidgetKind::TreeView => vec![EventType::SelectionChanged],
            WidgetKind::Tabs => vec![EventType::SelectionChanged],
            WidgetKind::ProgressBar => vec![EventType::ValueChanged],
            WidgetKind::Modal => vec![EventType::Click],
            WidgetKind::Tooltip => vec![EventType::MouseEnter, EventType::MouseLeave],
            WidgetKind::ScrollView => vec![],
            WidgetKind::Panel => vec![],
            WidgetKind::Separator => vec![],
            WidgetKind::Label => vec![],
            WidgetKind::Image => vec![],
            _ => vec![],
        }
    }
}

/// Generate event handler code for a widget
pub fn generate_widget_event_handler(
    widget: &PlacedWidget,
    event_type: &EventType,
    handler_body: &str,
) -> String {
    let var_name = format!("{}_{}", widget.kind.display_name().to_lowercase(), widget.id);
    
    match event_type {
        EventType::Click => {
            format!(
                "if {}.pressed {{\n    {}\n    {}.pressed = false;\n}}\n",
                var_name, handler_body, var_name
            )
        }
        EventType::ValueChanged => {
            format!(
                "if {}.changed {{\n    {}\n    {}.changed = false;\n}}\n",
                var_name, handler_body, var_name
            )
        }
        EventType::TextChanged => {
            format!(
                "if {}.text_changed {{\n    {}\n    {}.text_changed = false;\n}}\n",
                var_name, handler_body, var_name
            )
        }
        EventType::CheckChanged => {
            format!(
                "if {}.checked != {}_prev {{\n    {}\n    {}_prev = {}.checked;\n}}\n",
                var_name, var_name, handler_body, var_name, var_name
            )
        }
        EventType::SelectionChanged => {
            format!(
                "if {}.selection_changed {{\n    {}\n    {}.selection_changed = false;\n}}\n",
                var_name, handler_body, var_name
            )
        }
        _ => format!("// TODO: implement {} event handler for {}\n", event_type.as_str(), var_name),
    }
}