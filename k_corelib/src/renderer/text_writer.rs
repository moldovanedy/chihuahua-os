use crate::{log, renderer};
use dog_essentials::geometry::rect::Rect;
use dog_essentials::lazy_static::lazy_static;
use dog_essentials::static_cell::StaticCell;
use dog_essentials::sync::mutex::Mutex;
use psf::ascii_psf_font::AsciiPsfFont;

lazy_static! {
    static ref FONT: AsciiPsfFont = {
        unsafe {
            let font: Option<AsciiPsfFont> =
                AsciiPsfFont::from_raw(&_binary_res_Tamsyn8x16r_psf_start as *const u8);
            if font.is_some() {
                let font = font.unwrap();
                return font;
            } else {
                return AsciiPsfFont::default();
            }
        }
    };
}

static mut IS_INIT: bool = false;

static CURR_ROW: StaticCell<u32> = StaticCell::new(0);
static CURR_COLUMN: StaticCell<u32> = StaticCell::new(0);

static WIDTH_IN_CHARS: StaticCell<u32> = StaticCell::new(0);
static HEIGHT_IN_CHARS: StaticCell<u32> = StaticCell::new(0);
static WRITE_LOCK: Mutex<bool> = Mutex::new(false);

/// The width (in characters) for each line of text (max. 100 lines).
static mut LINES_WIDTH: [u32; 100] = [0; 100];

const HARDCODED_FONT_WIDTH: u32 = 8;
const HARDCODED_FONT_HEIGHT: u32 = 16;

#[allow(dead_code)]
unsafe extern "C" {
    static _binary_res_Tamsyn8x16r_psf_start: u8;
    static _binary_res_Tamsyn8x16r_psf_end: u8;
    static _binary_res_Tamsyn8x16r_psf_size: u8;
}

pub fn init() {
    unsafe {
        if IS_INIT {
            return;
        }

        IS_INIT = true;
        WIDTH_IN_CHARS.set_value_unsafe(renderer::fb_width() / FONT.width());
        HEIGHT_IN_CHARS.set_value_unsafe(renderer::fb_height() / FONT.height());
    }
}

pub fn write(raw_string: &[u8], fg_color: renderer::Color, bg_color: renderer::Color) {
    WRITE_LOCK.lock();
    
    unsafe {
        if FONT.width() != HARDCODED_FONT_WIDTH || FONT.height() != HARDCODED_FONT_HEIGHT {
            log::log_error("Font is in non-standard size. Can't write anything.");
            return;
        }

        let actual_string = core::str::from_utf8(raw_string);
        if actual_string.is_ok() {
            let actual_string = actual_string.unwrap();
            log::log_raw(actual_string);
        }

        for chr in raw_string.iter() {
            let curr_row = CURR_ROW.get_value_unsafe();
            let curr_column = CURR_COLUMN.get_value_unsafe();
            let width_in_chars = WIDTH_IN_CHARS.get_value_unsafe();
            let height_in_chars = HEIGHT_IN_CHARS.get_value_unsafe();

            //if newline
            if *chr == b'\n' || curr_column >= width_in_chars {
                if curr_row < &100 {
                    LINES_WIDTH[*curr_row as usize] = *curr_column;
                }

                CURR_ROW.set_value_unsafe(curr_row + 1);
                CURR_COLUMN.set_value_unsafe(0);
                continue;
            }

            if curr_row >= height_in_chars {
                scroll();
                update_lines_width_on_scroll();
                CURR_ROW.set_value_unsafe(height_in_chars - 1);
            }

            let c: char = core::char::from_u32(*chr as u32).unwrap_or('?');
            let mut glyph_data: *const u8 = FONT.get_glyph(c as u32);

            //currently only 8x16 as we don't have dynamic memory allocation
            let mut buffer: [renderer::Color;
                (HARDCODED_FONT_WIDTH * HARDCODED_FONT_HEIGHT) as usize] =
                core::array::from_fn(|_i| renderer::Color::new(0, 0, 0));

            for y in 0..FONT.height() {
                let mut mask: u32 = 1 << (FONT.width() - 1);

                for x in 0..FONT.width() {
                    if *(glyph_data as *const u32) & mask != 0 {
                        buffer[(y * FONT.width() + x) as usize] = fg_color;
                    } else {
                        buffer[(y * FONT.width() + x) as usize] = bg_color;
                    }

                    mask >>= 1;
                }

                glyph_data = glyph_data.byte_add((FONT.width() as usize + 7) / 8);
            }

            renderer::draw_rect_buffer(
                CURR_COLUMN.get_value_unsafe() * FONT.width(),
                CURR_ROW.get_value_unsafe() * FONT.height(),
                HARDCODED_FONT_WIDTH,
                HARDCODED_FONT_HEIGHT,
                &buffer,
            );

            CURR_COLUMN.set_value_unsafe(CURR_COLUMN.get_value_unsafe() + 1);
        }

        if CURR_ROW.get_value_unsafe() < &100 {
            LINES_WIDTH[*CURR_ROW.get_value_unsafe() as usize] = *CURR_COLUMN.get_value_unsafe();
        }
    }
}

fn update_lines_width_on_scroll() {
    unsafe {
        for i in 1..100 {
            LINES_WIDTH[i - 1] = LINES_WIDTH[i];
        }
    }
}

fn scroll() {
    let mut width: f32 = 0.0;

    if CURR_ROW.get_value_unsafe() >= &100 {
        width = renderer::fb_width() as f32;
    } else {
        unsafe {
            for i in 0..100 {
                let this_width = LINES_WIDTH[i] * HARDCODED_FONT_WIDTH;
                let this_width = this_width as f32;
                if this_width > width {
                    width = this_width;
                }
            }
        }
    }

    renderer::copy_region(
        &Rect::from_coords(
            0.0,
            16.0,
            width,
            (renderer::fb_height() - HARDCODED_FONT_HEIGHT) as f32,
        ),
        &Rect::from_coords(
            0.0,
            0.0,
            width,
            (renderer::fb_height() - HARDCODED_FONT_HEIGHT) as f32,
        ),
    );
}
