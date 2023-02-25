use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashSet},
    fs::File,
    io::Write,
};

use crate::def::*;

const DELTA: [(i64, i64); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

#[derive(Debug)]
pub struct Grid {
    pub total_score: i64,
    pub estimated_weight: Vec2d<i64>,
    pub is_used: Vec2d<bool>,
    pub house: Vec<Pos>,
    pub source: Vec<Pos>,
}

impl Grid {
    pub fn find_path_to_nearest_source(
        &self,
        start: &Pos,
        upper: i64,
        source: &Vec<Pos>,
        c: i64,
    ) -> (Vec<Pos>, i64) {
        let (dist, par) = self.dijkstra(start, upper, c);

        let mut best_source_pos = None;
        // 一番繋げるまでの距離が近い水源を探す
        for s_pos in source.iter() {
            if best_source_pos.is_none() || dist.get(s_pos) < dist.get(best_source_pos.unwrap()) {
                best_source_pos = Some(s_pos);
            }
        }

        // h -> best_source_posまでに通る辺を復元する
        let mut cur = best_source_pos.unwrap().clone();
        let mut path = vec![cur.clone()];
        while &cur != start {
            let parent = par.get(&cur).unwrap();
            path.push(parent);
            cur = parent;
        }
        path.reverse();
        (path, dist.get(&best_source_pos.unwrap()))
    }

    pub fn find_current_path_to_source(&self, start: &Pos) -> Option<(Vec<Pos>, i64)> {
        fn dfs(
            p: &Pos,
            par: &Pos,
            st: &mut Vec<Pos>,
            seen: &mut HashSet<Pos>,
            grid: &Grid,
        ) -> bool {
            if grid.source.contains(p) {
                return true;
            }
            for (dy, dx) in DELTA {
                let np = Pos {
                    y: p.y + dy,
                    x: p.x + dx,
                };
                if !np.is_valid() || par == &np {
                    continue;
                }
                // FIXME: is_usedに常にhouse、sourceが含まれるように修正
                let is_not_used = !grid.is_used.get(&np) && !grid.source.contains(&np);
                if is_not_used || seen.contains(&np) {
                    continue;
                }
                st.push(np.clone());
                seen.insert(np.clone());
                if dfs(&np, &p, st, seen, grid) {
                    return true;
                }
                st.pop();
                seen.remove(&np);
            }
            false
        }

        let mut st = vec![];
        st.push(start.clone());
        let mut seen = HashSet::new();
        seen.insert(start.clone());

        if dfs(start, &Pos { y: -1, x: -1 }, &mut st, &mut seen, &self) {
            let mut total_weight = 0;
            for p in st.iter() {
                total_weight += self.estimated_weight.get(p);
            }
            Some((st, total_weight))
        } else {
            None
        }
    }

    pub fn find_unconnected_houses(&self) -> Vec<usize> {
        let mut v = vec![];
        for (i, h_pos) in self.house.iter().enumerate() {
            let result = self.find_current_path_to_source(h_pos);
            if result.is_none() {
                v.push(i);
            }
        }
        v
    }

    pub fn set(&mut self, p: &Pos, v: bool) -> bool {
        if self.is_used.get(&p) == v {
            return false;
        }
        if self.is_used.get(&p) {
            self.total_score -= self.estimated_weight.get(&p);
        } else {
            self.total_score += self.estimated_weight.get(&p);
        }
        self.is_used.set(&p, v);
        return true;
    }

    fn dijkstra(&self, start: &Pos, upper: i64, c: i64) -> (Vec2d<i64>, Vec2d<Option<Pos>>) {
        let mut dist = Vec2d::new(N, N, INF);
        let mut par = Vec2d::new(N, N, None);

        let mut heap = BinaryHeap::new();
        dist.set(start, 0);
        heap.push((Reverse(0), start.clone()));

        while let Some((Reverse(d), p)) = heap.pop() {
            if dist.get(&p) < d {
                continue;
            }
            for (dy, dx) in DELTA {
                let np = Pos {
                    y: p.y + dy,
                    x: p.x + dx,
                };
                if !np.is_valid() {
                    continue;
                }
                let w = if self.is_used.get(&np) {
                    0
                } else {
                    self.estimated_weight.get(&np) + 2 * c
                };
                if dist.get(&np) <= d + w {
                    continue;
                }
                par.set(&np, Some(p.clone()));
                dist.set(&np, d + w);
                heap.push((Reverse(d + w), np.clone()));
            }
        }

        (dist, par)
    }

    #[allow(unused)]
    pub fn output_grid(&self, output_file: &str) {
        let mut file = File::create(output_file).unwrap();
        for y in 0..N {
            for x in 0..N {
                if self.is_used.get(&Pos {
                    y: y as i64,
                    x: x as i64,
                }) {
                    write!(file, "1 ").unwrap();
                } else {
                    write!(file, "0 ").unwrap();
                }
            }
            writeln!(file).unwrap();
        }

        for y in 0..N {
            for x in 0..N {
                write!(
                    file,
                    "{} ",
                    self.estimated_weight.get(&Pos {
                        y: y as i64,
                        x: x as i64
                    })
                )
                .unwrap();
            }
            writeln!(file).unwrap();
        }
    }
}
