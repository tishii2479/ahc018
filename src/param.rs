use crate::def::Input;

pub struct Param {
    pub c: i64,
    pub p1: usize,
    pub p_grid_size: usize,
    pub p_test_power: Vec<i64>,
    pub p_test_power2: Vec<i64>,
}

impl Param {
    pub fn new(input: &Input) -> Param {
        Param {
            c: input.c,
            p1: 10,
            p_grid_size: 20,
            p_test_power: vec![20, 60, 100],
            p_test_power2: vec![300, 500, 800],
        }
    }
}
