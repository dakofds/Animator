use crate::drawing::stroke::{Color, Stroke};
use crate::ui::font;
use crate::ui::{Area, Hit, Slider, SliderId, Ui, UiView};
use tiny_skia::{
    BlendMode, Color as SkColor, FillRule, LineCap, LineJoin, Paint, Path, PathBuilder, Pixmap,
    PixmapPaint, Stroke as SkStroke, Transform,
};

#[derive(Clone, Copy)]
enum Icon {
    Menu,
    Play,
    Pause,
    Onion,
    FrameMinus,
    FramePlus,
    Layers,
    Brush,
    Eraser,
    Undo,
    Plus,
    Minus,
    Up,
    Down,
    Eye,
    EyeOff,
}

pub struct Renderer {
    pixmap: Pixmap,
    layer: Pixmap,
    wheel: Option<(u32, Pixmap)>,
}

impl Renderer {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            pixmap: Pixmap::new(width, height).expect("tamanho invalido"),
            layer: Pixmap::new(width, height).expect("tamanho invalido"),
            wheel: None,
        }
    }

    pub fn clear(&mut self) {
        self.pixmap.fill(SkColor::WHITE);
    }

    pub fn draw_layer(
        &mut self,
        strokes: &[Stroke],
        active: Option<&Stroke>,
        tint: Option<(Color, f32)>,
    ) {
        if strokes.is_empty() && active.is_none() {
            return;
        }
        self.layer.fill(SkColor::TRANSPARENT);

        let rgb = tint.map(|(c, _)| c);
        for stroke in strokes.iter().chain(active) {
            paint_stroke(&mut self.layer, stroke, rgb);
        }

        let opacity = tint.map(|(_, o)| o).unwrap_or(1.0);
        let pp = PixmapPaint {
            opacity,
            ..PixmapPaint::default()
        };
        self.pixmap.draw_pixmap(
            0,
            0,
            self.layer.as_ref(),
            &pp,
            Transform::identity(),
            None,
        );
    }

    fn ensure_wheel(&mut self, r: f32) {
        let radius = r.round().max(1.0) as u32;
        if matches!(&self.wheel, Some((rp, _)) if *rp == radius) {
            return;
        }
        self.wheel = build_wheel(radius).map(|pm| (radius, pm));
    }

    pub fn draw_ui(&mut self, ui: &Ui, v: &UiView) {
        if v.picker_open {
            self.ensure_wheel(ui.picker.r);
        }
        let pm = &mut self.pixmap;
        let wheel = self.wheel.as_ref();

        let u = ui.scale;
        let w = pm.width() as f32;
        let h = pm.height() as f32;
        let bg = paint(28, 29, 34, 255);
        let border = paint(58, 60, 70, 255);
        let accent = paint(92, 134, 255, 255);

        fill_rrect(pm, 0.0, 0.0, w, ui.top_h, 0.0, &bg);
        fill_rrect(pm, 0.0, ui.top_h - 1.0, w, 1.0, 0.0, &border);
        fill_rrect(pm, 0.0, ui.top_h, ui.side_w, h - ui.top_h, 0.0, &bg);
        fill_rrect(pm, ui.side_w - 1.0, ui.top_h, 1.0, h - ui.top_h, 0.0, &border);

        draw_icon(pm, Icon::Menu, ui.menu_btn, v.menu_open, true);
        let play_icon = if v.playing { Icon::Pause } else { Icon::Play };
        draw_icon(pm, play_icon, ui.play, false, true);
        draw_icon(pm, Icon::Onion, ui.onion_btn, v.onion, true);

        for (i, a) in ui.timeline_cells(v.frames, v.current) {
            let current = i == v.current;
            let cell_paint = if current {
                accent.clone()
            } else {
                paint(48, 50, 60, 255)
            };
            fill_rrect(pm, a.x, a.y, a.w, a.h, 8.0 * u, &cell_paint);

            let label = (i + 1).to_string();
            let n = label.len();
            let px = (34.0 * u / (6 * n - 1) as f32).min(2.0 * u);
            let tw = font::width(&label, px);
            let ink = if current {
                paint(255, 255, 255, 255)
            } else {
                paint(150, 154, 168, 255)
            };
            draw_text(pm, &label, a.cx() - tw / 2.0, a.cy() - 3.5 * px, px, &ink);
        }

        draw_icon(pm, Icon::FrameMinus, ui.remove_btn, false, true);
        draw_icon(pm, Icon::FramePlus, ui.add_btn, false, true);
        draw_icon(pm, Icon::Layers, ui.layers_btn, v.layers_open, true);

        draw_icon(pm, Icon::Brush, ui.brush, !v.eraser, true);
        draw_icon(pm, Icon::Eraser, ui.eraser, v.eraser, true);
        draw_color_button(pm, ui.color, v);

        let pv = ui.preview;
        fill_rrect(pm, pv.x, pv.y, pv.w, pv.h, 8.0 * u, &paint(232, 233, 237, 255));
        let diameter = v.brush_size.min(pv.w - 8.0 * u);
        if let Some(path) = PathBuilder::from_circle(pv.cx(), pv.cy(), diameter / 2.0) {
            if v.eraser {
                outline(pm, &path, &paint(90, 92, 100, 255), 1.5 * u);
            } else {
                let c = Color::from_hsv(v.hue, v.sat, v.val, (v.opacity * 255.0).round() as u8);
                fill(pm, &path, &paint(c.r, c.g, c.b, c.a));
            }
        }

        draw_slider(pm, &ui.size_slider, v.size_t, u);
        draw_slider(pm, &ui.opacity_slider, v.opacity_t, u);
        draw_icon(pm, Icon::Undo, ui.undo, false, true);

        if v.layers_open {
            let p = ui.layers_panel;
            fill_rrect(pm, p.x, p.y, p.w, p.h, 0.0, &bg);
            fill_rrect(pm, p.x, p.y, 1.0, p.h, 0.0, &border);
            draw_text(
                pm,
                "CAMADAS",
                p.x + 14.0 * u,
                p.y + 15.0 * u,
                2.0 * u,
                &paint(225, 227, 235, 255),
            );

            for row in ui.layer_rows(v.layers.len(), v.active_layer) {
                let active = row.index == v.active_layer;
                let info = &v.layers[row.index];
                let row_paint = if active {
                    paint(52, 70, 120, 255)
                } else {
                    paint(38, 40, 48, 255)
                };
                fill_rrect(
                    pm,
                    row.area.x,
                    row.area.y,
                    row.area.w,
                    row.area.h,
                    8.0 * u,
                    &row_paint,
                );
                let eye = if info.visible { Icon::Eye } else { Icon::EyeOff };
                draw_icon(pm, eye, row.eye, false, info.visible);
                let name_paint = if info.visible {
                    paint(225, 227, 235, 255)
                } else {
                    paint(120, 123, 135, 255)
                };
                draw_text(
                    pm,
                    &info.name,
                    row.eye.x + row.eye.w + 6.0 * u,
                    row.area.cy() - 7.0 * u,
                    2.0 * u,
                    &name_paint,
                );
            }

            let n = v.layers.len();
            for (hit, a) in ui.layer_buttons() {
                let (icon, enabled) = match hit {
                    Hit::LayerAdd => (Icon::Plus, true),
                    Hit::LayerRemove => (Icon::Minus, n > 1),
                    Hit::LayerUp => (Icon::Up, v.active_layer + 1 < n),
                    _ => (Icon::Down, v.active_layer > 0),
                };
                fill_rrect(pm, a.x, a.y, a.w, a.h, 8.0 * u, &paint(38, 40, 48, 255));
                draw_icon(pm, icon, a, false, enabled);
            }
        }

        if v.menu_open {
            let m = ui.menu_panel;
            fill_rrect(pm, m.x + 2.0 * u, m.y + 4.0 * u, m.w, m.h, 12.0 * u, &paint(0, 0, 0, 90));
            fill_rrect(pm, m.x, m.y, m.w, m.h, 12.0 * u, &paint(40, 41, 49, 255));
            if let Some(path) = rrect(m.x + 0.5, m.y + 0.5, m.w - 1.0, m.h - 1.0, 12.0 * u) {
                outline(pm, &path, &border, 1.0);
            }
            for (i, item) in ui.menu_items.iter().enumerate() {
                if i == 3 || i == 5 {
                    fill_rrect(
                        pm,
                        item.area.x + 8.0 * u,
                        item.area.y - 1.0,
                        item.area.w - 16.0 * u,
                        1.0,
                        0.0,
                        &border,
                    );
                }
                let ink = if item.action.is_some() {
                    paint(225, 227, 235, 255)
                } else {
                    paint(100, 103, 115, 255)
                };
                draw_text(
                    pm,
                    item.label,
                    item.area.x + 14.0 * u,
                    item.area.cy() - 7.0 * u,
                    2.0 * u,
                    &ink,
                );
            }
        }

        if v.picker_open {
            let p = &ui.picker;
            let a = p.panel;
            fill_rrect(pm, a.x + 2.0 * u, a.y + 4.0 * u, a.w, a.h, 12.0 * u, &paint(0, 0, 0, 90));
            fill_rrect(pm, a.x, a.y, a.w, a.h, 12.0 * u, &paint(40, 41, 49, 255));
            if let Some(path) = rrect(a.x + 0.5, a.y + 0.5, a.w - 1.0, a.h - 1.0, 12.0 * u) {
                outline(pm, &path, &border, 1.0);
            }

            if let Some((rp, wheel_pm)) = wheel {
                let x = (p.cx - *rp as f32).round() as i32;
                let y = (p.cy - *rp as f32).round() as i32;
                pm.draw_pixmap(
                    x,
                    y,
                    wheel_pm.as_ref(),
                    &PixmapPaint::default(),
                    Transform::identity(),
                    None,
                );
                let shade = ((1.0 - v.val) * 255.0).round() as u8;
                if shade > 0 {
                    if let Some(path) = PathBuilder::from_circle(p.cx, p.cy, *rp as f32) {
                        fill(pm, &path, &paint(0, 0, 0, shade));
                    }
                }
            }

            let ang = v.hue.to_radians();
            let mx = p.cx + ang.cos() * v.sat * p.r;
            let my = p.cy + ang.sin() * v.sat * p.r;
            if let Some(path) = PathBuilder::from_circle(mx, my, 9.0 * u) {
                let c = Color::from_hsv(v.hue, v.sat, v.val, 255);
                fill(pm, &path, &paint(c.r, c.g, c.b, 255));
                outline(pm, &path, &paint(255, 255, 255, 255), 2.5 * u);
            }
            if let Some(path) = PathBuilder::from_circle(mx, my, 11.0 * u) {
                outline(pm, &path, &paint(0, 0, 0, 120), 1.2 * u);
            }

            draw_value_slider(pm, &p.value, v.hue, v.sat, v.val, u);
        }
    }

    pub fn data(&self) -> &[u8] {
        self.pixmap.data()
    }
}

