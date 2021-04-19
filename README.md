# Rusty Snakes

Fast [battlesnake](https://play.battlesnake.com) agents written in rust.

This project has been developed as part of an AI games course at the Leibniz University Hannover.
During this phase our tree agent ("ich heisse marvin") managed to score the second place in the global and dual arenas.
We even surpassed the best snake ([Kreuzotter](https://github.com/m-schier/battlesnake-2019)) of our university from the last year.

## Content

This repository contains a webserver that runs with the battlesnake api version 1
and abstractions for the used data types ([src/env.rs](src/env.rs)).

We developed multiple different agents ([src/agents](src/agents)),
from a simple random agent, a very fast area control agent,
to a minimax tree search agent, that combines multiple heuristics to find the best possible moves.
These heuristics are configurable and the impact of specific variables and
their influence over time can be specified on startup.
This allowed us to perform parameter optimization (bayesian optimization) to further improve the heuristic.

We also developed a fast simulator to execute moves and analyze their outcomes.

## Usage

### Running the Server

Fist the rust toolchain has to be installed (https://www.rust-lang.org/learn/get-started).

Starting the server:

```bash
cargo run --release -- [-h] [-p <port>] [--config <json>]
```

> There are additional options for `--runtime` and visual representation of the snake (`--head`, `--tail`, `--color`).
> Run `cargo run --release -- -h` to see all the commandline options.

`config` defines the agent to be used (`Tree`, `Mobility`, `Random`) and configures the agents heuristic.
The default config for the `Tree` agent is for example:

```json
{
  "Tree": {
    "mobility": 0.7,
    "mobility_decay": 0.0,
    "health": 0.012,
    "health_decay": 0.0,
    "len_advantage": 1.0,
    "len_advantage_decay": 0.0,
    "food_ownership": 0.65,
    "food_ownership_decay": 0.0,
    "centrality": 0.1,
    "centrality_decay": 0.0
  }
}
```

### Simulating Configs

This tool was developed to simulate different configurations.
The provided Configurations play a number of games against each other and the
number of wins of the first configuration is returned.

```bash
cargo run --release --bin simulate -- '{"Tree":{}}' '{"Tree":{"centrality":0}}' '{"Mobility":{}}' '{"Random":null}' -j 8 --game-count 8
```

> Play `--game-count` games on `-j` threads.

The last line of the standard output contains the number of wins of the first
snake and the total amount of games played:

```
Result: 3/8
```

### Testing moves

There is also an additional `move` program that outputs the chosen move for a given game input.

```bash
cargo run --release --bin move -- <json> [--config <json>] [--runtime]
```

### Running tests & benchmarks

There are multiple tests for the different modules that can be run as shown below.
For more information on unit-testing in rust see https://doc.rust-lang.org/book/ch11-01-writing-tests.html.

```bash
cargo test -p snork_core -- [--nocapture] [testname]
```

There are a number of benchmark tests that ignored when running normal unit tests, because they have a longer runtime.
These test are expected to be executed with the release config that include a number of compiler and linker optimizations.

```bash
cargo test -p snork_core --release -- --ignored --nocapture [testname]
```
