use std::{
    fs::File,
    io::{self, prelude::*, SeekFrom},
    marker::PhantomData,
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
};

use crossterm::{queue, style::*};
use structopt::StructOpt;

macro_rules! colorize {
    (reset) => {
        queue!(io::stdout(), ResetColor, SetAttribute(Attribute::Reset));
    };
    (Fg::$color:tt) => {
        SetForegroundColor(Color::$color)
    };
    (Bg::$color:tt) => {
        SetBackgroundColor(Color::$color)
    };
    (Attr::$attr:tt) => {
        SetAttribute(Attribute::$attr)
    };
    (in $($cmd:tt::$arg:tt),+) => {
        queue!(io::stdout(), $(colorize!($cmd::$arg)),+);
    };
    (if $flag:expr => $(in $($cmd:tt::$arg:tt),+ $code:block)+) => {
        #[allow(unused_must_use)] {
            $({
                if $flag {
                    colorize!(in $($cmd::$arg),+);
                }
                $code
            })+
            if $flag {
                colorize!(reset);
            }
        }
    };
}

#[derive(StructOpt)]
#[structopt(author, about)]
pub struct Hexed {
    /// The file to be dumped
    #[structopt(name = "FILE")]
    path: PathBuf,

    #[structopt(flatten)]
    opts: HexedOpts,

    #[structopt(flatten)]
    flags: HexedFlags,
}

#[derive(StructOpt)]
#[structopt(rename_all = "screaming-snake")]
struct HexedOpts {
    /// Limits the amount of bytes to display
    #[structopt(short = "n", long = "length")]
    length: Option<usize>,
    /// Skips the first `OFFSET` bytes
    #[structopt(short = "s", long = "skip")]
    offset: Option<u64>,
}

#[derive(StructOpt)]
struct HexedFlags {
    /// Displays the bytes as octal numbers
    #[structopt(short, long)]
    octal: bool,
    /// Disables offset guides
    #[structopt(short = "G", long)]
    no_guides: bool,
    /// Disables colors in the output
    #[structopt(short = "C", long)]
    no_colors: bool,
    /// Disables the ASCII sidebar
    #[structopt(short = "A", long)]
    no_ascii: bool,
}

struct Bytes<'b> {
    file: File,
    max: usize,
    buf: [u8; 0x10],
    phantom: PhantomData<&'b u8>,
}

impl Bytes<'_> {
    fn get(&self, n: usize) -> &[u8] {
        &self.buf[..n]
    }
}

impl Iterator for Bytes<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        match self.file.read(&mut self.buf[0..self.max]) {
            Ok(num) if num > 0 => Some(num),
            _ => None,
        }
    }
}

static ABORT: AtomicBool = AtomicBool::new(false);

impl Hexed {
    fn is_colored(&self) -> bool {
        !self.flags.no_colors
    }

    fn row_width(&self) -> usize {
        if self.flags.octal {
            0o10
        } else {
            0x10
        }
    }

    fn byte_width(&self) -> usize {
        if self.flags.octal {
            4
        } else {
            2
        }
    }

    fn row_character_width(&self) -> usize {
        self.row_width() * (self.byte_width() + 1)
    }

    fn bytes<'b>(&self) -> io::Result<Bytes<'b>> {
        let mut file = File::open(&self.path)?;
        file.seek(SeekFrom::Start(self.opts.offset.unwrap_or_default()))?;
        Ok(Bytes {
            file,
            max: self.row_width(),
            buf: Default::default(),
            phantom: PhantomData,
        })
    }

    fn print_ruler(&self, mid: char) {
        print!("─────────{}", mid);
        for _ in 0..self.row_character_width() {
            print!("─");
        }
    }

    fn print_header(&self) {
        if self.flags.no_guides {
            return;
        }
        colorize!(if self.is_colored() => in Fg::Green, Attr::Bold {
            print!("  Offset │ ");
            for i in 0..self.row_width() {
                if self.flags.octal {
                    print!("{:04o} ", i);
                } else {
                    print!("{:02X} ", i);
                }
            }
            println!();
            self.print_ruler('┼');
        });
    }

    fn print_vert_guide(&self, row: usize) {
        if self.flags.no_guides {
            return;
        }
        colorize!(if self.is_colored() => in Fg::Green, Attr::Bold {
            if self.flags.octal {
                print!("{:08o} │ ", row * 0o10);
            } else {
                print!("{:08X} │ ", row * 0x10);
            }
        });
    }

    #[allow(unused_must_use)]
    fn print_row(&self, bytes: &[u8]) {
        for i in 0..self.row_width() {
            let byte = bytes.get(i).unwrap_or(&0);
            if self.is_colored() {
                match byte {
                    // Null
                    0 => colorize!(in Fg::DarkGrey),
                    // Non-printable ASCII
                    0x01..=0x1F | 0x7F => colorize!(in Fg::Yellow),
                    // Printable ASCII
                    b' '..=b'~' => colorize!(in Fg::Cyan),
                    // Other
                    _ => colorize!(in Fg::Reset),
                };
            }
            if i >= bytes.len() {
                for _ in 0..self.byte_width() + 1 {
                    print!(" ");
                }
                continue;
            }
            if self.flags.octal {
                print!("{:04o} ", byte);
            } else {
                print!("{:02X} ", byte);
            }
        }
        if self.is_colored() {
            colorize!(reset);
        }
    }

    fn print_ascii_sidebar(&self, bytes: &[u8]) {
        if self.flags.no_ascii {
            return;
        }
        colorize!(if self.is_colored() => in Fg::DarkGrey {
            print!(" ");
            for mut ascii in bytes.iter().copied().map(<char as From<u8>>::from) {
                if !ascii.is_ascii_graphic() {
                    ascii = ' ';
                }
                print!("{}", ascii);
            }
        });
    }

    fn print_footer(&self, bytes: u64) {
        colorize!(if self.is_colored() =>
            in Fg::Green, Attr::Bold {
                if !self.flags.no_guides {
                    self.print_ruler('┴');
                }
            }
            in Attr::Reset {
                print!("\n{} bytes in `{}`", bytes, self.path.to_string_lossy());
            }
        );
    }

    #[allow(unused_must_use)]
    pub fn run(self) -> io::Result<()> {
        let limit = self.opts.length.unwrap_or_else(usize::max_value);
        let mut bytes_read = 0;
        let mut bytes = self.bytes()?;
        let mut row = 0;
        ctrlc::set_handler(|| ABORT.store(true, Ordering::Release));
        self.print_header();
        println!();
        while let Some(available_bytes) = bytes.next() {
            if ABORT.load(Ordering::Acquire) {
                break;
            }
            let bytes = bytes.get(available_bytes.min(limit - bytes_read));
            self.print_vert_guide(row);
            self.print_row(bytes);
            self.print_ascii_sidebar(bytes);
            println!();
            row += 1;
            bytes_read += available_bytes;
            if bytes_read >= limit {
                break;
            }
        }
        self.print_footer(bytes.file.seek(SeekFrom::End(0)).unwrap());
        println!();
        Ok(())
    }
}
