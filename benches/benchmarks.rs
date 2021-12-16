use criterion::{black_box, criterion_group, criterion_main, Criterion};

use rand::{SeedableRng, rngs::SmallRng};
use snork::agents::{maxn, FloodHeuristic, MobilityAgent, TreeHeuristic};
use snork::game::search::{self, Heuristic};
use snork::game::{FloodFill, Game, Outcome, Snake};
use snork::{env::*, logging};

#[derive(Debug, Clone, Default)]
struct TestH;

impl Heuristic for TestH {
    fn eval(&self, game: &Game) -> f64 {
        let mut flood_fill = FloodFill::new(game.grid.width, game.grid.height);
        if game.snake_is_alive(0) {
            flood_fill.flood_snakes(&game.grid, &game.snakes);
            flood_fill.count_space(true) as f64
        } else {
            0.0
        }
    }
}

fn game_step_circle(c: &mut Criterion) {
    logging();
    let mut game = Game::parse(
        r#"
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . 0 > v . . . .
            . . . . ^ . v . . . .
            . . . . ^ < < . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . ."#,
    )
    .unwrap();

    c.bench_function("game_step_circle", |b| {
        b.iter(|| {
            use Direction::*;
            game.step(black_box(&[Right]));
            game.step(black_box(&[Right]));
            game.step(black_box(&[Down]));
            game.step(black_box(&[Down]));
            game.step(black_box(&[Left]));
            game.step(black_box(&[Left]));
            game.step(black_box(&[Up]));
            game.step(black_box(&[Up]));
        })
    });
}

fn game_step_random(c: &mut Criterion) {
    logging();
    use rand::seq::IteratorRandom;

    let game = Game::parse(
        r#"
            . . . . . . . o . . .
            o . . . . . . . . o .
            . . o 3 . . . . . . .
            . . . . . . 0 o . . .
            . . o . . . . . . . .
            . . . . . o . . . o .
            . o . . o . . . . . .
            . . . . . . . 2 . o .
            . . . 1 . . . o . . .
            o . . o . . . . . . .
            . . . . . o . . o . ."#,
    )
    .unwrap();

    c.bench_function("game_step_random", |b| {
        b.iter(|| {
            let mut rng = SmallRng::from_entropy();
            // let mut turn = 0;
            let mut game = game.clone();
            loop {
                let mut moves = [Direction::Up; 4];
                for i in 0..4 {
                    moves[i as usize] = game
                        .valid_moves(i)
                        .choose(&mut rng)
                        .unwrap_or(Direction::Up);
                }
                game.step(&moves);

                if game.outcome() != Outcome::None {
                    // println!("{:?} after {} turns", game.outcome(), turn);
                    break;
                }
                // turn += 1;
            }
        })
    });
}

fn normal_max_n(c: &mut Criterion) {
    logging();
    let snakes = vec![
        Snake::new(vec![v2(0, 3), v2(1, 3), v2(2, 3), v2(3, 3)].into(), 100),
        Snake::new(vec![v2(3, 7), v2(3, 6), v2(3, 5)].into(), 100),
        Snake::new(vec![v2(10, 7), v2(10, 6), v2(10, 5)].into(), 100),
        Snake::new(vec![v2(10, 0), v2(9, 0), v2(8, 0)].into(), 100),
    ];
    let game = Game::new(0, 11, 11, snakes, &[], &[]);

    c.bench_function("normal_max_n", |b| {
        b.iter(|| search::max_n(black_box(&game), 2, &TestH))
    });
}

fn async_max_n(c: &mut Criterion) {
    logging();
    let snakes = vec![
        Snake::new(vec![v2(0, 3), v2(1, 3), v2(2, 3), v2(3, 3)].into(), 100),
        Snake::new(vec![v2(3, 7), v2(3, 6), v2(3, 5)].into(), 100),
        Snake::new(vec![v2(10, 7), v2(10, 6), v2(10, 5)].into(), 100),
        Snake::new(vec![v2(10, 0), v2(9, 0), v2(8, 0)].into(), 100),
    ];
    let game = Game::new(0, 11, 11, snakes, &[], &[]);

    c.bench_function("async_max_n", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| search::async_max_n(black_box(&game), 2, &TestH))
    });
}

