# Cunny

Cunny is a tiny, byte-based programming language whose programs use only two
tokens: `😭` and `💢`. Whitespace is ignored, and every instruction contains
exactly three tokens.

| Instruction | Meaning |
| --- | --- |
| `😭😭💢` | Move the pointer right |
| `😭😭😭` | Move the pointer left |
| `😭💢💢` | Increment the current byte |
| `😭💢😭` | Decrement the current byte |
| `💢😭💢` | Read one input byte |
| `💢😭😭` | Write the current byte |
| `💢💢💢` | Begin a loop |
| `💢💢😭` | End a loop |

Cells are wrapping 8-bit values. Moving right grows the tape; moving left from
its beginning is an error. Input sets the current cell to `0` at end-of-file.

## Install

On macOS or Linux, install the latest release with:

```sh
curl --proto '=https' --tlsv1.2 -LsSf \
  https://raw.githubusercontent.com/itsuki680/cunny-lang/main/install.sh | sh
```

The installer puts `cunny` in `~/.local/bin`. If your shell cannot find it, add
that directory to your `PATH`.

On any system with Rust installed, including Windows, use Cargo:

```sh
cargo install --git https://github.com/itsuki680/cunny-lang --locked
```

Prebuilt archives and checksums are also available on the
[latest release](https://github.com/itsuki680/cunny-lang/releases/latest).
Check the installation with:

```sh
cunny --version
```

Run a program with:

```sh
cunny examples/helloworld.cunny
```

Run it with the Imouto debugger to trace every instruction and tape state:

```sh
cunny --imouto examples/helloworld.cunny
```

The echo example reads and writes one byte:

```sh
printf Z | cunny examples/echo.cunny
```

## Run Bad Apple!!

`examples/bad-apple.cunny` is a 3,287-frame terminal animation paired with
`examples/bad-apple.m4a`. The program itself contains only Cunny tokens and
whitespace. Use a terminal at least 60 columns by 22 rows and run:

```sh
cunny examples/bad-apple.cunny
```

The runner recognizes this example, starts its audio track, and displays its
frame markers on a 15-frames-per-second clock. It uses elapsed time rather than
adding a fixed delay after each frame, so terminal rendering does not make the
animation gradually fall behind the music. macOS uses `afplay`; Linux uses the
first available player among `ffplay`, `mpv`, and VLC; Windows uses its built-in
PowerShell media player.

The demo clears the terminal and hides the cursor while it plays. If it is
interrupted, restore the cursor with `printf '\033[?25h'`.

This port is based on
[OpenSauce04's v1 Brainfuck demo](https://github.com/OpenSauce04/BadAppleBF/releases/tag/v1).
It uses fixed tape cells for common frame characters, keeping the complete
Cunny source below 60 MB instead of doing a much larger literal token swap.
The music is from the original shadow-art video: "Bad Apple!! feat. nomico" by
Masayoshi Minoshima (Alstroemeria Records), with vocals by nomico. The animation
was created by Anira. Those works remain the property of their respective
rights holders and are not covered by this repository's MIT license.

## Build a static site

A site uses this structure:

```text
my-site/
├── pages/
│   ├── index.cunny
│   └── about.cunny
├── layout/
│   ├── header.cunny
│   └── footer.cunny
└── assets/
    └── style.css
```

Only `pages/` is required. Every HTML page and layout must be programmed by
hand with `😭` and `💢`. Optional header and footer programs are added to every
page. CSS, JavaScript, images, and other files in `assets/` are copied unchanged.

Build it with:

```sh
cunny build my-site
```

The result is written to `my-site/dist/`. Every build replaces the previous
`dist/` directory so removed pages do not remain in the output.

Run the live development server with:

```sh
cunny brat correction my-site
```

It serves the site at `http://127.0.0.1:3000`, rebuilds when a Cunny program or
asset changes, and refreshes the browser automatically.

## Development

Build and test the project from source with:

```sh
cargo test
cargo clippy --all-targets -- -D warnings
```
