# Arquitectura de GUIMaker

Este documento describe la estructura interna de GUIMaker, cómo se relacionan los módulos entre sí y las decisiones de diseño tomadas.

---

## Vista general

GUIMaker sigue la arquitectura impuesta por `FerrousApp`: un único struct implementa el trait y recibe callbacks por frame. Los módulos internos son **datos puros** (`model`), **transformación de datos** (`codegen`) y **presentación** (`toolbox`, `properties`).

```
┌──────────────────────────────────────────────────────┐
│                     main.rs                          │
│                  GUIMakerApp                         │
│                                                      │
│  ┌─────────────┐  ┌──────────────┐  ┌────────────┐  │
│  │  Toolbox    │  │ CanvasState  │  │ Properties │  │
│  │ (toolbox.rs)│  │  (model.rs)  │  │  Panel     │  │
│  └──────┬──────┘  └──────┬───────┘  └─────┬──────┘  │
│         │                │                │          │
│         └────────────────▼────────────────┘          │
│                          │                           │
│                  ┌───────▼──────┐                    │
│                  │  codegen.rs  │                    │
│                  │  generate()  │                    │
│                  └──────────────┘                    │
└──────────────────────────────────────────────────────┘
```

---

## Módulos

### `model.rs` — Estado del dominio

Contiene únicamente **datos y lógica de dominio pura**, sin nada de UI.

```
WidgetKind          enum con los 10 tipos de widget
  └── display_name()
  └── preview_color()
  └── default_size()
  └── all()

WidgetNode          un widget concreto sobre el canvas
  └── id, kind, x, y, width, height
  └── label, var_name, value, radius

CanvasState         estado global del editor
  └── widgets: Vec<WidgetNode>
  └── selected_id, drag_widget_id, drag_offset
  └── project_name, generated_code, show_code_panel
  └── add_widget(), delete_selected(), hit_test()
  └── get(), get_mut()
```

**Decisión de diseño:** `CanvasState` no conoce coordenadas de ventana. Trabaja siempre en espacio-canvas. La traducción window↔canvas ocurre en `main.rs`.

---

### `codegen.rs` — Generación de código

Función pura: toma `&CanvasState` y devuelve un `String` con un `main.rs` compilable.

```
generate(state: &CanvasState) -> String
  ├── Cabecera con comentarios
  ├── Imports inteligentes (solo importa los tipos usados)
  ├── struct Application { campo por widget con tipo }
  ├── impl Default { constructor con coordenadas exactas }
  ├── impl FerrousApp
  │   ├── configure_ui  → ui.add(self.widget.clone())
  │   ├── update        → manejo de eventos + TODO por botón
  │   └── draw_ui       → widget.draw(gui) + labels como texto
  └── fn main()         → App::new().with_*().run()
```

**Helper `to_pascal_case`:** convierte `"my_gui_app"` → `"MyGuiApp"` para el nombre del struct.

**Imports inteligentes:** antes de escribir el `use ferrous_gui::{}`, el generador inspecciona qué `WidgetKind` están presentes en el canvas y solo incluye los tipos necesarios.

---

### `toolbox.rs` — Panel de herramientas

Renderiza el panel izquierdo con un `Button` por cada `WidgetKind`. Expone:

- `register(&self, ui)` — registra todos los botones en el sistema de UI del engine (llamado una sola vez en `configure_ui`)
- `consume_pressed() -> Option<WidgetKind>` — devuelve y consume el widget que se quiere añadir
- `draw(gui, text, font)` — dibuja el panel visualmente

Los botones de acción (`btn_generate`, `btn_clear`, `btn_close_code`) son campos públicos que `main.rs` lee directamente.

---

### `properties.rs` — Panel de propiedades

Renderiza el panel derecho con botones `−` / `+` para cada propiedad del widget seleccionado.

- `register(&self, ui)` — registra todos los botones
- `apply(&mut self, state) -> bool` — aplica los cambios al `CanvasState`
- `delete_pressed() -> bool` — consume el evento de borrado
- `draw(gui, text, font, state)` — dibuja el panel con los valores actuales

**Decisión de diseño:** `apply` toma el `&mut CanvasState` entero. El orden importa: primero `apply`, luego `delete_pressed`, porque `delete` necesita que `selected_id` todavía exista.

---

### `main.rs` — Orquestador

`GUIMakerApp` no contiene lógica de dominio propia. Su responsabilidad es:

1. **`configure_ui`** — registrar todos los widgets de UI permanentes
2. **`on_resize`** — reposicionar el panel de propiedades en el borde derecho
3. **`update`** — leer eventos (toolbox, propiedades, ratón, teclado) y mutar el `CanvasState`
4. **`draw_ui`** — dibujar la barra de menú, canvas, grid, widgets, tooltips, paneles y overlay de código

---

## Flujo de un frame

```
FerrousEngine frame tick
  │
  ├─▶ update()
  │     ├── Escape → request_exit
  │     ├── Delete → canvas.delete_selected()
  │     ├── toolbox.consume_pressed() → canvas.add_widget()
  │     ├── btn_generate → codegen::generate() → show_code_panel = true
  │     ├── btn_clear → limpiar canvas
  │     ├── properties.apply(&mut canvas)
  │     ├── properties.delete_pressed() → canvas.delete_selected()
  │     └── mouse hit-test + drag → mover widget
  │
  └─▶ draw_ui()
        ├── Barra de menú superior
        ├── Fondo del canvas
        ├── draw_grid()
        ├── Por cada widget → rect (sombra + relleno) + outline si seleccionado
        ├── Tooltips
        ├── toolbox.draw()
        ├── properties.draw()
        └── draw_code_panel() si show_code_panel
```

---

## Espacio de coordenadas

| Espacio | Origen | Usado en |
|---|---|---|
| **Ventana** | Esquina superior-izquierda de la ventana | `ctx.input.mouse_position()`, todos los `gui.rect()` |
| **Canvas** | Esquina superior-izquierda del área de canvas | `WidgetNode.x/y`, `hit_test()`, `drag_offset` |

La conversión es:
```
canvas_x = window_x - (TOOLBOX_W + 16.0)
canvas_y = window_y - (MENU_H + 8.0)
```

Al dibujar un widget del canvas en `draw_ui`:
```
screen_x = canvas_origin_x + widget.x
screen_y = canvas_origin_y + widget.y
```

---

## Extensibilidad

Para añadir un nuevo tipo de widget:

1. **`model.rs`** — añadir variante a `WidgetKind`, implementar `display_name`, `preview_color`, `default_size`, y añadirlo a `all()`
2. **`codegen.rs`** — añadir ramas en `widget_type_name()`, y en los bloques de `Default` / `update` / `draw_ui`
3. No se requieren cambios en `toolbox.rs` ni `properties.rs` — leen `WidgetKind::all()` automáticamente
