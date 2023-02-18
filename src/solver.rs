use crate::{def::*, io::IO};

pub fn solve_greedy(state: &mut State, input: &Input, io: &IO) {
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

        io.output(format!("{} {} {}", cell.y, cell.x, 10));
        state.damage.add(cell, 10);

        let r = io.read_result();

        if r == -1 {
            eprintln!("Invalid");
            break;
        } else if r == 0 {
            continue;
        } else if r == 1 {
            state.is_broken.set(cell, true);
        } else if r == 2 {
            state.is_broken.set(cell, true);
            break;
        }
    }
}
