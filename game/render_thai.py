#!/usr/bin/env python3
# Render Thai text -> PNG with proper shaping (HarfBuzz) so glyphs don't overlap
import freetype
import uharfbuzz as hb
from PIL import Image

FONT = "/root/rust-tutorial/miniroblox/assets/NotoSansThai.ttf"
OUT = "/root/rust-tutorial/miniroblox/assets/"

def render(text, out_name, size_px, color, pad=6):
    face = freetype.Face(FONT)
    face.set_pixel_sizes(0, size_px)

    blob = hb.Blob.from_file_path(FONT)
    hb_face = hb.Face(blob)
    hb_font = hb.Font(hb_face)
    hb_font.scale = (size_px * 64, size_px * 64)

    buf = hb.Buffer()
    buf.add_str(text)
    buf.guess_segment_properties()
    hb.shape(hb_font, buf)

    infos = buf.glyph_infos
    positions = buf.glyph_positions

    # pass 1: rasterize + collect glyphs + bounding box
    glyphs = []
    cursor_x = 0.0
    cursor_y = 0.0
    min_x = min_y = 1e18
    max_x = max_y = -1e18
    for info, pos in zip(infos, positions):
        gid = info.codepoint
        face.load_glyph(gid, freetype.FT_LOAD_RENDER)
        bmp = face.glyph.bitmap
        ox = cursor_x + pos.x_offset / 64.0
        oy = cursor_y + pos.y_offset / 64.0
        img_x = ox + face.glyph.bitmap_left
        img_y = -oy - face.glyph.bitmap_top   # baseline at y=0 (image: down positive)
        if bmp.width > 0 and bmp.rows > 0:
            min_x = min(min_x, img_x); max_x = max(max_x, img_x + bmp.width)
            min_y = min(min_y, img_y); max_y = max(max_y, img_y + bmp.rows)
        glyphs.append((gid, img_x, img_y, bmp))
        cursor_x += pos.x_advance / 64.0
        cursor_y += pos.y_advance / 64.0

    W = int(max_x - min_x) + pad * 2
    H = int(max_y - min_y) + pad * 2
    img = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    r, g, b = color
    for gid, img_x, img_y, bmp in glyphs:
        if bmp.width == 0 or bmp.rows == 0:
            continue
        gray = Image.frombytes("L", (bmp.width, bmp.rows), bytes(bmp.buffer), "raw", "L", bmp.pitch, 1)
        layer = Image.new("RGBA", (bmp.width, bmp.rows), (r, g, b, 255))
        layer.putalpha(gray)
        img.alpha_composite(layer, (int(img_x - min_x) + pad, int(img_y - min_y) + pad))
    img.save(OUT + out_name)
    print(f"{out_name}: {W}x{H}px, {len(glyphs)} glyphs")

render("รอดถึงเช้าแล้ว! ชนะ! เล่นต่อได้ หรือกด F10 ออก", "thai_win.png", 30, (255, 255, 255))
render("AUTO-PLAY (AI เล่นให้ดู) - กด P เพื่อคุมเอง", "thai_auto.png", 30, (255, 215, 0))
