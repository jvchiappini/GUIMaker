<div align="center">

# 🖼️ GUIMaker

**Visual GUI skeleton builder for FerrousEngine**

*Design your interface visually — get production-ready Rust code instantly.*

---

[![Rust](https://img.shields.io/badge/Rust-2021_Edition-orange?logo=rust)](https://www.rust-lang.org/)
[![Engine](https://img.shields.io/badge/FerrousEngine-Desktop2D-blueviolet)](#)
[![License](https://img.shields.io/badge/License-MIT-green)](#license)
[![Status](https://img.shields.io/badge/Status-In_Development-yellow)](#)

</div>

---

## ¿Qué es GUIMaker?

**GUIMaker** es una herramienta visual de escritorio construida sobre **FerrousEngine** que te permite diseñar el layout de una interfaz gráfica arrastrando widgets a un canvas y, con un solo clic, generar el **esqueleto completo de código Rust** para ese layout.

El objetivo es simple: tú diseñas la estructura visual, GUIMaker escribe la parte aburrida, y tú solo implementas la lógica.

```
Diseñas visualmente  →  Generas código  →  Añades tu lógica  →  ✅
```

---

## ✨ Características

| Función | Descripción |
|---|---|
| 🎨 **Canvas interactivo** | Cuadrícula con snap para colocar y mover widgets con precisión |
| 🧩 **10 tipos de widget** | Button, Label, Slider, Checkbox, TextInput, Panel, Separator, Image, ProgressBar, Dropdown |
| 🔧 **Panel de propiedades** | Ajusta posición, tamaño, radio de esquinas y valor directamente |
| ⚙️ **Generación de código** | Produce un `main.rs` completo con imports, struct, `Default`, `configure_ui`, `update` y `draw_ui` |
| 🏷️ **Tooltips** | Al pasar el ratón sobre un widget muestra su nombre de variable |
| 📋 **Visor de código** | Panel overlay con scroll y resaltado básico de sintaxis |
| ⌨️ **Atajos de teclado** | `Delete`/`Backspace` para eliminar, `Escape` para salir |
| 🔄 **Orden de capas** | Sube y baja widgets en el orden de renderizado |

---

## 🚀 Inicio rápido

### 1. Pre-requisitos

- [Rust](https://rustup.rs/) 1.75+ con `cargo`
- FerrousEngine clonado en `../FerrousEngine` (relativo a este proyecto)

### 2. Clonar y compilar

```bash
git clone https://github.com/tu-usuario/gui-maker
cd gui-maker
cargo build
```

### 3. Ejecutar

```bash
cargo run
```

> La ventana abre en **1280 × 720**. Asegúrate de tener una fuente en
> `assets/fonts/Roboto-Regular.ttf` o ajusta la ruta en `src/main.rs`.

---

## 🖥️ Interfaz

```
┌─────────────────────────────────────────────────────────────────────────┐
│  GUIMaker                   barra de menú superior                       │
├──────────────┬──────────────────────────────────┬────────────────────────┤
│  TOOLBOX     │                                  │  PROPIEDADES           │
│              │          CANVAS                  │                        │
│  • Button    │   (cuadrícula 20px × snap 4px)   │  X / Y  [ - ]  [ + ]  │
│  • Label     │                                  │  W / H  [ - ]  [ + ]  │
│  • Slider    │   [  Widget A  ]  [ Widget B ]   │  Radius [ - ]  [ + ]  │
│  • Checkbox  │                                  │  Value  [ - ]  [ + ]  │
│  • TextInput │      [ Selected Widget ]         │  ─────────────────     │
│  • Panel     │                                  │  [ Eliminar ]          │
│  • ...       │                                  │  [ ↑ ]  [ ↓ ]         │
│              │                                  │                        │
│  ───────────  │                                  │                        │
│  ⚙ Generar   │                                  │                        │
│  🗑 Limpiar  │                                  │                        │
│  ✕ Cerrar   │                                  │                        │
└──────────────┴──────────────────────────────────┴────────────────────────┘
```

---

## 🔄 Flujo de trabajo

```
1. Click en un widget de la Toolbox   →  aparece en el centro del canvas
2. Drag para reposicionarlo           →  snap automático a 4px
3. Click para seleccionarlo           →  panel de propiedades activo
4. Ajustar X, Y, W, H, Radius, Value →  con botones − / +
5. Repetir para todos los widgets     →  tu layout completo
6. ⚙ "Generar Código"                →  visor con el main.rs generado
7. Copiar y pegar en tu proyecto      →  implementar la lógica (TODO)
```

---

## 📄 Ejemplo de código generado

Dado un canvas con un `Button` y un `Slider`, GUIMaker produce algo así:

```rust
// Código generado por GUIMaker
// Proyecto: my_gui_app

use ferrous_app::{App, AppContext, AppMode, Color, FerrousApp, KeyCode};
use ferrous_assets::Font;
use ferrous_gui::{Button, GuiBatch, Slider, TextBatch, Ui};

struct MyGuiApp {
    button_1: Button,
    slider_2: Slider,
}

impl Default for MyGuiApp {
    fn default() -> Self {
        Self {
            button_1: Button::new(100.0, 80.0, 160.0, 40.0).with_radius(4.0),
            slider_2: Slider::new(100.0, 140.0, 200.0, 24.0, 0.50),
        }
    }
}

impl FerrousApp for MyGuiApp {
    fn configure_ui(&mut self, ui: &mut Ui) {
        ui.add(self.button_1.clone());
        ui.add(self.slider_2.clone());
    }

    fn update(&mut self, ctx: &mut AppContext) {
        if ctx.input.just_pressed(KeyCode::Escape) { ctx.request_exit(); }

        if self.button_1.pressed {
            // TODO: lógica al presionar "Button 1"
            self.button_1.pressed = false;
        }
    }

    fn draw_ui(&mut self, gui: &mut GuiBatch, text: &mut TextBatch,
               font: Option<&Font>, _ctx: &mut AppContext) {
        self.button_1.draw(gui);
        self.slider_2.draw(gui);
    }
}

fn main() {
    App::new(MyGuiApp::default())
        .with_title("my_gui_app")
        .with_size(1280, 720)
        .with_mode(AppMode::Desktop2D)
        .with_background_color(Color::rgb(0.10, 0.10, 0.12))
        .with_font("assets/fonts/Roboto-Regular.ttf")
        .run();
}
```

---

## 📁 Estructura del proyecto

```
GUIMaker/
├── Cargo.toml              # Dependencias (ferrous_app, ferrous_gui)
├── README.md               # Este archivo
├── docs/
│   ├── architecture.md     # Arquitectura interna y decisiones de diseño
│   ├── widgets.md          # Referencia de todos los widgets soportados
│   ├── codegen.md          # Cómo funciona el generador de código
│   ├── keybindings.md      # Atajos de teclado y controles del ratón
│   ├── roadmap.md          # Funcionalidades planeadas
│   └── contributing.md     # Guía para contribuir
└── src/
    ├── main.rs             # GUIMakerApp — punto de entrada y lógica de frame
    ├── model.rs            # WidgetKind, WidgetNode, CanvasState
    ├── codegen.rs          # Generador de código Rust
    ├── toolbox.rs          # Panel izquierdo — botones de widgets
    └── properties.rs       # Panel derecho — ajuste de propiedades
```

---

## ⌨️ Atajos

| Tecla / Acción | Efecto |
|---|---|
| `Click` en toolbox | Añade widget al centro del canvas |
| `Click` en widget | Lo selecciona |
| `Drag` sobre widget | Lo mueve (con snap a 4px) |
| `Delete` / `Backspace` | Elimina el widget seleccionado |
| `Scroll` en panel de código | Navega el código generado |
| `Escape` | Cierra la aplicación |

---

## 🗺️ Roadmap

- [ ] Edición de la etiqueta/texto del widget con doble click
- [ ] Guardar/cargar proyectos en `.json`
- [ ] Copiar al portapapeles del sistema
- [ ] Deshacer / rehacer (`Ctrl+Z` / `Ctrl+Y`)
- [ ] Alineación automática (centrar, distribuir)
- [ ] Múltiples páginas/pantallas en un mismo proyecto
- [ ] Exportar a archivo directamente (`Ctrl+S`)
- [ ] Temas de color del canvas

Ver el roadmap completo en [`docs/roadmap.md`](docs/roadmap.md).

---

## 📚 Documentación

| Documento | Contenido |
|---|---|
| [`docs/architecture.md`](docs/architecture.md) | Cómo encajan los módulos entre sí |
| [`docs/widgets.md`](docs/widgets.md) | Referencia de widgets y sus propiedades |
| [`docs/codegen.md`](docs/codegen.md) | Lógica interna del generador de código |
| [`docs/keybindings.md`](docs/keybindings.md) | Controles completos |
| [`docs/roadmap.md`](docs/roadmap.md) | Planes futuros |
| [`docs/contributing.md`](docs/contributing.md) | Cómo contribuir al proyecto |

---

## 📦 Dependencias

```toml
[dependencies]
ferrous_app = { path = "../FerrousEngine/crates/ferrous_app" }
ferrous_gui = { path = "../FerrousEngine/crates/ferrous_gui" }
```

GUIMaker no tiene dependencias externas más allá de FerrousEngine.

---

## 📝 Licencia

MIT License — ver [`LICENSE`](LICENSE) para detalles.

---

<div align="center">
Hecho con ❤️ y FerrousEngine
</div>
