mod def;
mod grid;
mod interactor;
mod param;
mod solver;
mod solver2;
mod util;

use def::State;
use interactor::Interactor;
use param::Param;
use util::*;

fn main() {
    time::start_clock();

    let mut interactor = Interactor::new();
    let input = interactor.read_input();
    let state = State::new(input.n);
    let param = Param::new(&input);

    if input.c <= 2 {
        let mut solver = solver::Solver {
            input,
            state,
            param,
            interactor,
        };
        solver.solve();
    } else {
        solver2::solve(&input, &mut interactor, &param);
    }
}
