use multiboot2::{BootInformation, BootInformationHeader, FramebufferType};

pub fn init_mb2_modules(multiboot2_info: u64) {
    unsafe {
        let boot_info: Result<BootInformation<'_>, multiboot2::LoadError> =
            BootInformation::load(multiboot2_info as *const BootInformationHeader);
        if boot_info.is_err() {
            return;
        }

        let boot_info: BootInformation<'_> = boot_info.unwrap();
        let fb_tag = boot_info.framebuffer_tag();
        if fb_tag.is_none() {
            return;
        }

        let fb_tag = fb_tag.unwrap();
        if fb_tag.is_err() {
            return;
        }

        let fb_tag: &multiboot2::FramebufferTag = fb_tag.unwrap();
        init_framebuffer(fb_tag);
    }
}

fn init_framebuffer(tag: &multiboot2::FramebufferTag) -> bool {
    use crate::renderer;

    if tag.buffer_type().is_err() {
        return false;
    }

    let color_info: multiboot2::FramebufferType<'_> = tag.buffer_type().unwrap();
    match color_info {
        FramebufferType::Indexed { .. } => return false,
        FramebufferType::Text => return false,
        FramebufferType::RGB { red, green, blue } => {
            let success: bool = renderer::setup_fb(renderer::FramebufferInfo {
                address: tag.address(),
                width: tag.width(),
                height: tag.height(),
                pitch: tag.pitch(),
                bits_per_pixel: tag.bpp(),

                red_pos: red.position,
                red_size: red.size,
                green_pos: green.position,
                green_size: green.size,
                blue_pos: blue.position,
                blue_size: blue.size,
            });

            if !success {
                return false;
            }
        }
    }

    return true;
}
