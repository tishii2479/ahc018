use crate::interactor::Interactor;

pub const INF: i64 = 100_000_000_000_000;
pub const NA: usize = 100_000_000_000_000;
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
    pub is_broken: Vec2dBool,
    pub damage: Vec2d,
}

impl State {
    pub fn new(n: usize) -> State {
        State {
            is_broken: Vec2dBool::new(n, n),
            damage: Vec2d::new(n, n),
        }
    }

    pub fn crack_point(&mut self, pos: &Pos, test_power: &Vec<i64>, interactor: &Interactor) {
        for test_power in test_power.iter() {
            if self.is_broken.get(pos) {
                break;
            }
            let power = test_power - self.damage.get(pos);
            if power <= 0 {
                break;
            }
            interactor.respond(pos, power, self);
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
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
}

#[allow(unused)]
pub struct Vec2d {
    vec: Vec<i64>,
    n: usize,
    m: usize,
}

#[allow(unused)]
impl Vec2d {
    pub fn new(n: usize, m: usize) -> Vec2d {
        Vec2d {
            vec: vec![0; n * m],
            n,
            m,
        }
    }

    pub fn get(&self, pos: &Pos) -> i64 {
        self.vec[pos.y as usize * self.m + pos.x as usize]
    }

    pub fn set(&mut self, pos: &Pos, val: i64) {
        self.vec[pos.y as usize * self.m + pos.x as usize] = val
    }

    pub fn add(&mut self, pos: &Pos, val: i64) {
        self.vec[pos.y as usize * self.m + pos.x as usize] += val
    }
}

#[allow(unused)]
pub struct Vec2dBool {
    vec: Vec<bool>,
    n: usize,
    m: usize,
}

#[allow(unused)]
impl Vec2dBool {
    pub fn new(n: usize, m: usize) -> Vec2dBool {
        Vec2dBool {
            vec: vec![false; n * m],
            n,
            m,
        }
    }

    pub fn get(&self, pos: &Pos) -> bool {
        self.vec[pos.y as usize * self.m + pos.x as usize]
    }

    pub fn set(&mut self, pos: &Pos, val: bool) {
        self.vec[pos.y as usize * self.m + pos.x as usize] = val
    }
}
