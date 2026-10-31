use super::*;

impl SceneState {
    /// Snaps world coordinates to grid and/or other widgets. Returns (snapped_x, snapped_y) and updates active_snap_lines.
    pub fn snap_coordinates(&mut self, active_id: u32, raw_x: f32, raw_y: f32, grid_size: f32) -> (f32, f32) {
        self.active_snap_lines.clear();

        let mut final_x = raw_x;
        let mut final_y = raw_y;

        let t = self.snap_threshold;

        if !self.snap_to_grid {
            return (final_x, final_y);
        }

        let aw = self.widgets.iter().find(|w| w.id == active_id).map(|w| w.w).unwrap_or(0.0);
        let ah = self.widgets.iter().find(|w| w.id == active_id).map(|w| w.h).unwrap_or(0.0);

        let a_left = raw_x - aw * 0.5;
        let a_right = raw_x + aw * 0.5;
        let a_top = raw_y - ah * 0.5;
        let a_bot = raw_y + ah * 0.5;
        let a_cx = raw_x;
        let a_cy = raw_y;

        if grid_size > 0.0 {
            let snapped_x = (raw_x / grid_size).round() * grid_size;
            let snapped_y = (raw_y / grid_size).round() * grid_size;

            if (raw_x - snapped_x).abs() < t { final_x = snapped_x; }
            if (raw_y - snapped_y).abs() < t { final_y = snapped_y; }
        }

        let mut best_dx = t;
        let mut best_dy = t;
        let mut snap_x_val = final_x;
        let mut snap_y_val = final_y;

        let mut lines = Vec::new();

        for w in &self.widgets {
            if w.id == active_id { continue; }

            let b_left = w.x - w.w * 0.5;
            let b_right = w.x + w.w * 0.5;
            let b_top = w.y - w.h * 0.5;
            let b_bot = w.y + w.h * 0.5;
            let b_cx = w.x;
            let b_cy = w.y;

            let x_points: [(f32, f32, f32); 5] = [
                (a_left, b_left, b_left + aw * 0.5),
                (a_left, b_right, b_right + aw * 0.5),
                (a_right, b_left, b_left - aw * 0.5),
                (a_right, b_right, b_right - aw * 0.5),
                (a_cx, b_cx, b_cx),
            ];

            for (a_pt, b_pt, new_cx) in x_points {
                let diff = (a_pt - b_pt).abs();
                if diff < best_dx {
                    best_dx = diff;
                    snap_x_val = new_cx;
                    lines.retain(|l: &SnapLine| !l.is_vertical);
                    lines.push(SnapLine {
                        pos: b_pt,
                        is_vertical: true,
                        length_start: a_cy.min(b_cy) - 100.0,
                        length_end: a_cy.max(b_cy) + 100.0,
                    });
                }
            }

            let y_points: [(f32, f32, f32); 5] = [
                (a_top, b_top, b_top + ah * 0.5),
                (a_top, b_bot, b_bot + ah * 0.5),
                (a_bot, b_top, b_top - ah * 0.5),
                (a_bot, b_bot, b_bot - ah * 0.5),
                (a_cy, b_cy, b_cy),
            ];

            for (a_pt, b_pt, new_cy) in y_points {
                let diff = (a_pt - b_pt).abs();
                if diff < best_dy {
                    best_dy = diff;
                    snap_y_val = new_cy;
                    lines.retain(|l: &SnapLine| l.is_vertical);
                    lines.push(SnapLine {
                        pos: b_pt,
                        is_vertical: false,
                        length_start: a_cx.min(b_cx) - 100.0,
                        length_end: a_cx.max(b_cx) + 100.0,
                    });
                }
            }
        }

        if best_dx < t { final_x = snap_x_val; }
        if best_dy < t { final_y = snap_y_val; }

        self.active_snap_lines = lines;

        (final_x, final_y)
    }
}
