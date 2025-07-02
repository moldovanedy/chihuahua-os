use crate::k_drivers::x86_64;
use core::str;
use dog_essentials::sync::mutex::Mutex;

static WRITE_LOCK: Mutex<bool> = Mutex::new(false);

pub fn log(severity: Severity, message: &str) {
    init();

    match severity {
        Severity::Verbose => {
            write("[VERBOSE]: ");
        }
        Severity::Debug => {
            write("[DEBUG]: ");
        }
        Severity::Info => {
            write("[INFO]: ");
        }
        Severity::Warn => {
            write("[WARN]: ");
        }
        Severity::Error => {
            write("[ERROR]: ");
        }
        Severity::Fatal => {
            write("[FATAL]: ");
        }
    }

    write(message);
    write("\n");
}

pub fn log_verbose(message: &str) {
    log(Severity::Verbose, message);
}

pub fn log_debug(message: &str) {
    log(Severity::Debug, message);
}

pub fn log_info(message: &str) {
    log(Severity::Info, message);
}

pub fn log_warn(message: &str) {
    log(Severity::Warn, message);
}

pub fn log_error(message: &str) {
    log(Severity::Error, message);
}

pub fn log_fatal(message: &str) {
    log(Severity::Fatal, message);
}

pub fn log_raw(message: &str) {
    init();
    write(message);
}

fn write(message: &str) {
    WRITE_LOCK.lock();

    for chr in message.as_bytes() {
        //if non-ASCII, replace with '?'
        if *chr >= 128 {
            write_to_serial(b'?');
            continue;
        }

        //Break
        if *chr == 3 {
            write_to_serial(b'^');
            write_to_serial(b'B');
            continue;
        }

        //Beep
        if *chr == 7 {
            write_to_serial(b'^');
            write_to_serial(b'G');
            continue;
        }

        //Backspace
        if *chr == 8 {
            write_to_serial(b'^');
            write_to_serial(b'H');
            continue;
        }

        //Tab
        if *chr == 9 {
            write_to_serial(b'^');
            write_to_serial(b'I');
            continue;
        }

        //LF or CR
        if *chr == 10 || *chr == 13 {
            write_to_serial(b'\n');
            continue;
        }

        //Escape
        if *chr == 27 {
            write_to_serial(b'\\');
            write_to_serial(b'0');
            write_to_serial(b'3');
            write_to_serial(b'3');
            continue;
        }

        //Delete
        if *chr == 127 {
            write_to_serial(b'\\');
            write_to_serial(b'0');
            write_to_serial(b'3');
            write_to_serial(b'3');
            write_to_serial(b'[');
            write_to_serial(b'3');
            write_to_serial(b'~');
            continue;
        }

        write_to_serial(*chr);
    }
}

fn write_to_serial(chr: u8) {
    #[cfg(target_arch = "x86_64")]
    x86_64::com_debug::write_char(chr);
}

fn init() {
    #[cfg(target_arch = "x86_64")]
    if x86_64::com_debug::is_initialized() {
        return;
    }

    #[cfg(target_arch = "x86_64")]
    x86_64::com_debug::init_serial();
}

pub enum Severity {
    Verbose = 1,
    Debug = 2,
    Info = 3,
    Warn = 4,
    Error = 5,
    Fatal = 6,
}
