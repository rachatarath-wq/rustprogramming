// Runtime Thai/English text rendering: rustybuzz (shaping) + fontdue (raster).
use fontdue::Font;
use macroquad::prelude::*;
use std::collections::HashMap;

const SARABUN: &[u8] = include_bytes!("../assets/Sarabun-Regular.ttf");

type GlyphCache = HashMap<(u32, u16), (Texture2D, f32, f32, f32, f32)>;

pub struct ThaiText {
    font: Font,
    glyphs: GlyphCache, // (glyph_id, px) -> (tex, w, h, xmin, ymin)
}

impl ThaiText {
    pub fn new() -> Self {
        let font = Font::from_bytes(SARABUN, fontdue::FontSettings::default()).unwrap();
        Self {
            font,
            glyphs: HashMap::new(),
        }
    }

    pub fn draw(&mut self, text: &str, x: f32, y: f32, px: f32, color: Color) {
        let face = rustybuzz::Face::from_slice(SARABUN, 0).unwrap();
        let mut buf = rustybuzz::UnicodeBuffer::new();
        buf.push_str(text);
        let out = rustybuzz::shape(&face, &[], buf);

        let upem = self.font.units_per_em() as f32;
        let scale = px / upem;
        let line = self.font.horizontal_line_metrics(px).unwrap();
        let baseline = y + line.ascent;

        let mut cx = x;
        let infos = out.glyph_infos();
        let positions = out.glyph_positions();
        for i in 0..infos.len() {
            let info = &infos[i];
            let p = &positions[i];
            let (tex, w, h, xmin, ymin) = self.glyph_tex(info.glyph_id, px as u16);
            if w <= 0.0 || h <= 0.0 {
                cx += p.x_advance as f32 * scale;
                continue;
            }
            let gx = cx + xmin + p.x_offset as f32 * scale;
            // fontdue ymin is y-UP (positive above baseline) -> flip to y-down screen
            let gy = baseline - h - ymin - p.y_offset as f32 * scale;
            draw_texture_ex(
                &tex,
                gx,
                gy,
                color,
                DrawTextureParams {
                    dest_size: Some(vec2(w, h)),
                    ..Default::default()
                },
            );
            cx += p.x_advance as f32 * scale;
        }
    }

    fn glyph_tex(&mut self, glyph_id: u32, px: u16) -> (Texture2D, f32, f32, f32, f32) {
        if let Some(t) = self.glyphs.get(&(glyph_id, px)) {
            return t.clone();
        }
        let (metrics, bitmap) = self.font.rasterize_indexed(glyph_id as u16, px as f32);
        let w = metrics.width as u16;
        let h = metrics.height as u16;
        let mut rgba = vec![0u8; (w as usize) * (h as usize) * 4];
        for i in 0..(w as usize * h as usize) {
            let cov = bitmap[i];
            let j = i * 4;
            rgba[j] = 255;
            rgba[j + 1] = 255;
            rgba[j + 2] = 255;
            rgba[j + 3] = cov;
        }
        let tex = Texture2D::from_rgba8(w, h, &rgba);
        tex.set_filter(FilterMode::Linear);
        let t = (
            tex,
            w as f32,
            h as f32,
            metrics.xmin as f32,
            metrics.ymin as f32,
        );
        self.glyphs.insert((glyph_id, px), t.clone());
        t
    }
}
