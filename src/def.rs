#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Pos {
    pub y: i64,
    pub x: i64,
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
