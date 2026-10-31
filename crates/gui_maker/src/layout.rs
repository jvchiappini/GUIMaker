//! Geometry and layout primitives.

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub const ZERO: Self = Self { width: 0.0, height: 0.0 };

    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, width: 0.0, height: 0.0 };

    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    pub const fn from_min_max(min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> Self {
        Self::new(min_x, min_y, (max_x - min_x).max(0.0), (max_y - min_y).max(0.0))
    }

    pub fn from_size(size: Size) -> Self {
        Self::new(0.0, 0.0, size.width, size.height)
    }

    pub fn right(self) -> f32 { self.x + self.width }
    pub fn bottom(self) -> f32 { self.y + self.height }
    pub fn center(self) -> [f32; 2] { [self.x + self.width * 0.5, self.y + self.height * 0.5] }
    pub fn size(self) -> Size { Size::new(self.width, self.height) }
    pub fn area(self) -> f32 { self.width * self.height }
    pub fn is_empty(self) -> bool { self.width <= 0.0 || self.height <= 0.0 }

    pub fn contains(self, point: [f32; 2]) -> bool {
        point[0] >= self.x
            && point[0] < self.right()
            && point[1] >= self.y
            && point[1] < self.bottom()
    }

    pub fn intersects(self, other: Rect) -> bool {
        self.x < other.right()
            && other.x < self.right()
            && self.y < other.bottom()
            && other.y < self.bottom()
    }

    pub fn intersect(self, other: Rect) -> Rect {
        let x0 = self.x.max(other.x);
        let y0 = self.y.max(other.y);
        let x1 = self.right().min(other.right());
        let y1 = self.bottom().min(other.bottom());
        Rect::from_min_max(x0, y0, x1, y1)
    }

    pub fn union(self, other: Rect) -> Rect {
        let x0 = self.x.min(other.x);
        let y0 = self.y.min(other.y);
        let x1 = self.right().max(other.right());
        let y1 = self.bottom().max(other.bottom());
        Rect::from_min_max(x0, y0, x1, y1)
    }

    pub fn translate(self, dx: f32, dy: f32) -> Self {
        Self::new(self.x + dx, self.y + dy, self.width, self.height)
    }

    pub fn shrink(self, amount: f32) -> Self {
        self.shrink_xy(amount, amount)
    }

    pub fn shrink_xy(self, horizontal: f32, vertical: f32) -> Self {
        Self::from_min_max(
            self.x + horizontal,
            self.y + vertical,
            self.right() - horizontal,
            self.bottom() - vertical,
        )
    }

    pub fn center_rect(self, width: f32, height: f32) -> Self {
        Self::new(
            self.x + (self.width - width) * 0.5,
            self.y + (self.height - height) * 0.5,
            width,
            height,
        )
    }

    /// Splits `amount` from the left and returns `(left, remaining)`.
    pub fn split_left(self, amount: f32) -> (Self, Self) {
        let amount = amount.clamp(0.0, self.width);
        (
            Self::new(self.x, self.y, amount, self.height),
            Self::new(self.x + amount, self.y, self.width - amount, self.height),
        )
    }

    /// Splits `amount` from the right and returns `(right, remaining)`.
    pub fn split_right(self, amount: f32) -> (Self, Self) {
        let amount = amount.clamp(0.0, self.width);
        (
            Self::new(self.right() - amount, self.y, amount, self.height),
            Self::new(self.x, self.y, self.width - amount, self.height),
        )
    }

    /// Splits `amount` from the top and returns `(top, remaining)`.
    pub fn split_top(self, amount: f32) -> (Self, Self) {
        let amount = amount.clamp(0.0, self.height);
        (
            Self::new(self.x, self.y, self.width, amount),
            Self::new(self.x, self.y + amount, self.width, self.height - amount),
        )
    }

    /// Splits `amount` from the bottom and returns `(bottom, remaining)`.
    pub fn split_bottom(self, amount: f32) -> (Self, Self) {
        let amount = amount.clamp(0.0, self.height);
        (
            Self::new(self.x, self.bottom() - amount, self.width, amount),
            Self::new(self.x, self.y, self.width, self.height - amount),
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Align {
    Start,
    Center,
    End,
}

impl Align {
    pub fn offset(self, available: f32, item: f32) -> f32 {
        match self {
            Align::Start => 0.0,
            Align::Center => (available - item) * 0.5,
            Align::End => available - item,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextAlign {
    pub h: Align,
    pub v: Align,
}

impl TextAlign {
    pub const LEFT_TOP: Self = Self { h: Align::Start, v: Align::Start };
    pub const LEFT_CENTER: Self = Self { h: Align::Start, v: Align::Center };
    pub const CENTER: Self = Self { h: Align::Center, v: Align::Center };
    pub const RIGHT_CENTER: Self = Self { h: Align::End, v: Align::Center };
}

impl Default for TextAlign {
    fn default() -> Self {
        Self::LEFT_TOP
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    Column,
    Row,
    Grid,
}

#[derive(Clone, Debug)]
pub(crate) struct LayoutState {
    pub rect: Rect,
    pub axis: Axis,
    pub gap: f32,
    pub cursor: f32,
    pub index: usize,
    pub columns: usize,
    pub row_height: f32,
    pub cell_width: f32,
}

impl LayoutState {
    pub fn column(rect: Rect, gap: f32) -> Self {
        Self {
            rect,
            axis: Axis::Column,
            gap,
            cursor: 0.0,
            index: 0,
            columns: 1,
            row_height: 0.0,
            cell_width: 0.0,
        }
    }

    pub fn row(rect: Rect, gap: f32) -> Self {
        Self {
            axis: Axis::Row,
            ..Self::column(rect, gap)
        }
    }

    pub fn grid(rect: Rect, columns: usize, row_height: f32, gap: f32) -> Self {
        let columns = columns.max(1);
        let total_gap = gap * (columns.saturating_sub(1)) as f32;
        Self {
            rect,
            axis: Axis::Grid,
            gap,
            cursor: 0.0,
            index: 0,
            columns,
            row_height: row_height.max(0.0),
            cell_width: ((rect.width - total_gap) / columns as f32).max(0.0),
        }
    }

    pub fn next(&mut self, along: f32) -> Rect {
        match self.axis {
            Axis::Column => {
                let height = along.max(0.0).min((self.rect.height - self.cursor).max(0.0));
                let rect = Rect::new(self.rect.x, self.rect.y + self.cursor, self.rect.width, height);
                self.cursor += height + self.gap;
                rect
            }
            Axis::Row => {
                let width = along.max(0.0).min((self.rect.width - self.cursor).max(0.0));
                let rect = Rect::new(self.rect.x + self.cursor, self.rect.y, width, self.rect.height);
                self.cursor += width + self.gap;
                rect
            }
            Axis::Grid => {
                let col = self.index % self.columns;
                let row = self.index / self.columns;
                self.index += 1;
                Rect::new(
                    self.rect.x + col as f32 * (self.cell_width + self.gap),
                    self.rect.y + row as f32 * (self.row_height + self.gap),
                    self.cell_width,
                    self.row_height,
                )
            }
        }
    }

    pub fn fill(&mut self) -> Rect {
        match self.axis {
            Axis::Column => {
                let height = (self.rect.height - self.cursor).max(0.0);
                let rect = Rect::new(self.rect.x, self.rect.y + self.cursor, self.rect.width, height);
                self.cursor = self.rect.height;
                rect
            }
            Axis::Row => {
                let width = (self.rect.width - self.cursor).max(0.0);
                let rect = Rect::new(self.rect.x + self.cursor, self.rect.y, width, self.rect.height);
                self.cursor = self.rect.width;
                rect
            }
            Axis::Grid => self.rect,
        }
    }

    pub fn add_space(&mut self, amount: f32) {
        let amount = amount.max(0.0);
        match self.axis {
            Axis::Column => self.cursor += amount,
            Axis::Row => self.cursor += amount,
            Axis::Grid => {}
        }
    }

    pub fn remaining(&self) -> Rect {
        match self.axis {
            Axis::Column => Rect::new(
                self.rect.x,
                self.rect.y + self.cursor,
                self.rect.width,
                (self.rect.height - self.cursor).max(0.0),
            ),
            Axis::Row => Rect::new(
                self.rect.x + self.cursor,
                self.rect.y,
                (self.rect.width - self.cursor).max(0.0),
                self.rect.height,
            ),
            Axis::Grid => self.rect,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_returns_taken_and_remaining() {
        let r = Rect::new(0.0, 0.0, 100.0, 50.0);
        let (left, rest) = r.split_left(20.0);
        assert_eq!(left, Rect::new(0.0, 0.0, 20.0, 50.0));
        assert_eq!(rest, Rect::new(20.0, 0.0, 80.0, 50.0));

        let (right, rest) = r.split_right(30.0);
        assert_eq!(right, Rect::new(70.0, 0.0, 30.0, 50.0));
        assert_eq!(rest, Rect::new(0.0, 0.0, 70.0, 50.0));

        let (top, rest) = r.split_top(10.0);
        assert_eq!(top, Rect::new(0.0, 0.0, 100.0, 10.0));
        assert_eq!(rest, Rect::new(0.0, 10.0, 100.0, 40.0));

        let (bottom, rest) = r.split_bottom(15.0);
        assert_eq!(bottom, Rect::new(0.0, 35.0, 100.0, 15.0));
        assert_eq!(rest, Rect::new(0.0, 0.0, 100.0, 35.0));
    }

    #[test]
    fn column_layout_stacks_with_gap() {
        let mut l = LayoutState::column(Rect::new(0.0, 0.0, 100.0, 100.0), 4.0);
        assert_eq!(l.next(20.0), Rect::new(0.0, 0.0, 100.0, 20.0));
        assert_eq!(l.next(10.0), Rect::new(0.0, 24.0, 100.0, 10.0));
        l.add_space(6.0);
        assert_eq!(l.next(5.0), Rect::new(0.0, 40.0, 100.0, 5.0));
    }

    #[test]
    fn grid_layout_uses_columns_and_rows() {
        let mut l = LayoutState::grid(Rect::new(0.0, 0.0, 100.0, 100.0), 2, 20.0, 4.0);
        assert_eq!(l.next(0.0), Rect::new(0.0, 0.0, 48.0, 20.0));
        assert_eq!(l.next(0.0), Rect::new(52.0, 0.0, 48.0, 20.0));
        assert_eq!(l.next(0.0), Rect::new(0.0, 24.0, 48.0, 20.0));
    }

    #[test]
    fn contains_is_half_open() {
        let r = Rect::new(10.0, 10.0, 20.0, 20.0);
        assert!(r.contains([10.0, 10.0]));
        assert!(r.contains([29.9, 29.9]));
        assert!(!r.contains([30.0, 20.0]));
    }
}
