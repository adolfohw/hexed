use colored::Colorize;
use std::{
    env, fs,
    io::{self, Write},
    iter,
};

const ROW_SIZE: usize = 0x10;

// Print without panicking
macro_rules! safe_print {
    (in $($color:ident)+: $($arg:tt)*) => {{
        let msg = format!("{}", format_args!($($arg)*));
        if write!(io::stdout(), "{}", msg$(.$color())+).is_err() {
            return;
        }
    }};

    ($($arg:tt)*) => {{
        if write!(io::stdout(), "{}", format_args!($($arg)*)).is_err() {
            return;
        }
    }};
}

macro_rules! safe_println {
    () => {
        safe_print!("\n");
    };

    (in $($color:ident)+: $($arg:tt)*) => {
        safe_print!(in $($color)+: $($arg)*);
        safe_println!();
    };

    ($($arg:tt)*) => {
        safe_print!($($arg)*);
        safe_println!();
    }
}

macro_rules! safe_eprint {
    ($($arg:tt)*) => {{
        let msg = format!("{}", format_args!($($arg)*));
        if write!(io::stderr(), "{}", msg.red()).is_err() {
            return;
        }
    }};
}

macro_rules! safe_eprintln {
    () => {
        safe_eprint!("\n");
    };

    ($($arg:tt)*) => {
        safe_eprint!($($arg)*);
        safe_eprintln!();
    };
}

fn main() {
    let filler: String = iter::repeat(' ').take(ROW_SIZE * 3).collect();
    let path = match env::args().nth(1) {
        Some(path) => path,
        None => {
            safe_eprintln!("You must provide a path");
            return;
        }
    };
    let file = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(_) => {
            safe_eprintln!("Invalid path");
            return;
        }
    };

    // Header
    safe_print!(in bold green: "  Offset ");
    for offset in 0..ROW_SIZE {
        safe_print!(in green: "{:02X} ", offset);
    }
    safe_println!();

    // Dump
    let rows = file.len() / ROW_SIZE + (file.len() % ROW_SIZE).min(1);
    for row in 0..rows {
        // Offset magnitude
        safe_print!(in green: "{:08X} ", row * ROW_SIZE);
        // Row dump
        let start = row * ROW_SIZE;
        let end = (start + ROW_SIZE).min(file.len());
        // Bytes
        let mut safe_printed_bytes = 0;
        for byte in &file[start..end] {
            match byte {
                // Null
                0 => safe_print!(in bright_black: "{:02X} ", byte),
                // Non-printable ASCII
                0x01..=0x1F | 0x7F => safe_print!(in bright_yellow: "{:02X} ", byte),
                // Printable ASCII
                b' '..=b'~' => safe_print!(in bright_cyan: "{:02X} ", byte),
                // Other
                _ => safe_print!("{:02X} ", byte),
            }
            safe_printed_bytes += 1;
        }
        if safe_printed_bytes < ROW_SIZE {
            safe_print!("{}", &filler[0..(ROW_SIZE - safe_printed_bytes) * 3]);
        }
        safe_print!(" ");
        // Text repr
        for byte in &file[start..end] {
            let mut ch = char::from(*byte);
            if !ch.is_ascii_graphic() {
                ch = ' ';
            }
            safe_print!(in bright_black: "{}", ch);
        }
        safe_println!();
    }
}
