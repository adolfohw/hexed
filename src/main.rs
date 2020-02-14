use std::{
    env, fs,
    io::{self, Write},
    iter,
};

const ROW_SIZE: usize = 0x10;

// Print without panicking
macro_rules! safe_print {
    ($($arg:tt)*) => {
        if write!(io::stdout(), "{}", format_args!($($arg)*)).is_err() {
            return;
        }
    };
}

macro_rules! safe_println {
    () => {
        safe_print!("\n");
    };

    ($($arg:tt)*) => {
        safe_print!($($arg)*);
        safe_println!();
    }
}

fn main() {
    let filler: String = iter::repeat(' ').take(ROW_SIZE * 3).collect();
    let path = match env::args().nth(1) {
        Some(path) => path,
        None => {
            safe_println!("You must provide a path");
            return;
        }
    };
    let file = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(_) => {
            safe_println!("Invalid path");
            return;
        }
    };

    // Header
    safe_print!("  Offset ");
    for offset in 0..ROW_SIZE {
        safe_print!("{:02X} ", offset);
    }
    safe_println!();

    // Dump
    let rows = file.len() / ROW_SIZE + (file.len() % ROW_SIZE).min(1);
    for row in 0..rows {
        // Offset magnitude
        safe_print!("{:08X} ", row * ROW_SIZE);
        // Row dump
        let start = row * ROW_SIZE;
        let end = (start + ROW_SIZE).min(file.len());
        // Bytes
        let mut safe_printed_bytes = 0;
        for byte in &file[start..end] {
            safe_print!("{:02X} ", byte);
            safe_printed_bytes += 1;
        }
        if safe_printed_bytes < ROW_SIZE {
            safe_print!("{}", &filler[0..(ROW_SIZE - safe_printed_bytes) * 3]);
        }
        // Text repr
        for byte in &file[start..end] {
            let mut ch = char::from(*byte);
            if !ch.is_ascii_graphic() {
                ch = '⋅';
            }
            safe_print!("{}", ch);
        }
        safe_println!();
    }
}
