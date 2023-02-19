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
