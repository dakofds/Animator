use crate::drawing::stroke::{Color, Stroke};
use crate::ui::text::Text;
use crate::ui::{Area, Hit, Overlay, SelTool, Slider, SliderId, Ui, UiView};
use tiny_skia::{
    BlendMode, Color as SkColor, FillRule, LineCap, LineJoin, Paint, Path, PathBuilder, Pixmap,
    PixmapPaint, Stroke as SkStroke, StrokeDash, Transform,
};

const BG: [u8; 3] = [43, 44, 48];
const EDGE: [u8; 3] = [26, 27, 30];
const SEL: [u8; 3] = [68, 69, 75];
const INK: [u8; 3] = [205, 206, 210];
const INK_DIM: [u8; 3] = [112, 113, 119];
const LIGHT: [u8; 3] = [216, 217, 221];
const CELL: [u8; 3] = [92, 93, 99];
const TRACK: [u8; 3] = [72, 73, 79];
const POPUP: [u8; 3] = [52, 53, 58];
const LINE: [u8; 3] = [68, 69, 75];
const TEXT: [u8; 4] = [226, 227, 231, 255];
const TEXT_DIM: [u8; 4] = [112, 113, 119, 255];

#[derive(Clone, Copy)]
enum Icon {
    Menu,
    Play,
    Pause,
    Onion,
    Layers,
    Brush,
    Eraser,
    Undo,
    Redo,
    Plus,
    Minus,
    Up,
    Down,
    Eye,
    EyeOff,
    Lasso,
    Free,
    Cage,
}

fn sel_icon(t: SelTool) -> Icon {
    match t {
        SelTool::Lasso => Icon::Lasso,
        SelTool::Free => Icon::Free,
        SelTool::Cage => Icon::Cage,
    }
}

pub struct Renderer {
    pixmap: Pixmap,
    layer: Pixmap,
    wheel: Option<(u32, Pixmap)>,
    text: Text,
}

impl Renderer {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            pixmap: Pixmap::new(width, height).expect("tamanho invalido"),
            layer: Pixmap::new(width, height).expect("tamanho invalido"),
            wheel: None,
            text: Text::new(),
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

    pub fn draw_overlay(&mut self, overlay: &Overlay, u: f32) {
        let pm = &mut self.pixmap;
        match overlay {
            Overlay::None => {}
            Overlay::Lasso(path) => two_tone(pm, path, false, true, u),
            Overlay::Selection(r) => {
                let pts = [(r.0, r.1), (r.2, r.1), (r.2, r.3), (r.0, r.3)];
                two_tone(pm, &pts, true, true, u);
            }
            Overlay::Free { quad, rot } => {
                two_tone(pm, quad, true, false, u);
                let top = (
                    (quad[0].0 + quad[1].0) / 2.0,
                    (quad[0].1 + quad[1].1) / 2.0,
                );
                two_tone(pm, &[top, *rot], false, false, u);
                for k in 0..4 {
                    let a = quad[k];
                    let b = quad[(k + 1) % 4];
                    handle_square(pm, a, u);
                    handle_square(pm, ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0), u);
                }
                handle_circle(pm, *rot, u);
            }
            Overlay::Cage(pts) => {
                for j in 0..3 {
                    two_tone(pm, &pts[j], false, false, u);
                }
                for i in 0..3 {
                    two_tone(pm, &[pts[0][i], pts[1][i], pts[2][i]], false, false, u);
                }
                for j in 0..3 {
                    for i in 0..3 {
                        handle_circle(pm, pts[j][i], u);
                    }
                }
            }
        }
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
        let text = &mut self.text;
        let wheel = self.wheel.as_ref();

        let u = ui.scale;
        let w = pm.width() as f32;
        let h = pm.height() as f32;

        fill_rrect(pm, 0.0, 0.0, w, ui.top_h, 0.0, &pc(BG, 255));
        fill_rrect(pm, 0.0, ui.top_h - 1.0, w, 1.0, 0.0, &pc(EDGE, 255));
        for sx in ui.separators {
            fill_rrect(pm, sx, ui.top_h * 0.25, 1.0, ui.top_h * 0.5, 0.0, &pc(LINE, 255));
        }

        draw_icon(pm, Icon::Menu, ui.menu_btn, v.menu_open, true);
        draw_icon(pm, Icon::Undo, ui.undo_btn, false, v.can_undo);
        draw_icon(pm, Icon::Redo, ui.redo_btn, false, v.can_redo);
        let play_icon = if v.playing { Icon::Pause } else { Icon::Play };
        draw_icon(pm, play_icon, ui.play, false, true);
        draw_icon(pm, Icon::Layers, ui.layers_btn, v.layers_open, true);

