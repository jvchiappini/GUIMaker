# Mejoras sugeridas para GUI Maker (layout responsivo y totalmente personalizable)

<!--
Archivo: GUI_MAKER_MEJORAS_RESPONSIVE.md
Descripción: colección de sugerencias, requisitos y caminos de mejora para la plataforma.
-->

## 1. Fundamentos de layout (must have)

- [ ] Modelo de layout principal unificado
  - Agregar soporte nativo para:
    - Grid (lineas/columnas, celdas, `fr`, `auto`, `minmax`)
    - Flexbox (row, column, wrap, gap, align-items, justify-content)
    - Absolute/relative (top/left/right/bottom, z-index)
  - Evitar estructuras blancas/locked que rompen con contenedores anidados.

- [ ] Soporte de unidades mixtas
  - `px`, `%`, `vw`, `vh`, `em`, `rem`, `fr`.
  - Validador en UI y runtime: no permitas valores inválidos.

- [ ] Layout basado en constraints/cadenas
  - Restricciones de anchura/altura mínimas/máximas y proporciones (aspect-ratio).
  - Sistema de compatibilidad de contenedores (overflow, clamp, snap).

- [ ]Magnitud de breakpoint adaptivo
  - Animar/mostrar en tiempo real 0..4 breakpoints con preview.
  - Propiedades por breakpoint (ej: `behavior@sm` / `layout@md`).

- [ ] Vista previa mutidispositivo
  - Lienzo con 3 vistas simultaneas: móvil, tablet, desktop.
  - Cambios visuales transicionables y toggle up/down.

## 2. Motor de estilo y temas

- [ ] Sistema de temas global + overrides locales
  - Variables CSS-like (`--accent`, `--bg`, `--radius`).
  - Scoped theming: sobrescribir dentro de contenedor.

- [ ] Diseño con tokens y prácticas de design system
  - `spacing-1..8`, tipografías, estados interactivos, elevación
  - Paleta de colores accesibles y validación de contraste.

- [ ] Editor de estilo especializado
  - Inserción de reglas expresivas (hover, active, focus), pseudo-selectores.
  - Propiedades condicionales (en base a estado, device, auth).

## 3. Experiencia de usuario del editor

- [ ] Interfaz de arrastrar soltar robusta
  - Rutas de drop aclaras con placeholder indicativo.
  - Auto-scroll de canvas durante arrastre.

- [ ] Interacción con nodos y modos (move/resize/rotate)
  - Modos combinables como Figma (move, resize edges/corners, scale).
  - Precisión con snap a columnas/rows y guías de grid.

- [ ] Paleta de componentes + plantillas
  - Soporte para colocar componentes personalizados (reusable) con props.
  - Guardar ‘sections’ complejas como componentes a reusar.

- [ ] Inspección y edición avanzada de props
  - Edición inline, junto a `property panel` (name/value/type).
  - Cambios en tiempo real con undo/redo.

- [ ] Operaciones en lote y bulk editing
  - Cambiar múltiples elementos seleccionados.
  - Copiar/pegar estilo y layout entre widgets.

## 4. Interoperabilidad y exportación

- [ ] Generación de código limpio y responsive
  - Salida compatible con frameworks (React, Vue, HTML/CSS puro).
  - Generar `media queries` con breakpoints detectados.

- [ ] Importación desde diseño estándar
  - Importar de JSON/DSL y reconstruir layout.
  - Genere historia de cambios por versión.

- [ ] Runtime optimizado
  - Calcular reflow incremental, no recalcular todo.
  - Hacer culling/virtualization de widgets en canvas grande.

## 5. Animación y transiciones

- [ ] Transiciones anidadas (estado, tema, device)
  - `transition` de movimiento, opacity, escala.
  - Modo “preview live” + keyframes simples.

- [ ] Interacción con lógica
  - Definir triggers y eventos (click, hover, time)
  - Poder exportar pseudo-código para lógica (state machine).

## 6. Accesibilidad

- [ ] Soporte WCAG 2.1
  - Contrast checker y auditoría con requerimientos mínimos.
  - Roles ARIA, foco keyboard y teclado total.

- [ ] Testing integrado de la accesibilidad
  - Reporte de problemas y recomendaciones (colores, tamaño, ALT).

## 7. Rendimiento

- [ ] Re-render selectivo
  - Solo calcula layout de ramas afectadas.
  - Evitar `O(n)` en cada frame con capas de caché.

- [ ] Carga perezosa de assets del editor
  - Texto, iconos y paletas cargadas cuando se usan.

- [ ] Guardado incremental / autosave
  - Serialización eficiente en background.

## 8. IA / asistente de mejoras

- [ ] Recomendador de layout adaptativo
  - Sugerir cambio de grid/flex según contenido.
  - “Optimize for mobile” one-click.

- [ ] Autocompletado de propiedades y snippets
  - Propiedad intuitiva basada en tipo de widget.

- [ ] Génesis de diseños a partir de prompt
  - Transformar idea textual a wireframe inicial.

