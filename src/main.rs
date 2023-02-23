mod def;
mod grid;
mod interactor;
mod solver;
mod util;

use util::*;

fn main() {
    time::start_clock();

    let mut solver = solver::Solver::new();
    solver.solve();
}
