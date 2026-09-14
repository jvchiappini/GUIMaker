//! Template system for GUIMaker
//!
//! This module provides a comprehensive template system for:
//! - Project templates (login, dashboard, settings, etc.)
//! - Widget templates (common UI patterns)
//! - Code templates (boilerplate, patterns)
//! - Layout templates (predefined layouts)
//!
//! # Architecture
//!
//! ```text
//! Template System
//! ├── ProjectTemplate    - Complete project structures
//! ├── WidgetTemplate     - Pre-configured widget groups
//! ├── CodeTemplate       - Code snippets and patterns
//! ├── LayoutTemplate     - Layout configurations
//! └── TemplateRegistry   - Template management
//! ```

use std::collections::HashMap;
use std::sync::Arc;

// ─────────────────────────────────────────────────────────────────────────────
// Template Types
// ─────────────────────────────────────────────────────────────────────────────

/// Template metadata
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TemplateMeta {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub author: String,
    pub version: String,
    pub tags: Vec<String>,
    pub preview_image: Option<String>,
}

/// Project template structure
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ProjectTemplate {
    pub meta: TemplateMeta,
    pub pages: Vec<PageTemplate>,
    pub assets: AssetTemplate,
    pub settings: ProjectSettingsTemplate,
}

/// Page template
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PageTemplate {
    pub name: String,
    pub widgets: Vec<WidgetInstanceTemplate>,
    pub layout: LayoutTemplate,
}

/// Widget instance template
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct WidgetInstanceTemplate {
    pub kind: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub properties: HashMap<String, serde_json::Value>,
}

/// Layout template
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct LayoutTemplate {
    pub kind: LayoutKind,
    pub config: HashMap<String, serde_json::Value>,
}

/// Layout kind enum
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum LayoutKind {
    Absolute,
    FlexHorizontal,
    FlexVertical,
    Grid {
        columns: usize,
        rows: usize,
        gap: f32,
    },
    Custom(String),
}

/// Asset template
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AssetTemplate {
    pub images: Vec<ImageAssetTemplate>,
    pub fonts: Vec<FontAssetTemplate>,
}

/// Image asset template
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ImageAssetTemplate {
    pub name: String,
    pub path: String,
    pub size: Option<(u32, u32)>,
}

/// Font asset template
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FontAssetTemplate {
    pub name: String,
    pub path: String,
}

