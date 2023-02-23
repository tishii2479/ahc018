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
    pub is_broken: Vec2d<bool>,
    pub damage: Vec2d<i64>,
    pub total_damage: i64,
}

impl State {
    pub fn new(n: usize) -> State {
        State {
            is_broken: Vec2d::new(n, n, false),
            damage: Vec2d::new(n, n, 0),
            total_damage: 0,
        }
    }

    pub fn crack_point(&mut self, pos: &Pos, test_power: &Vec<i64>, interactor: &mut Interactor) {
        for test_power in test_power.iter() {
            if self.is_broken.get(pos) {
                break;
            }
            let power = test_power - self.damage.get(pos);
            if power <= 0 {
                break;
            }
            self.total_damage += power;
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
        self.vec[pos.y as usize * self.m + pos.x as usize]
    }

    pub fn set(&mut self, pos: &Pos, val: T) {
        self.vec[pos.y as usize * self.m + pos.x as usize] = val
    }
}
