//! Plugin system for GUIMaker
//!
//! This module provides a comprehensive plugin architecture that allows
//! extending GUIMaker with custom widgets, code generators, UI extensions,
//! themes, and tools.
//!
//! # Architecture
//!
//! ```text
//! Plugin System
//! ├── WidgetRegistry      - Register custom widget types
//! ├── CodegenRegistry     - Register code generation strategies
//! ├── UIToolRegistry      - Register UI tools and panels
//! ├── ThemeRegistry       - Register visual themes
//! ├── EventRegistry       - Register event handlers
//! └── PluginManager       - Lifecycle management
//! ```

use std::any::Any;
use std::sync::Arc;

// ─────────────────────────────────────────────────────────────────────────────
// Plugin Traits
// ─────────────────────────────────────────────────────────────────────────────

/// Base trait for all plugins
pub trait Plugin: Any + Send + Sync {
    /// Unique identifier for the plugin
    fn id(&self) -> &'static str;
    
    /// Human-readable name
    fn name(&self) -> &'static str;
    
    /// Version string (e.g., "1.0.0")
    fn version(&self) -> &'static str;
    
    /// Plugin description
    fn description(&self) -> &'static str;
    
    /// Author information
    fn author(&self) -> &'static str;
    
    /// Initialize the plugin
    fn initialize(&self, context: &mut PluginContext) -> Result<(), PluginError>;
    
    /// Shutdown the plugin
    fn shutdown(&self, context: &mut PluginContext) -> Result<(), PluginError>;
    
    /// Get plugin as Any for downcasting
    fn as_any(&self) -> &dyn Any;
}

/// Context passed to plugins during initialization
pub struct PluginContext {
    pub widget_registry: WidgetRegistryRef,
    pub codegen_registry: CodegenRegistryRef,
    pub theme_registry: ThemeRegistryRef,
    pub tool_registry: ToolRegistryRef,
    pub event_registry: EventRegistryRef,
}

/// Plugin error type
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("Plugin initialization failed: {0}")]
    InitError(String),
    
    #[error("Plugin runtime error: {0}")]
    RuntimeError(String),
    
    #[error("Plugin already registered: {0}")]
    AlreadyRegistered(String),
    
    #[error("Plugin not found: {0}")]
    NotFound(String),
}

// ─────────────────────────────────────────────────────────────────────────────
// Widget Plugin System
// ─────────────────────────────────────────────────────────────────────────────

use crate::scene::{PlacedWidget, WidgetKind, WidgetProps};

/// Definition of a custom widget type
#[derive(Clone, Debug)]
pub struct WidgetDefinition {
    pub kind: WidgetKind,
    pub display_name: String,
    pub category: String,
    pub default_size: (f32, f32),
    pub properties: Vec<PropertyDefinition>,
    pub preview_color: [f32; 4],
}

/// Property definition for a widget
#[derive(Clone, Debug)]
pub struct PropertyDefinition {
    pub name: String,
    pub property_type: PropertyType,
    pub default_value: PropertyValue,
    pub min: Option<f32>,
    pub max: Option<f32>,
    pub step: Option<f32>,
    pub description: Option<String>,
}

/// Property type enum
#[derive(Clone, Debug, PartialEq)]
pub enum PropertyType {
    String,
    Number,
    Boolean,
    Color,
    Enum(Vec<String>),
    Range { min: f32, max: f32 },
    Array { element_type: Box<PropertyType> },
}

/// Property value enum
#[derive(Clone, Debug, PartialEq)]
pub enum PropertyValue {
    String(String),
    Number(f32),
    Boolean(bool),
    Color([f32; 4]),
    Enum(usize),
}

/// Widget registry for managing widget definitions
pub struct WidgetRegistry {
    widgets: std::collections::HashMap<String, WidgetDefinition>,
    custom_generators: std::collections::HashMap<String, Box<dyn WidgetGenerator>>,
}

pub type WidgetRegistryRef = Arc<std::sync::RwLock<WidgetRegistry>>;

