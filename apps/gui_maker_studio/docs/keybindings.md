# Controles y Atajos de Teclado

Referencia completa de todos los controles de GUIMaker.

---

## Teclado

| Tecla | Contexto | Acción |
|---|---|---|
| `Delete` | Widget seleccionado | Elimina el widget seleccionado del canvas |
| `Backspace` | Widget seleccionado | Elimina el widget seleccionado del canvas |
| `Escape` | Cualquiera | Cierra la aplicación |

> **Nota:** Más atajos de teclado están planificados (ver [`roadmap.md`](roadmap.md)).

---

## Ratón — Canvas

| Acción | Efecto |
|---|---|
| **Click izquierdo** sobre un widget | Selecciona ese widget (activa el panel de propiedades) |
| **Click izquierdo** en zona vacía | Deselecciona el widget actual |
| **Click izquierdo + arrastrar** sobre un widget | Mueve el widget en el canvas |
| **Soltar** tras arrastrar | El widget queda en la posición más cercana al grid de snap (4 px) |
| **Pasar el cursor** sobre un widget | Muestra un tooltip con el nombre de variable y tipo |

---

## Ratón — Panel de código generado

| Acción | Efecto |
|---|---|
| **Scroll arriba/abajo** | Navega el contenido del panel de código |

---

## Botones de la Toolbox (panel izquierdo)

| Botón | Acción |
|---|---|
| Nombre de cualquier widget | Añade ese widget al centro del canvas y lo selecciona |
| **⚙ Generar Código** | Genera el `main.rs` y abre el visor de código |
| **🗑 Limpiar Todo** | Elimina todos los widgets del canvas y cierra el visor de código |
| **✕ Cerrar Código** | Cierra el visor de código sin borrar nada |

---

## Botones del Panel de Propiedades (panel derecho)

Disponibles solo cuando hay un widget seleccionado.

| Botón | Propiedad | Incremento |
|---|---|---|
| `−` / `+` junto a **X** | Posición horizontal | ±20 px |
| `−` / `+` junto a **Y** | Posición vertical | ±20 px |
| `−` / `+` junto a **W** | Ancho | ±20 px (mín. 20) |
| `−` / `+` junto a **H** | Alto | ±20 px (mín. 12) |
| `−` / `+` junto a **Radius** | Radio de esquinas | ±4 px (mín. 0) |
| `−` / `+` junto a **Value** | Valor numérico | ±0.05 (rango 0–1) |
| **Eliminar** | — | Elimina el widget seleccionado |
| **↑** (to front) | Orden de capas | Sube un nivel en la pila de renderizado |
| **↓** (to back) | Orden de capas | Baja un nivel en la pila de renderizado |

---

## Snap y grid

- **Grid visual:** 20 × 20 px — solo referencia visual, no magnético
- **Snap al mover:** el widget se pega al múltiplo de 4 px más cercano al soltar
- **Snap al crear:** el nuevo widget aparece en el múltiplo de 4 px más cercano al centro del canvas

---

## Resaltado de sintaxis en el visor de código

El visor de código no es un editor, pero aplica colores básicos:

| Color | Tipo de línea |
|---|---|
| 🟢 Verde | Comentarios (`//`) |
| 🔵 Azul claro | Palabras clave (`fn`, `struct`, `impl`, `use`) |
| 🟣 Violeta | Modificador `pub` |
| 🟡 Amarillo | Líneas con `TODO` |
| ⚪ Blanco | Resto del código |
