use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub pressure: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub fn from_hsv(h: f32, s: f32, v: f32, a: u8) -> Color {
        let h = h.rem_euclid(360.0) / 60.0;
        let s = s.clamp(0.0, 1.0);
        let v = v.clamp(0.0, 1.0);
        let c = v * s;
        let x = c * (1.0 - (h % 2.0 - 1.0).abs());
        let (r, g, b) = match h as u32 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };
        let m = v - c;
        Color {
            r: ((r + m) * 255.0).round() as u8,
            g: ((g + m) * 255.0).round() as u8,
            b: ((b + m) * 255.0).round() as u8,
            a,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stroke {
    pub points: Vec<Point>,
    pub color: Color,
    pub width: f32,
    pub erase: bool,
}

impl Stroke {
    pub fn new(color: Color, width: f32) -> Self {
        Self {
            points: Vec::new(),
            color,
            width,
            erase: false,
        }
    }

    pub fn eraser(width: f32, opacity: f32) -> Self {
        Self {
            points: Vec::new(),
            color: Color {
                r: 0,
                g: 0,
                b: 0,
                a: (opacity.clamp(0.0, 1.0) * 255.0).round() as u8,
            },
            width,
            erase: true,
        }
    }

    pub fn add_point(&mut self, point: Point) {
        self.points.push(point);
    }
}
