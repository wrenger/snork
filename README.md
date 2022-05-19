# Rusty Snakes

Fast [battlesnake](https://play.battlesnake.com) agents written in rust.
This project has been developed as part of an AI games course at the Leibniz University Hannover.

Our `Tree` agent ("ich heisse marvin") managed to reach the second and first place in the global, dual and royale arenas.
In the Spring League 2021 we even surpassed the best snake ([Kreuzotter](https://github.com/m-schier/battlesnake-2019)) of our university from the last year.

At the end of 2021, the new `Flood` agent reached the second place in the Elite Division of the Winter Classic Invitational 2021.


## Structure of this Repository

This repository contains a web server that runs with the battlesnake API version 1
and abstractions for the used data types ([src/env.rs](src/env.rs)).

We developed multiple different agents ([src/agents](src/agents)),
from a simple random agent, a very fast area control agent,
to a minimax tree search agent, that combines multiple heuristics to find the best possible moves.
These heuristics are configurable and the impact of specific variables and
their influence over time can be specified on startup.
This allowed us to perform parameter optimization (Bayesian optimization) to further improve the heuristic.

We also developed a fast simulator to execute moves and analyze their outcomes.
It was used to evaluate the heuristics and tune their parameters.

The [hpo](hpo) directory contains the code for automatically optimizing the agent's hyperparameters.
It utilizes the simulator, mentioned below, to simulate the generated configs and find the best performing parameters.
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

`config` defines the agent to be used (`Flood`, `Tree`, `Mobility`, `Random`) and configures the agents heuristic.
The default config for the `Flood` agent is for example:

```json
{
  "Flood": {
    "space": 2.0,
    "health": 0.5,
    "size_adv": 4.0,
    "food_distance": 0.5
  }
}
```

> If a config parameter (like `health`) is excluded the default value is used.

### Simulating Configs

This tool was developed to simulate different configurations.
These configurations specify the agent and its hyperparameters.
If no parameters are provided, the default values for the agent are used.
The number of simulated games can be specified with `--game-count`.
Use `-h` for more information about other arguments to specify the board size and game rules.

The example below simulates the `Flood` and `Tree` agents for 10 games:

```bash
cargo run --release --bin simulate -- '{"Flood":{"space":8.0}}' '{"Tree":{"centrality":0}}' --game-count 10
```

The last line of the standard output contains the number of wins of the first
snake and the total number of games played:

```
Result: 3/10
```

### Testing moves

There is also an additional `move` program that outputs the chosen move for a given game state and agent configuration.
The game input can be downloaded from the [battlesnake](https://play.battlesnake.com) with this [Firefox extension](https://addons.mozilla.org/firefox/addon/battlesnake-downloader/).

```bash
cargo run --release --bin move -- [--config <json>] [--runtime] <json>
```

### Running tests & benchmarks

There are multiple tests for the different modules that can be run, as shown below.
For more information on unit-testing in rust see https://doc.rust-lang.org/book/ch11-01-writing-tests.html.

```bash
cargo test -- [--nocapture] [testname]
```

There are several benchmarks that are ignored when running normal unit tests because they have a longer runtime.
They are executed with the release config with compiler and linker optimizations.

```bash
cargo bench -- [testname]
```