fn build_wheel(radius: u32) -> Option<Pixmap> {
    let d = radius * 2;
    let mut pm = Pixmap::new(d, d)?;
    let rf = radius as f32;
    let data = pm.data_mut();
    for py in 0..d {
        for px in 0..d {
            let dx = px as f32 + 0.5 - rf;
            let dy = py as f32 + 0.5 - rf;
            let dist = (dx * dx + dy * dy).sqrt();
            let cover = (rf - dist).clamp(0.0, 1.0);
            if cover <= 0.0 {
                continue;
            }
            let hue = dy.atan2(dx).to_degrees().rem_euclid(360.0);
            let sat = (dist / rf).min(1.0);
            let c = Color::from_hsv(hue, sat, 1.0, 255);
            let a = (cover * 255.0).round() as u32;
            let i = ((py * d + px) * 4) as usize;
            data[i] = (c.r as u32 * a / 255) as u8;
            data[i + 1] = (c.g as u32 * a / 255) as u8;
            data[i + 2] = (c.b as u32 * a / 255) as u8;
            data[i + 3] = a as u8;
        }
    }
    Some(pm)
}

fn paint(r: u8, g: u8, b: u8, a: u8) -> Paint<'static> {
    let mut p = Paint::default();
    p.set_color_rgba8(r, g, b, a);
    p.anti_alias = true;
    p
}

