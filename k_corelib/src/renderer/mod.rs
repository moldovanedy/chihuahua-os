use core::fmt::Debug;

use crate::essentials_clone::geometry::rect::Rect;
use boot_info;

static mut BASE_ADDR: u64 = 0;
static mut WIDTH: u32 = 0;
static mut HEIGHT: u32 = 0;
static mut PITCH: u32 = 0;
static mut BITS_PER_PIXEL: u8 = 0;

static mut RED_MASK: u32 = 0;
static mut GREEN_MASK: u32 = 0;
static mut BLUE_MASK: u32 = 0;
static mut IS_RGB32: bool = false;

#[derive(Clone, Copy)]
pub struct Color {
    raw_color: u32,
}

impl Color {
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Color {
            raw_color: ((red as u32) << 16) | ((green as u32) << 8) | blue as u32,
        }
    }

    /// Returns a color from an u32 in the form of 0x00_RR_GG_BB (RGB on the least significant 24 bits).
    /// This will discard the most significant 8 bits.
    pub fn from_u32(color: u32) -> Self {
        if color > 0xff_ff_ff {
            return Color {
                raw_color: color & 0xff_ff_ff,
            };
        }

        return Color { raw_color: color };
    }

    pub fn get(&self) -> u32 {
        self.raw_color
    }

    pub fn r(&self) -> u8 {
        (self.raw_color >> 16) as u8
    }

    pub fn g(&self) -> u8 {
        (self.raw_color >> 8) as u8
    }

    pub fn b(&self) -> u8 {
        self.raw_color as u8
    }

    pub fn set_r(&mut self, red: u8) {
        self.raw_color |= (red as u32) << 16;
    }

    pub fn set_g(&mut self, green: u8) {
        self.raw_color |= (green as u32) << 8;
    }

    pub fn set_b(&mut self, blue: u8) {
        self.raw_color |= blue as u32;
    }
}

impl Debug for Color {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Color")
            .field("raw_color", &self.raw_color)
            .finish()
    }
}

pub fn fb_width() -> u32 {
    unsafe { WIDTH }
}

pub fn fb_height() -> u32 {
    unsafe { HEIGHT }
}

pub fn fb_pitch() -> u32 {
    unsafe { PITCH }
}

pub fn fb_bits_per_pixel() -> u8 {
    unsafe { BITS_PER_PIXEL }
}

pub fn fb_red_mask() -> u32 {
    unsafe { RED_MASK }
}

pub fn fb_green_mask() -> u32 {
    unsafe { GREEN_MASK }
}

pub fn fb_blue_mask() -> u32 {
    unsafe { BLUE_MASK }
}

pub fn setup_fb(fb_info: &boot_info::framebuffer::FramebufferData) -> bool {
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
    }

    return true;
}

pub fn draw_rect(x: u32, y: u32, width: u32, height: u32, color: Color) {
    unsafe {
        let fb: *mut u8 = BASE_ADDR as *mut u8;
        let bytes_per_pixel: u32 = (BITS_PER_PIXEL / 8) as u32;

        let color: u32 = if IS_RGB32 {
            color.get()
        } else {
            convert_color_format(color.get())
        };

        for y_pos in y..(y + height) {
            //can't draw lower
            if y_pos > HEIGHT {
                break;
            }

            for x_pos in x..(x + width) {
                //can't draw any more to the right, skip to the next row
                if x_pos > WIDTH {
                    break;
                }

                let base_addr: *mut u8 =
                    fb.add((bytes_per_pixel * y_pos * PITCH + bytes_per_pixel * x_pos) as usize);
                direct_draw(base_addr, color);
            }
        }
    }
}

pub fn draw_rect_buffer(x: u32, y: u32, width: u32, height: u32, buffer: &[Color]) {
    unsafe {
        if buffer.len() < (width * height) as usize {
            return;
        }

        let bytes_per_pixel: u32 = (BITS_PER_PIXEL / 8) as u32;
        let fb: *mut u8 = BASE_ADDR as *mut u8;
        let fb: *mut u8 =
            fb.byte_add(((bytes_per_pixel * y * PITCH) + (bytes_per_pixel * x)) as usize);

        for y_pos in 0..height {
            //can't draw lower
            if y_pos > HEIGHT {
                break;
            }

            for x_pos in 0..width {
                //can't draw any more to the right, skip to the next row
                if x_pos > WIDTH {
                    break;
                }

                let base_addr: *mut u8 =
                    fb.add((bytes_per_pixel * y_pos * PITCH + bytes_per_pixel * x_pos) as usize);
                direct_draw(base_addr, buffer[(y_pos * width + x_pos) as usize].get());
            }
        }
    }
}

pub fn clear_screen(color: Color) {
    unsafe {
        let bytes_per_pixel: u32 = (BITS_PER_PIXEL / 8) as u32;
        let fb: *mut u8 = BASE_ADDR as *mut u8;

        for y_pos in 0..HEIGHT {
            for x_pos in 0..WIDTH {
                let base_addr: *mut u8 =
                    fb.add((bytes_per_pixel * y_pos * PITCH + bytes_per_pixel * x_pos) as usize);
                direct_draw(base_addr, color.get());
            }
        }
    }
}

pub fn copy_region(src: &Rect, dest: &Rect) {
    unsafe {
        if src.width() > WIDTH as f32
            || src.width() <= 0.0
            || src.height() > HEIGHT as f32
            || src.height() <= 0.0
        {
            return;
        }

        if dest.width() > WIDTH as f32
            || dest.width() <= 0.0
            || dest.height() > HEIGHT as f32
            || dest.height() <= 0.0
        {
            return;
        }

        let fb: *mut u8 = BASE_ADDR as *mut u8;
        let bytes_per_pixel: u32 = (BITS_PER_PIXEL / 8) as u32;
        let src_offset: usize =
            (bytes_per_pixel * src.y() as u32 * PITCH + bytes_per_pixel * src.x() as u32) as usize;
        let dest_offset: usize = 
            (bytes_per_pixel * dest.y() as u32 * PITCH + bytes_per_pixel * dest.x() as u32) as usize;

        for y_pos in 0..src.height() as u32 {
            //can't draw lower
            if y_pos > dest.height() as u32
                || y_pos as i32 + src.y() as i32 > HEIGHT as i32
                || y_pos as i32 + dest.y() as i32 > HEIGHT as i32
            {
                break;
            }

            if y_pos as i32 + (dest.y() as i32) < 0 {
                continue;
            }

            for x_pos in 0..src.width() as u32 {
                //can't draw any more to the right, skip to the next row
                if x_pos > dest.width() as u32
                    || x_pos as i32 + src.x() as i32 > WIDTH as i32
                    || x_pos as i32 + dest.x() as i32 > WIDTH as i32
                {
                    break;
                }

                if x_pos as i32 + (dest.x() as i32) < 0 {
                    continue;
                }

                let off: usize =
                    (bytes_per_pixel * y_pos * PITCH + bytes_per_pixel * x_pos) as usize;
                let fb_src: *mut u8 = fb.add(src_offset + off);
                let fb_dest: *mut u8 = fb.add(dest_offset + off);

                *fb_dest.add(0) = *fb_src.add(0);
                *fb_dest.add(1) = *fb_src.add(1);
                *fb_dest.add(2) = *fb_src.add(2);
                *fb_dest.add(3) = *fb_src.add(3);
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