fn expectimax(c: &mut Criterion) {
    logging();
    let snakes = vec![
        Snake::new(vec![v2(0, 3), v2(1, 3), v2(2, 3), v2(3, 3)].into(), 100),
        Snake::new(vec![v2(3, 7), v2(3, 6), v2(3, 5)].into(), 100),
        Snake::new(vec![v2(10, 7), v2(10, 6), v2(10, 5)].into(), 100),
        Snake::new(vec![v2(10, 0), v2(9, 0), v2(8, 0)].into(), 100),
    ];
    let game = Game::new(0, 11, 11, snakes, &[], &[]);

    c.bench_function("expectimax", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| search::expectimax(black_box(&game), 2, &TestH))
    });
}

fn normal_alphabeta(c: &mut Criterion) {
    logging();
    let snakes = vec![
        Snake::new(vec![v2(0, 3), v2(1, 3), v2(2, 3), v2(3, 3)].into(), 100),
        Snake::new(vec![v2(10, 7), v2(10, 6), v2(10, 5)].into(), 100),
    ];
    let game = Game::new(0, 11, 11, snakes, &[], &[]);

    c.bench_function("normal_alphabeta", |b| {
        b.iter(|| search::alphabeta(black_box(&game), 5, &TestH))
    });
}

fn async_alphabeta(c: &mut Criterion) {
    logging();
    let snakes = vec![
        Snake::new(vec![v2(0, 3), v2(1, 3), v2(2, 3), v2(3, 3)].into(), 100),
        Snake::new(vec![v2(10, 7), v2(10, 6), v2(10, 5)].into(), 100),
    ];
    let game = Game::new(0, 11, 11, snakes, &[], &[]);

    c.bench_function("async_alphabeta", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| search::async_alphabeta(black_box(&game), 5, &TestH))
    });
}

fn tree_heuristic(c: &mut Criterion) {
    logging();
    let request: GameRequest = serde_json::from_str(
            r#"{"game":{"id":"bcb8c2e8-4fb7-485b-9ade-9df947dd9623","ruleset":{"name":"standard","version":"v1.0.15"},"timeout":500},"turn":69,"board":{"height":11,"width":11,"food":[{"x":7,"y":9},{"x":1,"y":0}],"hazards":[],"snakes":[{"id":"gs_3MjqcwQJxYG7VrvjbbkRW9JB","name":"Nessegrev-flood","health":85,"body":[{"x":7,"y":10},{"x":8,"y":10},{"x":8,"y":9},{"x":9,"y":9},{"x":10,"y":9},{"x":10,"y":8},{"x":10,"y":7}],"shout":""},{"id":"gs_c9JrKQcQqHHPJFm43W47RKMd","name":"Rufio the Tenacious","health":80,"body":[{"x":5,"y":8},{"x":4,"y":8},{"x":4,"y":9},{"x":3,"y":9},{"x":2,"y":9},{"x":2,"y":8},{"x":2,"y":7}],"shout":""},{"id":"gs_ffjK7pqCwVXYGtwhWtk3vtJX","name":"marrrvin","health":89,"body":[{"x":8,"y":7},{"x":8,"y":8},{"x":7,"y":8},{"x":7,"y":7},{"x":7,"y":6},{"x":6,"y":6},{"x":5,"y":6},{"x":5,"y":5},{"x":6,"y":5}],"shout":""},{"id":"gs_Kr6BCBwbDpdGDpWbw9vMS6qV","name":"kostka","health":93,"body":[{"x":7,"y":2},{"x":7,"y":3},{"x":6,"y":3},{"x":5,"y":3},{"x":4,"y":3},{"x":3,"y":3}],"shout":""}]},"you":{"id":"gs_ffjK7pqCwVXYGtwhWtk3vtJX","name":"marrrvin","health":89,"body":[{"x":8,"y":7},{"x":8,"y":8},{"x":7,"y":8},{"x":7,"y":7},{"x":7,"y":6},{"x":6,"y":6},{"x":5,"y":6},{"x":5,"y":5},{"x":6,"y":5}],"shout":""}}"#
        ).unwrap();

    let game = Game::from_request(&request);
    let heuristic = TreeHeuristic::default();

    c.bench_function("tree_heuristic", |b| {
        b.iter(|| heuristic.eval(black_box(&game)))
    });
}

