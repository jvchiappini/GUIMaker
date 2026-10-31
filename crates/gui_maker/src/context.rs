//! Frame context, widget memory, layout cursor and interaction state.

use std::collections::{HashMap, HashSet};
use std::fmt;

use crate::anim::AnimState;
use crate::backend::{Backend, Input, PointerButton};
use crate::interact::Response;
use crate::layout::{LayoutState, Rect};
use crate::style::Style;
use crate::widgets::text_edit::TextEditState;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct WidgetId(pub u64);

impl WidgetId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}

impl fmt::Display for WidgetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

const FNV_OFFSET: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

fn hash_bytes(hash: u64, bytes: &[u8]) -> u64 {
    let mut value = hash;
    for byte in bytes {
        value ^= *byte as u64;
        value = value.wrapping_mul(FNV_PRIME);
    }
    value
}

fn hash_str(value: u64, text: &str) -> u64 {
    hash_bytes(value, text.as_bytes())
}

#[derive(Clone, Debug, Default)]
pub struct IdStack {
    hashes: Vec<u64>,
}

impl IdStack {
    pub fn new() -> Self {
        Self { hashes: vec![FNV_OFFSET] }
    }

    pub fn push(&mut self, label: &str) -> u64 {
        let parent = *self.hashes.last().unwrap_or(&FNV_OFFSET);
        let hash = hash_str(hash_str(parent, "/"), label);
        self.hashes.push(hash);
        hash
    }

    pub fn pop(&mut self) {
        if self.hashes.len() > 1 {
            self.hashes.pop();
        }
    }

    pub fn current(&self) -> u64 {
        *self.hashes.last().unwrap_or(&FNV_OFFSET)
    }

    pub fn id(&self, label: &str) -> WidgetId {
        WidgetId(hash_str(hash_str(self.current(), "/"), label))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CursorIcon {
    #[default]
    Default,
    Pointer,
    Text,
    ResizeHorizontal,
    ResizeVertical,
    Crosshair,
    Move,
    NotAllowed,
}

#[derive(Clone, Debug, Default)]
pub struct UiOutput {
    pub wants_pointer: bool,
    pub wants_keyboard: bool,
    pub cursor_icon: CursorIcon,
    pub repaint: bool,
    pub regions: Vec<Rect>,
}

#[derive(Debug, Default)]
pub struct UiMemory {
    pub(crate) hot: Option<WidgetId>,
    pub(crate) active: Option<WidgetId>,
    pub(crate) active_started: f64,
    pub(crate) drag_origin: Option<[f32; 2]>,
    pub(crate) drag_started: bool,
    pub(crate) last_click: Option<(WidgetId, f64, [f32; 2])>,
    pub(crate) regions: Vec<Rect>,
    pub(crate) prev_output: UiOutput,
    pub(crate) animations: HashMap<WidgetId, AnimState>,
    pub(crate) wants_keyboard: bool,
    pub(crate) seen_ids: HashMap<u64, String>,
    pub(crate) warned_ids: HashSet<u64>,
    pub(crate) frame_index: u64,
    pub(crate) focus: Option<WidgetId>,
    pub(crate) focus_rect: Rect,
    pub(crate) focus_seen: bool,
    pub(crate) blur_requested: Option<WidgetId>,
    pub(crate) text_states: HashMap<WidgetId, TextEditState>,
    pub(crate) text_buffers: HashMap<WidgetId, String>,
    pub(crate) open_popup: Option<WidgetId>,
}

impl UiMemory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn last_output(&self) -> &UiOutput {
        &self.prev_output
    }

    pub fn active(&self) -> Option<WidgetId> {
        self.active
    }

    pub fn hot(&self) -> Option<WidgetId> {
        self.hot
    }

    pub fn frame_index(&self) -> u64 {
        self.frame_index
    }

    pub fn focused(&self) -> Option<WidgetId> {
        self.focus
    }

    pub fn has_keyboard_focus(&self) -> bool {
        self.focus.is_some()
    }
}

pub struct Ui<'a> {
    pub(crate) backend: Option<&'a mut dyn Backend>,
    pub(crate) style: &'a Style,
    pub(crate) memory: &'a mut UiMemory,
    pub(crate) input: Input,
    pub(crate) id_stack: IdStack,
    pub(crate) layouts: Vec<LayoutState>,
    pub(crate) wants_keyboard: bool,
    pub(crate) repaint: bool,
    pub(crate) cursor: CursorIcon,
    pub(crate) enabled: bool,
    pub(crate) clip_stack: Vec<Rect>,
}