fn fill(pm: &mut Pixmap, path: &Path, p: &Paint) {
    pm.fill_path(path, p, FillRule::Winding, Transform::identity(), None);
}

fn outline(pm: &mut Pixmap, path: &Path, p: &Paint, width: f32) {
    let style = SkStroke {
        width,
        line_cap: LineCap::Round,
        line_join: LineJoin::Round,
        ..Default::default()
    };
    pm.stroke_path(path, p, &style, Transform::identity(), None);
}

fn poly(points: &[(f32, f32)], closed: bool) -> Option<Path> {
    let (first, rest) = points.split_first()?;
    let mut pb = PathBuilder::new();
    pb.move_to(first.0, first.1);
    for pt in rest {
        pb.line_to(pt.0, pt.1);
    }
    if closed {
        pb.close();
    }
    pb.finish()
}

fn stroke_line(pm: &mut Pixmap, points: &[(f32, f32)], closed: bool, p: &Paint, width: f32) {
    if let Some(path) = poly(points, closed) {
        outline(pm, &path, p, width);
    }
}

fn rrect(x: f32, y: f32, w: f32, h: f32, r: f32) -> Option<Path> {
    let r = r.min(w / 2.0).min(h / 2.0);
    if r < 0.5 {
        return poly(&[(x, y), (x + w, y), (x + w, y + h), (x, y + h)], true);
    }
    let k = r * 0.5523;
    let mut pb = PathBuilder::new();
    pb.move_to(x + r, y);
    pb.line_to(x + w - r, y);
    pb.cubic_to(x + w - r + k, y, x + w, y + r - k, x + w, y + r);
    pb.line_to(x + w, y + h - r);
    pb.cubic_to(x + w, y + h - r + k, x + w - r + k, y + h, x + w - r, y + h);
    pb.line_to(x + r, y + h);
    pb.cubic_to(x + r - k, y + h, x, y + h - r + k, x, y + h - r);
    pb.line_to(x, y + r);
    pb.cubic_to(x, y + r - k, x + r - k, y, x + r, y);
    pb.close();
    pb.finish()
}

