pub fn solve_greedy(state: &mut State, input: &Input, interactor: &Interactor, param: &Param) {
    let mut cells = vec![];
    for h in input.house.iter() {
        let mut nearest_source = input.source.first().unwrap().clone();
        for src in input.source.iter() {
            if src.dist(h) < nearest_source.dist(h) {
                nearest_source = src.clone();
            }
        }
        let mut p = nearest_source;
        while p != *h {
            cells.push(p.clone());
            if p.y < h.y {
                p.y += 1;
            } else if p.y > h.y {
                p.y -= 1;
            } else {
                if p.x < h.x {
                    p.x += 1;
                } else if p.x > h.x {
                    p.x -= 1;
                }
            }
        }
        cells.push(p.clone());
    }

    loop {
        while let Some(cell) = cells.last() {
            if state.is_broken.get(cell) {
                cells.pop();
            } else {
                break;
            }
        }
        let cell = cells.last().unwrap();
        interactor.respond(cell, 10, state);
    }
}

fn create_grid_axis(input: &Input, p_grid_size: i64) -> (Vec<i64>, Vec<i64>) {
    fn create_equally_spaced_axis(ps: &Vec<i64>, p_grid_size: i64, n: usize) -> Vec<i64> {
        debug_assert!(ps.len() > 0);
        let mut new_ps = vec![];

        if ps.first().unwrap() >= &p_grid_size {
            new_ps.push(ps.first().unwrap() - p_grid_size)
        }
        for i in 0..(ps.len() - 1) {
            new_ps.push(ps[i]);
            let dist = ps[i + 1] - ps[i];
            let count = (dist + p_grid_size / 2) / p_grid_size;
            for j in 1..count {
                let real_p = j as f64 * (dist as f64 / count as f64) + ps[i] as f64;
                let approx_p = real_p.round() as i64;
                debug_assert!(new_ps.last().unwrap() != &approx_p);
                new_ps.push(approx_p);
            }
        }
        new_ps.push(*ps.last().unwrap());

        if ps.last().unwrap() + p_grid_size < n as i64 {
            new_ps.push(ps.last().unwrap() + p_grid_size)
        }
        new_ps
    }

    let mut xs = vec![];
    let mut ys = vec![];

    for src in input.source.iter() {
        if !xs.contains(&src.x) {
            xs.push(src.x);
        }
        if !ys.contains(&src.y) {
            ys.push(src.y);
        }
    }
    for h in input.house.iter() {
        if !xs.contains(&h.x) {
            xs.push(h.x);
        }
        if !ys.contains(&h.y) {
            ys.push(h.y);
        }
    }
    xs.sort();
    ys.sort();

    let new_xs = create_equally_spaced_axis(&xs, p_grid_size, input.n);
    let new_ys = create_equally_spaced_axis(&ys, p_grid_size, input.n);

    (new_xs, new_ys)
}
