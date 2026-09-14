//! Módulo de persistencia de proyectos GUIMaker
//!
//! Permite guardar y cargar proyectos en formato JSON.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::scene::{PlacedWidget, SceneState};

/// Representa un proyecto GUIMaker completo listo para serializar.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Project {
    /// Nombre del proyecto (para el título de la ventana)
    pub name: String,
    /// Versión del formato de archivo
    pub version: String,
    /// Páginas del proyecto (cada página tiene su propio estado de widgets)
    pub pages: Vec<Page>,
    /// Página actual activa
    pub current_page_index: usize,
    /// Configuración adicional del canvas
    pub canvas_config: CanvasConfig,
    /// Assets del proyecto
    pub assets: AssetRegistry,
    /// Timestamp de creación/última modificación
    pub last_modified: String,
    /// Configuración de tema
    pub theme_name: String,
}

/// Representa una página individual en un proyecto GUIMaker.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Page {
    /// Nombre de la página
    pub name: String,
    /// Widgets colocados en esta página
    pub widgets: Vec<PlacedWidget>,
    /// Configuración de canvas para esta página
    pub canvas_config: CanvasConfig,
}

/// Configuración adicional del canvas.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CanvasConfig {
    /// Factor de zoom (1.0 = 100%)
    pub zoom: f32,
    /// Desplazamiento pan X
    pub pan_x: f32,
    /// Desplazamiento pan Y
    pub pan_y: f32,
    /// Dimensiones de la ventana de vista previa
    pub preview_width: f32,
    pub preview_height: f32,
}

/// Registro de assets.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AssetRegistry {
    pub images: HashMap<String, ImageAsset>,
    pub fonts: HashMap<String, FontAsset>,
    pub styles: HashMap<String, StyleAsset>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ImageAsset {
    pub path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FontAsset {
    pub path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StyleAsset {
    pub path: String,
}

impl Page {
    /// Crea una nueva página vacía.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            widgets: Vec::new(),
            canvas_config: CanvasConfig::default(),
        }
    }
}

impl Project {
    /// Crea un nuevo proyecto vacío.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            pages: vec![Page::new("Página 1")],
            current_page_index: 0,
            canvas_config: CanvasConfig::default(),
            assets: AssetRegistry::default(),
            last_modified: chrono::Utc::now().to_rfc3339(),
            theme_name: "dark".to_string(),
        }
    }

    /// Crea un proyecto desde el estado actual de la escena.
    pub fn from_scene(scene: &SceneState, name: String, zoom: f32, pan_x: f32, pan_y: f32) -> Self {
        let mut page = Page::new("Página 1");
        page.widgets = scene.widgets.clone();
        page.canvas_config = CanvasConfig {
            zoom,
            pan_x,
            pan_y,
            preview_width: 800.0,
            preview_height: 600.0,
        };
        
        Self {
            name,
            version: env!("CARGO_PKG_VERSION").to_string(),
            pages: vec![page],
            current_page_index: 0,
            canvas_config: CanvasConfig::default(),
            assets: AssetRegistry::default(),
            last_modified: chrono::Utc::now().to_rfc3339(),
            theme_name: "dark".to_string(),
        }
    }

    /// Aplica el contenido de la página actual a una escena.
    pub fn apply_to_scene(&self, scene: &mut SceneState) {
        if let Some(page) = self.pages.get(self.current_page_index) {
            scene.widgets = page.widgets.clone();
            scene.selected_id = None;
            scene.next_id = scene.widgets.iter().map(|w| w.id).max().unwrap_or(0) + 1;
        }
    }

    /// Serializa el proyecto a JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserializa JSON a un proyecto.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Guarda el proyecto en un archivo.
    pub fn save_to_file(&self, path: &PathBuf) -> Result<(), std::io::Error> {
        let json = self.to_json().map_err(|e| std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Error serializando proyecto: {}", e),
        ))?;
        std::fs::write(path, json)
    }

    /// Obtiene la página actual activa.
    pub fn current_page(&self) -> Option<&Page> {
        self.pages.get(self.current_page_index)
    }

    /// Obtiene la página actual activa (mutable).
    pub fn current_page_mut(&mut self) -> Option<&mut Page> {
        self.pages.get_mut(self.current_page_index)
    }

    /// Cambia a la página especificada.
    pub fn set_current_page(&mut self, index: usize) -> bool {
        if index < self.pages.len() {
            self.current_page_index = index;
            true
        } else {
            false
        }
    }

    /// Añade una nueva página al proyecto.
    pub fn add_page(&mut self, name: impl Into<String>) -> usize {
        let page = Page::new(name);
        self.pages.push(page);
        self.pages.len() - 1
    }

    /// Elimina una página del proyecto.
    /// No permite eliminar la última página.
    pub fn remove_page(&mut self, index: usize) -> bool {
        if self.pages.len() <= 1 {
            return false;
        }
        if index < self.pages.len() {
            self.pages.remove(index);
            if self.current_page_index >= self.pages.len() {
                self.current_page_index = self.pages.len() - 1;
            }
            true
        } else {
            false
        }
    }

    /// Renombra una página.
    pub fn rename_page(&mut self, index: usize, new_name: impl Into<String>) -> bool {
        if let Some(page) = self.pages.get_mut(index) {
            page.name = new_name.into();
            true
        } else {
            false
        }
    }

    /// Carga un proyecto desde un archivo.
    pub fn load_from_file(path: &PathBuf) -> Result<Self, std::io::Error> {
        let json = std::fs::read_to_string(path)?;
        Self::from_json(&json).map_err(|e| std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Error deserializando proyecto: {}", e),
        ))
    }
}

impl Default for Project {
    fn default() -> Self {
        Self::new("Untitled")
    }
}