/// Trait for custom widget code generation
pub trait WidgetGenerator: Send + Sync {
    fn generate_code(&self, widget: &PlacedWidget) -> String;
    fn generate_default(&self, widget: &PlacedWidget) -> String;
    fn generate_draw(&self, widget: &PlacedWidget) -> String;
}

impl WidgetRegistry {
    pub fn new() -> Self {
        Self {
            widgets: std::collections::HashMap::new(),
            custom_generators: std::collections::HashMap::new(),
        }
    }
    
    pub fn register(&mut self, definition: WidgetDefinition) -> Result<(), PluginError> {
        let key = format!("{:?}", definition.kind);
        if self.widgets.contains_key(&key) {
            return Err(PluginError::AlreadyRegistered(key));
        }
        self.widgets.insert(key, definition);
        Ok(())
    }
    
    pub fn get(&self, kind: &WidgetKind) -> Option<&WidgetDefinition> {
        self.widgets.get(&format!("{:?}", kind))
    }
    
    pub fn all(&self) -> impl Iterator<Item = &WidgetDefinition> {
        self.widgets.values()
    }
    
    
    pub fn register_generator(
        &mut self,
        kind: WidgetKind,
        generator: Box<dyn WidgetGenerator>,
    ) -> Result<(), PluginError> {
        let key = format!("{:?}", kind);
        self.custom_generators.insert(key, generator);
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Codegen Plugin System
// ─────────────────────────────────────────────────────────────────────────────

use crate::scene::SceneState;

/// Code generation configuration
#[derive(Clone, Debug)]

pub struct CodegenConfig {
    pub output_format: OutputFormat,
    pub include_comments: bool,
    pub include_examples: bool,
    pub state_management: StateManagementStrategy,
    pub layout_strategy: LayoutStrategy,
    pub event_handling: EventHandlingStrategy,
}

/// Output format for code generation
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum OutputFormat {
    FerrousApp,
    AppBuilder,
    Plugins,
    MultiFile,
}

/// State management strategy
#[derive(Clone, Debug, PartialEq)]
pub enum StateManagementStrategy {
    StructFields,
    GlobalState,
    ReduxPattern,
    RefCell,
}

/// Layout strategy
#[derive(Clone, Debug, PartialEq)]
pub enum LayoutStrategy {
    Absolute,
    Percentages,
    Flexbox,
    Grid,
}

/// Event handling strategy
#[derive(Clone, Debug, PartialEq)]
pub enum EventHandlingStrategy {
    Callbacks,
    MessagePassing,
    EventLoop,
}

/// Codegen registry for managing code generation strategies
pub struct CodegenRegistry {
    generators: std::collections::HashMap<String, Box<dyn CodegenGenerator>>,
    default_format: OutputFormat,
}

pub type CodegenRegistryRef = Arc<std::sync::RwLock<CodegenRegistry>>;

/// Trait for custom code generation
pub trait CodegenGenerator: Send + Sync {
    fn generate(&self, scene: &SceneState, config: &CodegenConfig) -> Result<String, PluginError>;
    fn supported_formats(&self) -> Vec<OutputFormat>;
}

impl CodegenRegistry {
    pub fn new() -> Self {
        Self {
            generators: std::collections::HashMap::new(),
            default_format: OutputFormat::FerrousApp,
        }
    }
    
    pub fn register(
        &mut self,
        format: OutputFormat,
        generator: Box<dyn CodegenGenerator>,
    ) -> Result<(), PluginError> {
        let key = format!("{:?}", format);
        if self.generators.contains_key(&key) {
            return Err(PluginError::AlreadyRegistered(key));
        }
        self.generators.insert(key, generator);
        Ok(())
    }
    
    pub fn get(&self, format: OutputFormat) -> Option<&Box<dyn CodegenGenerator>> {
        let key = format!("{:?}", format);
        self.generators.get(&key)
    }
    
    pub fn generate(
        &self,
        scene: &SceneState,
        config: &CodegenConfig,
    ) -> Result<String, PluginError> {
        let generator = self.get(config.output_format.clone())
            .ok_or_else(|| PluginError::NotFound(format!("{:?}", config.output_format)))?;
        generator.generate(scene, config)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Theme Plugin System
// ─────────────────────────────────────────────────────────────────────────────

/// Visual theme definition
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Theme {
    pub name: String,
    pub colors: ThemeColors,
    pub typography: ThemeTypography,
    pub spacing: ThemeSpacing,
    pub borders: ThemeBorders,
}

/// Theme color palette
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ThemeColors {
    pub primary: String,
    pub primary_variant: String,
    pub secondary: String,
    pub secondary_variant: String,
    pub background: String,
    pub surface: String,
    pub error: String,
    pub on_primary: String,
    pub on_secondary: String,
    pub on_background: String,
    pub on_surface: String,
    pub on_error: String,
}

/// Theme typography settings
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ThemeTypography {
    pub font_family: String,
    pub font_size_base: f32,
    pub font_size_small: f32,
    pub font_size_medium: f32,
    pub font_size_large: f32,
    pub font_size_xlarge: f32,
}

/// Theme spacing settings
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ThemeSpacing {
    pub unit: f32,
    pub tight: f32,
    pub regular: f32,
    pub wide: f32,
}

/// Theme border settings
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ThemeBorders {
    pub radius_small: f32,
    pub radius_medium: f32,
    pub radius_large: f32,
    pub width_thin: f32,
    pub width_regular: f32,
}

/// Theme registry for managing themes
pub struct ThemeRegistry {
    themes: std::collections::HashMap<String, Theme>,
    current: String,
}

pub type ThemeRegistryRef = Arc<std::sync::RwLock<ThemeRegistry>>;

impl ThemeRegistry {
    pub fn new() -> Self {
        let mut themes = std::collections::HashMap::new();
        themes.insert("dark".to_string(), Theme::dark());
        themes.insert("light".to_string(), Theme::light());
        
        Self {
            themes,
            current: "dark".to_string(),
        }
    }
    
    pub fn register(&mut self, theme: Theme) -> Result<(), PluginError> {
        if self.themes.contains_key(&theme.name) {
            return Err(PluginError::AlreadyRegistered(theme.name));
        }
        self.themes.insert(theme.name.clone(), theme);
        Ok(())
    }
    
    pub fn get(&self, name: &str) -> Option<&Theme> {
        self.themes.get(name)
    }
    
    pub fn current(&self) -> &Theme {
        self.themes.get(&self.current).unwrap()
    }
    
    pub fn set_current(&mut self, name: String) -> Result<(), PluginError> {
        if !self.themes.contains_key(&name) {
            return Err(PluginError::NotFound(name));
        }
        self.current = name;
        Ok(())
    }
    
    pub fn all(&self) -> impl Iterator<Item = &Theme> {
        self.themes.values()
    }
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            name: "dark".to_string(),
            colors: ThemeColors {
                primary: "#6C63FF".to_string(),
                primary_variant: "#5C53EF".to_string(),
                secondary: "#FF63A5".to_string(),
                secondary_variant: "#EF5395".to_string(),
                background: "#1E1E1E".to_string(),
                surface: "#2D2D2D".to_string(),
                error: "#FF5252".to_string(),
                on_primary: "#FFFFFF".to_string(),
                on_secondary: "#FFFFFF".to_string(),
                on_background: "#FFFFFF".to_string(),
                on_surface: "#FFFFFF".to_string(),
                on_error: "#FFFFFF".to_string(),
            },
            typography: ThemeTypography {
                font_family: "Roboto".to_string(),
                font_size_base: 14.0,
                font_size_small: 12.0,
                font_size_medium: 14.0,
                font_size_large: 16.0,
                font_size_xlarge: 20.0,
            },
            spacing: ThemeSpacing {
                unit: 4.0,
                tight: 4.0,
                regular: 8.0,
                wide: 16.0,
            },
            borders: ThemeBorders {
                radius_small: 2.0,
                radius_medium: 4.0,
                radius_large: 8.0,
                width_thin: 1.0,
                width_regular: 2.0,
            },
        }
    }
    
    pub fn light() -> Self {
        Self {
            name: "light".to_string(),
            colors: ThemeColors {
                primary: "#6200EE".to_string(),
                primary_variant: "#3700B3".to_string(),
                secondary: "#03DAC6".to_string(),
                secondary_variant: "#018786".to_string(),
                background: "#FFFFFF".to_string(),
                surface: "#F5F5F5".to_string(),
                error: "#B00020".to_string(),
                on_primary: "#FFFFFF".to_string(),
                on_secondary: "#000000".to_string(),
                on_background: "#000000".to_string(),
                on_surface: "#000000".to_string(),
                on_error: "#FFFFFF".to_string(),
            },
            typography: ThemeTypography {
                font_family: "Roboto".to_string(),
                font_size_base: 14.0,
                font_size_small: 12.0,
                font_size_medium: 14.0,
                font_size_large: 16.0,
                font_size_xlarge: 20.0,
            },
            spacing: ThemeSpacing {
                unit: 4.0,
                tight: 4.0,
                regular: 8.0,
                wide: 16.0,
            },
            borders: ThemeBorders {
                radius_small: 2.0,
                radius_medium: 4.0,
                radius_large: 8.0,
                width_thin: 1.0,
                width_regular: 2.0,
            },
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tool Plugin System
// ─────────────────────────────────────────────────────────────────────────────

/// Tool definition for UI extensions
#[derive(Clone, Debug)]
pub struct ToolDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub icon: Option<String>,
    pub shortcut: Option<String>,
}

/// Tool registry for managing UI tools
pub struct ToolRegistry {
    tools: std::collections::HashMap<String, ToolDefinition>,
}

pub type ToolRegistryRef = Arc<std::sync::RwLock<ToolRegistry>>;

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: std::collections::HashMap::new(),
        }
    }
    
    pub fn register(&mut self, tool: ToolDefinition) -> Result<(), PluginError> {
        if self.tools.contains_key(&tool.id) {
            return Err(PluginError::AlreadyRegistered(tool.id));
        }
        self.tools.insert(tool.id.clone(), tool);
        Ok(())
    }
    
    pub fn get(&self, id: &str) -> Option<&ToolDefinition> {
        self.tools.get(id)
    }
    
    pub fn all(&self) -> impl Iterator<Item = &ToolDefinition> {
        self.tools.values()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Event Plugin System
// ─────────────────────────────────────────────────────────────────────────────

/// Event handler definition
#[derive(Clone, Debug)]
pub struct EventHandlerDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub event_type: String,
    pub handler_fn: String,
    pub priority: i32,
}

/// Event registry for managing event handlers
pub struct EventRegistry {
    handlers: std::collections::HashMap<String, EventHandlerDefinition>,
}

pub type EventRegistryRef = Arc<std::sync::RwLock<EventRegistry>>;

impl EventRegistry {
    pub fn new() -> Self {
        Self {
            handlers: std::collections::HashMap::new(),
        }
    }
    
    pub fn register(&mut self, handler: EventHandlerDefinition) -> Result<(), PluginError> {
        if self.handlers.contains_key(&handler.id) {
            return Err(PluginError::AlreadyRegistered(handler.id));
        }
        self.handlers.insert(handler.id.clone(), handler);
        Ok(())
    }
    
    pub fn get(&self, id: &str) -> Option<&EventHandlerDefinition> {
        self.handlers.get(id)
    }
    
    pub fn all(&self) -> impl Iterator<Item = &EventHandlerDefinition> {
        self.handlers.values()
    }
    
    pub fn handlers_for_event(&self, event_type: &str) -> Vec<&EventHandlerDefinition> {
        self.handlers.values()
            .filter(|h| h.event_type == event_type)
            .collect()
    }
}
// ─────────────────────────────────────────────────────────────────────────────
// Plugin Manager
// ─────────────────────────────────────────────────────────────────────────────

pub struct PluginManager {
    plugins: std::collections::HashMap<String, Arc<dyn Plugin>>,
    widget_registry: WidgetRegistryRef,
    codegen_registry: CodegenRegistryRef,
    theme_registry: ThemeRegistryRef,
    tool_registry: ToolRegistryRef,
    event_registry: EventRegistryRef,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: std::collections::HashMap::new(),
            widget_registry: Arc::new(std::sync::RwLock::new(WidgetRegistry::new())),
            codegen_registry: Arc::new(std::sync::RwLock::new(CodegenRegistry::new())),
            theme_registry: Arc::new(std::sync::RwLock::new(ThemeRegistry::new())),
            tool_registry: Arc::new(std::sync::RwLock::new(ToolRegistry::new())),
            event_registry: Arc::new(std::sync::RwLock::new(EventRegistry::new())),
        }
    }
    
