# Haikunator Generator via Loco(Rust)

This is a web service that generates Heroku-like memorable random strings.

![Screenshot](docs/screenshot_top.png)

# Usage

Web:

[https://haikunator-generator.shuttle.app/](https://haikunator-generator.shuttle.app/)

API:

```console
// JSON
$ curl https://haikunator-generator.shuttle.app/api/gen
{"name":"falling-disk-1736"}

// Plain text
$ curl https://haikunator-generator.shuttle.app/api/gen.txt
broken-wildflower-1928
```

# Development

## In DevContainer

```console
$ rustc --version
rustc 1.98.1 (48a229cea 2026-09-01)
$ cargo --version
cargo 1.98.1 (797e8a9bc 2026-08-05)
$ cargo loco --version
loco-rs 1.2.0
$ npm --version
11.19.0
$ pnpm --version
12.8.1
```

### Start frontend

Run the following commands:

```console
$ cd frontend
$ pnpm dev
```

### Start backend

On another terminal window, run:

```console
$ cargo loco start
```

Then open http://localhost:5150/ in your browser.

## Linting

To format and auto-fix your code, you can use the following command:

```console
$ cargo fmt --all
```

# Dependencies

| | |
| -- | -- |
| Rust | [https://www.rust-lang.org/](https://www.rust-lang.org/) |
| Loco | [https://loco.rs/](https://loco.rs/) |
| Shuttle | [https://www.shuttle.dev/](https://www.shuttle.dev/) |
| rust-haikunator | [https://github.com/nishanths/rust-haikunator](https://github.com/nishanths/rust-haikunator) |

