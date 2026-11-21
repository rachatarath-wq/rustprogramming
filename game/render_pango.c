// Render text -> PNG with Pango (full Thai shaping) — transparent bg, colored text
#include <cairo.h>
#include <pango/pangocairo.h>
#include <stdlib.h>

int main(int argc, char** argv) {
    if (argc < 7) { return 1; }
    const char* text = argv[1];
    int size = atoi(argv[2]);
    double r = atoi(argv[3]) / 255.0;
    double g = atoi(argv[4]) / 255.0;
    double b = atoi(argv[5]) / 255.0;
    const char* out = argv[6];

    PangoFontMap* fontmap = pango_cairo_font_map_new();
    PangoContext* ctx = pango_font_map_create_context(fontmap);
    PangoLayout* layout = pango_layout_new(ctx);
    PangoFontDescription* desc = pango_font_description_from_string("Noto Sans Thai");
    pango_font_description_set_size(desc, size * PANGO_SCALE);
    pango_layout_set_font_description(layout, desc);
    pango_layout_set_text(layout, text, -1);

    int w = 0, h = 0;
    pango_layout_get_pixel_size(layout, &w, &h);

    cairo_surface_t* surf = cairo_image_surface_create(CAIRO_FORMAT_ARGB32, w, h);
    cairo_t* cr = cairo_create(surf);
    cairo_set_source_rgba(cr, r, g, b, 1.0);
    pango_cairo_show_layout(cr, layout);
    cairo_surface_write_to_png(surf, out);

    printf("%s: %dx%d px\n", out, w, h);

    pango_font_description_free(desc);
    g_object_unref(layout);
    g_object_unref(ctx);
    g_object_unref(fontmap);
    cairo_destroy(cr);
    cairo_surface_destroy(surf);
    return 0;
}
