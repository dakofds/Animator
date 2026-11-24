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

fn lerp_point(a: crate::drawing::stroke::Point, b: crate::drawing::stroke::Point, t: f32) -> crate::drawing::stroke::Point {
    crate::drawing::stroke::Point {
        x: a.x + (b.x - a.x) * t,
        y: a.y + (b.y - a.y) * t,
        pressure: a.pressure + (b.pressure - a.pressure) * t,
    }
}

fn cross(ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    ax * by - ay * bx
}

// Returns the position t on segment AB where it intersects segment CD.
fn segment_intersection_t(
    a: (f32, f32),
    b: (f32, f32),
    c: (f32, f32),
    d: (f32, f32),
) -> Option<f32> {
    let r = (b.0 - a.0, b.1 - a.1);
    let s = (d.0 - c.0, d.1 - c.1);
    let denom = cross(r.0, r.1, s.0, s.1);
    let q = (c.0 - a.0, c.1 - a.1);
    let eps = 1.0e-6;

    if denom.abs() < eps {
        return None;
    }

    let t = cross(q.0, q.1, s.0, s.1) / denom;
    let u = cross(q.0, q.1, r.0, r.1) / denom;
    if t >= -eps && t <= 1.0 + eps && u >= -eps && u <= 1.0 + eps {
        Some(t.clamp(0.0, 1.0))
    } else {
        None
    }
}

fn dedup_sorted(values: &mut Vec<f32>) {
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    values.dedup_by(|a, b| (*a - *b).abs() < 1.0e-5);
}

/// Splits a stroke at the lasso boundary. Each returned stroke is marked by
/// the bool indicating whether its centerline is inside the lasso.
///
/// Unlike a point-count test, this also inserts the exact line/polygon
/// crossing points, so a sparse stroke is clipped correctly even when the
/// lasso crosses between two sampled points.
pub fn split_stroke_by_polygon(
    stroke: &Stroke,
    poly: &[(f32, f32)],
) -> Vec<(Stroke, bool)> {
    if stroke.points.is_empty() {
        return Vec::new();
    }
    if stroke.points.len() == 1 {
        return vec![(stroke.clone(), point_in_polygon(
            stroke.points[0].x,
            stroke.points[0].y,
            poly,
        ))];
    }
    if poly.len() < 3 {
        return vec![(stroke.clone(), false)];
    }

    let mut out: Vec<(Stroke, bool)> = Vec::new();
    let mut current: Option<Stroke> = None;
    let mut current_inside = false;

    let flush = |out: &mut Vec<(Stroke, bool)>, current: &mut Option<Stroke>, inside: bool| {
        if let Some(s) = current.take() {
            if s.points.len() >= 2 {
                out.push((s, inside));
            }
        }
    };

    for i in 0..stroke.points.len() - 1 {
        let a = stroke.points[i];
        let b = stroke.points[i + 1];
        let mut ts = vec![0.0, 1.0];

        // Close the lasso polygon by connecting its last point to its first.
        for j in 0..poly.len() {
            let c = poly[j];
            let d = poly[(j + 1) % poly.len()];
            if let Some(t) = segment_intersection_t((a.x, a.y), (b.x, b.y), c, d) {
                ts.push(t);
            }
        }
        dedup_sorted(&mut ts);

        for k in 0..ts.len() - 1 {
            let t0 = ts[k];
            let t1 = ts[k + 1];
            if (t1 - t0).abs() < 1.0e-6 {
                continue;
            }
            let p0 = lerp_point(a, b, t0);
            let p1 = lerp_point(a, b, t1);
            let tm = (t0 + t1) * 0.5;
            let pm = lerp_point(a, b, tm);
            let inside = point_in_polygon(pm.x, pm.y, poly);

            if current.is_none() || current_inside != inside {
                if current.is_some() {
                    flush(&mut out, &mut current, current_inside);
                }
                let mut s = Stroke {
                    points: Vec::new(),
                    color: stroke.color,
                    width: stroke.width,
                    erase: stroke.erase,
                };
                s.points.push(p0);
                s.points.push(p1);
                current = Some(s);
                current_inside = inside;
            } else if let Some(s) = current.as_mut() {
                // The previous interval ends exactly where this one starts.
                if s.points.last().map_or(true, |q|
                    (q.x - p0.x).abs() > 1.0e-5 || (q.y - p0.y).abs() > 1.0e-5)
                {
                    s.points.push(p0);
                }
                s.points.push(p1);
            }
        }
    }

    if current.is_some() {
        flush(&mut out, &mut current, current_inside);
    }
    out
}

// Kept for callers that only need the old coarse metric. The lasso itself
// now uses split_stroke_by_polygon instead.
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
