mod def;
mod io;
mod solver;
mod util;

use def::*;
use util::*;

fn main() {
    time::start_clock();

    let io = io::IO::new();

    let input = io.read_input();
    let mut state = State::new(input.n);

    solver::solve_greedy(&mut state, &input, &io);

    eprintln!("elapsed seconds: {:.4}", time::elapsed_seconds());
}