fn fill_rrect(pm: &mut Pixmap, x: f32, y: f32, w: f32, h: f32, r: f32, p: &Paint) {
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    if let Some(path) = rrect(x, y, w, h, r) {
        fill(pm, &path, p);
    }
}

fn draw_text(pm: &mut Pixmap, text: &str, x: f32, y: f32, px: f32, p: &Paint) {
    let mut cx = x;
    for ch in text.chars() {
        if let Some(rows) = font::glyph(ch) {
            for (r, &bits) in rows.iter().enumerate() {
                let on = |c: usize| (bits >> (4 - c)) & 1 == 1;
                let mut c = 0usize;
                while c < 5 {
                    if on(c) {
                        let start = c;
                        while c < 5 && on(c) {
                            c += 1;
                        }
                        fill_rrect(
                            pm,
                            cx + start as f32 * px,
                            y + r as f32 * px,
                            (c - start) as f32 * px,
                            px,
                            0.0,
                            p,
                        );
                    } else {
                        c += 1;
                    }
                }
            }
        }
        cx += 6.0 * px;
    }
}

fn draw_color_button(pm: &mut Pixmap, a: Area, v: &UiView) {
    let k = a.w / 44.0;
    if v.picker_open {
        fill_rrect(pm, a.x, a.y, a.w, a.h, 10.0 * k, &paint(52, 70, 120, 255));
    }
    for i in 0..10 {
        let ang = i as f32 * 36.0;
        let rad = ang.to_radians();
        let c = Color::from_hsv(ang, 1.0, 1.0, 255);
        let px = a.cx() + rad.cos() * 16.0 * k;
        let py = a.cy() + rad.sin() * 16.0 * k;
        if let Some(path) = PathBuilder::from_circle(px, py, 3.4 * k) {
            fill(pm, &path, &paint(c.r, c.g, c.b, 255));
        }
    }
    let c = Color::from_hsv(v.hue, v.sat, v.val, 255);
    if let Some(path) = PathBuilder::from_circle(a.cx(), a.cy(), 10.0 * k) {
        fill(pm, &path, &paint(c.r, c.g, c.b, 255));
        outline(pm, &path, &paint(205, 208, 218, 255), 1.5 * k);
    }
}

