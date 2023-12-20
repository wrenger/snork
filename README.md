# Rusty Snakes

Fast [Battlesnake](https://play.battlesnake.com) agents written in rust.
This project has been developed as part of an AI games course at the Leibniz University Hannover.

Our `Tree` agent ("ich heisse marvin") reached second and first place in the global, dual, and royale arenas.
In the Spring League 2021, we surpassed last year's best snake ([Kreuzotter](https://github.com/m-schier/battlesnake-2019)) from our university.

At the end of 2021, the new `Flood` agent reached second place in the Elite Division of the Winter Classic Invitational 2021.


## Structure of this Repository

This repository contains a web server compatible with the Battlesnake API version 1 and abstractions for the data types ([src/env.rs](src/env.rs)).

We developed multiple different agents ([src/agents](src/agents)), from a simple random agent, a very fast area control agent, to a minimax tree search agent combining multiple heuristics to find the best possible moves.
These heuristics are configurable, and the impact of specific variables and their influence over time can be specified on startup.
This allows us to perform parameter optimization (Bayesian optimization) to improve the heuristic further.

We also developed a fast simulator to execute moves and analyze their outcomes.
It is used to evaluate the heuristics and tune their parameters.

The [hpo](hpo) directory contains the code for automatically optimizing the agent's hyperparameters.
It utilizes the simulator to simulate the generated configs and find the best-performing parameters.
The current default configurations of the `Flood`, `Tree`, and `Mobility` agents are the results of several optimization campaigns.


## Usage

### Running the Server

First, the rust toolchain has to be installed (https://www.rust-lang.org/learn/get-started).

Starting the server:

```bash
cargo run --release -- [-h] [--host <ip:port>] [--config <json>]
```

> There are additional options for `--runtime` and visual representation of the snake (`--head`, `--tail`, `--color`).
> Run `cargo run --release -- -h` to see all the commandline options.

`config` defines the agent to be used (`Flood`, `Tree`, `Mobility`, `Random`) and configures the agent's heuristic.
The default config for the `Flood` agent is, for example:

```json
{
  "Flood": {
    "health": 0.00044,
    "food_distance": 0.173,
    "space": 0.0026,
    "space_adv": 0.108,
    "size_adv": 7.049,
    "size_adv_decay": 0.041,
  }
}
```

> If a config parameter (like `health`) is excluded the default value is used.

### Simulating Configs

This tool can be used to simulate different configurations.
These configurations specify the agent and its hyperparameters.
If no parameters are provided, the default values for the agent are used.
The number of simulated games can be specified with `--game-count`.
Use `-h` for more information about other arguments to define the board size and game rules.

The example below simulates the `Flood` and `Tree` agents for 10 games:

```bash
cargo run --release --bin simulate -- '{"Flood":{"space":8.0}}' '{"Tree":{"centrality":0}}' --game-count 10
```

The last line of the standard output contains the number of wins of the first snake and the total number of games played:

```
Result: 3/10
```

### Testing moves

The `move` program outputs the chosen move for a given game state and agent configuration.
This can be useful for debugging situational bugs.
The game input can be downloaded from the [battlesnake](https://play.battlesnake.com) with this [Firefox extension](https://addons.mozilla.org/firefox/addon/battlesnake-downloader/).

```bash
cargo run --release --bin move -- [--config <json>] [--runtime] <json>
```

### Running tests & benchmarks

There are multiple tests for the different modules that can be run, as shown below.
For more information on unit testing in Rust, see https://doc.rust-lang.org/book/ch11-01-writing-tests.html.

```bash
cargo test -- [--nocapture] [testname]
```

Besides the functional tests, there are several performance benchmarks.
They are executed with the release config with compiler and linker optimizations.
The criterion benchmark runner tracks the execution times from previous runs and reports any improvements or degradations.

```bash
cargo bench -- [testname]
```
