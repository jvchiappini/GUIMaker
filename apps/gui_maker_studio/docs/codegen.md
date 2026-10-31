# Generador de Código (`codegen.rs`)

Este documento explica cómo funciona internamente el generador de código Rust de GUIMaker.

---

## Visión general

La función `codegen::generate(state: &CanvasState) -> String` toma una foto del estado actual del canvas y construye un `String` que contiene un archivo `main.rs` compilable y completo para FerrousEngine.

El generador es una **función pura**: no tiene efectos secundarios, no modifica el estado, y siempre produce el mismo resultado para el mismo input.

---

## Estructura del archivo generado

```
1. Cabecera con comentarios (nombre del proyecto, aviso de generación)
2. Imports (use ferrous_app::..., use ferrous_gui::{...})
3. Struct de la aplicación (un campo por widget con tipo ferrous_gui)
4. impl Default (constructores con coordenadas exactas)
5. impl FerrousApp
   5a. configure_ui  → ui.add(self.widget.clone())
   5b. update        → manejo de eventos por tipo de widget + TODO
   5c. draw_ui       → widget.draw(gui) + Labels como texto
6. fn main()         → App builder con título, tamaño, modo Desktop2D
```

---

## Imports inteligentes

Antes de escribir la línea `use ferrous_gui::{...}`, el generador inspecciona qué `WidgetKind` están presentes en el canvas:

```rust
let has_slider     = state.widgets.iter().any(|w| w.kind == WidgetKind::Slider);
let has_checkbox   = state.widgets.iter().any(|w| w.kind == WidgetKind::Checkbox);
// ...
```

Solo se incluyen los tipos que realmente se usan. Un canvas con solo `Button` y `Label` generará:

```rust
use ferrous_gui::{Button, GuiBatch, TextBatch, Ui};
```

En lugar del import completo con todos los tipos.

---

## Nombre del struct: `to_pascal_case`

El nombre del proyecto (p.ej. `"my_gui_app"`) se convierte a PascalCase para usarlo como nombre del struct:

```
"my_gui_app"    →  "MyGuiApp"
"editor"        →  "Editor"
"todo-list-app" →  "TodoListApp"
```

La función `to_pascal_case` divide por `_`, `-` y espacios, y capitaliza cada segmento.

---

## Widgets con tipo `ferrous_gui` vs. solo visuales

| Widget | Tipo Rust generado | Notas |
|---|---|---|
| `Button` | `Button` | Campo en struct + `ui.add` + `draw` |
| `Slider` | `Slider` | Campo en struct + `ui.add` + `draw` |
| `Checkbox` | `Checkbox` | + campo `{var}_checked: bool` |
| `TextInput` | `TextInput` | + campo `{var}_text: String` |
| `ProgressBar` | `ProgressBar` | Campo en struct + `draw` |
| `Dropdown` | `Dropdown` | + campo `{var}_index: usize` + opciones de ejemplo |
| `Label` | — | Solo genera `text.push_str(...)` en `draw_ui` |
| `Panel` | — | Sin código (solo visual en el editor) |
| `Separator` | — | Sin código (solo visual en el editor) |
| `Image` | — | Sin código (placeholder visual) |

---

## Bloques `TODO`

Para cada `Button`, el generador inserta un bloque con un comentario `TODO`:

```rust
if self.button_1.pressed {

    // TODO: lógica al presionar "Button 1"
    self.button_1.pressed = false; // consumir evento
}
```

Al final de `update` también hay un `TODO` genérico:

```rust
// TODO: lógica de actualización del frame
```

Estos son los únicos puntos que el usuario necesita rellenar.

---

## Ejemplo de transformación

**Canvas:** 1 Button en (100, 80), 1 Slider en (100, 140, valor 0.7)

**Genera:**

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
            slider_2: Slider::new(100.0, 140.0, 200.0, 24.0, 0.70),
        }
    }
}

impl FerrousApp for MyGuiApp {
    fn configure_ui(&mut self, ui: &mut Ui) {
        ui.add(self.button_1.clone());
        ui.add(self.slider_2.clone());
    }

    fn update(&mut self, ctx: &mut AppContext) {
        if ctx.input.just_pressed(KeyCode::Escape) {
            ctx.request_exit();
        }

        if self.button_1.pressed {

            // TODO: lógica al presionar "Button 1"
            self.button_1.pressed = false;
        }

        // TODO: lógica de actualización del frame
    }

    fn draw_ui(
        &mut self,
        gui:  &mut GuiBatch,
        text: &mut TextBatch,
        font: Option<&Font>,
        _ctx: &mut AppContext,
    ) {
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

## Limitaciones actuales

- **Solo un widget del mismo ID** — los nombres de variable son `{kind}_{id}`, lo que garantiza unicidad.
- **Sin jerarquía de contenedores** — todos los widgets son planos. No se generan Panels con hijos.
- **Opciones del Dropdown fijas** — se generan siempre 3 opciones de ejemplo que el usuario debe editar.
- **Labels sin edición de texto** — el label visible en el generador es el mismo que en el editor; para cambiarlo hay que editar el código generado.
- **Sin coordinación con el sistema de assets** — los `Image` no generan código de carga de textura.

---

## Extender el generador

Para añadir soporte a un nuevo tipo de widget, editar `codegen.rs` en estos puntos:

1. **`widget_type_name()`** — mapear `WidgetKind::NuevoWidget` a su tipo Rust
2. **Bloque `has_*`** de imports — añadir la detección del tipo
3. **Bloque `Default`** — añadir el constructor
4. **Bloque `update`** — añadir el manejo de eventos si aplica
5. **Bloque `draw_ui`** — añadir la llamada a `draw`
