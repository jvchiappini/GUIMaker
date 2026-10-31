//! FerrousEngine backend: `GuiBatch` paint primitives and `InputState` input.

use std::sync::Arc;

use ferrous_assets::Texture2d;
use ferrous_engine::{DrawContext, KeyCode};

use crate::backend::{Backend, Key, Modifiers, PointerButton};
use crate::layout::{Rect, TextAlign};
use crate::style::Color;

pub struct FerrousBackend<'a, 'b, 'c> {
    pub dc: &'a mut DrawContext<'b, 'c>,
    atlas: Option<Arc<Texture2d>>,
    scale: f32,
}

impl<'a, 'b, 'c> FerrousBackend<'a, 'b, 'c> {
    pub fn new(dc: &'a mut DrawContext<'b, 'c>) -> Self {
        Self { dc, atlas: None, scale: 1.0 }
    }

    pub fn with_scale(dc: &'a mut DrawContext<'b, 'c>, scale: f32) -> Self {
        Self {
            dc,
            atlas: None,
            scale: if scale.is_finite() && scale > 0.0 { scale } else { 1.0 },
        }
    }

    pub fn with_atlas(mut self, atlas: Option<Arc<Texture2d>>) -> Self {
        self.atlas = atlas;
        self
    }

    #[inline]
    fn s(&self, value: f32) -> f32 {
        value * self.scale
    }

    #[inline]
    fn sr(&self, rect: Rect) -> Rect {
        Rect::new(self.s(rect.x), self.s(rect.y), self.s(rect.width), self.s(rect.height))
    }
}

fn fe_button(button: PointerButton) -> ferrous_engine::MouseButton {
    match button {
        PointerButton::Left => ferrous_engine::MouseButton::Left,
        PointerButton::Middle => ferrous_engine::MouseButton::Middle,
        PointerButton::Right => ferrous_engine::MouseButton::Right,
    }
}

fn fe_key(key: Key) -> KeyCode {
    match key {
        Key::A => KeyCode::KeyA, Key::B => KeyCode::KeyB, Key::C => KeyCode::KeyC,
        Key::D => KeyCode::KeyD, Key::E => KeyCode::KeyE, Key::F => KeyCode::KeyF,
        Key::G => KeyCode::KeyG, Key::H => KeyCode::KeyH, Key::I => KeyCode::KeyI,
        Key::J => KeyCode::KeyJ, Key::K => KeyCode::KeyK, Key::L => KeyCode::KeyL,
        Key::M => KeyCode::KeyM, Key::N => KeyCode::KeyN, Key::O => KeyCode::KeyO,
        Key::P => KeyCode::KeyP, Key::Q => KeyCode::KeyQ, Key::R => KeyCode::KeyR,
        Key::S => KeyCode::KeyS, Key::T => KeyCode::KeyT, Key::U => KeyCode::KeyU,
        Key::V => KeyCode::KeyV, Key::W => KeyCode::KeyW, Key::X => KeyCode::KeyX,
        Key::Y => KeyCode::KeyY, Key::Z => KeyCode::KeyZ,
        Key::Tab => KeyCode::Tab, Key::Space => KeyCode::Space, Key::Comma => KeyCode::Comma,
        Key::Escape => KeyCode::Escape, Key::Enter => KeyCode::Enter,
        Key::Backspace => KeyCode::Backspace, Key::Minus => KeyCode::Minus,
        Key::ArrowLeft => KeyCode::ArrowLeft, Key::ArrowRight => KeyCode::ArrowRight,
        Key::ArrowUp => KeyCode::ArrowUp, Key::ArrowDown => KeyCode::ArrowDown,
        Key::Home => KeyCode::Home, Key::End => KeyCode::End, Key::Delete => KeyCode::Delete,
        Key::Digit0 => KeyCode::Digit0, Key::Digit1 => KeyCode::Digit1,
        Key::Digit2 => KeyCode::Digit2, Key::Digit3 => KeyCode::Digit3,
        Key::Digit4 => KeyCode::Digit4, Key::Digit5 => KeyCode::Digit5,
        Key::Digit6 => KeyCode::Digit6, Key::Digit7 => KeyCode::Digit7,
        Key::Digit8 => KeyCode::Digit8, Key::Digit9 => KeyCode::Digit9,
        Key::Period => KeyCode::Period,
        Key::Numpad0 => KeyCode::Numpad0, Key::Numpad1 => KeyCode::Numpad1,
        Key::Numpad2 => KeyCode::Numpad2, Key::Numpad3 => KeyCode::Numpad3,
        Key::Numpad4 => KeyCode::Numpad4, Key::Numpad5 => KeyCode::Numpad5,
        Key::Numpad6 => KeyCode::Numpad6, Key::Numpad7 => KeyCode::Numpad7,
        Key::Numpad8 => KeyCode::Numpad8, Key::Numpad9 => KeyCode::Numpad9,
        Key::NumpadDecimal => KeyCode::NumpadDecimal,
        Key::NumpadSubtract => KeyCode::NumpadSubtract,
        Key::NumpadEnter => KeyCode::NumpadEnter,
        Key::ControlLeft => KeyCode::ControlLeft, Key::ControlRight => KeyCode::ControlRight,
        Key::ShiftLeft => KeyCode::ShiftLeft, Key::ShiftRight => KeyCode::ShiftRight,
        Key::AltLeft => KeyCode::AltLeft, Key::AltRight => KeyCode::AltRight,
    }
}

