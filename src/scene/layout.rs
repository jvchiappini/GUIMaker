// Modular scene layout engine (no god file). 

use super::*;

impl SceneState {
    /// Resolves layouts for all widgets based on their constraints and parent size.
    pub fn resolve_all_layouts(&mut self, preview_w: f32, preview_h: f32) {
        if self.widgets.is_empty() { return; }

        let root_rect = [-preview_w/2.0, -preview_h/2.0, preview_w, preview_h];

        // Build hierarchy sorted list of IDs to ensure parents are resolved before children
        let mut sorted_ids = Vec::new();
        let mut to_process: Vec<(u32, u32)> = self.widgets.iter().map(|w| (w.id, w.parent_id.unwrap_or(0))).collect();

        let mut depth = 0;
        while !to_process.is_empty() && depth < 20 {
            let depth_ids: Vec<u32> = to_process.iter()
                .filter(|(_, pid)| *pid == 0 || sorted_ids.contains(pid))
                .map(|(id, _)| *id)
                .collect();

            if depth_ids.is_empty() { break; }
            sorted_ids.extend(depth_ids.iter().cloned());
            to_process.retain(|(id, _)| !depth_ids.contains(id));
            depth += 1;
        }

        for id in &sorted_ids {
            self.apply_layout_to_widget(*id, root_rect);
        }
        for id in &sorted_ids {
            self.apply_auto_layout_to_container(*id);
        }
    }

    fn apply_auto_layout_to_container(&mut self, container_id: u32) {
        let idx = match self.widgets.iter().position(|w| w.id == container_id) {
            Some(i) => i,
            None => return,
        };
        let container = &self.widgets[idx];
        if container.kind != WidgetKind::Panel || !container.props.auto_layout { return; }

        let auto_layout_direction = container.props.auto_layout_direction;
        let container_y = container.y;
        let container_x = container.x;
        let container_w = container.w;
        let container_h = container.h;
        let child_left = container_x - container_w * 0.5 + container.props.padding[3].resolve(container_h);
        let child_top = container_y - container_h * 0.5 + container.props.padding[0].resolve(container_h);
        let auto_layout_gap = container.props.auto_layout_gap;

        let _ = container;

        let child_ids: Vec<u32> = self.widgets.iter().filter(|w| w.parent_id == Some(container_id)).map(|w| w.id).collect();
        if child_ids.is_empty() { return; }

        let mut cursor_x = child_left;
        let mut cursor_y = child_top;

        for child_id in child_ids {
            if let Some(child) = self.widgets.iter_mut().find(|w| w.id == child_id) {
                match auto_layout_direction {
                    AutoLayoutDirection::Horizontal => {
                        child.x = cursor_x + child.w * 0.5;
                        child.y = container_y;
                        cursor_x += child.w + auto_layout_gap;
                    }
                    AutoLayoutDirection::Vertical => {
                        child.y = cursor_y + child.h * 0.5;
                        child.x = container_x;
                        cursor_y += child.h + auto_layout_gap;
                    }
                }
            }
        }
    }

