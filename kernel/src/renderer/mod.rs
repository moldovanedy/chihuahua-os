use dog_essentials::static_cell::StaticCell;
use k_corelib::boot_info;
use psf::ascii_psf_font::AsciiPsfFont;

static mut BASE_ADDR: u64 = 0;
static mut WIDTH: u32 = 0;
static mut HEIGHT: u32 = 0;
static mut PITCH: u32 = 0;
static mut BITS_PER_PIXEL: u8 = 0;

static mut RED_MASK: u32 = 0;
static mut GREEN_MASK: u32 = 0;
static mut BLUE_MASK: u32 = 0;
static mut IS_RGB32: bool = false;

static FONT: StaticCell<AsciiPsfFont> = StaticCell::new(AsciiPsfFont::default());

unsafe extern "C" {
    unsafe static _binary_res_Tamsyn8x16r_psf_start: u8;
    unsafe static _binary_res_Tamsyn8x16r_psf_end: u8;
    unsafe static _binary_res_Tamsyn8x16r_psf_size: u8;
}

pub fn setup_fb(fb_info: &boot_info::FramebufferData) -> bool {
    if fb_info.width() == 0
        || fb_info.height() == 0
        || fb_info.red_bitmask() == 0
        || fb_info.green_bitmask() == 0
        || fb_info.blue_bitmask() == 0
    {
        return false;
    }

    unsafe {
        BASE_ADDR = fb_info.address();
        WIDTH = fb_info.width();
        HEIGHT = fb_info.height();
        PITCH = fb_info.pitch();
        BITS_PER_PIXEL = fb_info.bits_per_pixel();

        RED_MASK = fb_info.red_bitmask();
        GREEN_MASK = fb_info.green_bitmask();
        BLUE_MASK = fb_info.blue_bitmask();

        if RED_MASK == 0xff_00_00 && GREEN_MASK == 0xff_00 && BLUE_MASK == 0xff {
            IS_RGB32 = true;
        }

        let font: Option<AsciiPsfFont> =
            AsciiPsfFont::from_raw(&_binary_res_Tamsyn8x16r_psf_start as *const u8);
        if font.is_some() {
            FONT.set_value_unsafe(font.unwrap());
        }
    }

    return true;
}

pub fn draw_test() {
    draw_rect(20, 40, 250, 300, 0xff_ff_00);
    draw_char(b'2', 2, 1, 0xff_ff_ff, 0xff_00_00);
    draw_char(b'A', 3, 1, 0xff_ff_ff, 0xff_00_00);
    draw_char(b'b', 4, 1, 0xff_ff_ff, 0xff_00_00);
}

pub fn draw_rect(x: u32, y: u32, width: u32, height: u32, color: u32) {
    unsafe {
        if x + width > WIDTH || y + height > HEIGHT {
            return;
        }

        let fb: *mut u8 = BASE_ADDR as *mut u8;
        let bytes_per_pixel: u32 = (BITS_PER_PIXEL / 8) as u32;

        let color: u32 = if IS_RGB32 {
            color
        } else {
            convert_color_format(color)
        };

        for y_pos in y..(y + height) {
            for x_pos in x..(x + width) {
                let base_addr: *mut u8 =
                    fb.add((bytes_per_pixel * y_pos * PITCH + bytes_per_pixel * x_pos) as usize);
                *base_addr.add(0) = color as u8;
                *base_addr.add(1) = (color >> 8) as u8;
                *base_addr.add(2) = (color >> 16) as u8;
                *base_addr.add(3) = (color >> 24) as u8;
            }
        }
    }
}

pub fn draw_char(char: u8, cx: u32, cy: u32, fg_color: u32, bg_color: u32) {
    unsafe {
        let font: &AsciiPsfFont = FONT.get_value_unsafe();
        let mut glyph_data: *const u8 = font.get_glyph(char as u32);
        let bytes_per_pixel: u32 = (BITS_PER_PIXEL / 8) as u32;

        let mut fb: *mut u8 = BASE_ADDR as *mut u8;
        //TODO: see if we need to add 1 pixel horizontally to prevent text overlapping
        fb = fb.byte_add(
            ((cy * font.height() * bytes_per_pixel * PITCH)
                + (cx * font.width() * bytes_per_pixel))
                .try_into()
                .unwrap(),
        );

        for _y in 0..font.height() {
            let mut mask: u32 = 1 << (font.width() - 1);

            for _x in 0..font.width() {
                if *(glyph_data as *const u32) & mask != 0 {
                    direct_draw(fb, fg_color);
                } else {
                    direct_draw(fb, bg_color);
                }

                /* next pixel */
                mask >>= 1;
                fb = fb.byte_add(bytes_per_pixel as usize);
            }

            //we need +7 in case the pixels are not divisible by 8 bits; in that case, the next byte will take these values
            glyph_data = glyph_data.byte_add((font.width() as usize + 7) / 8);
            //advance to next row, but go back to the column start
            fb = fb
                .byte_add(((bytes_per_pixel * PITCH) - (bytes_per_pixel * font.width())) as usize);
        }
    }
}

fn direct_draw(fb: *mut u8, color: u32) {
    unsafe {
        *fb.add(0) = color as u8;
        *fb.add(1) = (color >> 8) as u8;
        *fb.add(2) = (color >> 16) as u8;
        *fb.add(3) = (color >> 24) as u8;
    }
}

/// Converts the RGBA32 color format to the one appropriate for the provided framebuffer.
fn convert_color_format(color: u32) -> u32 {
    unsafe {
        if IS_RGB32 {
            return color;
        }

        //if BGR32, skip expensive calculations and just rotate bits
        if RED_MASK == 0xff && GREEN_MASK == 0xff_00 && BLUE_MASK == 0xff_00_00 {
            //red from byte 3 to byte 2, green from byte 2 to byte 3, and blue from byte 1 to byte 4
            //last byte is reserved
            return ((color & 0xff_00_00) >> 8) | ((color & 0xff_00) << 8) | ((color & 0xff) << 24);
        }
    }
    color
}
