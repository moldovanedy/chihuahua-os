use core::cmp::max_by;
use core::slice::Iter;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Rect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl Rect {
    pub fn x(&self) -> f32 {
        self.x
    }
    pub fn y(&self) -> f32 {
        self.y
    }
    pub fn width(&self) -> f32 {
        self.width
    }
    pub fn height(&self) -> f32 {
        self.height
    }
    pub fn center_x(&self) -> f32 {
        self.x + self.width / 2.0
    }
    pub fn center_y(&self) -> f32 {
        self.y + self.height / 2.0
    }
    pub fn end_x(&self) -> f32 {
        self.x + self.width
    }
    pub fn end_y(&self) -> f32 {
        self.y + self.height
    }

    pub fn new() -> Self {
        Rect {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        }
    }

    pub fn from_coords(x: f32, y: f32, width: f32, height: f32) -> Self {
        Rect {
            x,
            y,
            width,
            height,
        }
    }

    pub fn get_common_bounding_rect(mut iter: Iter<Rect>) -> Rect {
        let mut next_rect: Option<&Rect> = iter.next();
        if next_rect.is_none() {
            return Rect::new();
        }

        let rect = next_rect.unwrap();
        let mut x = rect.x();
        let mut y = rect.y();
        let mut end_x = rect.end_x();
        let mut end_y = rect.end_y();

        while next_rect.is_some() {
            let rect = next_rect.unwrap();

            if x > rect.x() {
                x = rect.x();
            }
            if y > rect.y() {
                y = rect.y();
            }
            if end_x < rect.end_x() {
                end_x = rect.end_x();
            }
            if end_y < rect.end_y() {
                end_y = rect.end_y();
            }

            next_rect = iter.next();
        }

        return Rect::from_coords(x, y, end_x - x, end_y - y);
    }

    pub fn get_intersecting_rect(rect1: &Rect, rect2: &Rect) -> Rect {
        if Self::do_rects_intersect(rect1, rect2) {
            let x: f32 = max_by(rect1.x(), rect2.x(), |r1, r2| r1.partial_cmp(r2).unwrap());
            let y: f32 = max_by(rect1.y(), rect2.y(), |r1, r2| r1.partial_cmp(r2).unwrap());
            let width: f32 = max_by(rect1.width(), rect2.width(), |r1, r2| {
                r1.partial_cmp(r2).unwrap()
            });
            let height: f32 = max_by(rect1.height(), rect2.height(), |r1, r2| {
                r1.partial_cmp(r2).unwrap()
            });

            return Rect::from_coords(x, y, width, height);
        }

        return Rect::new();
    }

    pub fn do_rects_intersect(rect1: &Rect, rect2: &Rect) -> bool {
        if rect2.x() >= rect1.end_x() {
            return false;
        }

        if rect2.end_x() <= rect1.x() {
            return false;
        }

        if rect2.y() >= rect1.end_y() {
            return false;
        }

        if rect2.end_y() <= rect1.y() {
            return false;
        }

        return true;
    }

    pub fn is_point_inside(rect: &Rect, point: (f32, f32)) -> bool {
        return (point.0 >= rect.x())
            && (point.0 <= rect.end_x())
            && (point.1 >= rect.y())
            && (point.1 <= rect.end_y());
    }
}
