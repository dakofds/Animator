use crate::animation::frame::Frame;
use crate::animation::layer::LayerInfo;
use crate::drawing::stroke::{Color, Point, Stroke};
use crate::render::Renderer;
use crate::transform::{
<<<<<<< HEAD
    apply_affine, apply_cage, bounds_of, regular_lattice, split_stroke_by_polygon, Affine, Bounds, Lattice,
=======
    apply_affine, apply_cage, bounds_of, inside_fraction, regular_lattice, Affine, Bounds, Lattice,
>>>>>>> 0382d5c740de222f462987adb9c98c8dc80d4237
};
use crate::ui::{FrameDragView, Hit, MenuAction, Overlay, SelTool, SliderId, Ui, UiView};
use serde::{Deserialize, Serialize};
use std::time::Duration;

const MIN_SIZE: f32 = 1.0;
const MAX_SIZE: f32 = 80.0;
const MIN_OPACITY: f32 = 0.05;
const LONG_PRESS: Duration = Duration::from_millis(450);

#[derive(Clone, Copy, PartialEq)]
enum Tool {
    Pen,
    Eraser,
    Sel,
}

#[derive(Clone, Copy)]
enum Drag {
    Panel,
    Slider(SliderId),
    Wheel,
    Frame {
        from: usize,
        start_x: f32,
        x: f32,
        moved: bool,
        held: Duration,
    },
}

#[derive(Clone, Copy)]
struct Brush {
    size: f32,
    opacity: f32,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Command {
    Save,
    Open,
    Quit,
}

enum Edit {
    Add {
        frame: usize,
        layer: usize,
        stroke: Option<Stroke>,
    },
    Replace {
        frame: usize,
        layer: usize,
        before: Vec<Stroke>,
        after: Vec<Stroke>,
    },
}

impl Edit {
    fn target(&self) -> (usize, usize) {
        match self {
            Edit::Add { frame, layer, .. } | Edit::Replace { frame, layer, .. } => (*frame, *layer),
        }
    }

    fn set_target(&mut self, f: usize, l: usize) {
        match self {
            Edit::Add { frame, layer, .. } | Edit::Replace { frame, layer, .. } => {
                *frame = f;
                *layer = l;
            }
        }
    }
}

struct Selection {
    frame: usize,
    layer: usize,
    indices: Vec<usize>,
<<<<<<< HEAD
    path: Vec<(f32, f32)>,
=======
>>>>>>> 0382d5c740de222f462987adb9c98c8dc80d4237
}

#[derive(Clone, Copy)]
enum XKind {
    Move,
    Scale {
        hx: f32,
        hy: f32,
        ax: f32,
        ay: f32,
        fx: bool,
        fy: bool,
    },
    Rotate {
        cx: f32,
        cy: f32,
        a0: f32,
    },
    CagePoint {
        i: usize,
        j: usize,
        gx: f32,
        gy: f32,
    },
}

struct Xform {
    kind: XKind,
    frame: usize,
    layer: usize,
    indices: Vec<usize>,
    originals: Vec<Stroke>,
    layer_before: Vec<Stroke>,
    rect: Bounds,
    start: (f32, f32),
    affine: Affine,
    lattice: Lattice,
    moved: bool,
}

#[derive(Serialize)]
struct ProjectRef<'a> {
    layers: &'a [LayerInfo],
    frames: &'a [Frame],
    layer_counter: usize,
}

#[derive(Deserialize)]
struct Project {
    layers: Vec<LayerInfo>,
    frames: Vec<Frame>,
    layer_counter: usize,
}

pub struct App {
    frames: Vec<Frame>,
    layers: Vec<LayerInfo>,
    active_layer: usize,
    layer_counter: usize,
    current: usize,
    active: Option<Stroke>,
    undo_stack: Vec<Edit>,
    redo_stack: Vec<Edit>,
    lasso: Option<Vec<(f32, f32)>>,
    selection: Option<Selection>,
    xform: Option<Xform>,
    playing: bool,
    onion: bool,
    dirty: bool,
    ui: Ui,
    drag: Option<Drag>,
    menu_open: bool,
    picker_open: bool,
    layers_open: bool,
    flyout_open: bool,
    frame_menu: Option<f32>,
    command: Option<Command>,
    accum: Duration,
    frame_time: Duration,
    tool: Tool,
    sel_tool: SelTool,
    hue: f32,
    sat: f32,
    val: f32,
    pen: Brush,
    eraser: Brush,
}

