#![allow(dead_code)]

use std::sync::atomic::{AtomicBool, Ordering};

static COLOR_ENABLED: AtomicBool = AtomicBool::new(true);

pub fn init_console() {
    if std::env::var("NO_COLOR").is_ok() {
        COLOR_ENABLED.store(false, Ordering::Relaxed);
        return;
    }

    #[cfg(windows)]
    enable_windows_vt();
}

#[cfg(windows)]
fn enable_windows_vt() {
    unsafe {
        type Handle = *mut std::ffi::c_void;
        type Dword = u32;
        type Bool = i32;

        const STD_OUTPUT_HANDLE: Dword = -11i32 as u32;
        const ENABLE_VIRTUAL_TERMINAL_PROCESSING: Dword = 0x0004;

        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn GetStdHandle(nStdHandle: Dword) -> Handle;
            fn GetConsoleMode(hConsoleHandle: Handle, lpMode: *mut Dword) -> Bool;
            fn SetConsoleMode(hConsoleHandle: Handle, dwMode: Dword) -> Bool;
        }

        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        if !handle.is_null() && handle != -1isize as Handle {
            let mut mode: Dword = 0;
            if GetConsoleMode(handle, &mut mode) != 0 {
                SetConsoleMode(handle, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING);
            }
        }
    }
}

pub fn is_color_enabled() -> bool {
    COLOR_ENABLED.load(Ordering::Relaxed)
}

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const ITALIC: &str = "\x1b[3m";
const UNDERLINE: &str = "\x1b[4m";

const RED: &str = "\x1b[38;5;203m";
const GREEN: &str = "\x1b[38;5;120m";
const YELLOW: &str = "\x1b[38;5;221m";
const BLUE: &str = "\x1b[38;5;75m";
const MAGENTA: &str = "\x1b[38;5;176m";
const CYAN: &str = "\x1b[38;5;81m";
const WHITE: &str = "\x1b[38;5;255m";
const GRAY: &str = "\x1b[38;5;244m";
const DARK_GRAY: &str = "\x1b[38;5;238m";

pub fn bold(s: &str) -> String {
    format_styled(BOLD, s)
}

pub fn dim(s: &str) -> String {
    format_styled(DIM, s)
}

pub fn italic(s: &str) -> String {
    format_styled(ITALIC, s)
}

pub fn underline(s: &str) -> String {
    format_styled(UNDERLINE, s)
}

pub fn red(s: &str) -> String {
    format_styled(RED, s)
}

pub fn green(s: &str) -> String {
    format_styled(GREEN, s)
}

pub fn yellow(s: &str) -> String {
    format_styled(YELLOW, s)
}

pub fn blue(s: &str) -> String {
    format_styled(BLUE, s)
}

pub fn magenta(s: &str) -> String {
    format_styled(MAGENTA, s)
}

pub fn cyan(s: &str) -> String {
    format_styled(CYAN, s)
}

pub fn white(s: &str) -> String {
    format_styled(WHITE, s)
}

pub fn gray(s: &str) -> String {
    format_styled(GRAY, s)
}

pub fn dark_gray(s: &str) -> String {
    format_styled(DARK_GRAY, s)
}

fn format_styled(code: &str, s: &str) -> String {
    if is_color_enabled() {
        format!("{}{}{}", code, s, RESET)
    } else {
        s.to_string()
    }
}
