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
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Pos {
    pub y: i64,
    pub x: i64,
}

impl Pos {
    pub fn dist(&self, to: &Pos) -> i64 {
        i64::abs(to.y - self.y) + i64::abs(to.x - self.x)
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
