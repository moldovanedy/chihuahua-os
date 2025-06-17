use core::str;

use crate::k_drivers::com_debug;

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

fn write(message: &str) {
    for chr in message.as_bytes() {
        //if non-ASCII, replace with '?'
        if *chr >= 128 {
            com_debug::write_char(b'?');
            continue;
        }

        com_debug::write_char(*chr);
    }
}

fn init() {
    if com_debug::is_initialized() {
        return;
    }

    com_debug::init_serial();
}

pub enum Severity {
    Verbose = 1,
    Debug = 2,
    Info = 3,
    Warn = 4,
    Error = 5,
    Fatal = 6,
}