/// Project settings template
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ProjectSettingsTemplate {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub background_color: String,
    pub font: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// Widget Templates
// ─────────────────────────────────────────────────────────────────────────────

/// Widget template (pre-configured widget)
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct WidgetTemplate {
    pub meta: TemplateMeta,
    pub widget_kind: String,
    pub default_properties: HashMap<String, serde_json::Value>,
    pub required_properties: Vec<String>,
    pub sample_usage: String,
}

/// Template group for common patterns
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct WidgetTemplateGroup {
    pub name: String,
    pub description: String,
    pub templates: Vec<WidgetTemplate>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Code Templates
// ─────────────────────────────────────────────────────────────────────────────

/// Code template for snippets and patterns
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CodeTemplate {
    pub meta: TemplateMeta,
    pub language: String,
    pub content: String,
    pub placeholders: Vec<CodePlaceholder>,
}

/// Code placeholder definition
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CodePlaceholder {
    pub name: String,
    pub description: String,
    pub default_value: String,
    pub required: bool,
}

/// Generated code result
pub struct GeneratedCode {
    pub main: String,
    pub ui: String,
    pub events: String,
    pub state: String,
}

/// Event template for event handlers
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EventTemplate {
    pub meta: TemplateMeta,
    pub event_type: String,
    pub handler_code: String,
    pub parameters: Vec<EventParameter>,
}

/// Event parameter definition
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EventParameter {
    pub name: String,
    pub param_type: String,
    pub description: Option<String>,
}
pub struct TemplateRegistry {
    project_templates: HashMap<String, ProjectTemplate>,
    widget_templates: HashMap<String, WidgetTemplate>,
    widget_template_groups: HashMap<String, WidgetTemplateGroup>,
    code_templates: HashMap<String, CodeTemplate>,
    event_templates: HashMap<String, EventTemplate>,
}

pub type TemplateRegistryRef = Arc<std::sync::RwLock<TemplateRegistry>>;

impl TemplateRegistry {
    pub fn new() -> Self {
        Self {
            project_templates: HashMap::new(),
            widget_templates: HashMap::new(),
            widget_template_groups: HashMap::new(),
            code_templates: HashMap::new(),
            event_templates: HashMap::new(),
        }
    }

    // Project templates
    // Project templates
    pub fn register_project_template(&mut self, template: ProjectTemplate) {
        self.project_templates.insert(template.meta.id.clone(), template);
    }
    
    pub fn get_project_template(&self, id: &str) -> Option<&ProjectTemplate> {
        self.project_templates.get(id)
    }
    
    pub fn list_project_templates(&self) -> impl Iterator<Item = &ProjectTemplate> {
        self.project_templates.values()
    }

    pub fn list_project_templates_by_category<'a>(&'a self, category: &'a str) -> impl Iterator<Item = &'a ProjectTemplate> + 'a {
        self.project_templates.values().filter(move |t| t.meta.category == category)
    }

    
    // Widget templates
    pub fn register_widget_template(&mut self, template: WidgetTemplate) {
        self.widget_templates.insert(template.meta.id.clone(), template);
    }
    
    pub fn get_widget_template(&self, id: &str) -> Option<&WidgetTemplate> {
        self.widget_templates.get(id)
    }
    
    pub fn list_widget_templates(&self) -> impl Iterator<Item = &WidgetTemplate> {
        self.widget_templates.values()
    }
    
    // Widget template groups
    pub fn register_widget_template_group(&mut self, group: WidgetTemplateGroup) {
        self.widget_template_groups.insert(group.name.clone(), group);
    }
    
    pub fn get_widget_template_group(&self, name: &str) -> Option<&WidgetTemplateGroup> {
        self.widget_template_groups.get(name)
    }
    
    pub fn list_widget_template_groups(&self) -> impl Iterator<Item = &WidgetTemplateGroup> {
        self.widget_template_groups.values()
    }
    
    // Code templates
    pub fn register_code_template(&mut self, template: CodeTemplate) {
        self.code_templates.insert(template.meta.id.clone(), template);
    }
    
    pub fn get_code_template(&self, id: &str) -> Option<&CodeTemplate> {
        self.code_templates.get(id)
    }
    
    pub fn list_code_templates(&self) -> impl Iterator<Item = &CodeTemplate> {
        self.code_templates.values()
    }
    
    // Event templates
    pub fn register_event_template(&mut self, template: EventTemplate) {
        self.event_templates.insert(template.meta.id.clone(), template);
    }
    
    pub fn get_event_template(&self, id: &str) -> Option<&EventTemplate> {
        self.event_templates.get(id)
    }
    
    pub fn list_event_templates(&self) -> impl Iterator<Item = &EventTemplate> {
        self.event_templates.values()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Template Engine
// ─────────────────────────────────────────────────────────────────────────────

pub struct TemplateEngine {
    registry: TemplateRegistryRef,
}

impl TemplateEngine {
    pub fn new(registry: TemplateRegistryRef) -> Self {
        Self { registry }
    }
    
    /// Generate project from template
    pub fn generate_project(
        &self,
        template_id: &str,
        settings: HashMap<String, serde_json::Value>,
    ) -> Result<ProjectTemplate, TemplateError> {
        let registry = self.registry.read().unwrap();
        let template = registry.get_project_template(template_id)
            .ok_or_else(|| TemplateError::NotFound(template_id.to_string()))?;
        
        let mut generated = template.clone();
        
        // Apply settings overrides
        if let Some(serde_json::Value::String(title)) = settings.get("title") {
            generated.settings.title = title.clone();
        }
        
        Ok(generated)
    }
    
    /// Apply widget template to create widget instances
    pub fn apply_widget_template(
        &self,
        template_id: &str,
        position: (f32, f32),
        overrides: HashMap<String, serde_json::Value>,
    ) -> Result<WidgetInstanceTemplate, TemplateError> {
        let registry = self.registry.read().unwrap();
        let template = registry.get_widget_template(template_id)
            .ok_or_else(|| TemplateError::NotFound(template_id.to_string()))?;
        
        let mut properties = template.default_properties.clone();
        for (key, value) in overrides {
            properties.insert(key, value);
        }
        
        Ok(WidgetInstanceTemplate {
            kind: template.widget_kind.clone(),
            x: position.0,
            y: position.1,
            width: 200.0, // Default width
            height: 40.0, // Default height
            properties,
        })
    }
    
    /// Generate code from template with placeholders
    pub fn generate_code(
        &self,
        template_id: &str,
        variables: HashMap<String, String>,
    ) -> Result<String, TemplateError> {
        let registry = self.registry.read().unwrap();
        let template = registry.get_code_template(template_id)
            .ok_or_else(|| TemplateError::NotFound(template_id.to_string()))?;
        
        let mut content = template.content.clone();
        for (key, value) in variables {
            let placeholder = format!("{{{{{}}}}}", key);
            content = content.replace(&placeholder, &value);
        }
        
        Ok(content)
    }
}

/// Template error type
#[derive(Debug, thiserror::Error)]
pub enum TemplateError {
    #[error("Template not found: {0}")]
    NotFound(String),
    
    #[error("Template validation failed: {0}")]
    Validation(String),
    
    #[error("Template generation failed: {0}")]
    Generation(String),
}

// ─────────────────────────────────────────────────────────────────────────────
// Built-in Templates
// ─────────────────────────────────────────────────────────────────────────────

pub struct BuiltInTemplates;

impl BuiltInTemplates {
    pub fn register_all(registry: &mut TemplateRegistry) {
        Self::register_project_templates(registry);
        Self::register_widget_templates(registry);
        Self::register_code_templates(registry);
    }
    
    fn register_project_templates(registry: &mut TemplateRegistry) {
        // Login Screen Template
        registry.register_project_template(ProjectTemplate {
            meta: TemplateMeta {
                id: "login-screen".to_string(),
                name: "Login Screen".to_string(),
                description: "Classic login form with username and password".to_string(),
                category: "auth".to_string(),
                author: "GUIMaker".to_string(),
                version: "1.0.0".to_string(),
                tags: vec!["auth".to_string(), "login".to_string(), "form".to_string()],
                preview_image: None,
            },
            pages: vec![PageTemplate {
                name: "Login".to_string(),
                widgets: vec![
                    WidgetInstanceTemplate {
                        kind: "Label".to_string(),
                        x: 400.0,
                        y: 200.0,
                        width: 200.0,
                        height: 30.0,
                        properties: serde_json::from_value(serde_json::json!({
                            "label": "Username"
                        })).unwrap(),
                    },

                    WidgetInstanceTemplate {
                        kind: "TextInput".to_string(),
                        x: 400.0,
                        y: 240.0,
                        width: 200.0,
                        height: 40.0,
                        properties: serde_json::from_value(serde_json::json!({
                            "label": "Enter username"
                        })).unwrap(),
                    },

                    WidgetInstanceTemplate {
                        kind: "Label".to_string(),
                        x: 400.0,
                        y: 300.0,
                        width: 200.0,
                        height: 30.0,
                        properties: serde_json::from_value(serde_json::json!({
                            "label": "Password"
                        })).unwrap(),
                    },
                    WidgetInstanceTemplate {
                        kind: "Label".to_string(),
                        x: 400.0,
                        y: 300.0,
                        width: 200.0,
                        height: 30.0,
                        properties: serde_json::from_value(serde_json::json!({
                            "label": "Password"
                        })).unwrap(),
                    },
                    WidgetInstanceTemplate {
                        kind: "TextInput".to_string(),
                        x: 400.0,
                        y: 340.0,
                        width: 200.0,
                        height: 40.0,
                        properties: serde_json::from_value(serde_json::json!({
                            "label": "Enter password"
                        })).unwrap(),
                    },
                    WidgetInstanceTemplate {
                        kind: "Button".to_string(),
                        x: 425.0,
                        y: 400.0,
                        width: 150.0,
                        height: 40.0,
                        properties: serde_json::from_value(serde_json::json!({
                            "label": "Login"
                        })).unwrap(),
                    },
                ],
                layout: LayoutTemplate {
                    kind: LayoutKind::Absolute,
                    config: HashMap::new(),
                },
            }],
            assets: AssetTemplate {
                images: vec![],
                fonts: vec![],
            },
            settings: ProjectSettingsTemplate {
                title: "Login".to_string(),
                width: 1280,
                height: 720,
                background_color: "#1E1E1E".to_string(),
                font: "Roboto".to_string(),
            },
        });
        
        // Dashboard Template
        registry.register_project_template(ProjectTemplate {
            meta: TemplateMeta {
                id: "dashboard".to_string(),
                name: "Dashboard".to_string(),
                description: "Dashboard layout with sidebar and main content area".to_string(),
                category: "layout".to_string(),
                author: "GUIMaker".to_string(),
                version: "1.0.0".to_string(),
                tags: vec!["dashboard".to_string(), "layout".to_string()],
                preview_image: None,
            },
            pages: vec![PageTemplate {
                name: "Dashboard".to_string(),
                widgets: vec![
                    WidgetInstanceTemplate {
                        kind: "Panel".to_string(),
                        x: 0.0,
                        y: 0.0,
                        width: 250.0,
                        height: 720.0,
                        properties: serde_json::from_value(serde_json::json!({
                            "label": "Sidebar"
                        })).unwrap(),
                    },
                    WidgetInstanceTemplate {
                        kind: "Panel".to_string(),
                        x: 250.0,
                        y: 0.0,
                        width: 1030.0,
                        height: 720.0,
                        properties: serde_json::from_value(serde_json::json!({
                            "label": "Main Content"
                        })).unwrap(),
                    },
                ],
                layout: LayoutTemplate {
                    kind: LayoutKind::Absolute,
                    config: HashMap::new(),
                },
            }],
            assets: AssetTemplate {
                images: vec![],
                fonts: vec![],
            },
            settings: ProjectSettingsTemplate {
                title: "Dashboard".to_string(),
                width: 1280,
                height: 720,
                background_color: "#1E1E1E".to_string(),
                font: "Roboto".to_string(),
            },
        });
        
        // Settings Template
        registry.register_project_template(ProjectTemplate {
            meta: TemplateMeta {
                id: "settings-panel".to_string(),
                name: "Settings Panel".to_string(),
                description: "Settings dialog with tabs and form fields".to_string(),
                category: "dialog".to_string(),
                author: "GUIMaker".to_string(),
                version: "1.0.0".to_string(),
                tags: vec!["settings".to_string(), "dialog".to_string(), "form".to_string()],
                preview_image: None,
            },
            pages: vec![PageTemplate {
                name: "Settings".to_string(),
                widgets: vec![
                    WidgetInstanceTemplate {
                        kind: "Label".to_string(),
                        x: 200.0,
                        y: 200.0,
                        width: 100.0,
                        height: 30.0,
                        properties: serde_json::from_value(serde_json::json!({
                            "label": "Theme:"
                        })).unwrap(),
                    },
                    WidgetInstanceTemplate {
                        kind: "DropDown".to_string(),
                        x: 320.0,
                        y: 200.0,
                        width: 150.0,
                        height: 30.0,
                        properties: serde_json::from_value(serde_json::json!({})).unwrap(),
                    },
                    WidgetInstanceTemplate {
                        kind: "Button".to_string(),
                        x: 480.0,
                        y: 280.0,
                        width: 100.0,
                        height: 35.0,
                        properties: serde_json::from_value(serde_json::json!({
                            "label": "Save"
                        })).unwrap(),
                    },
                ],
                layout: LayoutTemplate {
                    kind: LayoutKind::Absolute,
                    config: HashMap::new(),
                },
            }],
            assets: AssetTemplate {
                images: vec![],
                fonts: vec![],
            },
            settings: ProjectSettingsTemplate {
                title: "Settings".to_string(),
                width: 800,
                height: 600,
                background_color: "#1E1E1E".to_string(),
                font: "Roboto".to_string(),
            },
        });
    }
    
    fn register_widget_templates(registry: &mut TemplateRegistry) {
        // Button templates
        registry.register_widget_template(WidgetTemplate {
            meta: TemplateMeta {
                id: "button-primary".to_string(),
                name: "Primary Button".to_string(),
                description: "Standard primary action button".to_string(),
                category: "button".to_string(),
                author: "GUIMaker".to_string(),
                version: "1.0.0".to_string(),
                tags: vec!["button".to_string(), "primary".to_string()],
                preview_image: None,
            },
            widget_kind: "Button".to_string(),
            default_properties: serde_json::from_value(serde_json::json!({
                "label": "Click me",
                "bg_color_hex": "#6C63FF",
                "text_color_hex": "#FFFFFF",
                "border_radii": [6.0, 6.0, 6.0, 6.0]
            })).unwrap(),
            required_properties: vec!["label".to_string()],
            sample_usage: "A button for primary actions".to_string(),
        });

        
        // TextInput templates

        registry.register_widget_template(WidgetTemplate {
            meta: TemplateMeta {
                id: "input-text".to_string(),
                name: "Text Input".to_string(),
                description: "Standard text input field".to_string(),
                category: "input".to_string(),
                author: "GUIMaker".to_string(),
                version: "1.0.0".to_string(),
                tags: vec!["input".to_string(), "text".to_string()],
                preview_image: None,
            },
            widget_kind: "TextInput".to_string(),
            default_properties: serde_json::from_value(serde_json::json!({
                "label": "Enter text"
            })).unwrap(),
            required_properties: vec!["label".to_string()],
            sample_usage: "A text input field for user entry".to_string(),
        });


    }
    
    fn register_code_templates(registry: &mut TemplateRegistry) {
        // Button event handler template
        registry.register_code_template(CodeTemplate {
            meta: TemplateMeta {
                id: "button-event-handler".to_string(),
                name: "Button Event Handler".to_string(),
                description: "Template for button click event handling".to_string(),
                category: "events".to_string(),
                author: "GUIMaker".to_string(),
                version: "1.0.0".to_string(),
                tags: vec!["event".to_string(), "button".to_string(), "handler".to_string()],
                preview_image: None,
            },
            language: "rust".to_string(),
            content: r#"if self.{{button_name}}.pressed {
    // TODO: Handle {{button_name}} click
    println!("{{button_name}} clicked!");
    self.{{button_name}}.pressed = false;
}"#.to_string(),
            placeholders: vec![
                CodePlaceholder {
                    name: "button_name".to_string(),
                    description: "Name of the button variable".to_string(),
                    default_value: "button_1".to_string(),
                    required: true,
                },
            ],
        });
        
        // Layout template
        registry.register_code_template(CodeTemplate {
            meta: TemplateMeta {
                id: "flex-layout".to_string(),
                name: "Flex Layout".to_string(),
                description: "Template for flexible layout".to_string(),
                category: "layout".to_string(),
                author: "GUIMaker".to_string(),
                version: "1.0.0".to_string(),
                tags: vec!["layout".to_string(), "flex".to_string()],
                preview_image: None,
            },
            language: "rust".to_string(),
            content: r#"// Flex layout container
// Configure {{direction}} layout with gap {{gap}}
let container = Container::new()
    .with_layout(Layout::{{direction}})
    .with_gap({{gap}})
    .build();
"#.to_string(),
            placeholders: vec![
                CodePlaceholder {
                    name: "direction".to_string(),
                    description: "Layout direction (Horizontal/Vertical)".to_string(),
                    default_value: "Horizontal".to_string(),
                    required: true,
                },
                CodePlaceholder {
                    name: "gap".to_string(),
                    description: "Gap between elements".to_string(),
                    default_value: "8.0".to_string(),
                    required: false,
                },
            ],
        });
    }
}