    pub fn register_plugin(&mut self, plugin: Arc<dyn Plugin>) -> Result<(), PluginError> {
        let id = plugin.id().to_string();
        if self.plugins.contains_key(&id) {
            return Err(PluginError::AlreadyRegistered(id));
        }
        
        let mut context = PluginContext {
            widget_registry: self.widget_registry.clone(),
            codegen_registry: self.codegen_registry.clone(),
            theme_registry: self.theme_registry.clone(),
            tool_registry: self.tool_registry.clone(),
            event_registry: self.event_registry.clone(),
        };
        
        plugin.initialize(&mut context)?;
        self.plugins.insert(id, plugin);
        Ok(())
    }
    
    pub fn unregister_plugin(&mut self, id: &str) -> Result<(), PluginError> {
        if let Some(plugin) = self.plugins.remove(id) {
            let mut context = PluginContext {
                widget_registry: self.widget_registry.clone(),
                codegen_registry: self.codegen_registry.clone(),
                theme_registry: self.theme_registry.clone(),
                tool_registry: self.tool_registry.clone(),
                event_registry: self.event_registry.clone(),
            };
            plugin.shutdown(&mut context)?;
            Ok(())
        } else {
            Err(PluginError::NotFound(id.to_string()))
        }
    }
    
    pub fn get_plugin(&self, id: &str) -> Option<&Arc<dyn Plugin>> {
        self.plugins.get(id)
    }
    