impl<'a> Ui<'a> {
    pub fn begin(backend: &'a mut dyn Backend, style: &'a Style, memory: &'a mut UiMemory) -> Self {
        let input = Input::from_backend(&*backend);
        Self::from_parts(Some(backend), style, memory, input)
    }

    pub fn begin_regions(style: &'a Style, memory: &'a mut UiMemory, input: Input) -> Self {
        Self::from_parts(None, style, memory, input)
    }

    fn from_parts(
        backend: Option<&'a mut dyn Backend>,
        style: &'a Style,
        memory: &'a mut UiMemory,
        input: Input,
    ) -> Self {
        memory.frame_index = memory.frame_index.wrapping_add(1);
        memory.regions.clear();
        memory.hot = None;
        memory.wants_keyboard = false;
        memory.blur_requested = None;
        memory.focus_seen = false;

        if input.button_pressed(PointerButton::Left) {
            if let Some(focused) = memory.focus {
                if !memory.focus_rect.contains(input.pointer) {
                    memory.blur_requested = Some(focused);
                    memory.focus = None;
                }
            }
        }

        let [width, height] = input.window_size;
        let full = Rect::new(0.0, 0.0, width, height);
        Self {
            backend,
            style,
            memory,
            input,
            id_stack: IdStack::new(),
            layouts: vec![LayoutState::column(full, 0.0)],
            wants_keyboard: false,
            repaint: false,
            cursor: CursorIcon::Default,
            enabled: true,
            clip_stack: Vec::new(),
        }
    }

    pub fn end(self) -> UiOutput {
        let pointer = self.input.pointer;
        let wants_pointer = self.memory.active.is_some()
            || self.memory.hot.is_some()
            || self.memory.regions.iter().any(|region| region.contains(pointer));

        if self.input.button_released(PointerButton::Left) {
            self.memory.active = None;
            self.memory.drag_origin = None;
            self.memory.drag_started = false;
        }
        if self.memory.focus.is_some() && !self.memory.focus_seen {
            self.memory.focus = None;
            self.memory.focus_rect = Rect::ZERO;
        }
        self.memory.blur_requested = None;

        let output = UiOutput {
            wants_pointer,
            wants_keyboard: self.wants_keyboard || self.memory.wants_keyboard,
            cursor_icon: self.cursor,
            repaint: self.repaint || self.animations_running(),
            regions: self.memory.regions.clone(),
        };
        self.memory.prev_output = output.clone();
        self.memory.hot = None;
        output
    }

    fn animations_running(&self) -> bool {
        self.memory
            .animations
            .values()
            .any(|state| (state.value - state.target).abs() > 1e-3)
    }

    pub fn style(&self) -> &Style {
        self.style
    }

    pub fn input(&self) -> &Input {
        &self.input
    }

    pub fn memory(&self) -> &UiMemory {
        self.memory
    }

    pub fn window_size(&self) -> [f32; 2] {
        self.input.window_size
    }

    pub fn pointer(&self) -> [f32; 2] {
        self.input.pointer
    }

    pub fn id(&mut self, label: &str) -> WidgetId {
        let id = self.id_stack.id(label);
        match self.memory.seen_ids.get(&id.0) {
            Some(previous) if previous != label && !self.memory.warned_ids.contains(&id.0) => {
                self.memory.warned_ids.insert(id.0);
                #[cfg(debug_assertions)]
                eprintln!("gui_maker: WidgetId collision between {previous:?} and {label:?}");
            }
            Some(_) => {}
            None => {
                self.memory.seen_ids.insert(id.0, label.to_string());
            }
        }
        id
    }