        let tl = ui.timeline(v.frames, v.current);
        for (i, a) in &tl.cells {
            let current = *i == v.current;
            let dragging = matches!(v.frame_drag, Some(d) if d.from == *i);
            let col = if current { LIGHT } else { CELL };
            let alpha = if dragging { 90 } else { 255 };
            fill_rrect(pm, a.x, a.y, a.w, a.h, 2.0 * u, &pc(col, alpha));
        }
        let add = tl.add;
        if let Some(path) = rrect(add.x + 0.5, add.y + 0.5, add.w - 1.0, add.h - 1.0, 2.0 * u) {
            outline(pm, &path, &pc(CELL, 255), 1.0);
        }
        draw_icon(pm, Icon::Plus, add, false, true);

        if let Some(d) = v.frame_drag {
            if d.target != d.from {
                if let Some((_, a)) = tl.cells.iter().find(|(i, _)| *i == d.target) {
                    let lx = if d.target > d.from {
                        a.x + a.w + 2.0 * u
                    } else {
                        a.x - 2.0 * u
                    };
                    fill_rrect(
                        pm,
                        lx - 1.0 * u,
                        a.y - 4.0 * u,
                        2.0 * u,
                        a.h + 8.0 * u,
                        0.0,
                        &pc(LIGHT, 255),
                    );
                }
            }
            let cy = ui.top_h / 2.0 - 3.0 * u;
            fill_rrect(
                pm,
                d.x - 10.0 * u,
                cy - 17.0 * u,
                20.0 * u,
                34.0 * u,
                2.0 * u,
                &pc(LIGHT, 230),
            );
        }

        fill_rrect(pm, 0.0, ui.top_h, ui.side_w, h - ui.top_h, 0.0, &pc(BG, 255));
        fill_rrect(pm, ui.side_w - 1.0, ui.top_h, 1.0, h - ui.top_h, 0.0, &pc(EDGE, 255));

        draw_icon(pm, Icon::Brush, ui.brush, v.pen_active, true);
        draw_icon(pm, sel_icon(v.sel_tool), ui.sel_btn, v.sel_active, true);
        draw_icon(pm, Icon::Eraser, ui.eraser, v.eraser, true);
        draw_color_button(pm, ui.color, v);

        let pv = ui.preview;
        fill_rrect(pm, pv.x, pv.y, pv.w, pv.h, 2.0 * u, &paint(226, 227, 230, 255));
        let diameter = v.brush_size.min(pv.w - 6.0 * u);
        if let Some(path) = PathBuilder::from_circle(pv.cx(), pv.cy(), diameter / 2.0) {
            if v.eraser {
                outline(pm, &path, &paint(90, 92, 100, 255), 1.2 * u);
            } else {
                let c = Color::from_hsv(v.hue, v.sat, v.val, (v.opacity * 255.0).round() as u8);
                fill(pm, &path, &paint(c.r, c.g, c.b, c.a));
            }
        }

        draw_slider(pm, &ui.size_slider, v.size_t, u);
        draw_slider(pm, &ui.opacity_slider, v.opacity_t, u);
        draw_icon(pm, Icon::Onion, ui.onion_btn, v.onion, true);

        if v.layers_open {
            let p = ui.layers_panel;
            fill_rrect(pm, p.x, p.y, p.w, p.h, 0.0, &pc(BG, 255));
            fill_rrect(pm, p.x, p.y, 1.0, p.h, 0.0, &pc(EDGE, 255));
            text.draw(pm, "Camadas", p.x + 12.0 * u, p.y + 20.0 * u, 13.0 * u, TEXT_DIM);

            for row in ui.layer_rows(v.layers.len(), v.active_layer) {
                let active = row.index == v.active_layer;
                let info = &v.layers[row.index];
                if active {
                    fill_rrect(
                        pm,
                        row.area.x,
                        row.area.y,
                        row.area.w,
                        row.area.h,
                        3.0 * u,
                        &pc(SEL, 255),
                    );
                }
                let eye = if info.visible { Icon::Eye } else { Icon::EyeOff };
                draw_icon(pm, eye, row.eye, false, info.visible);
                let color = if info.visible { TEXT } else { TEXT_DIM };
                text.draw(
                    pm,
                    &info.name,
                    row.eye.x + row.eye.w + 4.0 * u,
                    row.area.cy(),
                    14.0 * u,
                    color,
                );
            }

            let buttons = ui.layer_buttons();
            let strip = buttons[0].1.y - 8.0 * u;
            fill_rrect(pm, p.x + 1.0, strip, p.w - 1.0, 1.0, 0.0, &pc(EDGE, 255));

            let n = v.layers.len();
            for (hit, a) in buttons {
                let (icon, enabled) = match hit {
                    Hit::LayerAdd => (Icon::Plus, true),
                    Hit::LayerRemove => (Icon::Minus, n > 1),
                    Hit::LayerUp => (Icon::Up, v.active_layer + 1 < n),
                    _ => (Icon::Down, v.active_layer > 0),
                };
                draw_icon(pm, icon, a, false, enabled);
            }
        }

