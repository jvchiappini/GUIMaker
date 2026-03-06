# Referencia de Widgets

GUIMaker soporta **10 tipos de widget**. Esta página describe cada uno: sus propiedades editables, el tipo que genera en el código Rust, sus dimensiones por defecto y notas de uso.

---

## Tabla resumen

| Widget | Tipo Rust | Tamaño por defecto | Genera estado extra |
|---|---|---|---|
| Button | `Button` | 160 × 40 | — |
| Label | *(texto)* | 180 × 24 | — |
| Slider | `Slider` | 200 × 24 | — |
| Checkbox | `Checkbox` | 140 × 28 | `bool` checked |
| TextInput | `TextInput` | 200 × 32 | `String` text |
| Panel | *(rect)* | 240 × 160 | — |
| Separator | *(rect)* | 200 × 6 | — |
| Image | *(rect)* | 120 × 120 | — |
| ProgressBar | `ProgressBar` | 200 × 20 | — |
| Dropdown | `Dropdown` | 180 × 32 | `usize` index |

---

## Propiedades comunes

Todos los widgets comparten estas propiedades editables desde el panel de propiedades:

| Propiedad | Descripción | Incremento |
|---|---|---|
| **X** | Posición horizontal en el canvas (píxeles) | ±20 px |
| **Y** | Posición vertical en el canvas (píxeles) | ±20 px |
| **W** (Width) | Ancho del widget | ±20 px (mín. 20) |
| **H** (Height) | Alto del widget | ±20 px (mín. 12) |
| **Radius** | Radio de esquinas redondeadas | ±4 px (mín. 0) |
| **Value** | Valor numérico (0.0–1.0) | ±0.05 |

> El campo **Value** es relevante para Slider y ProgressBar. En el resto de widgets se ignora en el código generado.

---

## Button

El widget interactivo más común. Genera un `Button` de `ferrous_gui` con coordenadas y radio exactos.

**Color de previsualización:** azul (`[0.25, 0.52, 0.96]`)

**Código generado:**
```rust
// En Default:
button_1: Button::new(100.0, 80.0, 160.0, 40.0).with_radius(4.0),

// En configure_ui:
ui.add(self.button_1.clone());

// En update (con TODO):
if self.button_1.pressed {
    // TODO: lógica al presionar "Button 1"
    self.button_1.pressed = false;
}

// En draw_ui:
self.button_1.draw(gui);
```

---

## Label

Texto estático. No tiene tipo `ferrous_gui` — se renderiza directamente con `TextBatch`.

**Color de previsualización:** gris (`[0.60, 0.60, 0.60]`)

**Código generado:**
```rust
// En draw_ui (requiere font):
if let Some(f) = font {
    text.push_str("Label 2", 200.0, 50.0, 16.0, [0.9, 0.9, 0.9, 1.0], f);
}
```

> Las propiedades W, H, Radius y Value no tienen efecto en Labels.

---

## Slider

Control deslizante con valor entre 0.0 y 1.0. La propiedad **Value** establece el valor inicial.

**Color de previsualización:** verde (`[0.30, 0.78, 0.54]`)

**Código generado:**
```rust
// En Default:
slider_3: Slider::new(100.0, 140.0, 200.0, 24.0, 0.50),

// En configure_ui:
ui.add(self.slider_3.clone());

// En draw_ui:
self.slider_3.draw(gui);
```

Para leer el valor actual en `update`: `self.slider_3.value`

---

## Checkbox

Casilla de verificación booleana. Genera un campo adicional `{var}_checked: bool` en la aplicación.

**Color de previsualización:** amarillo (`[0.96, 0.78, 0.25]`)

**Código generado:**
```rust
// En Default:
checkbox_4:         Checkbox::new(50.0, 180.0, 140.0, 28.0),
checkbox_4_checked: false,

// En update (sincroniza estado):
self.checkbox_4_checked = self.checkbox_4.checked;

// En draw_ui:
self.checkbox_4.draw(gui);
```

---

## TextInput

Campo de texto editable. Genera un campo adicional `{var}_text: String`.

**Color de previsualización:** gris claro (`[0.80, 0.80, 0.80]`)

**Código generado:**
```rust
// En Default:
text_input_5:      TextInput::new(100.0, 220.0, 200.0, 32.0),
text_input_5_text: String::new(),

// En update:
self.text_input_5_text = self.text_input_5.text.clone();

// En draw_ui:
self.text_input_5.draw(gui);
```

---

## Panel

Contenedor visual (rectángulo semitransparente). Solo visual — no genera tipo `ferrous_gui`. Útil para agrupar widgets visualmente en el diseño.

**Color de previsualización:** gris oscuro semitransparente (`[0.20, 0.20, 0.25, 0.85]`)

> Los Panels no generan código de widget — sirven como guía visual durante el diseño. Para incluir un contenedor en el código real, añade un `gui.rect()` manualmente.

---

## Separator

Línea divisoria horizontal. Solo visual, no genera tipo `ferrous_gui`.

**Color de previsualización:** gris medio (`[0.45, 0.45, 0.50]`)

---

## Image

Placeholder para una imagen o icono. Solo visual en GUIMaker — no genera código de carga de imagen (ese paso depende de los assets de tu proyecto).

**Color de previsualización:** azul claro (`[0.40, 0.65, 0.90]`)

> Marca el espacio donde irá tu imagen; en el proyecto generado deberás añadir la lógica de carga de textura manualmente.

---

## ProgressBar

Barra de progreso. La propiedad **Value** establece el relleno inicial (0.0 = vacío, 1.0 = completo).

**Color de previsualización:** verde (`[0.30, 0.70, 0.30]`)

**Código generado:**
```rust
// En Default:
progress_bar_8: ProgressBar::new(100.0, 300.0, 200.0, 20.0, 0.75),

// En draw_ui:
self.progress_bar_8.draw(gui);
```

Para actualizar el valor en tiempo de ejecución: `self.progress_bar_8.value = 0.5;`

---

## Dropdown

Menú desplegable con opciones. Genera un campo `{var}_index: usize` y opciones de ejemplo (`"Option 1"`, `"Option 2"`, `"Option 3"`).

**Color de previsualización:** naranja (`[0.80, 0.50, 0.20]`)

**Código generado:**
```rust
// En Default:
dropdown_9: Dropdown::new(100.0, 340.0, 180.0, 32.0,
                          vec!["Option 1", "Option 2", "Option 3"]),
dropdown_9_index: 0,

// En update:
self.dropdown_9_index = self.dropdown_9.selected_index;

// En draw_ui:
self.dropdown_9.draw(gui);
```

> Reemplaza el `vec!["Option 1", ...]` con tus opciones reales en el código generado.

---

## Snap y grid

Todos los widgets se posicionan con **snap a 4 píxeles** al soltarlos o moverlos con el ratón. La cuadrícula visual tiene celdas de **20 × 20 píxeles** y sirve como referencia.

Al ajustar con los botones `−` / `+` del panel de propiedades, el incremento es de **20 píxeles** para posición y tamaño, y **4 píxeles** para el radio.