fn first_layer() -> LayerInfo {
    LayerInfo {
        name: "Camada 1".to_string(),
        visible: true,
    }
}

fn scale_ratio(num: f32, den: f32) -> f32 {
    if den.abs() < 1.0 {
        return 1.0;
    }
    let r = num / den;
    if r.abs() < 0.05 {
        0.05f32.copysign(r)
    } else {
        r
    }
}

impl App {
    pub fn new(width: f32, height: f32, scale: f32) -> Self {
        Self {
            frames: vec![Frame::new(1)],
            layers: vec![first_layer()],
            active_layer: 0,
            layer_counter: 1,
            current: 0,
            active: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            lasso: None,
            selection: None,
            xform: None,
            playing: false,
            onion: true,
            dirty: true,
            ui: Ui::new(width, height, scale),
            drag: None,
            menu_open: false,
            picker_open: false,
            layers_open: false,
            flyout_open: false,
            frame_menu: None,
            command: None,
            accum: Duration::ZERO,
            frame_time: Duration::from_millis(1000 / 12),
            tool: Tool::Pen,
            sel_tool: SelTool::Lasso,
            hue: 0.0,
            sat: 0.0,
            val: 0.0,
            pen: Brush { size: 4.0, opacity: 1.0 },
            eraser: Brush { size: 24.0, opacity: 1.0 },
        }
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn take_dirty(&mut self) -> bool {
        std::mem::replace(&mut self.dirty, false)
    }

    pub fn take_command(&mut self) -> Option<Command> {
        self.command.take()
    }

    fn brush(&self) -> Brush {
        match self.tool {
            Tool::Eraser => self.eraser,
            _ => self.pen,
        }
    }

    fn brush_mut(&mut self) -> &mut Brush {
        match self.tool {
            Tool::Eraser => &mut self.eraser,
            _ => &mut self.pen,
        }
    }

    fn pen_color(&self) -> Color {
        let a = (self.pen.opacity.clamp(0.0, 1.0) * 255.0).round() as u8;
        Color::from_hsv(self.hue, self.sat, self.val, a)
    }

    fn close_popups(&mut self) {
        self.menu_open = false;
        self.picker_open = false;
        self.flyout_open = false;
        self.frame_menu = None;
    }

    fn layer_mut(&mut self, frame: usize, layer: usize) -> Option<&mut Vec<Stroke>> {
        self.frames
            .get_mut(frame)
            .and_then(|f| f.layers.get_mut(layer))
    }

    fn focus(&mut self, frame: usize, layer: usize) {
        if frame < self.frames.len() {
            self.current = frame;
        }
        if layer < self.layers.len() {
            self.active_layer = layer;
        }
    }

    fn remap_history(&mut self, map: impl Fn(usize, usize) -> Option<(usize, usize)>) {
        self.selection = None;
        for stack in [&mut self.undo_stack, &mut self.redo_stack] {
            stack.retain_mut(|edit| {
                let (f, l) = edit.target();
                match map(f, l) {
                    Some((nf, nl)) => {
                        edit.set_target(nf, nl);
                        true
                    }
                    None => false,
                }
            });
        }
    }

    fn selection_indices(&self) -> Option<Vec<usize>> {
        let sel = self.selection.as_ref()?;
        if sel.frame != self.current || sel.layer != self.active_layer {
            return None;
        }
        let strokes = self.frames.get(sel.frame)?.layers.get(sel.layer)?;
        if sel.indices.iter().all(|&i| i < strokes.len()) {
            Some(sel.indices.clone())
        } else {
            None
        }
    }

    fn target_rect(&self, selection_only: bool) -> Option<Bounds> {
        let strokes = self.frames.get(self.current)?.layers.get(self.active_layer)?;
        match self.selection_indices() {
            Some(ids) => bounds_of(ids.iter().filter_map(|&i| strokes.get(i))),
            None if !selection_only => bounds_of(strokes.iter()),
            None => None,
        }
    }

    fn overlay(&self) -> Overlay {
        if self.tool != Tool::Sel {
            return Overlay::None;
        }
        if let Some(path) = &self.lasso {
            return Overlay::Lasso(path.clone());
        }
        let u = self.ui.scale;
        match self.sel_tool {
<<<<<<< HEAD
            SelTool::Lasso => match self.selection.as_ref() {
                Some(sel) if sel.frame == self.current && sel.layer == self.active_layer => {
                    Overlay::SelectionPath(sel.path.clone())
                }
                _ => Overlay::None,
=======
            SelTool::Lasso => match self.target_rect(true) {
                Some(r) => Overlay::Selection(r),
                None => Overlay::None,
>>>>>>> 0382d5c740de222f462987adb9c98c8dc80d4237
            },
            SelTool::Free => {
                let (rect, aff) = match &self.xform {
                    Some(xf) => (xf.rect, xf.affine),
                    None => match self.target_rect(false) {
                        Some(r) => (r, Affine::identity()),
                        None => return Overlay::None,
                    },
                };
                let (x0, y0, x1, y1) = rect;
                let tl = aff.apply(x0, y0);
                let tr = aff.apply(x1, y0);
                let br = aff.apply(x1, y1);
                let bl = aff.apply(x0, y1);
                let top = aff.apply((x0 + x1) / 2.0, y0);
                let center = aff.apply((x0 + x1) / 2.0, (y0 + y1) / 2.0);
                let dx = top.0 - center.0;
                let dy = top.1 - center.1;
                let len = dx.hypot(dy);
                let (nx, ny) = if len > 0.001 {
                    (dx / len, dy / len)
                } else {
                    (0.0, -1.0)
                };
                let rot = (top.0 + nx * 28.0 * u, top.1 + ny * 28.0 * u);
                Overlay::Free {
                    quad: [tl, tr, br, bl],
                    rot,
                }
            }
            SelTool::Cage => match &self.xform {
                Some(xf) => Overlay::Cage(xf.lattice),
                None => match self.target_rect(false) {
                    Some(r) => Overlay::Cage(regular_lattice(r)),
                    None => Overlay::None,
                },
            },
        }
    }

    fn view(&self) -> UiView<'_> {
        let brush = self.brush();
        let frame_drag = match self.drag {
            Some(Drag::Frame { from, x, moved: true, .. }) => Some(FrameDragView {
                from,
                target: self.ui.frame_target(self.frames.len(), self.current, x),
                x,
            }),
            _ => None,
        };
        UiView {
            eraser: self.tool == Tool::Eraser,
            pen_active: self.tool == Tool::Pen,
            sel_active: self.tool == Tool::Sel,
            sel_tool: self.sel_tool,
            playing: self.playing,
            onion: self.onion,
            frames: self.frames.len(),
            current: self.current,
            can_undo: !self.undo_stack.is_empty(),
            can_redo: !self.redo_stack.is_empty(),
            menu_open: self.menu_open,
            picker_open: self.picker_open,
            layers_open: self.layers_open,
            flyout_open: self.flyout_open,
            frame_menu: self.frame_menu,
            frame_drag,
            hue: self.hue,
            sat: self.sat,
            val: self.val,
            size_t: ((brush.size - MIN_SIZE) / (MAX_SIZE - MIN_SIZE)).max(0.0).sqrt(),
            opacity_t: (brush.opacity - MIN_OPACITY) / (1.0 - MIN_OPACITY),
            brush_size: brush.size,
            opacity: self.pen.opacity,
            layers: &self.layers,
            active_layer: self.active_layer,
        }
    }

    fn set_slider(&mut self, id: SliderId, x: f32, y: f32) {
        let t = self.ui.slider(id).t_at(x, y);
        match id {
            SliderId::Size => {
                self.brush_mut().size = MIN_SIZE + (MAX_SIZE - MIN_SIZE) * t * t;
            }
            SliderId::Opacity => {
                self.brush_mut().opacity = MIN_OPACITY + (1.0 - MIN_OPACITY) * t;
            }
            SliderId::Value => {
                self.val = t;
            }
        }
        self.dirty = true;
    }

    fn set_wheel(&mut self, x: f32, y: f32) {
        let (hue, sat) = self.ui.picker.hue_sat(x, y);
        self.hue = hue;
        self.sat = sat;
        if self.val < 0.05 {
            self.val = 1.0;
        }
        self.tool = Tool::Pen;
        self.dirty = true;
    }

    fn handle_hit(&mut self, hit: Hit, x: f32, y: f32) {
        self.drag = Some(Drag::Panel);
        match hit {
            Hit::Dismiss => self.close_popups(),
            Hit::Panel => {}
            Hit::MenuButton => {
                self.close_popups();
                self.menu_open = true;
            }
            Hit::Menu(action) => {
                self.menu_open = false;
                match action {
                    MenuAction::New => self.new_project(),
                    MenuAction::Open => self.command = Some(Command::Open),
                    MenuAction::Save => self.command = Some(Command::Save),
                    MenuAction::Quit => self.command = Some(Command::Quit),
                }
            }
            Hit::Undo => self.undo(),
            Hit::Redo => self.redo(),
            Hit::Play => self.toggle_play(),
            Hit::Onion => self.toggle_onion(),
            Hit::AddFrame => self.add_frame(),
            Hit::ToggleLayers => self.layers_open = !self.layers_open,
            Hit::Frame(i) => {
                if i < self.frames.len() {
                    self.current = i;
                    self.drag = Some(Drag::Frame {
                        from: i,
                        start_x: x,
                        x,
                        moved: false,
                        held: Duration::ZERO,
                    });
                }
            }
            Hit::Brush => self.use_pen(),
            Hit::SelButton => {
                self.close_popups();
                self.tool = Tool::Sel;
                self.flyout_open = true;
            }
            Hit::SelTool(t) => {
                self.sel_tool = t;
                self.tool = Tool::Sel;
                self.flyout_open = false;
            }
            Hit::Eraser => self.use_eraser(),
            Hit::ColorButton => {
                self.close_popups();
                self.picker_open = true;
            }
            Hit::Slider(id) => {
                self.drag = Some(Drag::Slider(id));
                self.set_slider(id, x, y);
            }
            Hit::Wheel => {
                self.drag = Some(Drag::Wheel);
                self.set_wheel(x, y);
            }
            Hit::FrameDuplicate => {
                self.frame_menu = None;
                self.duplicate_frame();
            }
            Hit::FrameDelete => {
                self.frame_menu = None;
                self.remove_frame();
            }
            Hit::LayerSelect(i) => {
                if i < self.layers.len() {
                    self.active_layer = i;
                }
            }
            Hit::LayerEye(i) => {
                if let Some(layer) = self.layers.get_mut(i) {
                    layer.visible = !layer.visible;
                }
            }
            Hit::LayerAdd => self.add_layer(),
            Hit::LayerRemove => self.remove_layer(),
            Hit::LayerUp => self.move_layer_up(),
            Hit::LayerDown => self.move_layer_down(),
        }
        self.dirty = true;
    }

    pub fn pointer_down(&mut self, x: f32, y: f32, pressure: f32) {
        let hit = {
            let view = self.view();
            self.ui.hit(x, y, &view)
        };
        if let Some(hit) = hit {
            self.handle_hit(hit, x, y);
            return;
        }
        if self.playing {
            return;
        }
        if !self.layers[self.active_layer].visible {
            return;
        }

        if self.tool == Tool::Sel {
            match self.sel_tool {
                SelTool::Lasso => {
                    self.selection = None;
                    self.lasso = Some(vec![(x, y)]);
                }
                SelTool::Free | SelTool::Cage => self.begin_xform(x, y),
            }
            self.dirty = true;
            return;
        }

        let brush = self.brush();
        let mut stroke = match self.tool {
            Tool::Eraser => Stroke::eraser(brush.size, brush.opacity),
            _ => Stroke::new(self.pen_color(), brush.size),
        };
        stroke.add_point(Point { x, y, pressure });
        self.active = Some(stroke);
        self.dirty = true;
    }

    fn begin_xform(&mut self, x: f32, y: f32) {
        let frame = self.current;
        let layer = self.active_layer;
        let Some(layer_before) = self
            .frames
            .get(frame)
            .and_then(|f| f.layers.get(layer))
            .cloned()
        else {
            return;
        };
        let indices = match self.selection_indices() {
            Some(i) => i,
            None => (0..layer_before.len()).collect(),
        };
        if indices.is_empty() {
            return;
        }
        let originals: Vec<Stroke> = indices.iter().map(|&i| layer_before[i].clone()).collect();
        let Some(rect) = bounds_of(originals.iter()) else {
            return;
        };

        let u = self.ui.scale;
        let reach = 16.0 * u;
        let (x0, y0, x1, y1) = rect;
        let mx = (x0 + x1) / 2.0;
        let my = (y0 + y1) / 2.0;
        let near = |px: f32, py: f32| (x - px).hypot(y - py) <= reach;
        let lattice = regular_lattice(rect);
        let mut kind: Option<XKind> = None;

        match self.sel_tool {
            SelTool::Free => {
                if near(mx, y0 - 28.0 * u) {
                    kind = Some(XKind::Rotate {
                        cx: mx,
                        cy: my,
                        a0: (y - my).atan2(x - mx),
                    });
                } else {
                    let handles = [
                        (x0, y0),
                        (mx, y0),
                        (x1, y0),
                        (x1, my),
                        (x1, y1),
                        (mx, y1),
                        (x0, y1),
                        (x0, my),
                    ];
                    for (hx, hy) in handles {
                        if near(hx, hy) {
                            kind = Some(XKind::Scale {
                                hx,
                                hy,
                                ax: x0 + x1 - hx,
                                ay: y0 + y1 - hy,
                                fx: (hx - mx).abs() > 0.5,
                                fy: (hy - my).abs() > 0.5,
                            });
                            break;
                        }
                    }
                    if kind.is_none() && x >= x0 && x <= x1 && y >= y0 && y <= y1 {
                        kind = Some(XKind::Move);
                    }
                }
            }
            SelTool::Cage => {
                let mut best: Option<(usize, usize, f32)> = None;
                for j in 0..3 {
                    for i in 0..3 {
                        let (px, py) = lattice[j][i];
                        let d = (x - px).hypot(y - py);
                        if d <= reach && best.map_or(true, |b| d < b.2) {
                            best = Some((i, j, d));
                        }
                    }
                }
                if let Some((i, j, _)) = best {
                    let (px, py) = lattice[j][i];
                    kind = Some(XKind::CagePoint {
                        i,
                        j,
                        gx: px - x,
                        gy: py - y,
                    });
                }
            }
            SelTool::Lasso => {}
        }

        let Some(kind) = kind else {
            self.selection = None;
            return;
        };

        self.xform = Some(Xform {
            kind,
            frame,
            layer,
            indices,
            originals,
            layer_before,
            rect,
            start: (x, y),
            affine: Affine::identity(),
            lattice,
            moved: false,
        });
    }

    fn update_xform(&mut self, x: f32, y: f32) {
        let Some(xf) = self.xform.as_mut() else {
            return;
        };
        xf.moved = true;
        match xf.kind {
            XKind::Move => {
                xf.affine = Affine::translate(x - xf.start.0, y - xf.start.1);
            }
            XKind::Scale { hx, hy, ax, ay, fx, fy } => {
                let sx = if fx { scale_ratio(x - ax, hx - ax) } else { 1.0 };
                let sy = if fy { scale_ratio(y - ay, hy - ay) } else { 1.0 };
                xf.affine = Affine::scale_about(ax, ay, sx, sy);
            }
            XKind::Rotate { cx, cy, a0 } => {
                xf.affine = Affine::rotate_about(cx, cy, (y - cy).atan2(x - cx) - a0);
            }
            XKind::CagePoint { i, j, gx, gy } => {
                xf.lattice[j][i] = (x + gx, y + gy);
            }
        }
        let new_strokes = match xf.kind {
            XKind::CagePoint { .. } => apply_cage(&xf.originals, xf.rect, &xf.lattice),
            _ => apply_affine(&xf.originals, xf.affine),
        };
        let frame = xf.frame;
        let layer = xf.layer;
        let indices = xf.indices.clone();
        if let Some(strokes) = self.layer_mut(frame, layer) {
            for (idx, s) in indices.into_iter().zip(new_strokes) {
                if idx < strokes.len() {
                    strokes[idx] = s;
                }
            }
        }
        self.dirty = true;
    }

    fn finish_xform(&mut self, xf: Xform) {
        if xf.moved {
            let frame = xf.frame;
            let layer = xf.layer;
            let after = self
                .frames
                .get(frame)
                .and_then(|f| f.layers.get(layer))
                .cloned();
            if let Some(after) = after {
                self.undo_stack.push(Edit::Replace {
                    frame,
                    layer,
                    before: xf.layer_before,
                    after,
                });
                self.redo_stack.clear();
            }
        }
        self.dirty = true;
    }

    fn finish_lasso(&mut self, path: Vec<(f32, f32)>) {
        self.selection = None;
        if path.len() < 3 {
            return;
        }
<<<<<<< HEAD

        let frame = self.current;
        let layer = self.active_layer;
        let Some(before) = self
            .frames
            .get(frame)
            .and_then(|f| f.layers.get(layer))
            .cloned()
        else {
            return;
        };

        // Rebuild the active layer using clipped stroke fragments. Fragments
        // stay on the SAME layer; the lasso never creates new layers.
        let mut after = Vec::new();
        let mut selected_indices = Vec::new();
        for stroke in &before {
            for (fragment, inside) in split_stroke_by_polygon(stroke, &path) {
                let idx = after.len();
                after.push(fragment);
                if inside {
                    selected_indices.push(idx);
                }
            }
        }

        if selected_indices.is_empty() {
            return;
        }

        if let Some(strokes) = self.layer_mut(frame, layer) {
            *strokes = after.clone();
        } else {
            return;
        }

        self.undo_stack.push(Edit::Replace {
            frame,
            layer,
            before,
            after,
        });
        self.redo_stack.clear();
        self.selection = Some(Selection {
            frame,
            layer,
            indices: selected_indices,
            path,
        });
        self.dirty = true;
=======
        let indices: Vec<usize> = {
            let Some(strokes) = self
                .frames
                .get(self.current)
                .and_then(|f| f.layers.get(self.active_layer))
            else {
                return;
            };
            strokes
                .iter()
                .enumerate()
                .filter(|(_, s)| inside_fraction(s, &path) >= 0.5)
                .map(|(i, _)| i)
                .collect()
        };
        if !indices.is_empty() {
            self.selection = Some(Selection {
                frame: self.current,
                layer: self.active_layer,
                indices,
            });
        }
>>>>>>> 0382d5c740de222f462987adb9c98c8dc80d4237
    }

    pub fn pointer_move(&mut self, x: f32, y: f32, pressure: f32) {
        if let Some(drag) = self.drag {
            match drag {
                Drag::Slider(id) => self.set_slider(id, x, y),
                Drag::Wheel => self.set_wheel(x, y),
                Drag::Frame { from, start_x, moved, held, .. } => {
                    let threshold = 8.0 * self.ui.scale;
                    let moved = moved || (x - start_x).abs() > threshold;
                    self.drag = Some(Drag::Frame { from, start_x, x, moved, held });
                    if moved {
                        self.dirty = true;
                    }
                }
                Drag::Panel => {}
            }
            return;
        }
        if let Some(path) = self.lasso.as_mut() {
            if path.last() != Some(&(x, y)) {
                path.push((x, y));
                self.dirty = true;
            }
            return;
        }
        if self.xform.is_some() {
            self.update_xform(x, y);
            return;
        }
        if let Some(stroke) = self.active.as_mut() {
            if let Some(last) = stroke.points.last() {
                if last.x == x && last.y == y {
                    return;
                }
            }
            stroke.add_point(Point { x, y, pressure });
            self.dirty = true;
        }
    }

    pub fn pointer_up(&mut self) {
        if let Some(drag) = self.drag.take() {
            if let Drag::Frame { from, x, moved: true, .. } = drag {
                let to = self.ui.frame_target(self.frames.len(), self.current, x);
                self.move_frame(from, to);
            }
            self.dirty = true;
            return;
        }
        if let Some(path) = self.lasso.take() {
            self.finish_lasso(path);
            self.dirty = true;
            return;
        }
        if let Some(xf) = self.xform.take() {
            self.finish_xform(xf);
            return;
        }
        if let Some(stroke) = self.active.take() {
            let frame = self.current;
            let layer = self.active_layer;
            self.frames[frame].layers[layer].push(stroke);
            self.undo_stack.push(Edit::Add {
                frame,
                layer,
                stroke: None,
            });
            self.redo_stack.clear();
            self.selection = None;
            self.dirty = true;
        }
    }

    pub fn use_pen(&mut self) {
        self.tool = Tool::Pen;
        self.dirty = true;
    }

    pub fn use_eraser(&mut self) {
        self.tool = Tool::Eraser;
        self.dirty = true;
    }

    pub fn change_width(&mut self, delta: f32) {
        let brush = self.brush_mut();
        brush.size = (brush.size + delta).clamp(MIN_SIZE, MAX_SIZE);
        self.dirty = true;
    }

    pub fn next_frame(&mut self) {
        if self.current + 1 < self.frames.len() {
            self.current += 1;
            self.dirty = true;
        }
    }

    pub fn prev_frame(&mut self) {
        if self.current > 0 {
            self.current -= 1;
            self.dirty = true;
        }
    }

    pub fn add_frame(&mut self) {
        self.frames.push(Frame::new(self.layers.len()));
        self.current = self.frames.len() - 1;
        self.dirty = true;
    }

    pub fn duplicate_frame(&mut self) {
        let c = self.current;
        let copy = self.frames[c].clone();
        self.frames.insert(c + 1, copy);
        self.current = c + 1;
        self.remap_history(move |f, l| Some((if f > c { f + 1 } else { f }, l)));
        self.dirty = true;
    }

    pub fn remove_frame(&mut self) {
        if self.frames.len() > 1 {
            let c = self.current;
            self.frames.remove(c);
            if self.current >= self.frames.len() {
                self.current = self.frames.len() - 1;
            }
            self.remap_history(move |f, l| {
                if f == c {
                    None
                } else if f > c {
                    Some((f - 1, l))
                } else {
                    Some((f, l))
                }
            });
        } else {
            self.frames[0] = Frame::new(self.layers.len());
            self.undo_stack.clear();
            self.redo_stack.clear();
            self.selection = None;
        }
        self.dirty = true;
    }

    pub fn move_frame(&mut self, from: usize, to: usize) {
        if from == to || from >= self.frames.len() || to >= self.frames.len() {
            return;
        }
        let frame = self.frames.remove(from);
        self.frames.insert(to, frame);
        self.current = to;
        self.remap_history(move |f, l| {
            let nf = if f == from {
                to
            } else if from < to && f > from && f <= to {
                f - 1
            } else if from > to && f >= to && f < from {
                f + 1
            } else {
                f
            };
            Some((nf, l))
        });
        self.dirty = true;
    }

    pub fn add_layer(&mut self) {
        let at = self.active_layer + 1;
        self.layer_counter += 1;
        self.layers.insert(
            at,
            LayerInfo {
                name: format!("Camada {}", self.layer_counter),
                visible: true,
            },
        );
        for frame in &mut self.frames {
            frame.layers.insert(at, Vec::new());
        }
        self.active_layer = at;
        self.remap_history(move |f, l| Some((f, if l >= at { l + 1 } else { l })));
        self.dirty = true;
    }

    pub fn remove_layer(&mut self) {
        if self.layers.len() <= 1 {
            return;
        }
        let i = self.active_layer;
        self.layers.remove(i);
        for frame in &mut self.frames {
            frame.layers.remove(i);
        }
        self.active_layer = i.min(self.layers.len() - 1);
        self.remap_history(move |f, l| {
            if l == i {
                None
            } else if l > i {
                Some((f, l - 1))
            } else {
                Some((f, l))
            }
        });
        self.dirty = true;
    }

    pub fn move_layer_up(&mut self) {
        let i = self.active_layer;
        if i + 1 < self.layers.len() {
            self.layers.swap(i, i + 1);
            for frame in &mut self.frames {
                frame.layers.swap(i, i + 1);
            }
            self.active_layer = i + 1;
            self.remap_history(move |f, l| {
                Some((f, if l == i { i + 1 } else if l == i + 1 { i } else { l }))
            });
            self.dirty = true;
        }
    }

    pub fn move_layer_down(&mut self) {
        let i = self.active_layer;
        if i > 0 {
            self.layers.swap(i, i - 1);
            for frame in &mut self.frames {
                frame.layers.swap(i, i - 1);
            }
            self.active_layer = i - 1;
            self.remap_history(move |f, l| {
                Some((f, if l == i { i - 1 } else if l == i - 1 { i } else { l }))
            });
            self.dirty = true;
        }
    }

    pub fn undo(&mut self) {
        let Some(edit) = self.undo_stack.pop() else {
            return;
        };
        self.selection = None;
        match edit {
            Edit::Add { frame, layer, .. } => {
                let popped = self.layer_mut(frame, layer).and_then(|s| s.pop());
                if let Some(stroke) = popped {
                    self.redo_stack.push(Edit::Add {
                        frame,
                        layer,
                        stroke: Some(stroke),
                    });
                }
                self.focus(frame, layer);
            }
            Edit::Replace { frame, layer, before, after } => {
                if let Some(strokes) = self.layer_mut(frame, layer) {
                    *strokes = before.clone();
                }
                self.redo_stack.push(Edit::Replace { frame, layer, before, after });
                self.focus(frame, layer);
            }
        }
        self.dirty = true;
    }

    pub fn redo(&mut self) {
        let Some(edit) = self.redo_stack.pop() else {
            return;
        };
        self.selection = None;
        match edit {
            Edit::Add { frame, layer, stroke } => {
                if let Some(stroke) = stroke {
                    if let Some(strokes) = self.layer_mut(frame, layer) {
                        strokes.push(stroke);
                    }
                }
                self.undo_stack.push(Edit::Add {
                    frame,
                    layer,
                    stroke: None,
                });
                self.focus(frame, layer);
            }
            Edit::Replace { frame, layer, before, after } => {
                if let Some(strokes) = self.layer_mut(frame, layer) {
                    *strokes = after.clone();
                }
                self.undo_stack.push(Edit::Replace { frame, layer, before, after });
                self.focus(frame, layer);
            }
        }
        self.dirty = true;
    }

    pub fn toggle_play(&mut self) {
        self.playing = !self.playing;
        self.accum = Duration::ZERO;
        self.dirty = true;
    }

    pub fn toggle_onion(&mut self) {
        self.onion = !self.onion;
        self.dirty = true;
    }

    pub fn new_project(&mut self) {
        self.layers = vec![first_layer()];
        self.layer_counter = 1;
        self.active_layer = 0;
        self.frames = vec![Frame::new(1)];
        self.current = 0;
        self.active = None;
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.selection = None;
        self.lasso = None;
        self.xform = None;
        self.playing = false;
        self.dirty = true;
    }

    pub fn update(&mut self, dt: Duration) {
        if let Some(Drag::Frame { from, start_x, x, moved: false, held }) = self.drag {
            let held = held + dt;
            if held >= LONG_PRESS {
                self.frame_menu = Some(start_x);
                self.drag = Some(Drag::Panel);
                self.dirty = true;
            } else {
                self.drag = Some(Drag::Frame { from, start_x, x, moved: false, held });
            }
        }

        if !self.playing {
            return;
        }
        let before = self.current;
        self.accum += dt;
        while self.accum >= self.frame_time {
            self.accum -= self.frame_time;
            self.current = (self.current + 1) % self.frames.len();
        }
        if self.current != before {
            self.dirty = true;
        }
    }

    pub fn save(&self, path: &str) -> Result<(), String> {
        let project = ProjectRef {
            layers: &self.layers,
            frames: &self.frames,
            layer_counter: self.layer_counter,
        };
        let json = serde_json::to_string(&project).map_err(|e| e.to_string())?;
        std::fs::write(path, json).map_err(|e| e.to_string())
    }

    pub fn load(&mut self, path: &str) -> Result<(), String> {
        let json = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        let project: Project = serde_json::from_str(&json).map_err(|e| e.to_string())?;
        let valid = !project.layers.is_empty()
            && !project.frames.is_empty()
            && project
                .frames
                .iter()
                .all(|f| f.layers.len() == project.layers.len());
        if !valid {
            return Err("projeto invalido".to_string());
        }
        self.layers = project.layers;
        self.frames = project.frames;
        self.layer_counter = project.layer_counter.max(self.layers.len());
        self.current = 0;
        self.active_layer = 0;
        self.active = None;
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.selection = None;
        self.lasso = None;
        self.xform = None;
        self.dirty = true;
        Ok(())
    }

    fn draw_frame_layers(
        &self,
        renderer: &mut Renderer,
        index: usize,
        with_active: bool,
        tint: Option<(Color, f32)>,
    ) {
        for (li, info) in self.layers.iter().enumerate() {
            if !info.visible {
                continue;
            }
            let active = if with_active && li == self.active_layer {
                self.active.as_ref()
            } else {
                None
            };
            renderer.draw_layer(&self.frames[index].layers[li], active, tint);
        }
    }

    pub fn render(&self, renderer: &mut Renderer) {
        renderer.clear();

        if self.onion && !self.playing {
            if self.current > 0 {
                let red = Color { r: 230, g: 60, b: 60, a: 255 };
                self.draw_frame_layers(renderer, self.current - 1, false, Some((red, 0.28)));
            }
            if self.current + 1 < self.frames.len() {
                let green = Color { r: 60, g: 180, b: 90, a: 255 };
                self.draw_frame_layers(renderer, self.current + 1, false, Some((green, 0.28)));
            }
        }

        self.draw_frame_layers(renderer, self.current, true, None);

        let overlay = self.overlay();
        renderer.draw_overlay(&overlay, self.ui.scale);

        let view = self.view();
        renderer.draw_ui(&self.ui, &view);
    }
}