    fn apply_layout_to_widget(&mut self, id: u32, root_rect: [f32; 4]) {
        let (parent_id, anchor_top, anchor_right, anchor_bottom, anchor_left, min_w, min_h, max_w, max_h) = {
            let w = self.widgets.iter().find(|w| w.id == id).unwrap();
            (w.parent_id,
             w.props.anchor_top.clone(),
             w.props.anchor_right.clone(),
             w.props.anchor_bottom.clone(),
             w.props.anchor_left.clone(),
             w.props.min_width,
             w.props.min_height,
             w.props.max_width,
             w.props.max_height)
        };

        let parent_rect = if let Some(pid) = parent_id {
            if let Some(pw) = self.widgets.iter().find(|w| w.id == pid) {
                let pad_top = pw.props.padding[0].resolve(pw.h);
                let pad_right = pw.props.padding[1].resolve(pw.w);
                let pad_bottom = pw.props.padding[2].resolve(pw.h);
                let pad_left = pw.props.padding[3].resolve(pw.w);
                [pw.x - pw.w/2.0 + pad_left, pw.y - pw.h/2.0 + pad_top, pw.w - pad_right - pad_left, pw.h - pad_top - pad_bottom]
            } else {
                root_rect
            }
        } else {
            root_rect
        };

        if let Some(w) = self.widgets.iter_mut().find(|w| w.id == id) {
            let left_edge = parent_rect[0];
            let right_edge = parent_rect[0] + parent_rect[2];
            let top_edge = parent_rect[1];
            let bottom_edge = parent_rect[1] + parent_rect[3];

            let mut left = w.x - w.w/2.0;
            let mut right = w.x + w.w/2.0;
            let mut top = w.y - w.h/2.0;
            let mut bottom = w.y + w.h/2.0;

            if let Some(at) = &anchor_top {
                if !matches!(at, LayoutValue::Auto) {
                    top = top_edge + at.resolve(parent_rect[3]);
                    if anchor_bottom.is_none() { bottom = top + w.h; }
                }
            }
            if let Some(ab) = &anchor_bottom {
                if !matches!(ab, LayoutValue::Auto) {
                    bottom = bottom_edge - ab.resolve(parent_rect[3]);
                    if anchor_top.is_none() { top = bottom - w.h; }
                }
            }
            if let Some(al) = &anchor_left {
                if !matches!(al, LayoutValue::Auto) {
                    left = left_edge + al.resolve(parent_rect[2]);
                    if anchor_right.is_none() { right = left + w.w; }
                }
            }
            if let Some(ar) = &anchor_right {
                if !matches!(ar, LayoutValue::Auto) {
                    right = right_edge - ar.resolve(parent_rect[2]);
                    if anchor_left.is_none() { left = right - w.w; }
                }
            }

            let mut fw = right - left;
            let mut fh = bottom - top;
            if let Some(mw) = min_w { fw = fw.max(mw); }
            if let Some(mh) = min_h { fh = fh.max(mh); }
            if let Some(mw) = max_w { fw = fw.min(mw); }
            if let Some(mh) = max_h { fh = fh.min(mh); }

            if anchor_left.is_some() && anchor_right.is_none() { right = left + fw; }
            else if anchor_right.is_some() && anchor_left.is_none() { left = right - fw; }
            else { right = left + fw; }

            if anchor_top.is_some() && anchor_bottom.is_none() { bottom = top + fh; }
            else if anchor_bottom.is_some() && anchor_top.is_none() { top = bottom - fh; }
            else { bottom = top + fh; }

            w.w = right - left;
            w.h = bottom - top;
            w.x = left + w.w/2.0;
            w.y = top + w.h/2.0;
        }
    }

