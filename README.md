# gui_maker

Workspace for the Koncreta UI toolkit.

| Path | Role |
|---|---|
| `crates/gui_maker` | **Library** consumed by Koncreta (`packages/core_viewer`). Immediate-mode facade over FerrousEngine. |
| `apps/gui_maker_studio` | Desktop visual UI builder (parked; not used by Koncreta). |

Koncreta depends on the library only:

```toml
gui_maker = { path = ".../gui_maker/crates/gui_maker", features = ["ferrous"] }
```

The library must stay free of desktop-only dependencies (`rfd`, `chrono`,
`winit`, `arboard`). Those belong to `apps/gui_maker_studio`.

→ Library API and examples: [`crates/gui_maker/README.md`](crates/gui_maker/README.md)
→ Visual builder: [`apps/gui_maker_studio/README.md`](apps/gui_maker_studio/README.md)
