# Roadmap de GUIMaker

Este documento lista las funcionalidades planeadas, organizadas por prioridad y categoría. El estado de cada ítem refleja la situación actual del proyecto.

---

## Estado del proyecto

**Versión actual:** `0.1.0` — MVP funcional

La versión actual permite diseñar, ajustar y generar código para los 10 tipos de widget soportados. Las iteraciones siguientes se centran en **persistencia**, **ergonomía** y **calidad del código generado**.

---

## 🔴 Alta prioridad

### Guardar y cargar proyectos
- Serializar `CanvasState` a JSON con `serde`
- Guardar con `Ctrl+S` / botón "Guardar"
- Cargar desde archivo con un diálogo nativo
- Formato de archivo: `.guimaker.json`

### Copiar código al portapapeles del sistema
- Integrar `arboard` o similar
- Botón "📋 Copiar" junto al visor de código
- Notificación visual de confirmación

### Editar la etiqueta del widget (doble click)
- Doble click sobre un widget activa un `TextInput` flotante
- Al confirmar (`Enter`) actualiza `widget.label` y `widget.var_name`
- Validación: `var_name` solo acepta identificadores Rust válidos

### Deshacer / rehacer
- Stack de snapshots de `CanvasState` (máx. 50)
- `Ctrl+Z` → deshacer, `Ctrl+Y` / `Ctrl+Shift+Z` → rehacer
- Indicador en la barra de menú del número de pasos disponibles

---

## 🟡 Media prioridad

### Exportar código directamente a archivo
- Diálogo "Guardar como..." para el código `.rs`
- Opción de exportar directamente con la ruta del proyecto destino
- Crear la estructura `src/main.rs` completa si no existe

### Múltiples páginas / pantallas
- Pestañas en la barra superior para navegar entre pantallas
- Cada pantalla tiene su propio `CanvasState`
- El generador produce enums de estado para navegar entre pantallas

### Alineación automática
- Botones: centrar horizontal, centrar vertical, distribuir uniformemente
- Líneas guía (guidelines) al arrastrar — snap magnético a otros widgets
- Alineación al borde del canvas

### Edición de nombre de proyecto
- Campo de texto editable en la barra de menú superior
- El nombre se refleja en tiempo real en el código generado

### Selección múltiple
- `Ctrl+Click` para añadir a la selección
- `Drag` sobre zona vacía → rectángulo de selección
- Mover / eliminar varios widgets a la vez

---

## 🟢 Baja prioridad / Ideas futuras

### Temas del canvas
- Tema oscuro (actual), claro, alto contraste
- Personalizar el color de la cuadrícula y el fondo

### Zoom del canvas
- `Ctrl+Scroll` para hacer zoom in/out
- `Ctrl+0` para resetear al 100 %
- El snap y la cuadrícula se adaptan al nivel de zoom

### Grupos de widgets
- Selección múltiple → "Agrupar" (`Ctrl+G`)
- Mover, escalar y eliminar grupos como unidad

### Previsualización de fuente personalizada
- Cargar cualquier `.ttf` para previsualizar el texto real de Labels

### Generación de código avanzada
- Opción de generar estructura multi-archivo (`main.rs` + `ui.rs`)
- Soporte para `AppBuilder` + `Plugin` en lugar de `FerrousApp` directamente
- Generación de layouts responsivos (porcentajes en lugar de píxeles fijos)

### Plantillas de layout predefinidas
- "Login screen", "Settings panel", "Dashboard", etc.
- El usuario las carga como punto de partida

### Historial de proyectos recientes
- Lista en pantalla de inicio de los últimos proyectos abiertos

---

## ✅ Completado (v0.1.0)

- [x] Canvas interactivo con cuadrícula y snap
- [x] 10 tipos de widget con previsualización de color
- [x] Drag & drop de widgets en el canvas
- [x] Hit-test y selección con click
- [x] Panel de propiedades con botones − / +
- [x] Orden de capas (subir / bajar)
- [x] Eliminación de widgets (botón + teclado)
- [x] Tooltips al pasar el cursor
- [x] Generación completa de `main.rs` con imports inteligentes
- [x] Visor de código con scroll y resaltado básico
- [x] Limpieza total del canvas
- [x] Soporte para `on_resize` (reposicionamiento del panel de propiedades)

---

## Notas de versioning

| Versión | Hito principal |
|---|---|
| `0.1.0` | MVP — canvas, 10 widgets, generación de código |
| `0.2.0` | Persistencia — guardar/cargar JSON + copiar al portapapeles |
| `0.3.0` | Ergonomía — deshacer/rehacer + editar labels + selección múltiple |
| `0.4.0` | Multi-pantalla + exportar a archivo |
| `1.0.0` | Feature-complete para uso cotidiano en proyectos FerrousEngine |
