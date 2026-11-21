use crate::drawing::stroke::Stroke;

pub type Lattice = [[(f32, f32); 3]; 3];
pub type Bounds = (f32, f32, f32, f32);

#[derive(Clone, Copy)]
pub struct Affine {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub e: f32,
    pub f: f32,
}

impl Affine {
    pub fn identity() -> Self {
        Self { a: 1.0, b: 0.0, c: 0.0, d: 1.0, e: 0.0, f: 0.0 }
    }

    pub fn translate(dx: f32, dy: f32) -> Self {
        Self { a: 1.0, b: 0.0, c: 0.0, d: 1.0, e: dx, f: dy }
    }

    pub fn scale_about(ax: f32, ay: f32, sx: f32, sy: f32) -> Self {
        Self {
            a: sx,
            b: 0.0,
            c: 0.0,
            d: sy,
            e: ax - sx * ax,
            f: ay - sy * ay,
        }
    }

    pub fn rotate_about(cx: f32, cy: f32, angle: f32) -> Self {
        let (s, c) = angle.sin_cos();
        Self {
            a: c,
            b: s,
            c: -s,
            d: c,
            e: cx - c * cx + s * cy,
            f: cy - s * cx - c * cy,
        }
    }

    pub fn apply(&self, x: f32, y: f32) -> (f32, f32) {
        (self.a * x + self.c * y + self.e, self.b * x + self.d * y + self.f)
    }

    pub fn width_factor(&self) -> f32 {
        (self.a * self.d - self.b * self.c).abs().sqrt()
    }
}

pub fn bounds_of<'a>(strokes: impl Iterator<Item = &'a Stroke>) -> Option<Bounds> {
    let mut r: Option<Bounds> = None;
    for s in strokes {
        for p in &s.points {
            r = Some(match r {
                None => (p.x, p.y, p.x, p.y),
                Some((a, b, c, d)) => (a.min(p.x), b.min(p.y), c.max(p.x), d.max(p.y)),
            });
        }
    }
    r.map(|(mut x0, mut y0, mut x1, mut y1)| {
        if x1 - x0 < 2.0 {
            x0 -= 1.0;
            x1 += 1.0;
        }
        if y1 - y0 < 2.0 {
            y0 -= 1.0;
            y1 += 1.0;
        }
        (x0, y0, x1, y1)
    })
}

pub fn regular_lattice(r: Bounds) -> Lattice {
    let (x0, y0, x1, y1) = r;
    let mut l: Lattice = [[(0.0, 0.0); 3]; 3];
    for j in 0..3 {
        for i in 0..3 {
            l[j][i] = (
                x0 + (x1 - x0) * i as f32 / 2.0,
                y0 + (y1 - y0) * j as f32 / 2.0,
            );
        }
    }
    l
}

pub fn apply_affine(originals: &[Stroke], aff: Affine) -> Vec<Stroke> {
    let k = aff.width_factor();
    originals
        .iter()
        .map(|s| {
            let mut n = s.clone();
            for p in &mut n.points {
                let (x, y) = aff.apply(p.x, p.y);
                p.x = x;
                p.y = y;
            }
            n.width = (s.width * k).max(0.5);
            n
        })
        .collect()
}

fn catmull(p: &[(f32, f32); 3], s: usize, t: f32) -> (f32, f32) {
    let get = |k: i32| -> (f32, f32) {
        if k < 0 {
            (2.0 * p[0].0 - p[1].0, 2.0 * p[0].1 - p[1].1)
        } else if k > 2 {
            (2.0 * p[2].0 - p[1].0, 2.0 * p[2].1 - p[1].1)
        } else {
            p[k as usize]
        }
    };
    let a = get(s as i32 - 1);
    let b = get(s as i32);
    let c = get(s as i32 + 1);
    let d = get(s as i32 + 2);
    let t2 = t * t;
    let t3 = t2 * t;
    let f = |a: f32, b: f32, c: f32, d: f32| {
        0.5 * ((2.0 * b)
            + (-a + c) * t
            + (2.0 * a - 5.0 * b + 4.0 * c - d) * t2
            + (-a + 3.0 * b - 3.0 * c + d) * t3)
    };
    (f(a.0, b.0, c.0, d.0), f(a.1, b.1, c.1, d.1))
}

pub fn cage_map(rect: Bounds, lat: &Lattice, x: f32, y: f32) -> (f32, f32) {
    let (x0, y0, x1, y1) = rect;
    let u = ((x - x0) / (x1 - x0).max(1.0)).clamp(0.0, 1.0) * 2.0;
    let v = ((y - y0) / (y1 - y0).max(1.0)).clamp(0.0, 1.0) * 2.0;
    let su = (u.floor() as usize).min(1);
    let tu = u - su as f32;
    let sv = (v.floor() as usize).min(1);
    let tv = v - sv as f32;
    let mut col = [(0.0f32, 0.0f32); 3];
    for j in 0..3 {
        col[j] = catmull(&lat[j], su, tu);
    }
    catmull(&col, sv, tv)
}

pub fn apply_cage(originals: &[Stroke], rect: Bounds, lat: &Lattice) -> Vec<Stroke> {
    originals
        .iter()
        .map(|s| {
            let mut n = s.clone();
            for p in &mut n.points {
                let (x, y) = cage_map(rect, lat, p.x, p.y);
                p.x = x;
                p.y = y;
            }
            n
        })
        .collect()
}

pub fn point_in_polygon(x: f32, y: f32, poly: &[(f32, f32)]) -> bool {
    let n = poly.len();
    if n < 3 {
        return false;
    }
    let mut inside = false;
    let mut j = n - 1;
    for i in 0..n {
        let (xi, yi) = poly[i];
        let (xj, yj) = poly[j];
        if (yi > y) != (yj > y) && x < (xj - xi) * (y - yi) / (yj - yi) + xi {
            inside = !inside;
        }
        j = i;
    }
    inside
}

pub fn inside_fraction(stroke: &Stroke, poly: &[(f32, f32)]) -> f32 {
    if stroke.points.is_empty() {
        return 0.0;
    }
    let n = stroke
        .points
        .iter()
        .filter(|p| point_in_polygon(p.x, p.y, poly))
        .count();
    n as f32 / stroke.points.len() as f32
}
