use dog_essentials::static_cell::StaticCell;
use k_corelib::{log, renderer};
use psf::ascii_psf_font::AsciiPsfFont;

static FONT: StaticCell<AsciiPsfFont> = StaticCell::new(AsciiPsfFont::default());
static CURR_ROW: StaticCell<u32> = StaticCell::new(0);
static CURR_COLUMN: StaticCell<u32> = StaticCell::new(0);

static WIDTH_IN_CHARS: StaticCell<u32> = StaticCell::new(0);
static HEIGHT_IN_CHARS: StaticCell<u32> = StaticCell::new(0);

unsafe extern "C" {
    unsafe static _binary_res_Tamsyn8x16r_psf_start: u8;
    unsafe static _binary_res_Tamsyn8x16r_psf_end: u8;
    unsafe static _binary_res_Tamsyn8x16r_psf_size: u8;
}

pub(crate) fn init() {
    unsafe {
        let font: Option<AsciiPsfFont> =
            AsciiPsfFont::from_raw(&_binary_res_Tamsyn8x16r_psf_start as *const u8);
        if font.is_some() {
            let font = font.unwrap();
            FONT.set_value_unsafe(font);

            let font = FONT.get_value_unsafe();
            WIDTH_IN_CHARS.set_value_unsafe(renderer::fb_width() / font.width());
            HEIGHT_IN_CHARS.set_value_unsafe(renderer::fb_height() / font.height());
        }
    }
}

pub fn write(raw_string: &[u8], fg_color: renderer::Color, bg_color: renderer::Color) {
    unsafe {
        let font: &AsciiPsfFont = FONT.get_value_unsafe();

        if font.width() != 8 || font.height() != 16 {
            log::log_error("Font is in non-standard size. Can't write anything.");
            return;
        }

        for chr in raw_string.iter() {
            let curr_row = CURR_ROW.get_value_unsafe();
            let curr_column = CURR_COLUMN.get_value_unsafe();
            let width_in_chars = WIDTH_IN_CHARS.get_value_unsafe();
            let height_in_chars = HEIGHT_IN_CHARS.get_value_unsafe();
            
            //if newline
            if *chr == b'\n' || curr_column >= width_in_chars {
                CURR_ROW.set_value_unsafe(curr_row + 1);
                CURR_COLUMN.set_value_unsafe(0);
                continue;
            }
            
            if curr_row >= height_in_chars {
                scroll();
                CURR_ROW.set_value_unsafe(height_in_chars - 1);
            }

            let c: char = core::char::from_u32(*chr as u32).unwrap_or('?');
            let mut glyph_data: *const u8 = font.get_glyph(c as u32);

            //currently only 8x16 as we don't have dynamic memory allocation
            let mut buffer: [renderer::Color; 8 * 16] =
                core::array::from_fn(|_i| renderer::Color::new(0, 0, 0));

            for y in 0..font.height() {
                let mut mask: u32 = 1 << (font.width() - 1);

                for x in 0..font.width() {
                    if *(glyph_data as *const u32) & mask != 0 {
                        buffer[(y * font.width() + x) as usize] = fg_color;
                    } else {
                        buffer[(y * font.width() + x) as usize] = bg_color;
                    }

                    mask >>= 1;
                }

                glyph_data = glyph_data.byte_add((font.width() as usize + 7) / 8);
            }

            renderer::draw_rect_buffer(
                CURR_COLUMN.get_value_unsafe() * font.width(),
                CURR_ROW.get_value_unsafe() * font.height(),
                8,
                16,
                &buffer,
            );

            CURR_COLUMN.set_value_unsafe(CURR_COLUMN.get_value_unsafe() + 1);
        }
    }
}

fn scroll() {
    use k_corelib::essentials_clone::geometry::rect as rect_clone;
    renderer::copy_region(
        &rect_clone::Rect::from_coords(0.0, 0.0, renderer::fb_width() as f32, renderer::fb_height() as f32), 
        &rect_clone::Rect::from_coords(0.0, -16.0, renderer::fb_width() as f32, renderer::fb_height() as f32));
}