        if v.flyout_open {
            draw_popup_panel(pm, ui.flyout_panel, u);
            for (tool, label, a) in &ui.flyout_items {
                if *tool == v.sel_tool {
                    fill_rrect(pm, a.x, a.y, a.w, a.h, 3.0 * u, &pc(SEL, 255));
                }
                let icon_area = Area::new(a.x, a.y, a.h, a.h);
                draw_icon(pm, sel_icon(*tool), icon_area, false, true);
                text.draw(pm, label, a.x + a.h + 2.0 * u, a.cy(), 14.0 * u, TEXT);
            }
        }

        if v.menu_open {
            draw_popup_panel(pm, ui.menu_panel, u);
            for (i, item) in ui.menu_items.iter().enumerate() {
                if i == 3 || i == 5 {
                    fill_rrect(
                        pm,
                        item.area.x + 6.0 * u,
                        item.area.y - 1.0,
                        item.area.w - 12.0 * u,
                        1.0,
                        0.0,
                        &pc(LINE, 255),
                    );
                }
                let color = if item.action.is_some() { TEXT } else { TEXT_DIM };
                text.draw(
                    pm,
                    item.label,
                    item.area.x + 10.0 * u,
                    item.area.cy(),
                    14.0 * u,
                    color,
                );
            }
        }

        if let Some(anchor) = v.frame_menu {
            let (panel, items) = ui.frame_menu_layout(anchor);
            draw_popup_panel(pm, panel, u);
            for (_, label, a) in items {
                text.draw(pm, label, a.x + 10.0 * u, a.cy(), 14.0 * u, TEXT);
            }
        }

