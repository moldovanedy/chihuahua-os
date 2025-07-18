use dog_essentials::pointer_ops;

pub const PSF2_MAGIC: u32 = 0x864ab572;

pub struct AsciiPsfFont {
    magic: u32,
    version: u32,
    header_size: u32,
    flags: u32,
    num_glyphs: u32,
    bytes_per_glyph: u32,
    height: u32,
    width: u32,

    base_addr: u64,
}

impl AsciiPsfFont {
    pub fn magic(&self) -> u32 {
        self.magic
    }

    pub fn version(&self) -> u32 {
        self.version
    }

    pub fn header_size(&self) -> u32 {
        self.header_size
    }

    pub fn flags(&self) -> u32 {
        self.flags
    }

    pub fn num_glyphs(&self) -> u32 {
        self.num_glyphs
    }

    pub fn bytes_per_glyph(&self) -> u32 {
        self.bytes_per_glyph
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub const fn default() -> Self {
        AsciiPsfFont {
            magic: 0,
            version: 0,
            header_size: 0,
            flags: 0,
            num_glyphs: 0,
            bytes_per_glyph: 0,
            height: 0,
            width: 0,
            base_addr: 0,
        }
    }

    pub unsafe fn from_raw(bytes: *const u8) -> Option<Self> {
        let mut traverser = pointer_ops::PointerTraverser::new(bytes);

        unsafe {
            let base_addr: u64 = bytes as *const u64 as u64;

            let magic: u32 = u32::from_le(traverser.read_and_advance::<u32>());
            if magic != PSF2_MAGIC {
                return None;
            }

            let version: u32 = u32::from_le(traverser.read_and_advance::<u32>());
            if version != 0 {
                return None;
            }

            let header_size: u32 = u32::from_le(traverser.read_and_advance::<u32>());

            //if 1, it is Unicode, but we can just ignore the table (it's at the end of the file)
            let flags: u32 = u32::from_le(traverser.read_and_advance::<u32>());

            let num_glyphs: u32 = u32::from_le(traverser.read_and_advance::<u32>());
            let bytes_per_glyph: u32 = u32::from_le(traverser.read_and_advance::<u32>());
            let height: u32 = u32::from_le(traverser.read_and_advance::<u32>());
            let width: u32 = u32::from_le(traverser.read_and_advance::<u32>());

            return Some(AsciiPsfFont {
                magic,
                version,
                header_size,
                flags,
                num_glyphs,
                bytes_per_glyph,
                height,
                width,

                base_addr,
            });
        };
    }

    pub fn get_glyph(&self, code_point: u32) -> *const u8 {
        let all_glyphs_offset: u64 = self.base_addr + self.header_size as u64;
        let glyph_offset: u64 = all_glyphs_offset
            + (if code_point < self.num_glyphs {
            code_point
        } else {
            0
        } * self.bytes_per_glyph) as u64;

        return glyph_offset as *const u8;
    }
}
