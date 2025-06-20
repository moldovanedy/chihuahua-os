static DIGITS: &str = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const MIN_BASE: u64 = 2;
const MAX_BASE: u64 = 36;

//#region Structs
pub struct FmtResult {
    buffer: [u8; 21],
    len: u32,
}

impl FmtResult {
    pub fn new() -> Self {
        FmtResult {
            buffer: [0; 21],
            len: 0,
        }
    }

    pub fn to_str(&self) -> &str {
        if self.len > 21 {
            return "";
        }

        let result = core::str::from_utf8(&self.buffer[(21 - self.len) as usize..]);
        if result.is_err() {
            return "";
        }

        return result.unwrap();
    }

    fn push(&mut self, c: u8) {
        //silent fail
        if self.len >= 21 {
            return;
        }

        self.len += 1;
        self.buffer[(21 - self.len) as usize] = c;
    }
}

pub struct FmtResultBase {
    buffer: [u8; 65],
    len: u32,
}

impl FmtResultBase {
    pub fn new() -> Self {
        FmtResultBase {
            buffer: [0; 65],
            len: 0,
        }
    }

    pub fn to_str(&self) -> &str {
        if self.len > 65 {
            return "";
        }

        let result = core::str::from_utf8(&self.buffer[(65 - self.len) as usize..]);
        if result.is_err() {
            return "";
        }

        return result.unwrap();
    }

    fn push(&mut self, c: u8) {
        //silent fail
        if self.len >= 65 {
            return;
        }

        self.len += 1;
        self.buffer[(65 - self.len) as usize] = c;
    }
}
//#endregion

//#region Number to string
pub fn i64_to_str(value: i64) -> FmtResult {
    let mut result = FmtResult::new();
    let mut value: i64 = value;

    let neg = value < 0;

    loop {
        //20 so as to also adjust for the negative sign; an u64 should never exceed 20 characters, so reaching
        //this means there is an error
        if result.len >= 20 {
            break;
        }

        result.push(b'0' + (value % 10).abs() as u8);
        value /= 10;
        // check at end of loop, so 0 prints to "0"
        if value == 0 {
            break;
        }
    }

    if neg {
        result.push(b'-');
    }

    return result;
}

pub fn u64_to_str(value: u64) -> FmtResult {
    let mut result = FmtResult::new();
    let mut value: u64 = value;

    loop {
        //20 so as to also adjust for the negative sign; an u64 should never exceed 20 characters, so reaching
        //this means there is an error
        if result.len >= 20 {
            break;
        }

        result.push(b'0' + (value % 10) as u8);
        value /= 10;
        // check at end of loop, so 0 prints to "0"
        if value == 0 {
            break;
        }
    }

    return result;
}

pub fn i32_to_str(value: i32) -> FmtResult {
    let mut result = FmtResult::new();
    let mut value: i32 = value;

    let neg = value < 0;

    loop {
        //20 so as to also adjust for the negative sign; an u64 should never exceed 20 characters, so reaching
        //this means there is an error
        if result.len >= 20 {
            break;
        }

        result.push(b'0' + (value % 10).abs() as u8);
        value /= 10;
        // check at end of loop, so 0 prints to "0"
        if value == 0 {
            break;
        }
    }

    if neg {
        result.push(b'-');
    }

    return result;
}

pub fn u32_to_str(value: u32) -> FmtResult {
    let mut result = FmtResult::new();
    let mut value: u32 = value;

    loop {
        //20 so as to also adjust for the negative sign; an u64 should never exceed 20 characters, so reaching
        //this means there is an error
        if result.len >= 20 {
            break;
        }

        result.push(b'0' + (value % 10) as u8);
        value /= 10;
        // check at end of loop, so 0 prints to "0"
        if value == 0 {
            break;
        }
    }

    return result;
}

pub fn i16_to_str(value: i16) -> FmtResult {
    let mut result = FmtResult::new();
    let mut value: i16 = value;

    let neg = value < 0;

    loop {
        //20 so as to also adjust for the negative sign; an u64 should never exceed 20 characters, so reaching
        //this means there is an error
        if result.len >= 20 {
            break;
        }

        result.push(b'0' + (value % 10).abs() as u8);
        value /= 10;
        // check at end of loop, so 0 prints to "0"
        if value == 0 {
            break;
        }
    }

    if neg {
        result.push(b'-');
    }

    return result;
}

