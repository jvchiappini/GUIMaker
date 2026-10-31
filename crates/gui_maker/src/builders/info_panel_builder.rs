use crate::Gui;

pub struct InfoPanelBuilder<'a, 'b, 'c, 'd> {
    gui: &'d mut Gui<'a, 'b, 'c>,
    rect: ferrous_ui_core::Rect,
    bg_color: [f32; 4],
    border_color: [f32; 4],
    accent_color: [f32; 4],
    fields: Vec<(String, String, String, bool)>, 
}

impl<'a, 'b, 'c, 'd> InfoPanelBuilder<'a, 'b, 'c, 'd> {
    pub(crate) fn new(gui: &'d mut Gui<'a, 'b, 'c>) -> Self {
        Self {
            gui,
            rect: ferrous_ui_core::Rect { x: 0.0, y: 0.0, width: 100.0, height: 100.0 },
            bg_color: [0.0, 0.0, 0.0, 1.0],
            border_color: [1.0, 1.0, 1.0, 1.0],
            accent_color: [1.0, 0.0, 0.0, 1.0],
            fields: Vec::new(),
        }
    }

    pub fn rect(mut self, x: f32, y: f32, w: f32, h: f32) -> Self {
        self.rect = ferrous_ui_core::Rect { x, y, width: w, height: h };
        self
    }
    pub fn bg(mut self, color: [f32; 4]) -> Self {
        self.bg_color = color;
        self
    }
    pub fn border(mut self, color: [f32; 4]) -> Self {
        self.border_color = color;
        self
    }
    pub fn accent(mut self, color: [f32; 4]) -> Self {
        self.accent_color = color;
        self
    }
    pub fn add_field(mut self, label: &str, value: &str, unit: &str, is_active: bool) -> Self {
        self.fields.push((label.to_string(), value.to_string(), unit.to_string(), is_active));
        self
    }
    
    pub fn render(self) {
        let x = self.rect.x;
        let y = self.rect.y;
        let w = self.rect.width;
        let h = self.rect.height;
        let alpha = self.bg_color[3];

        self.gui.dc.gui.rect_radii(x, y, w, h, self.bg_color, [10.0; 4]);
        self.gui.dc.gui.rect_radii(x + 8.0, y, w - 16.0, 1.0, self.border_color, [0.0; 4]);

        let num_fields = self.fields.len();
        if num_fields == 0 { return; }
        
        let field_w = w / num_fields as f32;
        
        for (i, (label, value, unit, is_active)) in self.fields.iter().enumerate() {
            let fx = x + (i as f32) * field_w;
            let fy = y;
            
            if *is_active {
                let mut red = self.accent_color;
                red[3] *= alpha;
                self.gui.dc.gui.rect_radii(fx + 4.0, fy + 4.0, field_w - 8.0, h - 8.0, red, [6.0; 4]);
                self.gui.dc.gui.rect_radii(fx + 5.0, fy + 5.0, field_w - 10.0, h - 10.0, self.bg_color, [5.0; 4]);
            }

            let text_y = fy + (h - 12.0) * 0.5;
            let mut label_col = [1.0, 1.0, 1.0, 0.4 * alpha];
            self.gui.dc.gui.draw_text(self.gui.dc.font, label, [fx + 16.0, text_y], 10.0, label_col);

            let val_col = [1.0, 1.0, 1.0, 1.0 * alpha];
            let val_x = fx + field_w - 50.0;
            self.gui.dc.gui.draw_text(self.gui.dc.font, value, [val_x, text_y], 12.0, val_col);

            self.gui.dc.gui.draw_text(self.gui.dc.font, unit, [fx + field_w - 18.0, text_y], 10.0, label_col);
        }
    }
}
