use crate::Gui;

pub enum Pivot {
    Center,
    TopLeft,
    BottomLeft,
}

pub struct PillTagBuilder<'a, 'b, 'c, 'd> {
    gui: &'d mut Gui<'a, 'b, 'c>,
    pos: [f32; 2],
    text: String,
    bg_color: [f32; 4],
    text_color: [f32; 4],
    text_size: f32,
    pivot: Pivot,
    offset: [f32; 2],
}

impl<'a, 'b, 'c, 'd> PillTagBuilder<'a, 'b, 'c, 'd> {
    pub(crate) fn new(gui: &'d mut Gui<'a, 'b, 'c>) -> Self {
        Self {
            gui,
            pos: [0.0, 0.0],
            text: String::new(),
            bg_color: [0.0, 0.0, 0.0, 1.0],
            text_color: [1.0, 1.0, 1.0, 1.0],
            text_size: 11.0,
            pivot: Pivot::Center,
            offset: [0.0, 0.0],
        }
    }

    pub fn at(mut self, pos: ferrous_engine::Vec2) -> Self {
        self.pos = pos.into();
        self
    }
    pub fn text(mut self, text: &str) -> Self {
        self.text = text.to_string();
        self
    }
    pub fn text_size(mut self, size: f32) -> Self {
        self.text_size = size;
        self
    }
    pub fn theme_bg(mut self, color: [f32; 4]) -> Self {
        self.bg_color = color;
        self
    }
    pub fn offset(mut self, offset: [f32; 2]) -> Self {
        self.offset = offset;
        self
    }
    pub fn pivot(mut self, pivot: Pivot) -> Self {
        self.pivot = pivot;
        self
    }

    pub fn render(self) {
        let padding_h = 8.0;
        let padding_v = 4.0;
        
        let text_w = self.text.len() as f32 * (self.text_size * 0.6); 
        let w = text_w + padding_h * 2.0;
        let h = self.text_size + padding_v * 2.0;

        let mut rx = self.pos[0] + self.offset[0];
        let mut ry = self.pos[1] + self.offset[1];

        match self.pivot {
            Pivot::Center => {
                rx -= w * 0.5;
                ry -= h * 0.5;
            }
            Pivot::BottomLeft => {
                ry -= h;
            }
            Pivot::TopLeft => {}
        }

        self.gui.dc.gui.rect_radii(rx, ry, w, h, self.bg_color, [h * 0.5; 4]);
        self.gui.dc.gui.draw_text(self.gui.dc.font, &self.text, [rx + padding_h, ry + padding_v], self.text_size, self.text_color);
    }
}
