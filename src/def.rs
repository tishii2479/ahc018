use std::{fs::File, io::Write};

pub const INF: i64 = 100_000_000_000_000;
pub const N: usize = 200;
pub const S_MAX: i64 = 5000;

pub struct Input {
    pub n: usize,
    pub w: usize,
    pub k: usize,
    pub c: i64,
    pub source: Vec<Pos>,
    pub house: Vec<Pos>,
}

pub struct State {
    pub is_broken: Vec2d<bool>,
    pub damage: Vec2d<i64>,
    pub damage_before_break: Vec2d<i64>,
    pub total_damage: i64,
}

impl State {
    pub fn new(n: usize) -> State {
        State {
            is_broken: Vec2d::new(n, n, false),
            damage: Vec2d::new(n, n, 0),
            damage_before_break: Vec2d::new(n, n, 0),
            total_damage: 0,
        }
    }

    #[allow(unused)]
    pub fn output_state(&self, output_file: &str) {
        let mut file = File::create(output_file).unwrap();
        for y in 0..N {
            for x in 0..N {
                write!(
                    file,
                    "{} ",
                    self.damage.get(&Pos {
                        y: y as i64,
                        x: x as i64
                    })
                )
                .unwrap();
            }
            writeln!(file).unwrap();
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash, PartialOrd, Ord)]
pub struct Pos {
    pub y: i64,
    pub x: i64,
}

impl Pos {
    pub fn dist(&self, to: &Pos) -> i64 {
        i64::abs(to.y - self.y) + i64::abs(to.x - self.x)
    }

    pub fn is_valid(&self) -> bool {
        self.x >= 0 && self.y >= 0 && self.x < 200 && self.y < 200
    }

    pub fn to_idx(&self) -> usize {
        self.y as usize * N + self.x as usize
    }
}

#[derive(Debug, Clone)]
#[allow(unused)]
pub struct Vec2d<T> {
    vec: Vec<T>,
    n: usize,
    m: usize,
}

#[allow(unused)]
impl<T> Vec2d<T>
where
    T: Copy + Clone,
{
    pub fn new(n: usize, m: usize, init_value: T) -> Vec2d<T> {
        Vec2d {
            vec: vec![init_value; n * m],
            n,
            m,
        }
    }

    pub fn get(&self, pos: &Pos) -> T {
        self.vec[pos.to_idx()]
    }

    pub fn set(&mut self, pos: &Pos, val: T) {
        self.vec[pos.to_idx()] = val
    }
}