    pub fn push_id(&mut self, label: &str) {
        self.id_stack.push(label);
    }

    pub fn pop_id(&mut self) {
        self.id_stack.pop();
    }

    pub fn with_id<R>(&mut self, label: &str, f: impl FnOnce(&mut Self) -> R) -> R {
        self.push_id(label);
        let result = f(self);
        self.pop_id();
        result
    }

    pub fn region(&mut self, rect: Rect) {
        if !rect.is_empty() {
            self.memory.regions.push(rect);
        }
    }

    pub fn regions(&self) -> &[Rect] {
        &self.memory.regions
    }

    pub fn set_wants_keyboard(&mut self, wants: bool) {
        self.wants_keyboard = wants;
        self.memory.wants_keyboard = wants;
    }

    pub fn request_repaint(&mut self) {
        self.repaint = true;
    }

    pub fn cursor_icon(&mut self, icon: CursorIcon) {
        self.cursor = icon;
    }

    pub fn col<R>(&mut self, rect: Rect, gap: f32, f: impl FnOnce(&mut Self) -> R) -> R {
        self.layouts.push(LayoutState::column(rect, gap));
        let result = f(self);
        self.layouts.pop();
        result
    }

    pub fn row<R>(&mut self, rect: Rect, gap: f32, f: impl FnOnce(&mut Self) -> R) -> R {
        self.layouts.push(LayoutState::row(rect, gap));
        let result = f(self);
        self.layouts.pop();
        result
    }

    pub fn grid<R>(&mut self, rect: Rect, columns: usize, row_height: f32, gap: f32, f: impl FnOnce(&mut Self) -> R) -> R {
        self.layouts.push(LayoutState::grid(rect, columns, row_height, gap));
        let result = f(self);
        self.layouts.pop();
        result
    }

    pub fn next(&mut self, along: f32) -> Rect {
        self.layouts
            .last_mut()
            .map(|layout| layout.next(along))
            .unwrap_or(Rect::ZERO)
    }

    pub fn allocate_size(&mut self, size: crate::layout::Size) -> Rect {
        let along = match self.layouts.last() {
            Some(layout) => match layout.axis {
                crate::layout::Axis::Row => size.width,
                _ => size.height,
            },
            None => size.height,
        };
        self.next(along)
    }

    pub fn add_space(&mut self, amount: f32) {
        if let Some(layout) = self.layouts.last_mut() {
            layout.add_space(amount);
        }
    }

    pub fn fill(&mut self) -> Rect {
        self.layouts
            .last_mut()
            .map(|layout| layout.fill())
            .unwrap_or(Rect::ZERO)
    }

    pub fn remaining(&self) -> Rect {
        self.layouts
            .last()
            .map(|layout| layout.remaining())
            .unwrap_or(Rect::ZERO)
    }

    pub fn measure_text(&self, text: &str, font_size: f32) -> f32 {
        self.backend
            .as_ref()
            .map(|backend| backend.measure_text(text, font_size))
            .unwrap_or_else(|| text.chars().count() as f32 * font_size * 0.55)
    }

    pub fn backend_mut(&mut self) -> Option<&mut dyn Backend> {
        match self.backend.as_mut() {
            Some(backend) => Some(&mut **backend),
            None => None,
        }
    }

    pub fn rect(&mut self, rect: Rect, color: crate::style::Color) {
        if let Some(backend) = self.backend.as_mut() {
            backend.rect(rect, color);
        }
    }

    pub fn rect_radii(&mut self, rect: Rect, color: crate::style::Color, radii: [f32; 4]) {
        if let Some(backend) = self.backend.as_mut() {
            backend.rect_radii(rect, color, radii);
        }
    }

