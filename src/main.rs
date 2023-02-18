mod def;
mod util;

use def::*;
use std::{cell, io, io::Write};
use util::*;

struct State {
    is_broken: Vec2d,
    damage: Vec2d,
}

impl State {
    fn new(n: usize) -> State {
        State {
            is_broken: Vec2d::new(n, n),
            damage: Vec2d::new(n, n),
        }
    }
}

#[derive(Debug)]
struct Input {
    n: usize,
    w: usize,
    k: usize,
    c: i64,
    source: Vec<Pos>,
    house: Vec<Pos>,
}

fn read_input(stdin: &io::Stdin) -> Input {
    let mut user_input = String::new();
    stdin.read_line(&mut user_input).unwrap();
    let mut v = vec![];
    for e in user_input.trim().split(" ") {
        v.push(e.to_string());
    }
    let (n, w, k, c): (usize, usize, usize, i64) = (
        v[0].parse().unwrap(),
        v[1].parse().unwrap(),
        v[2].parse().unwrap(),
        v[3].parse().unwrap(),
    );
    let mut source = vec![];
    for _ in 0..w {
        let mut user_input = String::new();
        stdin.read_line(&mut user_input).unwrap();
        let mut v = vec![];
        for e in user_input.trim().split(" ") {
            v.push(e.to_string());
        }
        source.push(Pos {
            y: v[0].parse().unwrap(),
            x: v[1].parse().unwrap(),
        });
    }
    let mut house = vec![];
    for _ in 0..k {
        let mut user_input = String::new();
        stdin.read_line(&mut user_input).unwrap();
        let mut v = vec![];
        for e in user_input.trim().split(" ") {
            v.push(e.to_string());
        }
        house.push(Pos {
            y: v[0].parse().unwrap(),
            x: v[1].parse().unwrap(),
        });
    }
    Input {
        n,
        w,
        k,
        c,
        source,
        house,
    }
}

fn read_result(stdin: &io::Stdin) -> i64 {
    let mut user_input = String::new();
    stdin.read_line(&mut user_input).unwrap();
    user_input.trim().parse().unwrap()
}

fn main() {
    time::start_clock();

    let stdin = io::stdin();
    let stdout = io::stdout();
    let flush = || stdout.lock().flush().unwrap();

    let input = read_input(&stdin);
    let mut state = State::new(input.n);

    let mut cells = vec![];
    for h in input.house.iter() {
        let mut p = input.source.first().unwrap().clone();
        while p != *h {
            cells.push(p.clone());
            if p.y < h.y {
                p.y += 1;
            } else if p.y > h.y {
                p.y -= 1;
            } else {
                if p.x < h.x {
                    p.x += 1;
                } else if p.x > h.x {
                    p.x -= 1;
                }
            }
        }
        cells.push(p.clone());
    }

    loop {
        while let Some(cell) = cells.last() {
            if state.is_broken.get(cell) == 1 {
                cells.pop();
            } else {
                break;
            }
        }
        let cell = cells.last().unwrap();

        println!("{} {} {}", cell.y, cell.x, 1000);
        flush();
        state.damage.add(cell, 1000);

        let r = read_result(&stdin);

        if r == -1 {
            eprintln!("Invalid");
            break;
        } else if r == 0 {
            continue;
        } else if r == 1 {
            state.is_broken.set(cell, 1);
        } else if r == 2 {
            state.is_broken.set(cell, 1);
            eprintln!("End");
            break;
        }
    }

    eprintln!("elapsed seconds: {:.4}", time::elapsed_seconds());
}
