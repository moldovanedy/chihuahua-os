use boot_info::framebuffer::FramebufferData;
use uefi::{
    boot::{self},
    proto::console::gop::{self, GraphicsOutput, PixelFormat},
    Handle,
};

pub fn set_appropriate_framebuffer(pref_width: u32, pref_height: u32) -> Option<FramebufferData> {
    let gop_handle: Result<Handle, uefi::Error> = boot::get_handle_for_protocol::<GraphicsOutput>();
    if gop_handle.is_err() {
        return None;
    }

    let gop_handle: Handle = gop_handle.unwrap();
    let gop: Result<boot::ScopedProtocol<GraphicsOutput>, uefi::Error> =
        boot::open_protocol_exclusive::<GraphicsOutput>(gop_handle);
    if gop.is_err() {
        return None;
    }

    let mut gop: boot::ScopedProtocol<GraphicsOutput> = gop.unwrap();

    let mut best_mode: Option<gop::Mode> = None;
    let mut best_mode_info: gop::ModeInfo = gop.current_mode_info();
    let mut deviation: (i32, i32) = (
        (pref_width as i32) - (best_mode_info.resolution().0 as i32),
        (pref_height as i32) - (best_mode_info.resolution().1 as i32),
    );

    //query all modes and pick the most appropriate one
    gop.modes().for_each(|x| {
        let res: (i32, i32) = (
            x.info().resolution().0 as i32,
            x.info().resolution().1 as i32,
        );

        let this_deviation: (i32, i32) =
            ((pref_width as i32) - res.0, (pref_height as i32) - res.1);

        if abs(this_deviation.0 + this_deviation.1) < abs(deviation.0 + deviation.1) {
            best_mode = Some(x);
            best_mode_info = *x.info();
            deviation = this_deviation;
        }
    });

    if best_mode.is_none() {
        return None;
    }

    let result: Result<(), uefi::Error> = gop.set_mode(&best_mode.unwrap());
    if result.is_err() {
        return None;
    }

    return fb_data_from_mode_info(best_mode_info);
}

fn abs(val: i32) -> i32 {
    if val < 0 { -val } else { val }
}

pub fn fb_data_from_mode_info(mode_info: gop::ModeInfo) -> Option<FramebufferData> {
    let mut red_mask: u32 = 0;
    let mut green_mask: u32 = 0;
    let mut blue_mask: u32 = 0;
    let mut bits_per_pixel: u8 = 0;

    match mode_info.pixel_format() {
        PixelFormat::Bgr => {
            red_mask = 0xff;
            green_mask = 0xff_00;
            blue_mask = 0xff_00_00;
            bits_per_pixel = 32;
        }
        PixelFormat::Rgb => {
            red_mask = 0xff_00_00;
            green_mask = 0xff_00;
            blue_mask = 0xff;
            bits_per_pixel = 32;
        }
        PixelFormat::Bitmask => {
            let mask: Option<gop::PixelBitmask> = mode_info.pixel_bitmask();
            if mask.is_none() {
                return None;
            }

            let mask: gop::PixelBitmask = mask.unwrap();
            red_mask = mask.red;
            green_mask = mask.green;
            blue_mask = mask.blue;

            //get BPP
            for mask in [mask.red, mask.green, mask.blue, mask.reserved] {
                let mut bit_pos: u8 = 31;
                while bit_pos > bits_per_pixel {
                    if mask & (1 << bit_pos) != 0 {
                        bits_per_pixel = bit_pos + 1;
                        break;
                    }

                    bit_pos -= 1;
                }
            }
        }
        _ => {}
    }

    if red_mask == 0 || green_mask == 0 || blue_mask == 0 {
        return None;
    }

    return Some(FramebufferData::new(
        boot_info::GOP_VIRTUAL_ADDRESS,
        mode_info.resolution().0 as u32,
        mode_info.resolution().1 as u32,
        mode_info.stride() as u32,
        bits_per_pixel,
        red_mask,
        green_mask,
        blue_mask,
    ));
}
