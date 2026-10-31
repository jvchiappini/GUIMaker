//! Diálogos de archivo para GUIMaker
//!
//! Proporciona funciones para abrir y guardar archivos de proyecto.

use std::path::{Path, PathBuf};

/// Abre un diálogo para seleccionar un archivo de proyecto existente.
/// Devuelve la ruta seleccionada o None si el usuario cancela.
pub fn open_project_dialog() -> Option<PathBuf> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let dialog = rfd::FileDialog::new()
            .add_filter("GUIMaker Project", &["guimaker.json", "json"])
            .set_title("Abrir proyecto GUIMaker")
            .pick_file();
        
        dialog
    }
    #[cfg(target_arch = "wasm32")]
    {
        None
    }
}

/// Abre un diálogo para guardar un proyecto.
/// Devuelve la ruta seleccionada o None si el usuario cancela.
pub fn save_project_dialog(default_name: &str) -> Option<PathBuf> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let dialog = rfd::FileDialog::new()
            .add_filter("GUIMaker Project", &["guimaker.json"])
            .set_file_name(default_name)
            .set_title("Guardar proyecto GUIMaker")
            .save_file();
        
        dialog
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = default_name;
        None
    }
}

/// Abre un diálogo para exportar código Rust.
/// Devuelve la ruta seleccionada o None si el usuario cancela.
pub fn export_code_dialog(default_name: &str) -> Option<PathBuf> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let dialog = rfd::FileDialog::new()
            .add_filter("Rust Source", &["rs"])
            .set_file_name(default_name)
            .set_title("Exportar código Rust")
            .save_file();
        
        dialog
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = default_name;
        None
    }
}

/// Obtiene el nombre del archivo sin extensión de una ruta.
pub fn get_file_stem(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("untitled")
        .to_string()
}

/// Obtiene la extensión de un archivo.
pub fn get_extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase())
}