    pub fn list_plugins(&self) -> Vec<&Arc<dyn Plugin>> {
        self.plugins.values().collect()
    }
    
    pub fn widget_registry(&self) -> WidgetRegistryRef {
        self.widget_registry.clone()
    }
    
    pub fn codegen_registry(&self) -> CodegenRegistryRef {
        self.codegen_registry.clone()
    }
    
    pub fn theme_registry(&self) -> ThemeRegistryRef {
        self.theme_registry.clone()
    }
    
    pub fn tool_registry(&self) -> ToolRegistryRef {
        self.tool_registry.clone()
    }
    
    pub fn event_registry(&self) -> EventRegistryRef {
        self.event_registry.clone()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Example Built-in Plugins
// ─────────────────────────────────────────────────────────────────────────────

/// Built-in widgets plugin
pub struct BuiltInWidgetsPlugin;

impl Plugin for BuiltInWidgetsPlugin {
    fn id(&self) -> &'static str { "built-in-widgets" }
    fn name(&self) -> &'static str { "Built-in Widgets" }
    fn version(&self) -> &'static str { "1.0.0" }
    fn description(&self) -> &'static str { "Provides standard GUI widgets" }
    fn author(&self) -> &'static str { "GUIMaker Team" }
    
    fn initialize(&self, context: &mut PluginContext) -> Result<(), PluginError> {
        // Register standard widgets (this would be expanded in actual implementation)
        Ok(())
    }
    
    fn shutdown(&self, _context: &mut PluginContext) -> Result<(), PluginError> {
        Ok(())
    }
    
    fn as_any(&self) -> &dyn Any { self }
}
