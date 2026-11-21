use crate::animation::frame::Frame;
use crate::animation::layer::LayerInfo;
use crate::drawing::stroke::{Color, Point, Stroke};
use crate::render::Renderer;
use crate::ui::{Hit, MenuAction, SliderId, Ui, UiView};
use serde::{Deserialize, Serialize};
use std::time::Duration;

const MIN_SIZE: f32 = 1.0;
const MAX_SIZE: f32 = 80.0;
const MIN_OPACITY: f32 = 0.05;

#[derive(Clone, Copy, PartialEq)]
enum Tool {
    Pen,
    Eraser,
}

#[derive(Clone, Copy)]
enum Drag {
    Panel,
    Slider(SliderId),
    Wheel,
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
    playing: bool,
    onion: bool,
    dirty: bool,
    ui: Ui,
    drag: Option<Drag>,
    menu_open: bool,
    picker_open: bool,
    layers_open: bool,
    command: Option<Command>,
    accum: Duration,
    frame_time: Duration,
    tool: Tool,
    hue: f32,
    sat: f32,
    val: f32,
    pen: Brush,
    eraser: Brush,
}

fn first_layer() -> LayerInfo {
    LayerInfo {
        name: "CAMADA 1".to_string(),
        visible: true,
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
            playing: false,
            onion: true,
            dirty: true,
            ui: Ui::new(width, height, scale),
            drag: None,
            menu_open: false,
            picker_open: false,
            layers_open: false,
            command: None,
            accum: Duration::ZERO,
            frame_time: Duration::from_millis(1000 / 12),
            tool: Tool::Pen,
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
            Tool::Pen => self.pen,
            Tool::Eraser => self.eraser,
        }
    }

    fn brush_mut(&mut self) -> &mut Brush {
        match self.tool {
            Tool::Pen => &mut self.pen,
            Tool::Eraser => &mut self.eraser,
        }
    }

    fn pen_color(&self) -> Color {
        let a = (self.pen.opacity.clamp(0.0, 1.0) * 255.0).round() as u8;
        Color::from_hsv(self.hue, self.sat, self.val, a)
    }

    fn view(&self) -> UiView<'_> {
        let brush = self.brush();
        UiView {
            eraser: self.tool == Tool::Eraser,
            playing: self.playing,
            onion: self.onion,
            frames: self.frames.len(),
            current: self.current,
            menu_open: self.menu_open,
            picker_open: self.picker_open,
            layers_open: self.layers_open,
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
            Hit::Dismiss => {
                self.menu_open = false;
                self.picker_open = false;
            }
            Hit::Panel => {}
            Hit::MenuButton => {
                self.menu_open = true;
                self.picker_open = false;
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
            Hit::Play => self.toggle_play(),
            Hit::Onion => self.toggle_onion(),
            Hit::RemoveFrame => self.remove_frame(),
            Hit::AddFrame => self.add_frame(),
            Hit::ToggleLayers => self.layers_open = !self.layers_open,
            Hit::Frame(i) => {
                if i < self.frames.len() {
                    self.current = i;
                }
            }
            Hit::Brush => self.use_pen(),
            Hit::Eraser => self.use_eraser(),
            Hit::ColorButton => {
                self.picker_open = true;
                self.menu_open = false;
            }
            Hit::Undo => self.undo(),
            Hit::Slider(id) => {
                self.drag = Some(Drag::Slider(id));
                self.set_slider(id, x, y);
            }
            Hit::Wheel => {
                self.drag = Some(Drag::Wheel);
                self.set_wheel(x, y);
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

        let brush = self.brush();
        let mut stroke = match self.tool {
            Tool::Pen => Stroke::new(self.pen_color(), brush.size),
            Tool::Eraser => Stroke::eraser(brush.size, brush.opacity),
        };
        stroke.add_point(Point { x, y, pressure });
        self.active = Some(stroke);
        self.dirty = true;
    }

    pub fn pointer_move(&mut self, x: f32, y: f32, pressure: f32) {
        if let Some(drag) = self.drag {
            match drag {
                Drag::Slider(id) => self.set_slider(id, x, y),
                Drag::Wheel => self.set_wheel(x, y),
                Drag::Panel => {}
            }
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
        if self.drag.take().is_some() {
            return;
        }
        if let Some(stroke) = self.active.take() {
            self.frames[self.current].layers[self.active_layer].push(stroke);
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
        let frame = Frame::new(self.layers.len());
        self.frames.insert(self.current + 1, frame);
        self.current += 1;
        self.dirty = true;
    }

    pub fn remove_frame(&mut self) {
        if self.frames.len() > 1 {
            self.frames.remove(self.current);
            if self.current >= self.frames.len() {
                self.current = self.frames.len() - 1;
            }
        } else {
            self.frames[0] = Frame::new(self.layers.len());
        }
        self.dirty = true;
    }

    pub fn add_layer(&mut self) {
        let at = self.active_layer + 1;
        self.layer_counter += 1;
        self.layers.insert(
            at,
            LayerInfo {
                name: format!("CAMADA {}", self.layer_counter),
                visible: true,
            },
        );
        for frame in &mut self.frames {
            frame.layers.insert(at, Vec::new());
        }
        self.active_layer = at;
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
            self.dirty = true;
        }
    }

    pub fn undo(&mut self) {
        self.frames[self.current].layers[self.active_layer].pop();
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
        self.playing = false;
        self.dirty = true;
    }

    pub fn update(&mut self, dt: Duration) {
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

        let view = self.view();
        renderer.draw_ui(&self.ui, &view);
    }
}