impl Backend for FerrousBackend<'_, '_, '_> {
    fn window_size(&self) -> [f32; 2] {
        let (w, h) = self.dc.ctx.window_size;
        [w as f32 / self.scale, h as f32 / self.scale]
    }

    fn pointer_pos(&self) -> [f32; 2] {
        let (x, y) = self.dc.ctx.input.mouse_pos_f32();
        [x / self.scale, y / self.scale]
    }

    fn pointer_delta(&self) -> [f32; 2] {
        let (x, y) = self.dc.ctx.input.mouse_delta();
        [x / self.scale, y / self.scale]
    }

    fn button_down(&self, button: PointerButton) -> bool {
        self.dc.ctx.input.is_button_down(fe_button(button))
    }

    fn button_pressed(&self, button: PointerButton) -> bool {
        self.dc.ctx.input.button_just_pressed(fe_button(button))
    }

    fn button_released(&self, button: PointerButton) -> bool {
        self.dc.ctx.input.button_just_released(fe_button(button))
    }

    fn scroll(&self) -> [f32; 2] {
        let (x, y) = self.dc.ctx.input.scroll_delta();
        [x, y]
    }

    fn typed_chars(&self) -> &[char] {
        self.dc.ctx.input.typed_chars()
    }

    fn key_down(&self, key: Key) -> bool {
        self.dc.ctx.input.is_key_pressed(fe_key(key))
    }

    fn key_pressed(&self, key: Key) -> bool {
        self.dc.ctx.input.just_pressed(fe_key(key))
    }

    fn modifiers(&self) -> Modifiers {
        let input = &self.dc.ctx.input;
        Modifiers {
            alt: input.is_key_pressed(KeyCode::AltLeft) || input.is_key_pressed(KeyCode::AltRight),
            ctrl: input.is_key_pressed(KeyCode::ControlLeft) || input.is_key_pressed(KeyCode::ControlRight),
            shift: input.is_key_pressed(KeyCode::ShiftLeft) || input.is_key_pressed(KeyCode::ShiftRight),
            logo: false,
        }
    }

    fn time(&self) -> f64 {
        self.dc.ctx.time.elapsed
    }

    fn delta_time(&self) -> f32 {
        self.dc.ctx.time.delta as f32
    }

    fn measure_text(&self, text: &str, font_size: f32) -> f32 {
        ferrous_gui::GuiBatch::measure_text(self.dc.font, text, self.s(font_size)) / self.scale
    }

    fn rect(&mut self, rect: Rect, color: Color) {
        let r = self.sr(rect);
        self.dc.gui.rect(r.x, r.y, r.width, r.height, color.to_array());
    }

    fn rect_radii(&mut self, rect: Rect, color: Color, radii: [f32; 4]) {
        let r = self.sr(rect);
        let radii = radii.map(|radius| self.s(radius));
        self.dc.gui.rect_radii(r.x, r.y, r.width, r.height, color.to_array(), radii);
    }

    fn border(&mut self, rect: Rect, color: Color, width: f32, radii: [f32; 4]) {
        let r = self.sr(rect);
        let radii = radii.map(|radius| self.s(radius));
        self.dc.gui.border(r.x, r.y, r.width, r.height, color.to_array(), radii, self.s(width));
    }

    fn shadow(&mut self, rect: Rect, color: Color, blur: f32, spread: f32, offset: [f32; 2], radii: [f32; 4]) {
        let r = self.sr(rect);
        let radii = radii.map(|radius| self.s(radius));
        self.dc.gui.shadow(
            r.x,
            r.y,
            r.width,
            r.height,
            color.to_array(),
            self.s(blur),
            self.s(spread),
            [self.s(offset[0]), self.s(offset[1])],
            radii,
        );
    }

    fn gradient(&mut self, rect: Rect, from: Color, to: Color, vertical: bool, _radii: [f32; 4]) {
        const BANDS: usize = 24;
        for i in 0..BANDS {
            let t = i as f32 / BANDS as f32;
            let next = (i + 1) as f32 / BANDS as f32;
            let color = Color::linear(
                from.r + (to.r - from.r) * t,
                from.g + (to.g - from.g) * t,
                from.b + (to.b - from.b) * t,
                from.a + (to.a - from.a) * t,
            );
            let band = if vertical {
                Rect::new(rect.x, rect.y + rect.height * t, rect.width, rect.height * (next - t) + 0.5)
            } else {
                Rect::new(rect.x + rect.width * t, rect.y, rect.width * (next - t) + 0.5, rect.height)
            };
            let r = self.sr(band);
            self.dc.gui.rect(r.x, r.y, r.width, r.height, color.to_array());
        }
    }

    fn text(&mut self, rect: Rect, text: &str, font_size: f32, color: Color, align: TextAlign) {
        let size = self.s(font_size);
        let width = ferrous_gui::GuiBatch::measure_text(self.dc.font, text, size);
        let x = rect.x + align.h.offset(rect.width, width / self.scale);
        let y = rect.y + align.v.offset(rect.height, font_size);
        self.dc.gui.draw_text(
            self.dc.font,
            text,
            [self.s(x), self.s(y)],
            size,
            color.to_array(),
        );
    }

    fn icon_uv(&mut self, rect: Rect, uv0: [f32; 2], uv1: [f32; 2], tint: Color) {
        if let Some(atlas) = self.atlas.clone() {
            let r = self.sr(rect);
            let slot = self.dc.gui.reserve_texture_slot(atlas);
            self.dc.gui.rect_textured(
                r.x, r.y, r.width, r.height,
                tint.to_array(), uv0, uv1, slot,
            );
        }
    }

    fn push_clip(&mut self, rect: Rect) {
        let r = self.sr(rect);
        self.dc.gui.push_clip(ferrous_ui_core::Rect {
            x: r.x,
            y: r.y,
            width: r.width,
            height: r.height,
        });
    }

    fn pop_clip(&mut self) {
        self.dc.gui.pop_clip();
    }

    fn begin_layer(&mut self) {
        self.dc.gui.begin_layer();
    }
}
