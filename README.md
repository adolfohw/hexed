# Hexed

A simple CLI to visualize the bytes of a file, in a similar fashion to `hexdump`, but prettier*!

\* Disclaimer: you might not find it prettier.

## Installation

```ps
cargo install hexed
```

## Usage

```ps
hexed path/to/file
```

## Features

Using a shameless copy-paste of `hexed --help`:

```ps
FLAGS:
    -A, --no-ascii     Disables the ASCII sidebar
    -C, --no-colors    Disables colors in the output
    -G, --no-guides    Disables offset guides
    -o, --octal        Displays the bytes as octal numbers
    -h, --help         Prints help information
    -V, --version      Prints version information

OPTIONS:
    -n, --length <LENGTH>    Limits the amount of bytes to display
    -s, --skip <OFFSET>      Skips the first offset bytes
```

## You said it was prettier

![Hexed preview](/assets/preview.png)
