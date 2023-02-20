pub struct Param {
    pub c: i64,
    pub p_grid_size: i64,
    pub p_test_power: Vec<i64>,
    pub p_test_power2: Vec<i64>,
}

impl Param {
    pub fn new(c: i64) -> Param {
        Param {
            c,
            p_grid_size: 20,
            p_test_power: vec![20, 50, 80, 110],
            p_test_power2: vec![200, 300, 500, 700, 900],
        }
    }
}
