pub mod text;

use crate::animation::layer::LayerInfo;

#[derive(Clone, Copy, Debug)]
pub struct Area {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Area {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px < self.x + self.w && py >= self.y && py < self.y + self.h
    }

    pub fn cx(&self) -> f32 {
        self.x + self.w / 2.0
    }

    pub fn cy(&self) -> f32 {
        self.y + self.h / 2.0
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum MenuAction {
    New,
    Open,
    Save,
    Quit,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SliderId {
    Size,
    Opacity,
    Value,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SelTool {
    Lasso,
    Free,
    Cage,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Hit {
    Dismiss,
    Panel,
    MenuButton,
    Undo,
    Redo,
    Play,
    Onion,
    AddFrame,
    ToggleLayers,
    Frame(usize),
    Brush,
    SelButton,
    SelTool(SelTool),
    Eraser,
    ColorButton,
    Slider(SliderId),
    Menu(MenuAction),
    Wheel,
    FrameDuplicate,
    FrameDelete,
    LayerSelect(usize),
    LayerEye(usize),
    LayerAdd,
    LayerRemove,
    LayerUp,
    LayerDown,
}

#[derive(Clone)]
pub enum Overlay {
    None,
    Lasso(Vec<(f32, f32)>),
    Selection((f32, f32, f32, f32)),
    Free { quad: [(f32, f32); 4], rot: (f32, f32) },
    Cage([[(f32, f32); 3]; 3]),
}

#[derive(Clone, Copy)]
pub struct Slider {
    pub id: SliderId,
    pub area: Area,
    pub edge: f32,
    pub vertical: bool,
}

impl Slider {
    pub fn start(&self) -> (f32, f32) {
        if self.vertical {
            (self.area.cx(), self.area.y + self.area.h - self.edge)
        } else {
            (self.area.x + self.edge, self.area.cy())
        }
    }

    pub fn end(&self) -> (f32, f32) {
        if self.vertical {
            (self.area.cx(), self.area.y + self.edge)
        } else {
            (self.area.x + self.area.w - self.edge, self.area.cy())
        }
    }

    pub fn point_at(&self, t: f32) -> (f32, f32) {
        let (sx, sy) = self.start();
        let (ex, ey) = self.end();
        (sx + (ex - sx) * t, sy + (ey - sy) * t)
    }

    pub fn t_at(&self, px: f32, py: f32) -> f32 {
        let (sx, sy) = self.start();
        let (ex, ey) = self.end();
        let dx = ex - sx;
        let dy = ey - sy;
        let len2 = dx * dx + dy * dy;
        if len2 <= 0.0 {
            return 0.0;
        }
        (((px - sx) * dx + (py - sy) * dy) / len2).clamp(0.0, 1.0)
    }
}

#[derive(Clone, Copy)]
pub struct Picker {
    pub panel: Area,
    pub cx: f32,
    pub cy: f32,
    pub r: f32,
    pub value: Slider,
}

impl Picker {
    pub fn hue_sat(&self, px: f32, py: f32) -> (f32, f32) {
        let dx = px - self.cx;
        let dy = py - self.cy;
        let hue = dy.atan2(dx).to_degrees().rem_euclid(360.0);
        let sat = ((dx * dx + dy * dy).sqrt() / self.r).min(1.0);
        (hue, sat)
    }
}

#[derive(Clone, Copy)]
pub struct MenuItem {
    pub area: Area,
    pub action: Option<MenuAction>,
    pub label: &'static str,
}

#[derive(Clone, Copy)]
pub struct LayerRow {
    pub index: usize,
    pub area: Area,
    pub eye: Area,
}

#[derive(Clone, Copy)]
pub struct FrameDragView {
    pub from: usize,
    pub target: usize,
    pub x: f32,
}

pub struct Timeline {
    pub cells: Vec<(usize, Area)>,
    pub add: Area,
}

pub struct UiView<'a> {
    pub eraser: bool,
    pub pen_active: bool,
    pub sel_active: bool,
    pub sel_tool: SelTool,
    pub playing: bool,
    pub onion: bool,
    pub frames: usize,
    pub current: usize,
    pub can_undo: bool,
    pub can_redo: bool,
    pub menu_open: bool,
    pub picker_open: bool,
    pub layers_open: bool,
    pub flyout_open: bool,
    pub frame_menu: Option<f32>,
    pub frame_drag: Option<FrameDragView>,
    pub hue: f32,
    pub sat: f32,
    pub val: f32,
    pub size_t: f32,
    pub opacity_t: f32,
    pub brush_size: f32,
    pub opacity: f32,
    pub layers: &'a [LayerInfo],
    pub active_layer: usize,
}

pub struct Ui {
    pub scale: f32,
    pub width: f32,
    pub top_h: f32,
    pub side_w: f32,
    pub separators: [f32; 4],
    pub menu_btn: Area,
    pub undo_btn: Area,
    pub redo_btn: Area,
    pub play: Area,
    pub layers_btn: Area,
    pub timeline_area: Area,
    pub brush: Area,
    pub sel_btn: Area,
    pub eraser: Area,
    pub color: Area,
    pub preview: Area,
    pub onion_btn: Area,
    pub size_slider: Slider,
    pub opacity_slider: Slider,
    pub menu_panel: Area,
    pub menu_items: Vec<MenuItem>,
    pub flyout_panel: Area,
    pub flyout_items: Vec<(SelTool, &'static str, Area)>,
    pub picker: Picker,
    pub layers_panel: Area,
}

impl Ui {
    pub fn new(width: f32, height: f32, scale: f32) -> Self {
        let u = scale;
        let top_h = 52.0 * u;
        let pad = 8.0 * u;
        let gap = 4.0 * u;
        let btn = 40.0 * u;
        let group = 12.0 * u;
        let by = (top_h - btn) / 2.0;

        let mut x = pad;
        let menu_btn = Area::new(x, by, btn, btn);
        x += btn + group;
        let sep0 = x - group / 2.0;
        let undo_btn = Area::new(x, by, btn, btn);
        x += btn + gap;
        let redo_btn = Area::new(x, by, btn, btn);
        x += btn + group;
        let sep1 = x - group / 2.0;
        let play = Area::new(x, by, btn, btn);
        x += btn + group;
        let sep2 = x - group / 2.0;

        let layers_btn = Area::new(width - pad - btn, by, btn, btn);
        let sep3 = layers_btn.x - group / 2.0;
        let timeline_area = Area::new(x, 0.0, (layers_btn.x - group - x).max(0.0), top_h);

        let side_w = 64.0 * u;
        let sb = 44.0 * u;
        let sbx = (side_w - sb) / 2.0;
        let mut y = top_h + 8.0 * u;
        let brush = Area::new(sbx, y, sb, sb);
        y += sb + gap;
        let sel_btn = Area::new(sbx, y, sb, sb);
        y += sb + gap;
        let eraser = Area::new(sbx, y, sb, sb);
        y += sb + gap;
        let color = Area::new(sbx, y, sb, sb);
        y += sb + gap;
        let pv = 36.0 * u;
        let preview = Area::new((side_w - pv) / 2.0, y, pv, pv);
        y += pv + gap * 2.0;
        let onion_btn = Area::new(sbx, height - 8.0 * u - sb, sb, sb);

        let sliders_top = y;
        let sliders_h = (onion_btn.y - gap - sliders_top).max(80.0 * u);
        let sw = 28.0 * u;
        let sx = (side_w - 2.0 * sw) / 2.0;
        let edge = 18.0 * u;
        let size_slider = Slider {
            id: SliderId::Size,
            area: Area::new(sx, sliders_top, sw, sliders_h),
            edge,
            vertical: true,
        };
        let opacity_slider = Slider {
            id: SliderId::Opacity,
            area: Area::new(sx + sw, sliders_top, sw, sliders_h),
            edge,
            vertical: true,
        };

        let fw = 236.0 * u;
        let fi_h = 40.0 * u;
        let fpad = 4.0 * u;
        let flyout_panel = Area::new(side_w + 4.0 * u, sel_btn.y, fw, 3.0 * fi_h + 2.0 * fpad);
        let tools: [(SelTool, &'static str); 3] = [
            (SelTool::Lasso, "Laço"),
            (SelTool::Free, "Transformação livre"),
            (SelTool::Cage, "Transformação em gaiola"),
        ];
        let flyout_items: Vec<(SelTool, &'static str, Area)> = tools
            .iter()
            .enumerate()
            .map(|(i, (tool, label))| {
                (
                    *tool,
                    *label,
                    Area::new(
                        flyout_panel.x + fpad,
                        flyout_panel.y + fpad + i as f32 * fi_h,
                        fw - 2.0 * fpad,
                        fi_h,
                    ),
                )
            })
            .collect();

        let entries: [(Option<MenuAction>, &'static str); 6] = [
            (Some(MenuAction::New), "Novo"),
            (Some(MenuAction::Open), "Abrir"),
            (Some(MenuAction::Save), "Salvar"),
            (None, "Importar imagem"),
            (None, "Exportar"),
            (Some(MenuAction::Quit), "Sair"),
        ];
        let item_h = 38.0 * u;
        let menu_w = 200.0 * u;
        let mpad = 4.0 * u;
        let menu_panel = Area::new(
            menu_btn.x,
            top_h + 2.0 * u,
            menu_w,
            entries.len() as f32 * item_h + 2.0 * mpad,
        );
        let menu_items: Vec<MenuItem> = entries
            .iter()
            .enumerate()
            .map(|(i, (action, label))| MenuItem {
                area: Area::new(
                    menu_panel.x + mpad,
                    menu_panel.y + mpad + i as f32 * item_h,
                    menu_w - 2.0 * mpad,
                    item_h,
                ),
                action: *action,
                label: *label,
            })
            .collect();

        let ppad = 12.0 * u;
        let vh = 30.0 * u;
        let avail = height - top_h - 16.0 * u;
        let r = (96.0 * u)
            .min((avail - 2.0 * ppad - 8.0 * u - vh) / 2.0)
            .max(40.0 * u);
        let pw = 2.0 * r + 2.0 * ppad;
        let ph = ppad + 2.0 * r + 8.0 * u + vh + ppad;
        let px = side_w + 6.0 * u;
        let min_y = top_h + 8.0 * u;
        let max_y = (height - ph - 8.0 * u).max(min_y);
        let py = (color.y - 20.0 * u).clamp(min_y, max_y);
        let pcx = px + pw / 2.0;
        let pcy = py + ppad + r;
        let picker = Picker {
            panel: Area::new(px, py, pw, ph),
            cx: pcx,
            cy: pcy,
            r,
            value: Slider {
                id: SliderId::Value,
                area: Area::new(px + ppad, pcy + r + 8.0 * u, 2.0 * r, vh),
                edge: 10.0 * u,
                vertical: false,
            },
        };

        let lp_w = 200.0 * u;
        let layers_panel = Area::new(width - lp_w, top_h, lp_w, height - top_h);

        Self {
            scale,
            width,
            top_h,
            side_w,
            separators: [sep0, sep1, sep2, sep3],
            menu_btn,
            undo_btn,
            redo_btn,
            play,
            layers_btn,
            timeline_area,
            brush,
            sel_btn,
            eraser,
            color,
            preview,
            onion_btn,
            size_slider,
            opacity_slider,
            menu_panel,
            menu_items,
            flyout_panel,
            flyout_items,
            picker,
            layers_panel,
        }
    }

    pub fn slider(&self, id: SliderId) -> &Slider {
        match id {
            SliderId::Size => &self.size_slider,
            SliderId::Opacity => &self.opacity_slider,
            SliderId::Value => &self.picker.value,
        }
    }

    pub fn timeline(&self, total: usize, current: usize) -> Timeline {
        let u = self.scale;
        let cw = 20.0 * u;
        let ch = 34.0 * u;
        let g = 4.0 * u;
        let add_w = 34.0 * u;
        let add_gap = 8.0 * u;
        let usable = (self.timeline_area.w - add_w - add_gap).max(0.0);
        let fit = (((usable + g) / (cw + g)).floor() as usize).max(1);
        let vis = fit.min(total);
        let first = if total <= vis {
            0
        } else {
            current.saturating_sub(vis / 2).min(total - vis)
        };
        let y = (self.top_h - ch) / 2.0;
        let cells = (0..vis)
            .map(|k| {
                (
                    first + k,
                    Area::new(self.timeline_area.x + k as f32 * (cw + g), y, cw, ch),
                )
            })
            .collect();
        let add_x = self.timeline_area.x + vis as f32 * (cw + g) - g + add_gap;
        Timeline {
            cells,
            add: Area::new(add_x, y, add_w, ch),
        }
    }

    pub fn frame_target(&self, total: usize, current: usize, x: f32) -> usize {
        let tl = self.timeline(total, current);
        tl.cells
            .iter()
            .min_by(|a, b| {
                (a.1.cx() - x)
                    .abs()
                    .partial_cmp(&(b.1.cx() - x).abs())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(i, _)| *i)
            .unwrap_or(current)
    }

    pub fn frame_menu_layout(&self, anchor_x: f32) -> (Area, Vec<(Hit, &'static str, Area)>) {
        let u = self.scale;
        let w = 160.0 * u;
        let item_h = 38.0 * u;
        let pad = 4.0 * u;
        let min_x = 4.0 * u;
        let max_x = (self.width - w - 4.0 * u).max(min_x);
        let x = (anchor_x - w / 2.0).clamp(min_x, max_x);
        let y = self.top_h + 2.0 * u;
        let panel = Area::new(x, y, w, 2.0 * item_h + 2.0 * pad);
        let items = vec![
            (
                Hit::FrameDuplicate,
                "Duplicar frame",
                Area::new(x + pad, y + pad, w - 2.0 * pad, item_h),
            ),
            (
                Hit::FrameDelete,
                "Excluir frame",
                Area::new(x + pad, y + pad + item_h, w - 2.0 * pad, item_h),
            ),
        ];
        (panel, items)
    }

    pub fn layer_rows(&self, count: usize, active: usize) -> Vec<LayerRow> {
        if count == 0 {
            return Vec::new();
        }
        let u = self.scale;
        let p = self.layers_panel;
        let row_h = 38.0 * u;
        let list_top = p.y + 40.0 * u;
        let list_bottom = p.y + p.h - 52.0 * u;
        let max_rows = (((list_bottom - list_top) / row_h).floor() as usize).max(1);
        let shown = count.min(max_rows);
        let active_row = count - 1 - active.min(count - 1);
        let first = if active_row >= shown {
            active_row + 1 - shown
        } else {
            0
        };
        (0..shown)
            .map(|k| {
                let row = first + k;
                let index = count - 1 - row;
                let area = Area::new(
                    p.x + 6.0 * u,
                    list_top + k as f32 * row_h,
                    p.w - 12.0 * u,
                    row_h - 2.0 * u,
                );
                let eye = Area::new(area.x, area.y, 34.0 * u, area.h);
                LayerRow { index, area, eye }
            })
            .collect()
    }

    pub fn layer_buttons(&self) -> [(Hit, Area); 4] {
        let u = self.scale;
        let p = self.layers_panel;
        let b = 36.0 * u;
        let pad = 10.0 * u;
        let gap = (p.w - 2.0 * pad - 4.0 * b) / 3.0;
        let y = p.y + p.h - 8.0 * u - b;
        let at = |i: usize| Area::new(p.x + pad + i as f32 * (b + gap), y, b, b);
        [
            (Hit::LayerAdd, at(0)),
            (Hit::LayerRemove, at(1)),
            (Hit::LayerUp, at(2)),
            (Hit::LayerDown, at(3)),
        ]
    }

    pub fn hit(&self, x: f32, y: f32, v: &UiView) -> Option<Hit> {
        if v.menu_open {
            if !self.menu_panel.contains(x, y) {
                return Some(Hit::Dismiss);
            }
            for item in &self.menu_items {
                if item.area.contains(x, y) {
                    return Some(match item.action {
                        Some(action) => Hit::Menu(action),
                        None => Hit::Panel,
                    });
                }
            }
            return Some(Hit::Panel);
        }

        if let Some(anchor) = v.frame_menu {
            let (panel, items) = self.frame_menu_layout(anchor);
            if !panel.contains(x, y) {
                return Some(Hit::Dismiss);
            }
            for (hit, _, area) in items {
                if area.contains(x, y) {
                    return Some(hit);
                }
            }
            return Some(Hit::Panel);
        }

        if v.flyout_open {
            if !self.flyout_panel.contains(x, y) {
                return Some(Hit::Dismiss);
            }
            for (tool, _, area) in &self.flyout_items {
                if area.contains(x, y) {
                    return Some(Hit::SelTool(*tool));
                }
            }
            return Some(Hit::Panel);
        }

        if v.picker_open {
            let p = &self.picker;
            if !p.panel.contains(x, y) {
                return Some(Hit::Dismiss);
            }
            let dx = x - p.cx;
            let dy = y - p.cy;
            let reach = p.r + 8.0 * self.scale;
            if dx * dx + dy * dy <= reach * reach {
                return Some(Hit::Wheel);
            }
            if p.value.area.contains(x, y) {
                return Some(Hit::Slider(SliderId::Value));
            }
            return Some(Hit::Panel);
        }

        if y < self.top_h {
            if self.menu_btn.contains(x, y) {
                return Some(Hit::MenuButton);
            }
            if self.undo_btn.contains(x, y) {
                return Some(Hit::Undo);
            }
            if self.redo_btn.contains(x, y) {
                return Some(Hit::Redo);
            }
            if self.play.contains(x, y) {
                return Some(Hit::Play);
            }
            if self.layers_btn.contains(x, y) {
                return Some(Hit::ToggleLayers);
            }
            let tl = self.timeline(v.frames, v.current);
            let margin = 2.0 * self.scale;
            for (i, a) in &tl.cells {
                let wide = Area::new(a.x - margin, a.y, a.w + 2.0 * margin, a.h);
                if wide.contains(x, y) {
                    return Some(Hit::Frame(*i));
                }
            }
            if tl.add.contains(x, y) {
                return Some(Hit::AddFrame);
            }
            return Some(Hit::Panel);
        }

        if x < self.side_w {
            if self.brush.contains(x, y) {
                return Some(Hit::Brush);
            }
            if self.sel_btn.contains(x, y) {
                return Some(Hit::SelButton);
            }
            if self.eraser.contains(x, y) {
                return Some(Hit::Eraser);
            }
            if self.color.contains(x, y) {
                return Some(Hit::ColorButton);
            }
            if self.onion_btn.contains(x, y) {
                return Some(Hit::Onion);
            }
            if self.size_slider.area.contains(x, y) {
                return Some(Hit::Slider(SliderId::Size));
            }
            if self.opacity_slider.area.contains(x, y) {
                return Some(Hit::Slider(SliderId::Opacity));
            }
            return Some(Hit::Panel);
        }

        if v.layers_open && self.layers_panel.contains(x, y) {
            for row in self.layer_rows(v.layers.len(), v.active_layer) {
                if row.eye.contains(x, y) {
                    return Some(Hit::LayerEye(row.index));
                }
                if row.area.contains(x, y) {
                    return Some(Hit::LayerSelect(row.index));
                }
            }
            for (hit, area) in self.layer_buttons() {
                if area.contains(x, y) {
                    return Some(hit);
                }
            }
            return Some(Hit::Panel);
        }

        None
    }
}
