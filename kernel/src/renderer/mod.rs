static mut BASE_ADDR: u64 = 0;
static mut WIDTH: u32 = 0;
static mut HEIGHT: u32 = 0;
static mut PITCH: u32 = 0;
static mut BITS_PER_PIXEL: u8 = 0;

static mut RED_MASK: u32 = 0;
static mut GREEN_MASK: u32 = 0;
static mut BLUE_MASK: u32 = 0;

static mut KNOWN_FORMAT: ColorFormat = ColorFormat::Unknown;

pub fn setup_fb(fb_info: FramebufferInfo) -> bool {
    if fb_info.width == 0 || fb_info.height == 0
    // || fb_info.red_size == 0
    // || fb_info.green_size == 0
    // || fb_info.blue_size == 0
    {
        // unsafe {
        //     core::arch::asm!("cli");
        //     core::arch::asm!("hlt");
        // }
        return false;
    }

    unsafe {
        BASE_ADDR = fb_info.address;
        WIDTH = fb_info.width;
        HEIGHT = fb_info.height;
        PITCH = fb_info.pitch;
        BITS_PER_PIXEL = fb_info.bits_per_pixel;

        RED_MASK = (1 << (fb_info.red_size - 1)) << fb_info.red_pos;
        GREEN_MASK = (1 << (fb_info.green_size - 1)) << fb_info.green_pos;
        BLUE_MASK = (1 << (fb_info.blue_size - 1)) << fb_info.blue_pos;

        if fb_info.red_size == 8 && fb_info.green_size == 8 && fb_info.blue_size == 8 {
            if fb_info.blue_pos == 0 && fb_info.green_pos == 8 && fb_info.red_pos == 16 {
                KNOWN_FORMAT = ColorFormat::Bgrx8888
            } else if fb_info.red_pos == 8 && fb_info.green_pos == 16 && fb_info.blue_pos == 24 {
                KNOWN_FORMAT = ColorFormat::Xrgb8888
            }
        }
    }

    return true;
}

pub fn draw_test() {
    unsafe {
        let fb: *mut u8 = BASE_ADDR as *mut u8;
        *fb.add(0) = 0x00; // blue
        *fb.add(1) = 0xFF; // green
        *fb.add(2) = 0xFF; // red
        *fb.add(3) = 0x00; // padding

        *fb.add(4) = 0x00; // blue
        *fb.add(5) = 0xFF; // green
        *fb.add(6) = 0xFF; // red
        *fb.add(7) = 0x00; // padding

        *fb.add(8) = 0x00; // blue
        *fb.add(9) = 0xFF; // green
        *fb.add(10) = 0xFF; // red
        *fb.add(11) = 0x00; // padding
    }

    draw_rect(20, 40, 250, 300, 0xff_ff_ff);
}

pub fn draw_rect(x: u32, y: u32, width: u32, height: u32, color: u32) -> bool {
    unsafe {
        if x + width > WIDTH || y + height > HEIGHT {
            return false;
        }

        let fb: *mut u8 = BASE_ADDR as *mut u8;
        // unsafe {
        //     core::arch::asm!("mov rax, {}", in(reg) BASE_ADDR);
        //     core::arch::asm!("cli");
        //     core::arch::asm!("hlt");
        // }

        let bytes_per_pixel: u32 = (BITS_PER_PIXEL / 8) as u32;

        // for y_pos in y..(y + height) {
        //     for x_pos in x..(x + width) {
        //         *fb.byte_add(
        //             (bytes_per_pixel * y_pos * PITCH + bytes_per_pixel * x_pos) as usize,
        //         ) = color;
        //     }
        // }

        for row in 0..height {
            let draw_y = y + row;
            if draw_y >= HEIGHT {
                break;
            }

            for col in 0..width {
                let draw_x = x + col;
                if draw_x >= WIDTH {
                    break;
                }

                let offset: usize =
                    (bytes_per_pixel * draw_y * PITCH + draw_x * bytes_per_pixel) as usize;
                let pixel_ptr: *mut u8 = fb.add(offset) as *mut u8;

                // Assuming 32bpp, format: [B, G, R, 0] (could vary)
                *pixel_ptr.add(0) = 0;
                *pixel_ptr.add(1) = 0xff;
                *pixel_ptr.add(2) = 0x40;
                *pixel_ptr.add(3) = 0; // Alpha or unused
            }
        }

        return true;
    }
}

pub struct FramebufferInfo {
    pub address: u64,
    pub width: u32,
    pub height: u32,
    pub pitch: u32,
    pub bits_per_pixel: u8,

    pub red_pos: u8,
    pub red_size: u8,
    pub green_pos: u8,
    pub green_size: u8,
    pub blue_pos: u8,
    pub blue_size: u8,
}

enum ColorFormat {
    Unknown,
    ///This means blue, green, red, reserved in the linear framebuffer, going from left to right (like little-endian).
    Bgrx8888,
    ///This means reserved, red, green, blue in the linear framebuffer, going from left to right (like big-endian).
    Xrgb8888,
}
