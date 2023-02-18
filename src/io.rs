use crate::def::*;
use std::{io, io::Write};

pub struct IO {
    stdin: io::Stdin,
    stdout: io::Stdout,
}

impl IO {
    pub fn new() -> IO {
        IO {
            stdin: io::stdin(),
            stdout: io::stdout(),
        }
    }

    pub fn output(&self, str: String) {
        println!("{}", str);
        self.stdout.lock().flush().unwrap();
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

    pub fn read_result(&self) -> i64 {
        let mut user_input = String::new();
        self.stdin.read_line(&mut user_input).unwrap();
        user_input.trim().parse().unwrap()
    }
}
