pub mod font;

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
pub enum Hit {
    Dismiss,
    Panel,
    MenuButton,
    Play,
    Onion,
    RemoveFrame,
    AddFrame,
    ToggleLayers,
    Frame(usize),
    Brush,
    Eraser,
    ColorButton,
    Undo,
    Slider(SliderId),
    Menu(MenuAction),
    Wheel,
    LayerSelect(usize),
    LayerEye(usize),
    LayerAdd,
    LayerRemove,
    LayerUp,
    LayerDown,
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

pub struct UiView<'a> {
    pub eraser: bool,
    pub playing: bool,
    pub onion: bool,
    pub frames: usize,
    pub current: usize,
    pub menu_open: bool,
    pub picker_open: bool,
    pub layers_open: bool,
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
    pub top_h: f32,
    pub side_w: f32,
    pub menu_btn: Area,
    pub play: Area,
    pub onion_btn: Area,
    pub remove_btn: Area,
    pub add_btn: Area,
    pub layers_btn: Area,
    pub timeline: Area,
    pub brush: Area,
    pub eraser: Area,
    pub color: Area,
    pub preview: Area,
    pub undo: Area,
    pub size_slider: Slider,
    pub opacity_slider: Slider,
    pub menu_panel: Area,
    pub menu_items: Vec<MenuItem>,
    pub picker: Picker,
    pub layers_panel: Area,
}

impl Ui {
    pub fn new(width: f32, height: f32, scale: f32) -> Self {
        let u = scale;
        let top_h = 56.0 * u;
        let pad = 10.0 * u;
        let gap = 6.0 * u;
        let btn = 44.0 * u;
        let by = (top_h - btn) / 2.0;

        let menu_btn = Area::new(pad, by, btn, btn);
        let play = Area::new(menu_btn.x + btn + gap, by, btn, btn);
        let onion_btn = Area::new(play.x + btn + gap, by, btn, btn);

        let layers_btn = Area::new(width - pad - btn, by, btn, btn);
        let add_btn = Area::new(layers_btn.x - gap - btn, by, btn, btn);
        let remove_btn = Area::new(add_btn.x - gap - btn, by, btn, btn);

        let tl_x = onion_btn.x + btn + 16.0 * u;
        let tl_end = remove_btn.x - 16.0 * u;
        let timeline = Area::new(tl_x, 0.0, (tl_end - tl_x).max(0.0), top_h);

        let side_w = 72.0 * u;
        let sb = 48.0 * u;
        let sbx = (side_w - sb) / 2.0;
        let mut y = top_h + 10.0 * u;
        let brush = Area::new(sbx, y, sb, sb);
        y += sb + gap;
        let eraser = Area::new(sbx, y, sb, sb);
        y += sb + gap;
        let color = Area::new(sbx, y, sb, sb);
        y += sb + gap;
        let pv = 40.0 * u;
        let preview = Area::new((side_w - pv) / 2.0, y, pv, pv);
        y += pv + gap;
        let undo = Area::new(sbx, height - 10.0 * u - sb, sb, sb);

        let sliders_top = y;
        let sliders_h = (undo.y - gap - sliders_top).max(80.0 * u);
        let sw = 32.0 * u;
        let sx = (side_w - 2.0 * sw) / 2.0;
        let edge = 20.0 * u;
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

        let entries: [(Option<MenuAction>, &'static str); 6] = [
            (Some(MenuAction::New), "NOVO"),
            (Some(MenuAction::Open), "ABRIR"),
            (Some(MenuAction::Save), "SALVAR"),
            (None, "IMPORTAR IMAGEM"),
            (None, "EXPORTAR"),
            (Some(MenuAction::Quit), "SAIR"),
        ];
        let item_h = 44.0 * u;
        let menu_w = 240.0 * u;
        let mpad = 6.0 * u;
        let menu_panel = Area::new(
            menu_btn.x,
            top_h + 4.0 * u,
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

        let ppad = 14.0 * u;
        let vh = 36.0 * u;
        let avail = height - top_h - 16.0 * u;
        let r = (100.0 * u)
            .min((avail - 2.0 * ppad - 10.0 * u - vh) / 2.0)
            .max(40.0 * u);
        let pw = 2.0 * r + 2.0 * ppad;
        let ph = ppad + 2.0 * r + 10.0 * u + vh + ppad;
        let px = side_w + 8.0 * u;
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
                area: Area::new(px + ppad, pcy + r + 10.0 * u, 2.0 * r, vh),
                edge: 12.0 * u,
                vertical: false,
            },
        };

        let lp_w = 220.0 * u;
        let layers_panel = Area::new(width - lp_w, top_h, lp_w, height - top_h);

        Self {
            scale,
            top_h,
            side_w,
            menu_btn,
            play,
            onion_btn,
            remove_btn,
            add_btn,
            layers_btn,
            timeline,
            brush,
            eraser,
            color,
            preview,
            undo,
            size_slider,
            opacity_slider,
            menu_panel,
            menu_items,
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

    pub fn timeline_cells(&self, total: usize, current: usize) -> Vec<(usize, Area)> {
        let u = self.scale;
        let c = 40.0 * u;
        let g = 6.0 * u;
        let fit = (((self.timeline.w + g) / (c + g)).floor() as usize).max(1);
        let vis = fit.min(total);
        let first = if total <= vis {
            0
        } else {
            current.saturating_sub(vis / 2).min(total - vis)
        };
        let y = (self.top_h - c) / 2.0;
        (0..vis)
            .map(|k| {
                (
                    first + k,
                    Area::new(self.timeline.x + k as f32 * (c + g), y, c, c),
                )
            })
            .collect()
    }

    pub fn layer_rows(&self, count: usize, active: usize) -> Vec<LayerRow> {
        if count == 0 {
            return Vec::new();
        }
        let u = self.scale;
        let p = self.layers_panel;
        let row_h = 44.0 * u;
        let list_top = p.y + 44.0 * u;
        let list_bottom = p.y + p.h - 60.0 * u;
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
                    p.x + 8.0 * u,
                    list_top + k as f32 * row_h,
                    p.w - 16.0 * u,
                    row_h - 4.0 * u,
                );
                let eye = Area::new(area.x, area.y, 40.0 * u, area.h);
                LayerRow { index, area, eye }
            })
            .collect()
    }

    pub fn layer_buttons(&self) -> [(Hit, Area); 4] {
        let u = self.scale;
        let p = self.layers_panel;
        let b = 40.0 * u;
        let pad = 12.0 * u;
        let gap = (p.w - 2.0 * pad - 4.0 * b) / 3.0;
        let y = p.y + p.h - 10.0 * u - b;
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
            if self.play.contains(x, y) {
                return Some(Hit::Play);
            }
            if self.onion_btn.contains(x, y) {
                return Some(Hit::Onion);
            }
            if self.remove_btn.contains(x, y) {
                return Some(Hit::RemoveFrame);
            }
            if self.add_btn.contains(x, y) {
                return Some(Hit::AddFrame);
            }
            if self.layers_btn.contains(x, y) {
                return Some(Hit::ToggleLayers);
            }
            for (i, area) in self.timeline_cells(v.frames, v.current) {
                if area.contains(x, y) {
                    return Some(Hit::Frame(i));
                }
            }
            return Some(Hit::Panel);
        }

        if x < self.side_w {
            if self.brush.contains(x, y) {
                return Some(Hit::Brush);
            }
            if self.eraser.contains(x, y) {
                return Some(Hit::Eraser);
            }
            if self.color.contains(x, y) {
                return Some(Hit::ColorButton);
            }
            if self.undo.contains(x, y) {
                return Some(Hit::Undo);
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