pub fn u16_to_str(value: u16) -> FmtResult {
    let mut result = FmtResult::new();
    let mut value: u16 = value;

    loop {
        //20 so as to also adjust for the negative sign; an u64 should never exceed 20 characters, so reaching
        //this means there is an error
        if result.len >= 20 {
            break;
        }

        result.push(b'0' + (value % 10) as u8);
        value /= 10;
        // check at end of loop, so 0 prints to "0"
        if value == 0 {
            break;
        }
    }

    return result;
}

pub fn i8_to_str(value: i8) -> FmtResult {
    let mut result = FmtResult::new();
    let mut value: i8 = value;

    let neg = value < 0;

    loop {
        //20 so as to also adjust for the negative sign; an u64 should never exceed 20 characters, so reaching
        //this means there is an error
        if result.len >= 20 {
            break;
        }

        result.push(b'0' + (value % 10).abs() as u8);
        value /= 10;
        // check at end of loop, so 0 prints to "0"
        if value == 0 {
            break;
        }
    }

    if neg {
        result.push(b'-');
    }

    return result;
}

pub fn u8_to_str(value: u8) -> FmtResult {
    let mut result = FmtResult::new();
    let mut value: u8 = value;

    loop {
        //20 so as to also adjust for the negative sign; an u64 should never exceed 20 characters, so reaching
        //this means there is an error
        if result.len >= 20 {
            break;
        }

        result.push(b'0' + (value % 10) as u8);
        value /= 10;
        // check at end of loop, so 0 prints to "0"
        if value == 0 {
            break;
        }
    }

    return result;
}
//#endregion

//#region Convert to any base
pub fn i64_to_str_base(value: i64, base: u32) -> FmtResultBase {
    let mut result = FmtResultBase::new();
    let mut value: i64 = value;
    let neg = value < 0;
    let base = base as u64;

    if base < MIN_BASE || base > MAX_BASE {
        return FmtResultBase::new();
    }

    loop {
        //64 so as to also adjust for the negative sign; an i64 should never exceed 64 characters in base 2,
        //so reaching this means there is an error
        if result.len >= 64 {
            break;
        }

        let idx: usize = (value % base as i64) as usize;
        if idx > 35 {
            return FmtResultBase::new();
        }

        result.push(DIGITS.as_bytes()[idx]);
        value /= base as i64;
        // check at end of loop, so 0 prints to "0"
        if value == 0 {
            break;
        }
    }

    if neg {
        result.push(b'-');
    }

    return result;
}

pub fn u64_to_str_base(value: u64, base: u32) -> FmtResultBase {
    let mut result = FmtResultBase::new();
    let mut value: u64 = value;
    let base: u64 = base as u64;

    if base < MIN_BASE || base > MAX_BASE {
        return FmtResultBase::new();
    }

    loop {
        //64 so as to also adjust for the negative sign; an u64 should never exceed 64 characters in binary,
        //so reaching this means there is an error
        if result.len >= 64 {
            break;
        }

        let idx: usize = (value % base) as usize;
        if idx > 35 {
            return FmtResultBase::new();
        }

        result.push(DIGITS.as_bytes()[idx]);
        value /= base;
        // check at end of loop, so 0 prints to "0"
        if value == 0 {
            break;
        }
    }

    return result;
}
//#endregion

//#region String to number
pub fn str_to_int(string: &str) -> Option<i64> {
    let mut value: i64 = 0;
    let mut is_negative: bool = false;

    let mut chars: core::str::Chars<'_> = string.chars();
    let first = chars.next();
    if first.is_none() {
        return Some(0);
    } else if first.unwrap() == '-' {
        is_negative = true;
    }

    let mut char: Option<char> = chars.next();
    let mut multiplier: u32 = 0;
    while char.is_some() {
        let c = char.unwrap();
        if c == '.' {
            //this is float
            return None;
        }

        let digit: Option<u32> = c.to_digit(10);
        if digit.is_none() {
            return None;
        }

        value = value + (digit.unwrap() * multiplier) as i64;
        char = chars.next();
        multiplier *= 10;
    }

    if is_negative {
        value = -value;
    }

    return Some(value);
}
//#endregion
