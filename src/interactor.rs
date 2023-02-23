use proconio::*;

use crate::{def::*, util::time};
use std::io::{Stdin, Write};

pub struct Interactor {
    source: proconio::source::line::LineSource<std::io::BufReader<Stdin>>,
}

impl Interactor {
    pub fn new() -> Interactor {
        Interactor {
            source: proconio::source::line::LineSource::new(std::io::BufReader::new(
                std::io::stdin(),
            )),
        }
    }

    pub fn read_input(&mut self) -> Input {
        input! {
            from &mut self.source,
            n: usize,
            w: usize,
            k: usize,
            c: i64,
        }
        let mut source = vec![];
        for _ in 0..w {
            input! {
                from &mut self.source,
                y: i64,
                x: i64
            }
            source.push(Pos { y, x });
        }
        let mut house = vec![];
        for _ in 0..k {
            input! {
                from &mut self.source,
                y: i64,
                x: i64
            }
            house.push(Pos { y, x });
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

    pub fn add_damage(&mut self, pos: &Pos, power: i64, state: &mut State) -> bool {
        println!("{} {} {}", pos.y, pos.x, power);
        std::io::stdout().flush().unwrap();

        let current_damage = state.damage.get(pos);
        state.damage_before_break.set(pos, current_damage);
        state.damage.set(pos, current_damage + power);

        input! {
            from &mut self.source,
            r: i64,
        }

        if r == 0 {
            return false;
        } else if r == 1 {
            state.is_broken.set(pos, true);
            return true;
        } else if r == 2 {
            // 終了する
            eprintln!("elapsed seconds: {:.4}", time::elapsed_seconds());
            std::process::exit(0);
        } else if r == -1 {
            panic!("Invalid operation");
        } else {
            panic!("Invalid result output");
        }
    }
}