    pub fn border(&mut self, rect: Rect, color: crate::style::Color, width: f32, radii: [f32; 4]) {
        if let Some(backend) = self.backend.as_mut() {
            backend.border(rect, color, width, radii);
        }
    }

    pub fn shadow(&mut self, rect: Rect, color: crate::style::Color, blur: f32, spread: f32, offset: [f32; 2], radii: [f32; 4]) {
        if let Some(backend) = self.backend.as_mut() {
            backend.shadow(rect, color, blur, spread, offset, radii);
        }
    }

    pub fn gradient(&mut self, rect: Rect, from: crate::style::Color, to: crate::style::Color, vertical: bool, radii: [f32; 4]) {
        if let Some(backend) = self.backend.as_mut() {
            backend.gradient(rect, from, to, vertical, radii);
        }
    }

    pub fn text(&mut self, rect: Rect, text: &str, font_size: f32, color: crate::style::Color, align: crate::layout::TextAlign) {
        if let Some(backend) = self.backend.as_mut() {
            backend.text(rect, text, font_size, color, align);
        }
    }

    pub fn icon_uv(&mut self, rect: Rect, uv0: [f32; 2], uv1: [f32; 2], tint: crate::style::Color) {
        if let Some(backend) = self.backend.as_mut() {
            backend.icon_uv(rect, uv0, uv1, tint);
        }
    }

    pub fn push_clip(&mut self, rect: Rect) {
        let clipped = match self.clip_stack.last() {
            Some(previous) => rect.intersect(*previous),
            None => rect,
        };
        self.clip_stack.push(clipped);
        if let Some(backend) = self.backend.as_mut() {
            backend.push_clip(rect);
        }
    }

    pub fn pop_clip(&mut self) {
        self.clip_stack.pop();
        if let Some(backend) = self.backend.as_mut() {
            backend.pop_clip();
        }
    }

    fn hit_rect(&self, rect: Rect) -> Rect {
        match self.clip_stack.last() {
            Some(clip) => rect.intersect(*clip),
            None => rect,
        }
    }

    pub fn begin_layer(&mut self) {
        if let Some(backend) = self.backend.as_mut() {
            backend.begin_layer();
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn interact(&mut self, id: WidgetId, rect: Rect, enabled: bool) -> Response {
        let enabled = enabled && self.enabled;
        crate::interact::interact(self.memory, &self.input, id, self.hit_rect(rect), enabled)
    }

    pub fn anim_bool(&mut self, id: WidgetId, on: bool, duration: f32) -> f32 {
        self.anim_value(id, if on { 1.0 } else { 0.0 }, duration)
    }

    pub fn anim_value(&mut self, id: WidgetId, target: f32, duration: f32) -> f32 {
        let dt = self.input.dt.max(0.0);
        let state = self
            .memory
            .animations
            .entry(id)
            .or_insert(AnimState::new(target));
        let value = state.advance(target, duration, dt);
        if !state.settled() {
            self.repaint = true;
        }
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_stack_is_stable_and_contextual() {
        let mut stack = IdStack::new();
        let root = stack.id("button");
        stack.push("panel");
        let nested = stack.id("button");
        stack.pop();
        assert_eq!(root, stack.id("button"));
        assert_ne!(root, nested);
    }

    #[test]
    fn regions_drive_wants_pointer() {
        let style = Style::koncreta_dark();
        let mut memory = UiMemory::new();
        let mut input = Input::default();
        input.window_size = [800.0, 600.0];
        input.pointer = [10.0, 10.0];
        let mut ui = Ui::begin_regions(&style, &mut memory, input.clone());
        ui.region(Rect::new(0.0, 0.0, 100.0, 100.0));
        assert!(ui.end().wants_pointer);

        input.pointer = [200.0, 200.0];
        let mut ui = Ui::begin_regions(&style, &mut memory, input);
        ui.region(Rect::new(0.0, 0.0, 100.0, 100.0));
        assert!(!ui.end().wants_pointer);
    }
}
