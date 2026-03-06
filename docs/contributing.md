# Guía para Contribuir

¡Gracias por querer mejorar GUIMaker! Este documento explica cómo está organizado el código, las convenciones que seguimos y los pasos para enviar cambios.

---

## Antes de empezar

1. Lee [`docs/architecture.md`](architecture.md) para entender cómo encajan los módulos.
2. Revisa el [`docs/roadmap.md`](roadmap.md) para ver qué está planificado y evitar trabajo duplicado.
3. Abre un **issue** antes de una PR grande para discutir el enfoque.

---

## Setup del entorno

```bash
# 1. Clona el repo
git clone https://github.com/tu-usuario/gui-maker
cd gui-maker

# 2. Asegúrate de tener FerrousEngine en la ruta correcta
# (../FerrousEngine relativo a este proyecto)

# 3. Compila para verificar que todo está en orden
cargo build

# 4. Ejecuta la app
cargo run
```

### Requisitos

- Rust 1.75+ (edición 2021)
- `rustfmt` — para formatear código (`rustup component add rustfmt`)
- `clippy` — para análisis estático (`rustup component add clippy`)

---

## Convenciones de código

### Formato

```bash
cargo fmt
```

Siempre formatea antes de hacer commit. La configuración por defecto de `rustfmt` es suficiente; no hay `rustfmt.toml` personalizado.

### Lint

```bash
cargo clippy -- -D warnings
```

El código no debe generar warnings de Clippy. Si hay un warning intencionalmente ignorado, documenta el motivo con `#[allow(...)]` y un comentario.

### Nomenclatura

| Elemento | Convención | Ejemplo |
|---|---|---|
| Tipos / Structs | `PascalCase` | `WidgetNode`, `CanvasState` |
| Funciones / métodos | `snake_case` | `hit_test`, `add_widget` |
| Constantes | `SCREAMING_SNAKE_CASE` | `TOOLBOX_W`, `SNAP` |
| Módulos | `snake_case` | `model`, `codegen` |
| Campos de UI en `main.rs` | `snake_case` con sufijo descriptivo | `btn_generate`, `code_scroll` |

### Comentarios

- Los módulos tienen un comentario de módulo `//!` en la primera línea (o se documenta en `architecture.md`).
- Las funciones públicas tienen doc-comment `///`.
- Los bloques de lógica no trivial tienen un comentario `// ── Descripción ──` al estilo del código existente.

---

## Cómo añadir un nuevo widget

Este es el caso de contribución más común. Sigue estos pasos en orden:

### 1. `src/model.rs`

Añade la variante al enum y todos sus métodos:

```rust
// En WidgetKind:
NuevoWidget,

// En display_name():
WidgetKind::NuevoWidget => "NuevoWidget",

// En preview_color():
WidgetKind::NuevoWidget => [r, g, b, 1.0],

// En default_size():
WidgetKind::NuevoWidget => (200.0, 36.0),

// En all():
WidgetKind::NuevoWidget,  // al final del vec!
```

### 2. `src/codegen.rs`

```rust
// En widget_type_name():
WidgetKind::NuevoWidget => Some("NuevoWidget"),

// En el bloque de imports:
let has_nuevo = state.widgets.iter().any(|w| w.kind == WidgetKind::NuevoWidget);
// ... añadir "NuevoWidget" a gui_imports si has_nuevo

// En el bloque Default:
WidgetKind::NuevoWidget => {
    out.push_str(&format!(
        "            {}: NuevoWidget::new({:.1}, {:.1}, {:.1}, {:.1}),\n",
        w.var_name, w.x, w.y, w.width, w.height
    ));
}

// En draw_ui (ya cubierto por el bucle genérico si widget_type_name retorna Some)
```

### 3. Verifica

- Abre la app con `cargo run`
- Comprueba que el nuevo widget aparece en la toolbox
- Coloca un widget, pulsa "Generar Código"
- Copia el código generado en un proyecto FerrousEngine de prueba y verifica que compila

No se necesitan cambios en `toolbox.rs` ni en `properties.rs` — leen `WidgetKind::all()` automáticamente.

---

## Flujo de trabajo con Git

```bash
# Crea una rama descriptiva
git checkout -b feat/add-toggle-widget
# o
git checkout -b fix/snap-off-by-one

# Haz tus cambios, luego:
cargo fmt
cargo clippy -- -D warnings
cargo build    # asegurarse que compila

git add .
git commit -m "feat: add Toggle widget to WidgetKind and codegen"

git push origin feat/add-toggle-widget
# → Abre una Pull Request en GitHub
```

### Prefijos de commit

| Prefijo | Uso |
|---|---|
| `feat:` | Nueva funcionalidad |
| `fix:` | Corrección de bug |
| `refactor:` | Reestructuración sin cambio de comportamiento |
| `docs:` | Solo cambios en documentación |
| `style:` | Formato, nombres, sin cambios lógicos |
| `chore:` | Tareas de mantenimiento (Cargo.toml, CI, etc.) |

---

## Pull Request checklist

Antes de enviar una PR, verifica:

- [ ] `cargo build` sin errores
- [ ] `cargo clippy -- -D warnings` sin warnings
- [ ] `cargo fmt` aplicado
- [ ] La funcionalidad fue probada manualmente en la app
- [ ] La documentación relevante en `docs/` fue actualizada (si aplica)
- [ ] `docs/roadmap.md` actualizado si la PR completa un ítem del roadmap

---

## Reportar bugs

Abre un issue con:

1. **Descripción** — qué ocurre vs. qué debería ocurrir
2. **Pasos para reproducir** — lo más concreto posible
3. **Entorno** — OS, versión de Rust, versión de FerrousEngine
4. **Logs o capturas** — si aplica

---

## Preguntas

Abre un issue con la etiqueta `question` o discútelo en las Discussions del repo.
