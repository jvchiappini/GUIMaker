//! # gui_maker
//!
//! Immediate-mode GUI facade used by Koncreta. This crate intentionally stays
//! small: it only wraps FerrousEngine's `DrawContext` with the primitives the
//! viewer needs. The visual builder application lives in
//! `apps/gui_maker_studio`.
//!
//! The FerrousEngine-backed implementation is behind the `ferrous` feature
//! (enabled by default); future backends will be added alongside it.

pub mod anim;
pub mod backend;
pub mod context;
pub mod interact;
pub mod layout;
pub mod style;
pub mod widgets;

#[cfg(feature = "ferrous")]
pub mod builders;
#[cfg(feature = "ferrous")]
pub mod ferrous;

#[cfg(feature = "ferrous")]
use ferrous_engine::DrawContext;

pub use backend::{Backend, DrawCommand, HeadlessBackend, Input, Key, Modifiers, PointerButton};
pub use context::{CursorIcon, IdStack, Ui, UiMemory, UiOutput, WidgetId};
pub use interact::Response;
pub use layout::{Align, Rect, Size, TextAlign};
pub use style::{Color, ControlHeights, Palette, Radius, Spacing, Style, TypeScale};
pub use widgets::text_edit::{TextEditState, TextEvent, TextFilter};
pub use widgets::ButtonKind;

// ── Intermediate Mode GUI API ────────────────────────────────────────────────
#[cfg(feature = "ferrous")]
pub struct Gui<'a, 'b, 'c> {
    pub dc: &'a mut ferrous_engine::DrawContext<'b, 'c>,
}

#[cfg(feature = "ferrous")]
impl<'a, 'b, 'c> Gui<'a, 'b, 'c> {
    pub fn new(dc: &'a mut DrawContext<'b, 'c>) -> Self {
        Self { dc }
    }

    pub fn draw_pill_tag<'d>(&'d mut self) -> builders::PillTagBuilder<'a, 'b, 'c, 'd> {
        builders::PillTagBuilder::new(self)
    }

    pub fn draw_info_panel<'d>(&'d mut self) -> builders::InfoPanelBuilder<'a, 'b, 'c, 'd> {
        builders::InfoPanelBuilder::new(self)
    }

    pub fn is_hovering(&self, x: f32, y: f32, w: f32, h: f32) -> bool {
        let (mx, my) = self.dc.ctx.input.mouse_pos_f32();
        mx >= x && mx <= x + w && my >= y && my <= y + h
    }

    pub fn bg_color(&self) -> [f32; 4] { [0.07, 0.07, 0.07, 1.0] }
    pub fn border_color(&self) -> [f32; 4] { [1.0, 1.0, 1.0, 0.08] }
    pub fn accent_color(&self) -> [f32; 4] { [0.2, 0.4, 0.8, 1.0] }

    pub fn toolbar(&mut self, rect: ferrous_ui_core::Rect, color: [f32; 4]) {
        // Main toolbar background with rounded corners
        self.dc.gui.rect_radii(
            rect.x,
            rect.y,
            rect.width,
            rect.height,
            color,
            [12.0; 4]
        );

        // Use the built-in shadow method for a better look
        self.dc.gui.shadow(
            rect.x,
            rect.y,
            rect.width,
            rect.height,
            [0.0, 0.0, 0.0, 0.2],
            8.0,
            0.0,
            [0.0, 4.0],
            [12.0; 4]
        );
    }

    pub fn icon(&mut self, texture: &std::sync::Arc<ferrous_assets::Texture2d>, rect: ferrous_ui_core::Rect, tint: [f32; 4]) {
        self.dc.gui.image(
            rect.x,
            rect.y,
            rect.width,
            rect.height,
            texture.clone(),
            [0.0, 0.0],
            [1.0, 1.0],
            tint
        );
    }

    /// Draws a sub-rectangle `[uv0, uv1]` of `texture` (e.g. one icon of an
    /// icon atlas). Many icons sharing one atlas use a single texture slot,
    /// which matters on WebGL/WebGPU where the UI shader has 8 slots.
    pub fn icon_uv(
        &mut self,
        texture: &std::sync::Arc<ferrous_assets::Texture2d>,
        rect: ferrous_ui_core::Rect,
        uv0: [f32; 2],
        uv1: [f32; 2],
        tint: [f32; 4],
    ) {
        self.dc.gui.image(rect.x, rect.y, rect.width, rect.height, texture.clone(), uv0, uv1, tint);
    }

    pub fn measure_text(&self, text: &str, size: f32) -> f32 {
        ferrous_gui::GuiBatch::measure_text(self.dc.font, text, size)
    }

    pub fn draw_text_aligned(&mut self, text: &str, rect: ferrous_ui_core::Rect, align: ferrous_ui_core::TextAlign, size: f32, color: [f32; 4]) {
        let tw = self.measure_text(text, size);
        let th = size;
        let tx = align.resolve_x(rect.x, rect.width, tw, 0.0);
        let ty = align.resolve_y(rect.y, rect.height, th, 0.0);
        self.dc.gui.draw_text(self.dc.font, text, [tx, ty], size, color);
    }

    pub fn draw_text_centered(&mut self, text: &str, rect: ferrous_ui_core::Rect, size: f32, color: [f32; 4]) {
        self.draw_text_aligned(text, rect, ferrous_ui_core::TextAlign::CENTER, size, color);
    }
}