fn draw_slider(pm: &mut Pixmap, s: &Slider, t: f32, u: f32) {
    let (sx, sy) = s.start();
    let (ex, ey) = s.end();
    let (tx, ty) = s.point_at(t);
    let accent = paint(92, 134, 255, 255);
    let muted = paint(130, 134, 148, 255);

    stroke_line(pm, &[(sx, sy), (ex, ey)], false, &paint(66, 68, 78, 255), 4.0 * u);
    if t > 0.01 {
        stroke_line(pm, &[(sx, sy), (tx, ty)], false, &accent, 4.0 * u);
    }

    let top = (s.area.cx(), s.area.y + s.edge / 2.0);
    let bottom = (s.area.cx(), s.area.y + s.area.h - s.edge / 2.0);
    match s.id {
        SliderId::Size => {
            if let Some(path) = PathBuilder::from_circle(bottom.0, bottom.1, 2.0 * u) {
                fill(pm, &path, &muted);
            }
            if let Some(path) = PathBuilder::from_circle(top.0, top.1, 6.0 * u) {
                fill(pm, &path, &muted);
            }
        }
        SliderId::Opacity => {
            if let Some(path) = PathBuilder::from_circle(bottom.0, bottom.1, 5.0 * u) {
                outline(pm, &path, &muted, 1.5 * u);
            }
            if let Some(path) = PathBuilder::from_circle(top.0, top.1, 5.0 * u) {
                fill(pm, &path, &muted);
            }
        }
        SliderId::Value => {}
    }

    draw_thumb(pm, tx, ty, u, &accent);
}

fn draw_value_slider(pm: &mut Pixmap, s: &Slider, hue: f32, sat: f32, t: f32, u: f32) {
    let (sx, sy) = s.start();
    let (ex, _) = s.end();
    let (tx, ty) = s.point_at(t);
    let segments = 24usize;
    let seg_w = (ex - sx) / segments as f32;
    for k in 0..segments {
        let c = Color::from_hsv(hue, sat, (k as f32 + 0.5) / segments as f32, 255);
        fill_rrect(
            pm,
            sx + k as f32 * seg_w,
            sy - 5.0 * u,
            seg_w + 0.6,
            10.0 * u,
            0.0,
            &paint(c.r, c.g, c.b, 255),
        );
    }
    if let Some(path) = rrect(sx, sy - 5.0 * u, ex - sx, 10.0 * u, 5.0 * u) {
        outline(pm, &path, &paint(90, 92, 105, 255), 1.0 * u);
    }
    draw_thumb(pm, tx, ty, u, &paint(92, 134, 255, 255));
}

fn draw_thumb(pm: &mut Pixmap, x: f32, y: f32, u: f32, accent: &Paint) {
    if let Some(path) = PathBuilder::from_circle(x, y + 1.0 * u, 10.0 * u) {
        fill(pm, &path, &paint(0, 0, 0, 70));
    }
    if let Some(path) = PathBuilder::from_circle(x, y, 9.0 * u) {
        fill(pm, &path, &paint(240, 241, 246, 255));
    }
    if let Some(path) = PathBuilder::from_circle(x, y, 3.0 * u) {
        fill(pm, &path, accent);
    }
}

