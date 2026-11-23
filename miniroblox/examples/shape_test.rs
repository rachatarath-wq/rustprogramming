use fontdue::Font;

fn main() {
    let font_bytes = std::fs::read("assets/Sarabun-Regular.ttf").unwrap();
    let font = Font::from_bytes(font_bytes.as_slice(), fontdue::FontSettings::default()).unwrap();
    let face = rustybuzz::Face::from_slice(font_bytes.as_slice(), 0).unwrap();
    let mut buf = rustybuzz::UnicodeBuffer::new();
    buf.push_str("ก่กู");
    let out = rustybuzz::shape(&face, &[], buf);
    let infos = out.glyph_infos();
    let positions = out.glyph_positions();
    let upem = font.units_per_em() as f32;
    println!("upem={}", upem);
    for i in 0..infos.len() {
        let info = &infos[i];
        let p = &positions[i];
        let (m, _) = font.rasterize_indexed(info.glyph_id as u16, 18.0);
        println!(
            "glyph{} id={} x_adv={} y_adv={} x_off={} y_off={} | fontdue xmin={} ymin={} w={} h={}",
            i, info.glyph_id, p.x_advance, p.y_advance, p.x_offset, p.y_offset,
            m.xmin, m.ymin, m.width, m.height
        );
    }
    let lm = font.horizontal_line_metrics(18.0).unwrap();
    println!("ascent={} descent={} newline={}", lm.ascent, lm.descent, lm.new_line_size);
}
