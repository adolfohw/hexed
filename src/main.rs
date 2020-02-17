use clap::clap_app;
use colored::Colorize;
use std::{env, fs, iter};

macro_rules! ceprint {
    ($($arg:tt)*) => {
        let msg = format!("{}", format_args!($($arg)*));
        eprint!("{}", msg.bright_red().on_black());
    }
}

macro_rules! ceprintln {
    () => {
        eprintln!();
        return;
    };

    ($($arg:tt)*) => {{
        ceprint!($($arg)*);
        ceprintln!();
    }}
}

fn main() {
    // CLI
    let app = clap_app!(hexed =>
        (version: env!("CARGO_PKG_VERSION"))
        (author: env!("CARGO_PKG_AUTHORS"))
        (about: env!("CARGO_PKG_DESCRIPTION"))
        (@arg FILE: * "The file to be dumped")
        // Bytes config
        (@arg LENGTH: -n --length [LENGTH] "Limits the amount of bytes to display")
        (@arg OFFSET: -s --skip [OFFSET] "Skips the first offset bytes")
        // Display config
        (@arg OCTAL: -o --octal "Displays the bytes as octal numbers")
        (@arg GUIDES: -G --("no-guides") "Disables offset guides")
        (@arg COLORS: -C --("no-colors") "Disables colors in the output")
        (@arg ASCII: -A --("no-ascii") "Disables the ASCII sidebar")
    );

    // Check for a valid FILE value being present
    let matches = match app.get_matches_safe() {
        Ok(matches) => matches,
        Err(err) => {
            eprintln!("{}", err.message);
            return;
        }
    };
    let path = matches.value_of("FILE").unwrap();
    let file = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(_) => ceprintln!("'{}' is not a valid path", path),
    };

    // Config
    let length = match matches.value_of("LENGTH") {
        Some(len) => {
            if let Ok(len) = len.parse() {
                len
            } else {
                ceprintln!("'{}' is not a valid length", len);
            }
        }
        None => file.len(),
    };
    let offset = match matches.value_of("OFFSET") {
        Some(os) => {
            if let Ok(os) = os.parse() {
                os
            } else {
                ceprintln!("'{}' is not a valid offset", os);
            }
        }
        None => 0_usize,
    };

    // Flags
    let is_octal = matches.is_present("OCTAL");
    let use_guides = !matches.is_present("GUIDES");
    let use_colors = !matches.is_present("COLORS");
    let show_sidebar = !matches.is_present("ASCII");

    // Control
    let row_size = if is_octal { 0o10 } else { 0x10 };
    let num_size = if is_octal { 5 } else { 3 };
    let filler: String = iter::repeat(' ').take(row_size * num_size).collect();

    macro_rules! cprint {
        (in $($color:ident)+: $($arg:tt)*) => {{
            if use_colors {
                let msg = format!("{}", format_args!($($arg)*));
                print!("{}", msg$(.$color())+);
            } else {
                print!($($arg)*);
            }
        }};

        ($($arg:tt)*) => {
            print!($($arg)*);
        };
    }

    macro_rules! cprintln {
        () => {
            cprint!("\n");
        };

        (in $($color:ident)+: $($arg:tt)*) => {
            cprint!(in $($color)+: $($arg)*);
            cprintln!();
        };

        ($($arg:tt)*) => {
            cprint!($($arg)*);
            cprintln!();
        }
    }

    // Set for octal or hex
    macro_rules! logic {
        ($num_format:literal, $mag_format:literal) => {
            // Header
            if use_guides {
                cprint!(in bold green: "  Offset ");
                for offset in 0..row_size {
                    cprint!(in green: $num_format, offset);
                }
                cprintln!();
            }

            // Dump
            let rows = (length - offset) / row_size + ((length - offset) % row_size).min(1);
            for row in 0..rows {
                // Offset magnitude
                if use_guides {
                    cprint!(in green: $mag_format, row * row_size);
                }

                // Row dump
                let start = offset + row * row_size;
                let end = (start + row_size).min(length);

                // Bytes
                let mut printed_bytes = 0;
                for byte in &file[start..end] {
                    match byte {
                        // Null
                        0 => cprint!(in bright_black: $num_format, byte),
                        // Non-printable ASCII
                        0x01..=0x1F | 0x7F => cprint!(in bright_yellow: $num_format, byte),
                        // Printable ASCII
                        b' '..=b'~' => cprint!(in bright_cyan: $num_format, byte),
                        // Other
                        _ => cprint!($num_format, byte),
                    }
                    printed_bytes += 1;
                }
                if printed_bytes < row_size {
                    cprint!("{}", &filler[0..(row_size - printed_bytes) * num_size]);
                }
                cprint!(" ");

                // ASCII repr
                if show_sidebar {
                    for byte in &file[start..end] {
                        let mut ch = char::from(*byte);
                        if !ch.is_ascii_graphic() {
                            ch = ' ';
                        }
                        cprint!(in bright_black: "{}", ch);
                    }
                }
                cprintln!();
            }
        };
    }
    if is_octal {
        logic!("{:04o} ", "{:08o} ");
    } else {
        logic!("{:02X} ", "{:08X} ");
    }
    cprintln!("{} bytes in {}", file.len(), path);
}
