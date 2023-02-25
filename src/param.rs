use crate::def::Input;

pub struct Param {
    pub c: i64,
    pub p1: usize,
}

impl Param {
    pub fn new(input: &Input) -> Param {
        Param { c: input.c, p1: 10 }
    }
}
