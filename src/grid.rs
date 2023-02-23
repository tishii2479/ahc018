use std::{cmp::Reverse, collections::BinaryHeap};

use crate::def::*;

const DELTA: [(i64, i64); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

pub struct Grid {
    pub total_score: i64,
    // TODO: すでに壊している箇所は重みをゼロにして、estimated_weightに変更する
    pub estimated_hardness: Vec2d<i64>,
    pub is_used: Vec2d<bool>,
}

impl Grid {
    pub fn find_path_to_nearest_source(
        &self,
        start: &Pos,
        upper: i64,
        source: &Vec<Pos>,
    ) -> (Vec<Pos>, i64) {
        let (dist, par) = self.dijkstra(start, upper);

        let mut best_source_pos = None;
        // 一番繋げるまでの距離が近い水源を探す
        for s_pos in source.iter() {
            if best_source_pos.is_none() || dist.get(s_pos) < dist.get(best_source_pos.unwrap()) {
                best_source_pos = Some(s_pos);
            }
        }

        // h -> best_source_posまでに通る辺を復元する
        let mut cur = best_source_pos.unwrap().clone();
        let mut edge_path = vec![cur.clone()];
        while &cur != start {
            let parent = par.get(&cur).unwrap();
            edge_path.push(parent);
            cur = parent;
        }
        edge_path.reverse();
        (edge_path, dist.get(&best_source_pos.unwrap()))
    }

    pub fn find_current_path_to_source(&self, start: &Pos) -> Option<(Vec<Pos>, i64)> {
        fn dfs(p: &Pos, st: &mut Vec<Pos>, grid: &Grid) -> bool {
            for (dy, dx) in DELTA {
                let np = Pos {
                    y: p.y + dy,
                    x: p.x + dx,
                };
                if !np.is_valid() || !grid.is_used.get(&np) {
                    continue;
                }
                st.push(np.clone());
                if dfs(&np, st, grid) {
                    return true;
                }
                st.pop();
            }
            false
        }

        let mut st = vec![];
        st.push(start.clone());

        if dfs(start, &mut st, &self) {
            let mut total_weight = 0;
            for p in st.iter() {
                total_weight += self.estimated_hardness.get(p);
            }
            Some((st, total_weight))
        } else {
            None
        }
    }

    pub fn find_unconnected_houses(&self, houses: &Vec<Pos>) -> Vec<usize> {
        let mut v = vec![];
        for (i, h_pos) in houses.iter().enumerate() {
            if self.find_current_path_to_source(h_pos).is_none() {
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
            self.total_score -= self.estimated_hardness.get(&p);
        } else {
            self.total_score += self.estimated_hardness.get(&p);
        }
        self.is_used.set(&p, v);
        return true;
    }

    fn dijkstra(&self, start: &Pos, upper: i64) -> (Vec2d<i64>, Vec2d<Option<Pos>>) {
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
                let w = self.estimated_hardness.get(&np);
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
}