## 9. Calidad de código y dev ergonomics

- [x] Estrategia de testing
  - Unit tests para motor de layout (calculado con casos extremos).
  - E2E y snapshots para UI actua.

- [ ] Documentación viva
  - `README` + `CHANGELOG` + guía de estilo.
  - `COMPONENT_GUIDE.md` para nuevo widget.

- [ ] Extensibilidad del sistema de plugins
  - Hooks desde layout engine (`onLayoutChanged`), nuevos widgets.

## 10. Roadmap Colombiano


---

## Cómo aplicar rápido (minimo viable)

1. En `scene.rs`/`layout_props.rs`, migrar de `single-value` a enum:
  - `Absolute(f32)`, `Percent(f32)`, `Fraction(f32)`, `Auto`, `MinMax(...)`. ✅ Hecho
2. En `panels/widgets/*`, añadir visual de contenedor con snap.
  - `cargo test` pasó con éxito (2 tests: roundtrip y parse).
  - `cargo test` pasó con éxito (unit tests añadidos para LayoutValue).


> Nota: si el documento es muy largo, está preparado para ser base de un epic de roadmap de producto.
## 11. Mejora intensa: características extra para ultra responsive

- [ ] `Smart Container` (auto-layout):
  - Detectar si es `row` o `column` según contenido.
  - Auto-wrap y shrink-follow: reduce elementos proporcionalmente cuando no cabe.
  - Config: `snapIncrement`, `gutter`, `maxChildren`, `minChildWidth`.

- [ ] `Responsive rules engine`:
  - `if width<=480 then set w=100%`, `if height>=900 then set maxHeight=92vh`.
  - Permitir reglas encadenadas `desktop->tablet->mobile` con overrides.
  - Soporte `fallback` por si no hay media query válido.

- [ ] `Dynamic content flow`:
  - IA / heurística para reorganizar contenido según orientacion (portrait/landscape).
  - Ejemplo: “Swap sidebar to bottom if viewport < 820px”.

- [ ] `Layout presets + responsive presets`:
  - Pestañas: `Desktop`, `Tablet`, `Mobile`, `Custom`.
  - Guardar preseteos (ej. 3-up cards, 2-col, one-line) en librería.

- [ ] `Visual snap guides` mejoradas:
  - Guías dinámicas con distancia en px/%.
  - Grids “smart” con arrastrar para ajustar columnas.

- [ ] `Auto contrast and font resize`:
  - Detecta cada widget y ajusta fuente a `clamp(min,max,calc(...))`.
  - Ajusta color de texto en base a fondo: black/white automático.

## 12. Mejora masiva: configurabilidad y extensión de propiedades

- [ ] `Property schema` en JSON:
  - Cada widget puede declarar `fields` con `type`, `validation`, `responsiveSupport`.
  - Ejemplo: `"padding": {"type":"spacing", "units": ["px", "%", "em"], "responsive": true}`.

- [ ] `Custom property editors`:
  - Permitir registrar editores a plugins (color picker, gradient picker, icon picker, data binding selector).
  - `layout-prop: { component: "range-map", min:0,max:100, step:1 }`.

- [ ] “inherit layout” entre nodos:
  - Parent puede dictar `layout-type` con `children.inherit = true`.
  - Soporta override manual, “lock/unlock” en nodo.

- [ ] `State-based layout`: estilos por estado runtime.
  - Widgets pueden tener `layoutStates: ["default","hover","focused","active"]`.
  - Cambiar de columna a fila según estado.

- [ ] `Layout tokens y variables globales` en GUI Maker.
  - Definir variables reutilizables: `--grid-gap-1`, `--safe-area-margin`, `--container-max-width`.
  - Sistema de herencia de variables entre `theme` y componente.

## 13. Workflow de diseño y despliegue responsive

- [ ] “Audit mode” integrado:
  - Analizar área visible / scroll / layout breaks cada 100px.
  - Recomendaciones: elemento demasiado alto, overflow hidden, no responsive.

- [ ] Modo “Diff” entre versiones:
  - Comparar dos estados de diseño y poner cambios en rojo/verde.
  - Ideal para iterar con equipos.

- [ ] Generador de tests visuales:
  - Captura snapshots en 4 resoluciones.
  - Testear conteo de containers, valores de layout y anchura.

- [ ] Exportar config JSON “responsive-ready”: esquema de `layout`, `breakpoints`, `style`.
- [ ] Importar estilo de Tailwind o Bootstrap como macros / presets.

## 14. Documentación para el equipo

- [ ] Ejemplos reales:
  - Card responsive con media queries.
  - Dashboard de datos con contenedor 12-column y column wrap.
  - Storybook (o sección “catalogo”) con patrones de layout.

- [ ] Checklist para revisiones:
  1. ¿Se usan `min/max`?
  2. ¿Funciona en 360px, 768px, 1200px?
  3. ¿Hay fallback para no-growing?
  4. ¿Cumple con WCAG y texto legible?

---
