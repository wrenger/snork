use criterion::{black_box, criterion_group, criterion_main, Criterion};

use snork::agents::{Agent, MobilityAgent, MobilityConfig, TreeAgent, TreeConfig};
use snork::env::{Direction, GameRequest, Vec2D};
use snork::game::{self, FloodFill, Game, Outcome, Snake};

fn game_step_circle(c: &mut Criterion) {
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
            let mut rng = rand::thread_rng();
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

fn tree_heuristic(c: &mut Criterion) {
    let game_req: GameRequest = serde_json::from_str(
            r#"{"game":{"id":"bcb8c2e8-4fb7-485b-9ade-9df947dd9623","ruleset":{"name":"standard","version":"v1.0.15"},"timeout":500},"turn":69,"board":{"height":11,"width":11,"food":[{"x":7,"y":9},{"x":1,"y":0}],"hazards":[],"snakes":[{"id":"gs_3MjqcwQJxYG7VrvjbbkRW9JB","name":"Nessegrev-flood","health":85,"body":[{"x":7,"y":10},{"x":8,"y":10},{"x":8,"y":9},{"x":9,"y":9},{"x":10,"y":9},{"x":10,"y":8},{"x":10,"y":7}],"shout":""},{"id":"gs_c9JrKQcQqHHPJFm43W47RKMd","name":"Rufio the Tenacious","health":80,"body":[{"x":5,"y":8},{"x":4,"y":8},{"x":4,"y":9},{"x":3,"y":9},{"x":2,"y":9},{"x":2,"y":8},{"x":2,"y":7}],"shout":""},{"id":"gs_ffjK7pqCwVXYGtwhWtk3vtJX","name":"marrrvin","health":89,"body":[{"x":8,"y":7},{"x":8,"y":8},{"x":7,"y":8},{"x":7,"y":7},{"x":7,"y":6},{"x":6,"y":6},{"x":5,"y":6},{"x":5,"y":5},{"x":6,"y":5}],"shout":""},{"id":"gs_Kr6BCBwbDpdGDpWbw9vMS6qV","name":"kostka","health":93,"body":[{"x":7,"y":2},{"x":7,"y":3},{"x":6,"y":3},{"x":5,"y":3},{"x":4,"y":3},{"x":3,"y":3}],"shout":""}]},"you":{"id":"gs_ffjK7pqCwVXYGtwhWtk3vtJX","name":"marrrvin","health":89,"body":[{"x":8,"y":7},{"x":8,"y":8},{"x":7,"y":8},{"x":7,"y":7},{"x":7,"y":6},{"x":6,"y":6},{"x":5,"y":6},{"x":5,"y":5},{"x":6,"y":5}],"shout":""}}"#
        ).unwrap();

    let food = &game_req.board.food[..];
    let mut flood_fill = FloodFill::new(game_req.board.width, game_req.board.height);
    let mut game = Game::new(game_req.board.width, game_req.board.height);
    game.reset_from_request(&game_req);
    let turn = game_req.turn;

    c.bench_function("tree_heuristic", |b| {
        b.iter(|| {
            TreeAgent::heuristic(
                black_box(food),
                black_box(&mut flood_fill),
                black_box(&game),
                turn,
                &TreeConfig::default(),
            )
        })
    });
}

fn max_n(c: &mut Criterion) {
    let snakes = vec![
        Snake::new(
            0,
            vec![
                Vec2D::new(0, 3),
                Vec2D::new(1, 3),
                Vec2D::new(2, 3),
                Vec2D::new(3, 3),
            ]
            .into(),
            100,
        ),
        Snake::new(
            1,
            vec![Vec2D::new(3, 7), Vec2D::new(3, 6), Vec2D::new(3, 5)].into(),
            100,
        ),
        Snake::new(
            2,
            vec![Vec2D::new(10, 7), Vec2D::new(10, 6), Vec2D::new(10, 5)].into(),
            100,
        ),
        Snake::new(
            3,
            vec![Vec2D::new(10, 0), Vec2D::new(9, 0), Vec2D::new(8, 0)].into(),
            100,
        ),
    ];

    let mut game = Game::new(11, 11);
    game.reset(snakes, &[], &[]);
    let mut flood_fill = FloodFill::new(game.grid.width, game.grid.height);

    c.bench_function("max_n", |b| {
        b.iter(|| {
            game::max_n(black_box(&game), black_box(2), |game| {
                if game.snake_is_alive(0) {
                    flood_fill.flood_snakes(&game.grid, &game.snakes);
                    flood_fill.count_space(true) as f64
                } else {
                    0.0
                }
            });
        })
    });
}

fn tree_search(c: &mut Criterion) {
    let game_req: GameRequest = serde_json::from_str(
            r#"{"game":{"id":"bcb8c2e8-4fb7-485b-9ade-9df947dd9623","ruleset":{"name":"standard","version":"v1.0.15"},"timeout":500},"turn":69,"board":{"height":11,"width":11,"food":[{"x":7,"y":9},{"x":1,"y":0}],"hazards":[],"snakes":[{"id":"gs_3MjqcwQJxYG7VrvjbbkRW9JB","name":"Nessegrev-flood","health":85,"body":[{"x":7,"y":10},{"x":8,"y":10},{"x":8,"y":9},{"x":9,"y":9},{"x":10,"y":9},{"x":10,"y":8},{"x":10,"y":7}],"shout":""},{"id":"gs_c9JrKQcQqHHPJFm43W47RKMd","name":"Rufio the Tenacious","health":80,"body":[{"x":5,"y":8},{"x":4,"y":8},{"x":4,"y":9},{"x":3,"y":9},{"x":2,"y":9},{"x":2,"y":8},{"x":2,"y":7}],"shout":""},{"id":"gs_ffjK7pqCwVXYGtwhWtk3vtJX","name":"marrrvin","health":89,"body":[{"x":8,"y":7},{"x":8,"y":8},{"x":7,"y":8},{"x":7,"y":7},{"x":7,"y":6},{"x":6,"y":6},{"x":5,"y":6},{"x":5,"y":5},{"x":6,"y":5}],"shout":""},{"id":"gs_Kr6BCBwbDpdGDpWbw9vMS6qV","name":"kostka","health":93,"body":[{"x":7,"y":2},{"x":7,"y":3},{"x":6,"y":3},{"x":5,"y":3},{"x":4,"y":3},{"x":3,"y":3}],"shout":""}]},"you":{"id":"gs_ffjK7pqCwVXYGtwhWtk3vtJX","name":"marrrvin","health":89,"body":[{"x":8,"y":7},{"x":8,"y":8},{"x":7,"y":8},{"x":7,"y":7},{"x":7,"y":6},{"x":6,"y":6},{"x":5,"y":6},{"x":5,"y":5},{"x":6,"y":5}],"shout":""}}"#
        ).unwrap();

    let food = &game_req.board.food[..];
    let mut game = Game::new(game_req.board.width, game_req.board.height);
    game.reset_from_request(&game_req);
    let turn = game_req.turn;

    c.bench_function("tree_search", |b| {
        b.iter(|| {
            TreeAgent::next_move(
                black_box(&game),
                black_box(turn),
                black_box(food),
                black_box(3),
                &TreeConfig::default(),
            )
        })
    });
}

fn mobility_agent(c: &mut Criterion) {
    let game_req: GameRequest = serde_json::from_str(
        r#"{"game":{"id":"bcb8c2e8-4fb7-485b-9ade-9df947dd9623","ruleset":{"name":"standard","version":"v1.0.15"},"timeout":500},"turn":69,"board":{"height":11,"width":11,"food":[{"x":7,"y":9},{"x":1,"y":0}],"hazards":[],"snakes":[{"id":"gs_3MjqcwQJxYG7VrvjbbkRW9JB","name":"Nessegrev-flood","health":85,"body":[{"x":7,"y":10},{"x":8,"y":10},{"x":8,"y":9},{"x":9,"y":9},{"x":10,"y":9},{"x":10,"y":8},{"x":10,"y":7}],"shout":""},{"id":"gs_c9JrKQcQqHHPJFm43W47RKMd","name":"Rufio the Tenacious","health":80,"body":[{"x":5,"y":8},{"x":4,"y":8},{"x":4,"y":9},{"x":3,"y":9},{"x":2,"y":9},{"x":2,"y":8},{"x":2,"y":7}],"shout":""},{"id":"gs_ffjK7pqCwVXYGtwhWtk3vtJX","name":"marrrvin","health":89,"body":[{"x":8,"y":7},{"x":8,"y":8},{"x":7,"y":8},{"x":7,"y":7},{"x":7,"y":6},{"x":6,"y":6},{"x":5,"y":6},{"x":5,"y":5},{"x":6,"y":5}],"shout":""},{"id":"gs_Kr6BCBwbDpdGDpWbw9vMS6qV","name":"kostka","health":93,"body":[{"x":7,"y":2},{"x":7,"y":3},{"x":6,"y":3},{"x":5,"y":3},{"x":4,"y":3},{"x":3,"y":3}],"shout":""}]},"you":{"id":"gs_ffjK7pqCwVXYGtwhWtk3vtJX","name":"marrrvin","health":89,"body":[{"x":8,"y":7},{"x":8,"y":8},{"x":7,"y":8},{"x":7,"y":7},{"x":7,"y":6},{"x":6,"y":6},{"x":5,"y":6},{"x":5,"y":5},{"x":6,"y":5}],"shout":""}}"#
    ).unwrap();

    let mut agent = MobilityAgent::new(&game_req, &MobilityConfig::default());

    c.bench_function("mobility", |b| {
        b.iter(|| black_box(&mut agent).step(black_box(&game_req), 200))
    });
}

criterion_group!(
    benches,
    game_step_circle,
    game_step_random,
    max_n,
    tree_heuristic,
    tree_search,
    mobility_agent,
);
criterion_main!(benches);
