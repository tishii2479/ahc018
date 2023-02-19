mod def;
mod interactor;
mod param;
mod solver;
mod util;

use def::*;
use param::*;
use util::*;

fn main() {
    time::start_clock();

    let interactor = interactor::Interactor::new();
    let input = interactor.read_input();
    let param = Param::new(input.c);

    let mut state = State::new(input.n);

    solver::solve(&mut state, &input, &interactor, &param);
}
