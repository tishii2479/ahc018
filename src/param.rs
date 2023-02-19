pub struct Param {
    pub c: i64,
    pub p_grid_size: i64,
    pub p_test_power: Vec<i64>,
    pub p_test_power2: Vec<i64>,
    pub p_hard_max: i64,
}

impl Param {
    pub fn new(c: i64) -> Param {
        Param {
            c,
            p_grid_size: 20,
            p_test_power: vec![20, 60, 100],
            p_test_power2: vec![300, 500, 1000],
            p_hard_max: 200,
        }
    }
}
