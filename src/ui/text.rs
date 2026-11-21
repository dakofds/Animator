use std::collections::HashMap;
use tiny_skia::Pixmap;

const FONT_DATA: &[u8] = include_bytes!("../../assets/font.ttf");

pub struct Text {
    font: fontdue::Font,
    cache: HashMap<(char, u32), (fontdue::Metrics, Vec<u8>)>,
}

impl Text {
    pub fn new() -> Self {
        let font = fontdue::Font::from_bytes(FONT_DATA, fontdue::FontSettings::default())
            .expect("fonte invalida");
        Self {
            font,
            cache: HashMap::new(),
        }
    }

    fn glyph(&mut self, ch: char, size: f32) -> &(fontdue::Metrics, Vec<u8>) {
        let q = (size * 4.0).round().max(1.0) as u32;
        let font = &self.font;
        self.cache
            .entry((ch, q))
            .or_insert_with(|| font.rasterize(ch, q as f32 / 4.0))
    }

    pub fn draw(
        &mut self,
        pm: &mut Pixmap,
        s: &str,
        x: f32,
        cy: f32,
        size: f32,
        color: [u8; 4],
    ) {
        let baseline = (cy + size * 0.36).round() as i32;
        let w = pm.width() as i32;
        let h = pm.height() as i32;
        let mut pen = x;

        for ch in s.chars() {
            let glyph = self.glyph(ch, size);
            let (m, bitmap) = (&glyph.0, &glyph.1);
            let gx0 = (pen + m.xmin as f32).round() as i32;
            let gy0 = baseline - m.ymin - m.height as i32;
            let data = pm.data_mut();

            for gy in 0..m.height {
                for gx in 0..m.width {
                    let cov = bitmap[gy * m.width + gx] as u32;
                    if cov == 0 {
                        continue;
                    }
                    let px = gx0 + gx as i32;
                    let py = gy0 + gy as i32;
                    if px < 0 || py < 0 || px >= w || py >= h {
                        continue;
                    }
                    let a = cov * color[3] as u32 / 255;
                    let inv = 255 - a;
                    let i = ((py * w + px) * 4) as usize;
                    data[i] = ((color[0] as u32 * a + data[i] as u32 * inv) / 255) as u8;
                    data[i + 1] = ((color[1] as u32 * a + data[i + 1] as u32 * inv) / 255) as u8;
                    data[i + 2] = ((color[2] as u32 * a + data[i + 2] as u32 * inv) / 255) as u8;
                    data[i + 3] = ((255 * a + data[i + 3] as u32 * inv) / 255) as u8;
                }
            }

            pen += m.advance_width;
        }
    }
}