    /// Synchronizes active anchors when a widget is manually moved or resized.
    pub fn sync_anchors(&mut self, id: u32, preview_w: f32, preview_h: f32) {
        let root_rect = [-preview_w/2.0, -preview_h/2.0, preview_w, preview_h];

        let parent_id = self.widgets.iter().find(|w| w.id == id).and_then(|w| w.parent_id);
        let parent_rect = if let Some(pid) = parent_id {
            if let Some(pw) = self.widgets.iter().find(|w| w.id == pid) {
                let pad_top = pw.props.padding[0].resolve(pw.h);
                let pad_right = pw.props.padding[1].resolve(pw.w);
                let pad_bottom = pw.props.padding[2].resolve(pw.h);
                let pad_left = pw.props.padding[3].resolve(pw.w);
                [pw.x - pw.w/2.0 + pad_left, pw.y - pw.h/2.0 + pad_top, pw.w - pad_right - pad_left, pw.h - pad_top - pad_bottom]
            } else {
                root_rect
            }
        } else {
            root_rect
        };

        if let Some(w) = self.widgets.iter_mut().find(|w| w.id == id) {
            let left_edge = parent_rect[0];
            let right_edge = parent_rect[0] + parent_rect[2];
            let top_edge = parent_rect[1];
            let bottom_edge = parent_rect[1] + parent_rect[3];

            if w.props.anchor_top.is_some() {
                let desired = w.y - w.h/2.0 - top_edge;
                let at = w.props.anchor_top.take().unwrap();
                w.props.anchor_top = Some(match at {
                    LayoutValue::Absolute(_) => LayoutValue::Absolute(desired),
                    LayoutValue::Percent(_) => LayoutValue::Percent((desired / parent_rect[3]) * 100.0),
                    LayoutValue::Fraction(_) => LayoutValue::Fraction(desired / parent_rect[3]),
                    LayoutValue::MinMax { min, max } => {
                        let minv = min.resolve(parent_rect[3]);
                        let maxv = max.resolve(parent_rect[3]);
                        let clamped = desired.max(minv).min(maxv);
                        LayoutValue::Absolute(clamped)
                    }
                    LayoutValue::Auto => LayoutValue::Auto,
                });
            }
            if let Some(ar) = &mut w.props.anchor_right {
                let desired = right_edge - (w.x + w.w/2.0);
                let ar_val = w.props.anchor_right.take().unwrap();
                w.props.anchor_right = Some(match ar_val {
                    LayoutValue::Absolute(_) => LayoutValue::Absolute(desired),
                    LayoutValue::Percent(_) => LayoutValue::Percent((desired / parent_rect[2]) * 100.0),
                    LayoutValue::Fraction(_) => LayoutValue::Fraction(desired / parent_rect[2]),
                    LayoutValue::MinMax { min, max } => {
                        let minv = min.resolve(parent_rect[2]);
                        let maxv = max.resolve(parent_rect[2]);
                        let clamped = desired.max(minv).min(maxv);
                        LayoutValue::Absolute(clamped)
                    }
                    LayoutValue::Auto => LayoutValue::Auto,
                });
            }
            if w.props.anchor_bottom.is_some() {
                let desired = bottom_edge - (w.y + w.h/2.0);
                let ab_val = w.props.anchor_bottom.take().unwrap();
                w.props.anchor_bottom = Some(match ab_val {
                    LayoutValue::Absolute(_) => LayoutValue::Absolute(desired),
                    LayoutValue::Percent(_) => LayoutValue::Percent((desired / parent_rect[3]) * 100.0),
                    LayoutValue::Fraction(_) => LayoutValue::Fraction(desired / parent_rect[3]),
                    LayoutValue::MinMax { min, max } => {
                        let minv = min.resolve(parent_rect[3]);
                        let maxv = max.resolve(parent_rect[3]);
                        let clamped = desired.max(minv).min(maxv);
                        LayoutValue::Absolute(clamped)
                    }
                    LayoutValue::Auto => LayoutValue::Auto,
                });
            }
            if w.props.anchor_left.is_some() {
                let desired = w.x - w.w/2.0 - left_edge;
                let al_val = w.props.anchor_left.take().unwrap();
                w.props.anchor_left = Some(match al_val {
                    LayoutValue::Absolute(_) => LayoutValue::Absolute(desired),
                    LayoutValue::Percent(_) => LayoutValue::Percent((desired / parent_rect[2]) * 100.0),
                    LayoutValue::Fraction(_) => LayoutValue::Fraction(desired / parent_rect[2]),
                    LayoutValue::MinMax { min, max } => {
                        let minv = min.resolve(parent_rect[2]);
                        let maxv = max.resolve(parent_rect[2]);
                        let clamped = desired.max(minv).min(maxv);
                        LayoutValue::Absolute(clamped)
                    }
                    LayoutValue::Auto => LayoutValue::Auto,
                });
            }
        }
    }
}
