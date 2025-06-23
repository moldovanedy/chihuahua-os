#[repr(C)]
pub struct FramebufferData {
    address: u64,
    width: u32,
    height: u32,
    pitch: u32,
    bits_per_pixel: u8,
    red_bitmask: u32,
    green_bitmask: u32,
    blue_bitmask: u32,
}

impl FramebufferData {
    pub fn address(&self) -> u64 {
        self.address
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn pitch(&self) -> u32 {
        self.pitch
    }

    pub fn bits_per_pixel(&self) -> u8 {
        self.bits_per_pixel
    }

    pub fn red_bitmask(&self) -> u32 {
        self.red_bitmask
    }

    pub fn green_bitmask(&self) -> u32 {
        self.green_bitmask
    }

    pub fn blue_bitmask(&self) -> u32 {
        self.blue_bitmask
    }

    pub fn new(
        address: u64,
        width: u32,
        height: u32,
        pitch: u32,
        bits_per_pixel: u8,
        red_bitmask: u32,
        green_bitmask: u32,
        blue_bitmask: u32,
    ) -> Self {
        FramebufferData {
            address: address,
            width: width,
            height: height,
            pitch: pitch,
            bits_per_pixel: bits_per_pixel,
            red_bitmask: red_bitmask,
            green_bitmask: green_bitmask,
            blue_bitmask: blue_bitmask,
        }
    }
}