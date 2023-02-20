use crate::{def::*, util::time};
use std::{io, io::Write};

pub struct Interactor {
    stdin: io::Stdin,
    stdout: io::Stdout,
}

impl Interactor {
    pub fn new() -> Interactor {
        Interactor {
            stdin: io::stdin(),
            stdout: io::stdout(),
        }
    }

    pub fn read_input(&self) -> Input {
        let mut user_input = String::new();
        self.stdin.read_line(&mut user_input).unwrap();
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
            self.stdin.read_line(&mut user_input).unwrap();
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
            self.stdin.read_line(&mut user_input).unwrap();
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

    pub fn respond(&self, pos: &Pos, power: i64, state: &mut State) -> bool {
        println!("{} {} {}", pos.y, pos.x, power);
        self.stdout.lock().flush().unwrap();
        state.damage.add(pos, power);

        let mut user_input = String::new();
        self.stdin.read_line(&mut user_input).unwrap();
        let r: i64 = user_input.trim().parse().unwrap();

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
