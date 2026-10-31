//! Widget categories for GUIMaker
//!
//! This module provides widget categories to organize widgets in the palette.

use crate::scene::WidgetKind;

/// Widget categories for organizing widgets in the palette
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum WidgetCategory {
    /// Input widgets (Button, TextInput, Slider, etc.)
    Input,
    /// Display widgets (Label, Image, ProgressBar, etc.)
    Display,
    /// Layout widgets (Panel, Separator, etc.)
    Layout,
    /// Navigation widgets (Tabs, TreeView, etc.)
    Navigation,
    /// Data widgets (ListView, TableView, etc.)
    Data,
    /// Advanced widgets (Modal, Tooltip, etc.)
    Advanced,
}

impl WidgetCategory {
    pub fn name(&self) -> &'static str {
        match self {
            WidgetCategory::Input => "Input",
            WidgetCategory::Display => "Display",
            WidgetCategory::Layout => "Layout",
            WidgetCategory::Navigation => "Navigation",
            WidgetCategory::Data => "Data",
            WidgetCategory::Advanced => "Advanced",
        }
    }

    pub fn widgets(&self) -> Vec<WidgetKind> {
        match self {
            WidgetCategory::Input => vec![
                WidgetKind::Button,
                WidgetKind::Slider,
                WidgetKind::Checkbox,
                WidgetKind::TextInput,
                WidgetKind::DropDown,
                WidgetKind::RadioButton, // Nueva
                WidgetKind::Toggle,      // Nueva
                WidgetKind::ColorPicker, // Nueva
                WidgetKind::FileInput,   // Nueva
            ],
            WidgetCategory::Display => vec![
                WidgetKind::Label,
                WidgetKind::Image,
                WidgetKind::ProgressBar,
                WidgetKind::ProgressBarCircular, // Nueva
            ],
            WidgetCategory::Layout => vec![
                WidgetKind::Panel,
                WidgetKind::Separator,
                WidgetKind::ScrollArea, // Nueva
                WidgetKind::Tabs,       // Nueva
            ],
            WidgetCategory::Navigation => vec![
                WidgetKind::TreeView,
                WidgetKind::Tabs,
            ],
            WidgetCategory::Data => vec![
                WidgetKind::ListView,
                WidgetKind::TableView,
            ],
            WidgetCategory::Advanced => vec![
                WidgetKind::Modal,
                WidgetKind::Tooltip,
                WidgetKind::DateTimePicker,
                WidgetKind::RangeSlider,
            ],
        }
    }
}

/// Registry for widget categories
pub struct WidgetCategoryRegistry {
    categories: std::collections::HashMap<WidgetCategory, Vec<WidgetKind>>,
}

impl WidgetCategoryRegistry {
    pub fn new() -> Self {
        let mut categories = std::collections::HashMap::new();
        
        for category in [
            WidgetCategory::Input,
            WidgetCategory::Display,
            WidgetCategory::Layout,
            WidgetCategory::Navigation,
            WidgetCategory::Data,
            WidgetCategory::Advanced,
        ] {
            categories.insert(category.clone(), category.widgets());
        }
        
        Self { categories }
    }
    
    pub fn get_category(&self, category: &WidgetCategory) -> Option<&Vec<WidgetKind>> {
        self.categories.get(category)
    }
    
    pub fn get_category_for_widget(&self, widget: &WidgetKind) -> Option<WidgetCategory> {
        for (category, widgets) in &self.categories {
            if widgets.contains(widget) {
                return Some(category.clone());
            }
        }
        None
    }
    
    pub fn all_categories(&self) -> impl Iterator<Item = &WidgetCategory> {
        self.categories.keys()
    }
    
    pub fn all_widgets(&self) -> impl Iterator<Item = &WidgetKind> {
        self.categories.values().flatten()
    }
}