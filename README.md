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
rustc 1.86.0 (05f9846f8 2025-03-31)
$ cargo --version
cargo 1.86.0 (adf9b6ad1 2025-02-28)
$ loco --version
loco 0.15.0
$ cargo shuttle --version
cargo-shuttle 0.53.0
$ npm --version
10.9.2
$ pnpm --version
10.9.0
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

# Deployment

Deploy to [Shuttle](https://console.shuttle.dev/):

```console
$ pushd frontend
$ pnpm build
$ popd
$ cargo shuttle deploy
```

# Dependencies

| | |
| -- | -- |
| Rust | [https://www.rust-lang.org/](https://www.rust-lang.org/) |
| Loco | [https://loco.rs/](https://loco.rs/) |
| Shuttle | [https://www.shuttle.dev/](https://www.shuttle.dev/) |
| rust-haikunator | [https://github.com/nishanths/rust-haikunator](https://github.com/nishanths/rust-haikunator) |