fn tree_search(c: &mut Criterion) {
    logging();
    let request: GameRequest = serde_json::from_str(
            r#"{"game":{"id":"bcb8c2e8-4fb7-485b-9ade-9df947dd9623","ruleset":{"name":"standard","version":"v1.0.15"},"timeout":500},"turn":69,"board":{"height":11,"width":11,"food":[{"x":7,"y":9},{"x":1,"y":0}],"hazards":[],"snakes":[{"id":"gs_3MjqcwQJxYG7VrvjbbkRW9JB","name":"Nessegrev-flood","health":85,"body":[{"x":7,"y":10},{"x":8,"y":10},{"x":8,"y":9},{"x":9,"y":9},{"x":10,"y":9},{"x":10,"y":8},{"x":10,"y":7}],"shout":""},{"id":"gs_c9JrKQcQqHHPJFm43W47RKMd","name":"Rufio the Tenacious","health":80,"body":[{"x":5,"y":8},{"x":4,"y":8},{"x":4,"y":9},{"x":3,"y":9},{"x":2,"y":9},{"x":2,"y":8},{"x":2,"y":7}],"shout":""},{"id":"gs_ffjK7pqCwVXYGtwhWtk3vtJX","name":"marrrvin","health":89,"body":[{"x":8,"y":7},{"x":8,"y":8},{"x":7,"y":8},{"x":7,"y":7},{"x":7,"y":6},{"x":6,"y":6},{"x":5,"y":6},{"x":5,"y":5},{"x":6,"y":5}],"shout":""},{"id":"gs_Kr6BCBwbDpdGDpWbw9vMS6qV","name":"kostka","health":93,"body":[{"x":7,"y":2},{"x":7,"y":3},{"x":6,"y":3},{"x":5,"y":3},{"x":4,"y":3},{"x":3,"y":3}],"shout":""}]},"you":{"id":"gs_ffjK7pqCwVXYGtwhWtk3vtJX","name":"marrrvin","health":89,"body":[{"x":8,"y":7},{"x":8,"y":8},{"x":7,"y":8},{"x":7,"y":7},{"x":7,"y":6},{"x":6,"y":6},{"x":5,"y":6},{"x":5,"y":5},{"x":6,"y":5}],"shout":""}}"#
        ).unwrap();

    let game = Game::from_request(&request);
    let heuristic = TreeHeuristic::default();

    c.bench_function("tree_search", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| maxn::tree_search(&heuristic, black_box(&game), 3))
    });
}

fn flood_heuristic(c: &mut Criterion) {
    logging();
    let request: GameRequest = serde_json::from_str(
            r#"{"game":{"id":"17d30fe5-a90f-45c0-bb81-1f8bd54781e1","ruleset":{"damagePerTurn":"14","foodSpawnChance":"15","minimumFood":"1","name":"royale","shrinkEveryNTurns":"25"},"timeout":500},"turn":64,"board":{"width":11,"height":11,"food":[{"x":10,"y":7}],"hazards":[{"x":0,"y":0},{"x":0,"y":1},{"x":0,"y":2},{"x":0,"y":3},{"x":0,"y":4},{"x":0,"y":5},{"x":0,"y":6},{"x":0,"y":7},{"x":0,"y":8},{"x":0,"y":9},{"x":0,"y":10},{"x":1,"y":0},{"x":2,"y":0},{"x":3,"y":0},{"x":4,"y":0},{"x":5,"y":0},{"x":6,"y":0},{"x":7,"y":0},{"x":8,"y":0},{"x":9,"y":0},{"x":10,"y":0}],"snakes":[{"id":"gs_c6BKHbpSr47cqd76mmWTj7dB","name":"unsigned long long","body":[{"x":5,"y":7},{"x":5,"y":6},{"x":5,"y":5},{"x":4,"y":5},{"x":3,"y":5},{"x":2,"y":5}],"health":93,"latency":471,"head":{"x":5,"y":7},"length":6,"shout":"","squad":""},{"id":"gs_BWkm6pVmC6kTmYShrGTrRHfW","name":"marrrvin","body":[{"x":4,"y":4},{"x":3,"y":4},{"x":3,"y":3},{"x":2,"y":3},{"x":1,"y":3}],"health":56,"latency":25,"head":{"x":4,"y":4},"length":5,"shout":"","squad":""},{"id":"gs_Q6FcKJtmmFCC6YtvTM4RVqXM","name":"marrrvin","body":[{"x":7,"y":7},{"x":7,"y":6},{"x":7,"y":5},{"x":8,"y":5},{"x":9,"y":5},{"x":9,"y":4}],"health":86,"latency":26,"head":{"x":7,"y":7},"length":6,"shout":"","squad":""},{"id":"gs_kqMqF4c7rCppw9mSm7vT6Xvb","name":"marrrvin","body":[{"x":9,"y":3},{"x":9,"y":2},{"x":8,"y":2},{"x":7,"y":2},{"x":7,"y":1}],"health":72,"latency":29,"head":{"x":9,"y":3},"length":5,"shout":"","squad":""}]},"you":{"id":"gs_c6BKHbpSr47cqd76mmWTj7dB","name":"unsigned long long","body":[{"x":5,"y":7},{"x":5,"y":6},{"x":5,"y":5},{"x":4,"y":5},{"x":3,"y":5},{"x":2,"y":5}],"health":93,"latency":471,"head":{"x":5,"y":7},"length":6,"shout":"","squad":""}}"#
        ).unwrap();

    let game = Game::from_request(&request);
    let heuristic = FloodHeuristic::default();

    c.bench_function("flood_heuristic", |b| {
        b.iter(|| heuristic.eval(black_box(&game)))
    });
}

