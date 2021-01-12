# Rusty Snake

High performant battlesnake agents written in rust.

## Running the Server

Fist the rust toolchain has to be installed (https://www.rust-lang.org/learn/get-started).

Starting the server:

```bash
cargo run --release -- [-h] [-p <port>] [--config <json>]
```

> There are additional options for `--config`, `--runtime` and visual representation of the snake (`--head`, `--tail`, `--color`).

## Testing moves

There is also an additional `move` program that outputs the chosen move for a given game input.

```bash
cargo run --release --bin move -- data <json>
# or
cargo run --release --bin move -- file <jsonFile>
```

## Running unit tests

```bash
cargo test -- [--nocapture] [testname]
```

## Running benchmarks

There are a number of benchmark tests that ignored when running normal unit tests, because they have a longer runtime.

```bash
cargo test --release -- --ignored --nocapture [testname]
```
