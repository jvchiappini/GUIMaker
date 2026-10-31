# gui_maker (library)

Immediate-mode GUI facade used by Koncreta's `core_viewer` WASM module.

- `Gui` wraps FerrousEngine's `DrawContext` and exposes the primitives the
  viewer already uses: `toolbar`, `icon`, `icon_uv`, `measure_text`,
  `draw_text_aligned`, `draw_text_centered`, plus the pill-tag and info-panel
  builders.
- Core modules: `backend` (input + paint trait, `HeadlessBackend`), `context`
  (`Ui`, `UiMemory`, `UiOutput`, `WidgetId`), `interact` (`Response`,
  hot/active), `layout` (`Rect`, `col`/`row`/`grid`), `style` (linear tokens),
  `anim`, and the optional `ferrous` backend over `DrawContext`/`GuiBatch`.
- Only `ferrous_engine`, `ferrous_gui`, `ferrous_ui_core` and `ferrous_assets`
  are dependencies, all behind the default `ferrous` feature. Do **not** add
  desktop crates (`rfd`, `chrono`, `winit`, `arboard`) here.

## Frame lifecycle

```rust
let mut ui = gui_maker::Ui::begin(&mut backend, &style, &mut memory);
ui.region(panel_rect);
let response = ui.interact(ui.id("ok"), button_rect, true);
let output = ui.end(); // wants_pointer / wants_keyboard / repaint
```

## Usage

```rust
use gui_maker::Gui;

fn draw(dc: &mut ferrous_engine::DrawContext<'_, '_>) {
    let mut gui = Gui::new(dc);

    gui.toolbar(
        ferrous_ui_core::Rect { x: 0.0, y: 0.0, width: 320.0, height: 40.0 },
        gui.bg_color(),
    );
}
```

`Gui::icon_uv` draws a sub-rectangle of an atlas texture so any number of icons
share one texture slot (the web UI shader has 8 slots).

```rust
gui.icon_uv(&atlas, rect, uv0, uv1, tint);
```

## Koncreta integration

`packages/core_viewer/src/render/gui_backend.rs` is the only file that imports
this crate. Everything else goes through the `GuiRenderer` trait.

```toml
gui_maker = { path = ".../gui_maker/crates/gui_maker", features = ["ferrous"] }
```