fn flood_search(c: &mut Criterion) {
    logging();
    let request: GameRequest = serde_json::from_str(
            r#"{"game":{"id":"17d30fe5-a90f-45c0-bb81-1f8bd54781e1","ruleset":{"damagePerTurn":"14","foodSpawnChance":"15","minimumFood":"1","name":"royale","shrinkEveryNTurns":"25"},"timeout":500},"turn":64,"board":{"width":11,"height":11,"food":[{"x":10,"y":7}],"hazards":[{"x":0,"y":0},{"x":0,"y":1},{"x":0,"y":2},{"x":0,"y":3},{"x":0,"y":4},{"x":0,"y":5},{"x":0,"y":6},{"x":0,"y":7},{"x":0,"y":8},{"x":0,"y":9},{"x":0,"y":10},{"x":1,"y":0},{"x":2,"y":0},{"x":3,"y":0},{"x":4,"y":0},{"x":5,"y":0},{"x":6,"y":0},{"x":7,"y":0},{"x":8,"y":0},{"x":9,"y":0},{"x":10,"y":0}],"snakes":[{"id":"gs_c6BKHbpSr47cqd76mmWTj7dB","name":"unsigned long long","body":[{"x":5,"y":7},{"x":5,"y":6},{"x":5,"y":5},{"x":4,"y":5},{"x":3,"y":5},{"x":2,"y":5}],"health":93,"latency":471,"head":{"x":5,"y":7},"length":6,"shout":"","squad":""},{"id":"gs_BWkm6pVmC6kTmYShrGTrRHfW","name":"marrrvin","body":[{"x":4,"y":4},{"x":3,"y":4},{"x":3,"y":3},{"x":2,"y":3},{"x":1,"y":3}],"health":56,"latency":25,"head":{"x":4,"y":4},"length":5,"shout":"","squad":""},{"id":"gs_Q6FcKJtmmFCC6YtvTM4RVqXM","name":"marrrvin","body":[{"x":7,"y":7},{"x":7,"y":6},{"x":7,"y":5},{"x":8,"y":5},{"x":9,"y":5},{"x":9,"y":4}],"health":86,"latency":26,"head":{"x":7,"y":7},"length":6,"shout":"","squad":""},{"id":"gs_kqMqF4c7rCppw9mSm7vT6Xvb","name":"marrrvin","body":[{"x":9,"y":3},{"x":9,"y":2},{"x":8,"y":2},{"x":7,"y":2},{"x":7,"y":1}],"health":72,"latency":29,"head":{"x":9,"y":3},"length":5,"shout":"","squad":""}]},"you":{"id":"gs_c6BKHbpSr47cqd76mmWTj7dB","name":"unsigned long long","body":[{"x":5,"y":7},{"x":5,"y":6},{"x":5,"y":5},{"x":4,"y":5},{"x":3,"y":5},{"x":2,"y":5}],"health":93,"latency":471,"head":{"x":5,"y":7},"length":6,"shout":"","squad":""}}"#
        ).unwrap();

    let game = Game::from_request(&request);
    let heuristic = FloodHeuristic::default();

    c.bench_function("flood_search", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| maxn::tree_search(&heuristic, black_box(&game), 3))
    });
}

