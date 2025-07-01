static mut BASE_ADDR: u64 = 0;
static mut WIDTH: u32 = 0;
static mut HEIGHT: u32 = 0;
static mut PITCH: u32 = 0;
static mut BITS_PER_PIXEL: u8 = 0;

static mut RED_MASK: u32 = 0;
static mut GREEN_MASK: u32 = 0;
static mut BLUE_MASK: u32 = 0;

pub fn setup_fb(
    base_addr: u64,
    width: u32,
    height: u32,
    pitch: u32,
    bpp: u8,
    red_mask: u32,
    green_mask: u32,
    blue_mask: u32,
) {
    if width == 0 || height == 0 || red_mask == 0 || green_mask == 0 || blue_mask == 0 {
        return;
    }

    unsafe {
        BASE_ADDR = base_addr;
        WIDTH = width;
        HEIGHT = height;
        PITCH = pitch;
        BITS_PER_PIXEL = bpp;

        RED_MASK = red_mask;
        GREEN_MASK = green_mask;
        BLUE_MASK = blue_mask;
    }
}

fn convert_color_format(color: u32) -> u32 {
    unsafe {
        //if RGB32, discard the reserved channel
        if RED_MASK == 0xff_00_00 && GREEN_MASK == 0xff_00 && BLUE_MASK == 0xff {
            return color << 8;
        }

        //if BGR32, skip expensive calculations and rotate bits
        if RED_MASK == 0xff && GREEN_MASK == 0xff_00 && BLUE_MASK == 0xff_00_00 {
            //red from byte 3 to byte 2, green from byte 2 to byte 3, and blue from byte 1 to byte 4
            //last byte is reserved
            return ((color & 0xff_00_00) >> 8) | ((color & 0xff_00) << 8) | ((color & 0xff) << 24);
        }
    }
    
    return color;
}

pub(crate) fn cover_screen() {
    unsafe {
        let fb: *mut u8 = BASE_ADDR as *mut u8;
        let bytes_per_pixel: u32 = (BITS_PER_PIXEL / 8) as u32;
        let color: u32 = convert_color_format(0x80_00_00);

        for y_pos in 0..HEIGHT {
            for x_pos in 0..WIDTH {
                let base_addr: *mut u8 =
                    fb.add((bytes_per_pixel * y_pos * PITCH + bytes_per_pixel * x_pos) as usize);
                direct_draw(base_addr, color);
            }
        }
    }
}

#[inline]
fn direct_draw(fb: *mut u8, color: u32) {
    unsafe {
        *fb.add(0) = color as u8;
        *fb.add(1) = (color >> 8) as u8;
        *fb.add(2) = (color >> 16) as u8;
        *fb.add(3) = (color >> 24) as u8;
    }
}