        if v.picker_open {
            let p = &ui.picker;
            draw_popup_panel(pm, p.panel, u);

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
            if let Some(path) = PathBuilder::from_circle(mx, my, 8.0 * u) {
                let c = Color::from_hsv(v.hue, v.sat, v.val, 255);
                fill(pm, &path, &paint(c.r, c.g, c.b, 255));
                outline(pm, &path, &paint(255, 255, 255, 255), 2.0 * u);
            }
            if let Some(path) = PathBuilder::from_circle(mx, my, 9.5 * u) {
                outline(pm, &path, &paint(0, 0, 0, 110), 1.0 * u);
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

fn pc(c: [u8; 3], a: u8) -> Paint<'static> {
    paint(c[0], c[1], c[2], a)
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

fn two_tone(pm: &mut Pixmap, pts: &[(f32, f32)], closed: bool, dashed: bool, u: f32) {
    let Some(path) = poly(pts, closed) else {
        return;
    };
    outline(pm, &path, &paint(255, 255, 255, 235), 3.2 * u);
    let dash = if dashed {
        StrokeDash::new(vec![6.0 * u, 4.0 * u], 0.0)
    } else {
        None
    };
    let style = SkStroke {
        width: 1.4 * u,
        line_cap: LineCap::Butt,
        line_join: LineJoin::Round,
        dash,
        ..Default::default()
    };
    pm.stroke_path(&path, &paint(30, 30, 36, 255), &style, Transform::identity(), None);
}

fn handle_square(pm: &mut Pixmap, p: (f32, f32), u: f32) {
    let s = 4.0 * u;
    fill_rrect(pm, p.0 - s, p.1 - s, 2.0 * s, 2.0 * s, 0.0, &paint(255, 255, 255, 255));
    if let Some(path) = rrect(p.0 - s, p.1 - s, 2.0 * s, 2.0 * s, 0.0) {
        outline(pm, &path, &paint(30, 30, 36, 255), 1.2 * u);
    }
}

fn handle_circle(pm: &mut Pixmap, p: (f32, f32), u: f32) {
    if let Some(path) = PathBuilder::from_circle(p.0, p.1, 5.5 * u) {
        fill(pm, &path, &paint(255, 255, 255, 255));
        outline(pm, &path, &paint(30, 30, 36, 255), 1.2 * u);
    }
}

fn draw_popup_panel(pm: &mut Pixmap, a: Area, u: f32) {
    fill_rrect(pm, a.x, a.y + 2.0 * u, a.w, a.h, 4.0 * u, &paint(0, 0, 0, 50));
    fill_rrect(pm, a.x, a.y, a.w, a.h, 4.0 * u, &pc(POPUP, 255));
    if let Some(path) = rrect(a.x + 0.5, a.y + 0.5, a.w - 1.0, a.h - 1.0, 4.0 * u) {
        outline(pm, &path, &pc(EDGE, 255), 1.0);
    }
}

fn draw_color_button(pm: &mut Pixmap, a: Area, v: &UiView) {
    let k = a.w / 44.0;
    if v.picker_open {
        fill_rrect(pm, a.x, a.y, a.w, a.h, 3.0 * k, &pc(SEL, 255));
    }
    let c = Color::from_hsv(v.hue, v.sat, v.val, 255);
    if let Some(path) = PathBuilder::from_circle(a.cx(), a.cy(), 12.0 * k) {
        fill(pm, &path, &paint(c.r, c.g, c.b, 255));
        outline(pm, &path, &pc(INK, 255), 1.2 * k);
    }
}

fn draw_thumb(pm: &mut Pixmap, x: f32, y: f32, u: f32) {
    if let Some(path) = PathBuilder::from_circle(x, y, 6.5 * u) {
        fill(pm, &path, &paint(236, 237, 240, 255));
        outline(pm, &path, &pc(EDGE, 255), 1.0 * u);
    }
}

fn draw_slider(pm: &mut Pixmap, s: &Slider, t: f32, u: f32) {
    let (sx, sy) = s.start();
    let (ex, ey) = s.end();
    let (tx, ty) = s.point_at(t);

    stroke_line(pm, &[(sx, sy), (ex, ey)], false, &pc(TRACK, 255), 2.5 * u);
    if t > 0.01 {
        stroke_line(pm, &[(sx, sy), (tx, ty)], false, &pc(LIGHT, 255), 2.5 * u);
    }

    let top = (s.area.cx(), s.area.y + s.edge / 2.0);
    let bottom = (s.area.cx(), s.area.y + s.area.h - s.edge / 2.0);
    let muted = pc(INK_DIM, 255);
    match s.id {
        SliderId::Size => {
            if let Some(path) = PathBuilder::from_circle(bottom.0, bottom.1, 1.8 * u) {
                fill(pm, &path, &muted);
            }
            if let Some(path) = PathBuilder::from_circle(top.0, top.1, 5.0 * u) {
                fill(pm, &path, &muted);
            }
        }
        SliderId::Opacity => {
            if let Some(path) = PathBuilder::from_circle(bottom.0, bottom.1, 4.5 * u) {
                outline(pm, &path, &muted, 1.2 * u);
            }
            if let Some(path) = PathBuilder::from_circle(top.0, top.1, 4.5 * u) {
                fill(pm, &path, &muted);
            }
        }
        SliderId::Value => {}
    }

    draw_thumb(pm, tx, ty, u);
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
            sy - 4.0 * u,
            seg_w + 0.6,
            8.0 * u,
            0.0,
            &paint(c.r, c.g, c.b, 255),
        );
    }
    if let Some(path) = rrect(sx, sy - 4.0 * u, ex - sx, 8.0 * u, 0.0) {
        outline(pm, &path, &pc(EDGE, 255), 1.0);
    }
    draw_thumb(pm, tx, ty, u);
}

fn draw_icon(pm: &mut Pixmap, icon: Icon, a: Area, selected: bool, enabled: bool) {
    let k = a.w / 44.0;
    let cx = a.cx();
    let cy = a.cy();
    let o = |dx: f32, dy: f32| (cx + dx * k, cy + dy * k);
    let q = |along: f32, side: f32| o((along + side) * 0.7071, (side - along) * 0.7071);

    if selected {
        fill_rrect(pm, a.x, a.y, a.w, a.h, 3.0 * k, &pc(SEL, 255));
    }

    let ink = if !enabled {
        pc(INK_DIM, 255)
    } else if selected {
        paint(255, 255, 255, 255)
    } else {
        pc(INK, 255)
    };
    let soft = if selected {
        paint(255, 255, 255, 90)
    } else {
        paint(205, 206, 210, 70)
    };
    let sw = 1.6 * k;

    match icon {
        Icon::Menu => {
            for dy in [-8.0, 0.0, 8.0] {
                stroke_line(pm, &[o(-11.0, dy), o(11.0, dy)], false, &ink, 2.0 * k);
            }
        }
        Icon::Play => {
            let bright = paint(236, 237, 240, 255);
            if let Some(path) = poly(&[o(-6.0, -10.0), o(-6.0, 10.0), o(10.0, 0.0)], true) {
                fill(pm, &path, &bright);
                outline(pm, &path, &bright, 1.2 * k);
            }
        }
        Icon::Pause => {
            let bright = paint(236, 237, 240, 255);
            fill_rrect(pm, cx - 8.0 * k, cy - 10.0 * k, 5.0 * k, 20.0 * k, 0.5 * k, &bright);
            fill_rrect(pm, cx + 3.0 * k, cy - 10.0 * k, 5.0 * k, 20.0 * k, 0.5 * k, &bright);
        }
        Icon::Onion => {
            if let Some(path) = PathBuilder::from_circle(cx - 4.5 * k, cy, 7.5 * k) {
                outline(pm, &path, &ink, sw);
            }
            if let Some(path) = PathBuilder::from_circle(cx + 4.5 * k, cy, 7.5 * k) {
                outline(pm, &path, &ink, sw);
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
        Icon::Redo => {
            let s = o(-8.0, 7.0);
            let c1 = o(-8.0, -1.0);
            let c2 = o(-3.0, -5.0);
            let e = o(5.0, -5.0);
            let mut pb = PathBuilder::new();
            pb.move_to(s.0, s.1);
            pb.cubic_to(c1.0, c1.1, c2.0, c2.1, e.0, e.1);
            if let Some(path) = pb.finish() {
                outline(pm, &path, &ink, sw);
            }
            stroke_line(pm, &[o(1.0, -10.0), o(6.0, -5.0), o(1.0, 0.0)], false, &ink, sw);
        }
        Icon::Plus => {
            stroke_line(pm, &[o(-8.0, 0.0), o(8.0, 0.0)], false, &ink, 2.0 * k);
            stroke_line(pm, &[o(0.0, -8.0), o(0.0, 8.0)], false, &ink, 2.0 * k);
        }
        Icon::Minus => {
            stroke_line(pm, &[o(-8.0, 0.0), o(8.0, 0.0)], false, &ink, 2.0 * k);
        }
        Icon::Up => {
            stroke_line(pm, &[o(-7.0, 3.0), o(0.0, -4.0), o(7.0, 3.0)], false, &ink, 2.0 * k);
        }
        Icon::Down => {
            stroke_line(pm, &[o(-7.0, -3.0), o(0.0, 4.0), o(7.0, -3.0)], false, &ink, 2.0 * k);
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
        Icon::Lasso => {
            let pts: Vec<(f32, f32)> = (0..24)
                .map(|n| {
                    let t = n as f32 / 24.0 * std::f32::consts::TAU;
                    o(t.cos() * 11.0, t.sin() * 7.5 - 3.0)
                })
                .collect();
            stroke_line(pm, &pts, true, &ink, sw);
            stroke_line(pm, &[o(-4.0, 4.0), o(-6.0, 8.0), o(-10.0, 11.0)], false, &ink, sw);
        }
        Icon::Free => {
            let corners = [o(-9.0, -9.0), o(9.0, -9.0), o(9.0, 9.0), o(-9.0, 9.0)];
            stroke_line(pm, &corners, true, &ink, sw);
            for (px, py) in corners {
                fill_rrect(pm, px - 2.5 * k, py - 2.5 * k, 5.0 * k, 5.0 * k, 0.0, &ink);
            }
        }
        Icon::Cage => {
            let g = [
                [o(-10.0, -9.0), o(0.0, -11.0), o(10.0, -9.0)],
                [o(-11.0, 0.0), o(0.0, 1.0), o(11.0, -1.0)],
                [o(-10.0, 9.0), o(0.0, 11.0), o(10.0, 10.0)],
            ];
            for j in 0..3 {
                stroke_line(pm, &g[j], false, &ink, sw);
            }
            for i in 0..3 {
                stroke_line(pm, &[g[0][i], g[1][i], g[2][i]], false, &ink, sw);
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
