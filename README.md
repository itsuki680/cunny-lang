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

Run a program with:

```sh
cargo run -- examples/a.cunny
```

Run it with the Imouto debugger to trace every instruction and tape state:

```sh
cargo run -- --imouto examples/a.cunny
```

The echo example reads and writes one byte:

```sh
printf Z | cargo run -- examples/echo.cunny
```

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
cargo run -- build my-site
```

The result is written to `my-site/dist/`. Every build replaces the previous
`dist/` directory so removed pages do not remain in the output.

Run the live development server with:

```sh
cargo run -- brat correction my-site
```

It serves the site at `http://127.0.0.1:3000`, rebuilds when a Cunny program or
asset changes, and refreshes the browser automatically.

Run the test suite with:

```sh
cargo test
```