fn flood_2_search(c: &mut Criterion) {
    logging();
    let request: GameRequest = serde_json::from_str(
            r#"{"game":{"id":"17d30fe5-a90f-45c0-bb81-1f8bd54781e1","ruleset":{"damagePerTurn":"14","foodSpawnChance":"15","minimumFood":"1","name":"royale","shrinkEveryNTurns":"25"},"timeout":500},"turn":64,"board":{"width":11,"height":11,"food":[{"x":10,"y":7}],"hazards":[{"x":0,"y":0},{"x":0,"y":1},{"x":0,"y":2},{"x":0,"y":3},{"x":0,"y":4},{"x":0,"y":5},{"x":0,"y":6},{"x":0,"y":7},{"x":0,"y":8},{"x":0,"y":9},{"x":0,"y":10},{"x":1,"y":0},{"x":2,"y":0},{"x":3,"y":0},{"x":4,"y":0},{"x":5,"y":0},{"x":6,"y":0},{"x":7,"y":0},{"x":8,"y":0},{"x":9,"y":0},{"x":10,"y":0}],"snakes":[{"id":"gs_c6BKHbpSr47cqd76mmWTj7dB","name":"unsigned long long","body":[{"x":5,"y":7},{"x":5,"y":6},{"x":5,"y":5},{"x":4,"y":5},{"x":3,"y":5},{"x":2,"y":5}],"health":93,"latency":471,"head":{"x":5,"y":7},"length":6,"shout":"","squad":""},{"id":"gs_BWkm6pVmC6kTmYShrGTrRHfW","name":"marrrvin","body":[{"x":4,"y":4},{"x":3,"y":4},{"x":3,"y":3},{"x":2,"y":3},{"x":1,"y":3}],"health":56,"latency":25,"head":{"x":4,"y":4},"length":5,"shout":"","squad":""}]},"you":{"id":"gs_c6BKHbpSr47cqd76mmWTj7dB","name":"unsigned long long","body":[{"x":5,"y":7},{"x":5,"y":6},{"x":5,"y":5},{"x":4,"y":5},{"x":3,"y":5},{"x":2,"y":5}],"health":93,"latency":471,"head":{"x":5,"y":7},"length":6,"shout":"","squad":""}}"#
        ).unwrap();

    let game = Game::from_request(&request);
    let heuristic = FloodHeuristic::default();

    c.bench_function("flood_2_search", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| maxn::tree_search(&heuristic, black_box(&game), 6))
    });
}

fn mobility_agent(c: &mut Criterion) {
    logging();
    let request: GameRequest = serde_json::from_str(
        r#"{"game":{"id":"bcb8c2e8-4fb7-485b-9ade-9df947dd9623","ruleset":{"name":"standard","version":"v1.0.15"},"timeout":500},"turn":69,"board":{"height":11,"width":11,"food":[{"x":7,"y":9},{"x":1,"y":0}],"hazards":[],"snakes":[{"id":"gs_3MjqcwQJxYG7VrvjbbkRW9JB","name":"Nessegrev-flood","health":85,"body":[{"x":7,"y":10},{"x":8,"y":10},{"x":8,"y":9},{"x":9,"y":9},{"x":10,"y":9},{"x":10,"y":8},{"x":10,"y":7}],"shout":""},{"id":"gs_c9JrKQcQqHHPJFm43W47RKMd","name":"Rufio the Tenacious","health":80,"body":[{"x":5,"y":8},{"x":4,"y":8},{"x":4,"y":9},{"x":3,"y":9},{"x":2,"y":9},{"x":2,"y":8},{"x":2,"y":7}],"shout":""},{"id":"gs_ffjK7pqCwVXYGtwhWtk3vtJX","name":"marrrvin","health":89,"body":[{"x":8,"y":7},{"x":8,"y":8},{"x":7,"y":8},{"x":7,"y":7},{"x":7,"y":6},{"x":6,"y":6},{"x":5,"y":6},{"x":5,"y":5},{"x":6,"y":5}],"shout":""},{"id":"gs_Kr6BCBwbDpdGDpWbw9vMS6qV","name":"kostka","health":93,"body":[{"x":7,"y":2},{"x":7,"y":3},{"x":6,"y":3},{"x":5,"y":3},{"x":4,"y":3},{"x":3,"y":3}],"shout":""}]},"you":{"id":"gs_ffjK7pqCwVXYGtwhWtk3vtJX","name":"marrrvin","health":89,"body":[{"x":8,"y":7},{"x":8,"y":8},{"x":7,"y":8},{"x":7,"y":7},{"x":7,"y":6},{"x":6,"y":6},{"x":5,"y":6},{"x":5,"y":5},{"x":6,"y":5}],"shout":""}}"#
    ).unwrap();

    let agent = MobilityAgent::default();
    let game = Game::from_request(&request);

    c.bench_function("mobility", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| agent.step(black_box(&game)))
    });
}

criterion_group!(
    benches,
    game_step_circle,
    game_step_random,
    async_max_n,
    normal_max_n,
    expectimax,
    async_alphabeta,
    normal_alphabeta,
    tree_heuristic,
    tree_search,
    flood_heuristic,
    flood_search,
    flood_2_search,
    mobility_agent,
);
criterion_main!(benches);