fn draw_icon(pm: &mut Pixmap, icon: Icon, a: Area, selected: bool, enabled: bool) {
    let k = a.w / 44.0;
    let cx = a.cx();
    let cy = a.cy();
    let o = |dx: f32, dy: f32| (cx + dx * k, cy + dy * k);
    let q = |along: f32, side: f32| o((along + side) * 0.7071, (side - along) * 0.7071);

    if selected && !matches!(icon, Icon::Play | Icon::Pause) {
        fill_rrect(pm, a.x, a.y, a.w, a.h, 10.0 * k, &paint(52, 70, 120, 255));
    }

    let ink = if !enabled {
        paint(105, 108, 120, 255)
    } else if selected {
        paint(255, 255, 255, 255)
    } else {
        paint(205, 208, 218, 255)
    };
    let soft = if selected {
        paint(255, 255, 255, 110)
    } else {
        paint(205, 208, 218, 80)
    };
    let sw = 1.9 * k;

    match icon {
        Icon::Menu => {
            for dy in [-8.0, 0.0, 8.0] {
                stroke_line(pm, &[o(-11.0, dy), o(11.0, dy)], false, &ink, 2.4 * k);
            }
        }
        Icon::Play | Icon::Pause => {
            if let Some(path) = PathBuilder::from_circle(cx, cy, 19.0 * k) {
                fill(pm, &path, &paint(92, 134, 255, 255));
            }
            let white = paint(255, 255, 255, 255);
            if matches!(icon, Icon::Pause) {
                fill_rrect(pm, cx - 6.0 * k, cy - 7.0 * k, 4.0 * k, 14.0 * k, 1.0 * k, &white);
                fill_rrect(pm, cx + 2.0 * k, cy - 7.0 * k, 4.0 * k, 14.0 * k, 1.0 * k, &white);
            } else if let Some(path) = poly(&[o(-4.0, -8.0), o(-4.0, 8.0), o(8.0, 0.0)], true) {
                fill(pm, &path, &white);
                outline(pm, &path, &white, 1.5 * k);
            }
        }
        Icon::Onion => {
            if let Some(path) = PathBuilder::from_circle(cx - 4.5 * k, cy, 7.5 * k) {
                outline(pm, &path, &ink, sw);
            }
            if let Some(path) = PathBuilder::from_circle(cx + 4.5 * k, cy, 7.5 * k) {
                outline(pm, &path, &ink, sw);
            }
        }
        Icon::FrameMinus | Icon::FramePlus => {
            stroke_line(
                pm,
                &[o(-10.0, -10.0), o(10.0, -10.0), o(10.0, 10.0), o(-10.0, 10.0)],
                true,
                &ink,
                sw,
            );
            stroke_line(pm, &[o(-5.0, 0.0), o(5.0, 0.0)], false, &ink, sw);
            if matches!(icon, Icon::FramePlus) {
                stroke_line(pm, &[o(0.0, -5.0), o(0.0, 5.0)], false, &ink, sw);
            }
        }
        Icon::Layers => {
            stroke_line(
                pm,
                &[o(-11.0, -6.0), o(0.0, -12.0), o(11.0, -6.0), o(0.0, 0.0)],
                true,
                &ink,
                sw,
            );
            stroke_line(pm, &[o(-11.0, 0.0), o(0.0, 6.0), o(11.0, 0.0)], false, &ink, sw);
            stroke_line(pm, &[o(-11.0, 6.0), o(0.0, 12.0), o(11.0, 6.0)], false, &ink, sw);
        }
        Icon::Brush => {
            stroke_line(
                pm,
                &[q(-11.0, 0.0), q(-6.0, 3.4), q(10.0, 3.4), q(10.0, -3.4), q(-6.0, -3.4)],
                true,
                &ink,
                sw,
            );
            stroke_line(pm, &[q(-6.0, 3.4), q(-6.0, -3.4)], false, &ink, sw);
        }
        Icon::Eraser => {
            if let Some(path) = poly(
                &[q(-10.0, -6.0), q(-2.0, -6.0), q(-2.0, 6.0), q(-10.0, 6.0)],
                true,
            ) {
                fill(pm, &path, &soft);
            }
            stroke_line(
                pm,
                &[q(-10.0, -6.0), q(10.0, -6.0), q(10.0, 6.0), q(-10.0, 6.0)],
                true,
                &ink,
                sw,
            );
            stroke_line(pm, &[q(-2.0, -6.0), q(-2.0, 6.0)], false, &ink, sw);
        }
        Icon::Undo => {
            let s = o(8.0, 7.0);
            let c1 = o(8.0, -1.0);
            let c2 = o(3.0, -5.0);
            let e = o(-5.0, -5.0);
            let mut pb = PathBuilder::new();
            pb.move_to(s.0, s.1);
            pb.cubic_to(c1.0, c1.1, c2.0, c2.1, e.0, e.1);
            if let Some(path) = pb.finish() {
                outline(pm, &path, &ink, sw);
            }
            stroke_line(pm, &[o(-1.0, -10.0), o(-6.0, -5.0), o(-1.0, 0.0)], false, &ink, sw);
        }
        Icon::Plus => {
            stroke_line(pm, &[o(-8.0, 0.0), o(8.0, 0.0)], false, &ink, 2.2 * k);
            stroke_line(pm, &[o(0.0, -8.0), o(0.0, 8.0)], false, &ink, 2.2 * k);
        }
        Icon::Minus => {
            stroke_line(pm, &[o(-8.0, 0.0), o(8.0, 0.0)], false, &ink, 2.2 * k);
        }
        Icon::Up => {
            stroke_line(pm, &[o(-7.0, 3.0), o(0.0, -4.0), o(7.0, 3.0)], false, &ink, 2.2 * k);
        }
        Icon::Down => {
            stroke_line(pm, &[o(-7.0, -3.0), o(0.0, 4.0), o(7.0, -3.0)], false, &ink, 2.2 * k);
        }
        Icon::Eye | Icon::EyeOff => {
            let l = o(-11.0, 0.0);
            let r = o(11.0, 0.0);
            let t = o(0.0, -11.0);
            let b = o(0.0, 11.0);
            let mut pb = PathBuilder::new();
            pb.move_to(l.0, l.1);
            pb.quad_to(t.0, t.1, r.0, r.1);
            pb.quad_to(b.0, b.1, l.0, l.1);
            pb.close();
            if let Some(path) = pb.finish() {
                outline(pm, &path, &ink, sw);
            }
            if let Some(path) = PathBuilder::from_circle(cx, cy, 3.2 * k) {
                fill(pm, &path, &ink);
            }
            if matches!(icon, Icon::EyeOff) {
                stroke_line(pm, &[o(-9.0, 9.0), o(9.0, -9.0)], false, &ink, sw);
            }
        }
    }
}

fn paint_stroke(target: &mut Pixmap, stroke: &Stroke, tint: Option<Color>) {
    let Some(first) = stroke.points.first() else {
        return;
    };

    let color = if stroke.erase {
        stroke.color
    } else if let Some(t) = tint {
        Color { a: 255, ..t }
    } else {
        stroke.color
    };

    let mut p = Paint::default();
    p.set_color_rgba8(color.r, color.g, color.b, color.a);
    p.anti_alias = true;
    if stroke.erase {
        p.blend_mode = BlendMode::DestinationOut;
    }

    if stroke.points.len() == 1 {
        if let Some(path) = PathBuilder::from_circle(first.x, first.y, stroke.width / 2.0) {
            target.fill_path(&path, &p, FillRule::Winding, Transform::identity(), None);
        }
        return;
    }

    let mut pb = PathBuilder::new();
    pb.move_to(first.x, first.y);
    for pt in &stroke.points[1..] {
        pb.line_to(pt.x, pt.y);
    }

    if let Some(path) = pb.finish() {
        let style = SkStroke {
            width: stroke.width,
            line_cap: LineCap::Round,
            line_join: LineJoin::Round,
            ..Default::default()
        };
        target.stroke_path(&path, &p, &style, Transform::identity(), None);
    }
}
