//! Backend boundary: input snapshot + paint primitives.

use crate::layout::{Rect, TextAlign};
use crate::style::Color;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerButton {
    Left,
    Middle,
    Right,
}

impl PointerButton {
    pub const ALL: [Self; 3] = [Self::Left, Self::Middle, Self::Right];

    pub const fn index(self) -> usize {
        match self {
            Self::Left => 0,
            Self::Middle => 1,
            Self::Right => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Modifiers {
    pub alt: bool,
    pub ctrl: bool,
    pub shift: bool,
    pub logo: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Key {
    A, B, C, D, E, F, G, H, I, J, K, L, M, N,
    O, P, Q, R, S, T, U, V, W, X, Y, Z,
    Tab,
    Space,
    Comma,
    Escape,
    Enter,
    Backspace,
    Minus,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
    Home,
    End,
    Delete,
    Digit0,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,
    Period,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    NumpadDecimal,
    NumpadSubtract,
    NumpadEnter,
    ControlLeft,
    ControlRight,
    ShiftLeft,
    ShiftRight,
    AltLeft,
    AltRight,
}

impl Key {
    pub const ALL: &'static [Key] = &[
        Key::A, Key::B, Key::C, Key::D, Key::E, Key::F, Key::G, Key::H, Key::I,
        Key::J, Key::K, Key::L, Key::M, Key::N, Key::O, Key::P, Key::Q, Key::R,
        Key::S, Key::T, Key::U, Key::V, Key::W, Key::X, Key::Y, Key::Z,
        Key::Tab, Key::Space, Key::Comma, Key::Escape, Key::Enter, Key::Backspace,
        Key::Minus, Key::ArrowLeft, Key::ArrowRight, Key::ArrowUp, Key::ArrowDown,
        Key::Home, Key::End, Key::Delete,
        Key::Digit0, Key::Digit1, Key::Digit2, Key::Digit3, Key::Digit4,
        Key::Digit5, Key::Digit6, Key::Digit7, Key::Digit8, Key::Digit9,
        Key::Period, Key::Numpad0, Key::Numpad1, Key::Numpad2, Key::Numpad3,
        Key::Numpad4, Key::Numpad5, Key::Numpad6, Key::Numpad7, Key::Numpad8,
        Key::Numpad9, Key::NumpadDecimal, Key::NumpadSubtract, Key::NumpadEnter,
        Key::ControlLeft, Key::ControlRight, Key::ShiftLeft, Key::ShiftRight,
        Key::AltLeft, Key::AltRight,
    ];
}

#[derive(Clone, Debug)]
pub struct Input {
    pub pointer: [f32; 2],
    pub pointer_delta: [f32; 2],
    pub window_size: [f32; 2],
    pub buttons_down: [bool; 3],
    pub buttons_pressed: [bool; 3],
    pub buttons_released: [bool; 3],
    pub scroll: [f32; 2],
    pub typed_chars: Vec<char>,
    pub keys_down: Vec<Key>,
    pub keys_pressed: Vec<Key>,
    pub modifiers: Modifiers,
    pub time: f64,
    pub dt: f32,
    pub clipboard: String,
}

impl Default for Input {
    fn default() -> Self {
        Self {
            pointer: [0.0; 2],
            pointer_delta: [0.0; 2],
            window_size: [0.0; 2],
            buttons_down: [false; 3],
            buttons_pressed: [false; 3],
            buttons_released: [false; 3],
            scroll: [0.0; 2],
            typed_chars: Vec::new(),
            keys_down: Vec::new(),
            keys_pressed: Vec::new(),
            modifiers: Modifiers::default(),
            time: 0.0,
            dt: 0.0,
            clipboard: String::new(),
        }
    }
}

impl Input {
    pub fn from_backend(backend: &dyn Backend) -> Self {
        let mut buttons_down = [false; 3];
        let mut buttons_pressed = [false; 3];
        let mut buttons_released = [false; 3];
        for button in PointerButton::ALL {
            let index = button.index();
            buttons_down[index] = backend.button_down(button);
            buttons_pressed[index] = backend.button_pressed(button);
            buttons_released[index] = backend.button_released(button);
        }
        Self {
            pointer: backend.pointer_pos(),
            pointer_delta: backend.pointer_delta(),
            window_size: backend.window_size(),
            buttons_down,
            buttons_pressed,
            buttons_released,
            scroll: backend.scroll(),
            typed_chars: backend.typed_chars().to_vec(),
            keys_down: Key::ALL.iter().copied().filter(|key| backend.key_down(*key)).collect(),
            keys_pressed: Key::ALL.iter().copied().filter(|key| backend.key_pressed(*key)).collect(),
            modifiers: backend.modifiers(),
            time: backend.time(),
            dt: backend.delta_time(),
            clipboard: String::new(),
        }
    }

    pub fn pointer_inside(&self, rect: Rect) -> bool {
        rect.contains(self.pointer)
    }

    pub fn button_down(&self, button: PointerButton) -> bool {
        self.buttons_down[button.index()]
    }

    pub fn button_pressed(&self, button: PointerButton) -> bool {
        self.buttons_pressed[button.index()]
    }

    pub fn button_released(&self, button: PointerButton) -> bool {
        self.buttons_released[button.index()]
    }

    pub fn key_down(&self, key: Key) -> bool {
        self.keys_down.contains(&key)
    }

    pub fn key_pressed(&self, key: Key) -> bool {
        self.keys_pressed.contains(&key)
    }

    pub fn ctrl(&self) -> bool { self.modifiers.ctrl || self.key_down(Key::ControlLeft) || self.key_down(Key::ControlRight) }
    pub fn shift(&self) -> bool { self.modifiers.shift || self.key_down(Key::ShiftLeft) || self.key_down(Key::ShiftRight) }
    pub fn alt(&self) -> bool { self.modifiers.alt || self.key_down(Key::AltLeft) || self.key_down(Key::AltRight) }
}

pub trait Backend {
    fn window_size(&self) -> [f32; 2];
    fn pointer_pos(&self) -> [f32; 2];
    fn pointer_delta(&self) -> [f32; 2] { [0.0; 2] }
    fn button_down(&self, button: PointerButton) -> bool;
    fn button_pressed(&self, button: PointerButton) -> bool;
    fn button_released(&self, button: PointerButton) -> bool { let _ = button; false }
    fn scroll(&self) -> [f32; 2] { [0.0; 2] }
    fn typed_chars(&self) -> &[char] { &[] }
    fn key_down(&self, key: Key) -> bool { let _ = key; false }
    fn key_pressed(&self, key: Key) -> bool { let _ = key; false }
    fn keys_down(&self) -> &[Key] { &[] }
    fn keys_pressed(&self) -> &[Key] { &[] }
    fn modifiers(&self) -> Modifiers { Modifiers::default() }
    fn time(&self) -> f64 { 0.0 }
    fn delta_time(&self) -> f32 { 0.0 }

    fn measure_text(&self, text: &str, font_size: f32) -> f32;

    fn rect(&mut self, rect: Rect, color: Color);
    fn rect_radii(&mut self, rect: Rect, color: Color, radii: [f32; 4]) { let _ = radii; self.rect(rect, color); }
    fn border(&mut self, rect: Rect, color: Color, width: f32, radii: [f32; 4]);
    fn shadow(&mut self, rect: Rect, color: Color, blur: f32, spread: f32, offset: [f32; 2], radii: [f32; 4]);
    fn gradient(&mut self, rect: Rect, from: Color, to: Color, vertical: bool, radii: [f32; 4]);
    fn text(&mut self, rect: Rect, text: &str, font_size: f32, color: Color, align: TextAlign);
    fn icon_uv(&mut self, rect: Rect, uv0: [f32; 2], uv1: [f32; 2], tint: Color);
    fn push_clip(&mut self, rect: Rect);
    fn pop_clip(&mut self);
    fn begin_layer(&mut self);

    fn clipboard_write(&mut self, text: &str) { let _ = text; }
    fn clipboard_request_read(&mut self) {}
    fn clipboard_take(&mut self) -> Option<String> { None }
}

#[derive(Clone, Debug, PartialEq)]
pub enum DrawCommand {
    Rect { rect: Rect, color: Color },
    RectRadii { rect: Rect, color: Color, radii: [f32; 4] },
    Border { rect: Rect, color: Color, width: f32, radii: [f32; 4] },
    Shadow { rect: Rect, color: Color, blur: f32, spread: f32, offset: [f32; 2], radii: [f32; 4] },
    Gradient { rect: Rect, from: Color, to: Color, vertical: bool, radii: [f32; 4] },
    Text { rect: Rect, text: String, font_size: f32, color: Color, align: TextAlign },
    Icon { rect: Rect, uv0: [f32; 2], uv1: [f32; 2], tint: Color },
    PushClip { rect: Rect },
    PopClip,
    BeginLayer,
}

#[derive(Clone, Debug)]
pub struct HeadlessBackend {
    pub input: Input,
    pub commands: Vec<DrawCommand>,
    pub clips: Vec<Rect>,
}

impl HeadlessBackend {
    pub fn new() -> Self {
        Self { input: Input::default(), commands: Vec::new(), clips: Vec::new() }
    }

    pub fn with_window(width: f32, height: f32) -> Self {
        let mut backend = Self::new();
        backend.input.window_size = [width, height];
        backend
    }

    pub fn set_pointer(&mut self, x: f32, y: f32) {
        self.input.pointer_delta = [x - self.input.pointer[0], y - self.input.pointer[1]];
        self.input.pointer = [x, y];
    }

    pub fn press(&mut self, button: PointerButton) {
        self.input.buttons_pressed[button.index()] = true;
        self.input.buttons_down[button.index()] = true;
    }

    pub fn release(&mut self, button: PointerButton) {
        self.input.buttons_released[button.index()] = true;
        self.input.buttons_down[button.index()] = false;
    }

    pub fn clear_frame_events(&mut self) {
        self.input.buttons_pressed = [false; 3];
        self.input.buttons_released = [false; 3];
        self.input.keys_pressed.clear();
        self.input.typed_chars.clear();
        self.input.pointer_delta = [0.0; 2];
        self.input.scroll = [0.0; 2];
    }
}

impl Default for HeadlessBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl Backend for HeadlessBackend {
    fn window_size(&self) -> [f32; 2] { self.input.window_size }
    fn pointer_pos(&self) -> [f32; 2] { self.input.pointer }
    fn pointer_delta(&self) -> [f32; 2] { self.input.pointer_delta }
    fn button_down(&self, button: PointerButton) -> bool { self.input.button_down(button) }
    fn button_pressed(&self, button: PointerButton) -> bool { self.input.button_pressed(button) }
    fn button_released(&self, button: PointerButton) -> bool { self.input.button_released(button) }
    fn scroll(&self) -> [f32; 2] { self.input.scroll }
    fn typed_chars(&self) -> &[char] { &self.input.typed_chars }
    fn key_down(&self, key: Key) -> bool { self.input.key_down(key) }
    fn key_pressed(&self, key: Key) -> bool { self.input.key_pressed(key) }
    fn keys_down(&self) -> &[Key] { &self.input.keys_down }
    fn keys_pressed(&self) -> &[Key] { &self.input.keys_pressed }
    fn modifiers(&self) -> Modifiers { self.input.modifiers }
    fn time(&self) -> f64 { self.input.time }
    fn delta_time(&self) -> f32 { self.input.dt }

    fn measure_text(&self, text: &str, font_size: f32) -> f32 {
        text.chars().count() as f32 * font_size * 0.55
    }

    fn rect(&mut self, rect: Rect, color: Color) {
        self.commands.push(DrawCommand::Rect { rect, color });
    }

    fn rect_radii(&mut self, rect: Rect, color: Color, radii: [f32; 4]) {
        self.commands.push(DrawCommand::RectRadii { rect, color, radii });
    }

    fn border(&mut self, rect: Rect, color: Color, width: f32, radii: [f32; 4]) {
        self.commands.push(DrawCommand::Border { rect, color, width, radii });
    }

    fn shadow(&mut self, rect: Rect, color: Color, blur: f32, spread: f32, offset: [f32; 2], radii: [f32; 4]) {
        self.commands.push(DrawCommand::Shadow { rect, color, blur, spread, offset, radii });
    }

    fn gradient(&mut self, rect: Rect, from: Color, to: Color, vertical: bool, radii: [f32; 4]) {
        self.commands.push(DrawCommand::Gradient { rect, from, to, vertical, radii });
    }

    fn text(&mut self, rect: Rect, text: &str, font_size: f32, color: Color, align: TextAlign) {
        self.commands.push(DrawCommand::Text {
            rect,
            text: text.to_string(),
            font_size,
            color,
            align,
        });
    }

    fn icon_uv(&mut self, rect: Rect, uv0: [f32; 2], uv1: [f32; 2], tint: Color) {
        self.commands.push(DrawCommand::Icon { rect, uv0, uv1, tint });
    }

    fn push_clip(&mut self, rect: Rect) {
        self.clips.push(rect);
        self.commands.push(DrawCommand::PushClip { rect });
    }

    fn pop_clip(&mut self) {
        self.clips.pop();
        self.commands.push(DrawCommand::PopClip);
    }

    fn begin_layer(&mut self) {
        self.commands.push(DrawCommand::BeginLayer);
    }

    fn clipboard_write(&mut self, text: &str) {
        self.input.clipboard = text.to_string();
    }

    fn clipboard_take(&mut self) -> Option<String> {
        if self.input.clipboard.is_empty() { None } else { Some(std::mem::take(&mut self.input.clipboard)) }
    }
}
