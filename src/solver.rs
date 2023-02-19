use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
};

use crate::{def::*, interactor::*, param::*};

struct Edge {
    u: usize,
    v: usize,
}

impl Edge {
    fn other_point(&self, v: usize) -> usize {
        debug_assert!(self.u == v || self.v == v);
        self.v + self.u - v
    }
}

struct Graph {
    points: Vec<Pos>,
    edges: Vec<Edge>,
    adj: Vec<Vec<usize>>,
    point_index: HashMap<Pos, usize>,
}

impl Graph {
    fn new(
        xs: &Vec<i64>,
        ys: &Vec<i64>,
        state: &mut State,
        p_test_power: &Vec<i64>,
        interactor: &Interactor,
    ) -> Graph {
        let mut points = vec![];
        let mut point_index = HashMap::new();
        for x in xs.iter() {
            for y in ys.iter() {
                let pos = Pos { x: *x, y: *y };
                crack_point(state, &pos, &p_test_power, interactor);
                point_index.insert(pos, points.len());
                points.push(pos);
            }
        }

        let mut edges = vec![];
        let mut adj = vec![vec![]; points.len()];
        for x in xs.iter() {
            for j in 0..(ys.len() - 1) {
                let u = Pos { x: *x, y: ys[j] };
                let v = Pos {
                    x: *x,
                    y: ys[j + 1],
                };
                let ui = point_index.get(&u).unwrap();
                let vi = point_index.get(&v).unwrap();
                adj[*ui].push(edges.len());
                adj[*vi].push(edges.len());
                edges.push(Edge { u: *ui, v: *vi });
            }
        }
        for y in ys.iter() {
            for i in 0..(xs.len() - 1) {
                let u = Pos { x: xs[i], y: *y };
                let v = Pos {
                    x: xs[i + 1],
                    y: *y,
                };
                let ui = point_index.get(&u).unwrap();
                let vi = point_index.get(&v).unwrap();
                adj[*ui].push(edges.len());
                adj[*vi].push(edges.len());
                edges.push(Edge { u: *ui, v: *vi });
            }
        }

        Graph {
            points,
            edges,
            adj,
            point_index,
        }
    }

    fn edge_weight(&self, edge_index: usize, c: i64, state: &State) -> i64 {
        let estimated_hardness = |v: usize| -> i64 {
            let p = &self.points[v];
            if state.is_broken.get(p) {
                state.damage.get(p)
            } else {
                // まだ壊れていなかったら、2倍の強度を想定する
                state.damage.get(p) * 2
            }
        };
        let edge = &self.edges[edge_index];
        let dist = self.points[edge.u].dist(&self.points[edge.v]);
        let hard_mean = (estimated_hardness(edge.u) + estimated_hardness(edge.v)) / 2;
        (hard_mean + c) * dist
    }

    fn pos_to_index(&self, pos: &Pos) -> usize {
        debug_assert!(self.point_index.contains_key(pos));
        *self.point_index.get(pos).unwrap()
    }

    fn dijkstra(
        &self,
        start: usize,
        c: i64,
        state: &State,
        edge_used: &Vec<i64>,
    ) -> (Vec<i64>, Vec<usize>) {
        let mut dist = vec![INF; self.points.len()];
        let mut par_edge = vec![NA; self.points.len()];

        // hから各頂点までの距離を計算する
        let mut heap = BinaryHeap::new();
        dist[start] = 0;
        heap.push((Reverse(0), start));

        // TODO: 枝刈り
        while let Some((Reverse(d), v)) = heap.pop() {
            if dist[v] < d {
                continue;
            }
            for edge_index in self.adj[v].iter() {
                let weight = if edge_used[*edge_index] == 0 {
                    // 使われていない辺なら、重みはself.edge_weight
                    self.edge_weight(*edge_index, c, state)
                } else {
                    // すでに使われている辺なら、重みは0
                    0
                };
                let u = self.edges[*edge_index].other_point(v);
                if dist[u] <= dist[v] + weight {
                    continue;
                }
                par_edge[u] = *edge_index;
                dist[u] = dist[v] + weight;
                heap.push((Reverse(dist[u]), u));
            }
        }

        (dist, par_edge)
    }
}

struct AnnealingState {
    edge_used: Vec<i64>,
    to_source_paths: Vec<Vec<usize>>,
    score: i64,
}

impl AnnealingState {
    fn new(input: &Input, graph: &Graph, state: &State, c: i64) -> AnnealingState {
        let mut annealing_state = AnnealingState {
            edge_used: vec![],
            to_source_paths: vec![],
            score: 0,
        };
        annealing_state.recalculate_all(input, graph, state, c);
        annealing_state
    }

    fn find_path_to_source(
        &self,
        point_idx: usize,
        input: &Input,
        graph: &Graph,
        state: &State,
        c: i64,
    ) -> Vec<usize> {
        let (dist, par_edge) = graph.dijkstra(point_idx, c, state, &self.edge_used);
        let mut best_source_index = NA;
        // 一番繋げるまでの距離が近い水源を探す
        for src in input.source.iter() {
            let point_index = graph.pos_to_index(&src);
            if best_source_index == NA || dist[point_index] < dist[best_source_index] {
                best_source_index = point_index;
            }
        }

        // h -> best_point_indexまでに通る辺を復元する
        let mut cur = best_source_index;
        let mut edge_path = vec![];
        while cur != point_idx {
            let par_edge_index = par_edge[cur];
            edge_path.push(par_edge_index);
            cur = graph.edges[par_edge_index].other_point(cur);
        }
        edge_path.reverse();

        edge_path
    }

    fn recalculate_all(&mut self, input: &Input, graph: &Graph, state: &State, c: i64) {
        self.edge_used = vec![0; graph.edges.len()];
        self.to_source_paths = vec![vec![]; input.house.len()];
        self.score = 0;
        for (i, h_pos) in input.house.iter().enumerate() {
            let point_idx = graph.pos_to_index(h_pos);
            let edge_path = self.find_path_to_source(point_idx, input, graph, state, c);
            self.set_edge_path(i, edge_path, graph, state, c);
        }
    }

    fn set_edge_path(
        &mut self,
        h_idx: usize,
        edge_path: Vec<usize>,
        graph: &Graph,
        state: &State,
        c: i64,
    ) {
        for edge_index in edge_path.iter() {
            if self.edge_used[*edge_index] == 0 {
                self.score += graph.edge_weight(*edge_index, c, state);
            }
            self.edge_used[*edge_index] += 1;
        }
        self.to_source_paths[h_idx] = edge_path;
    }

    #[allow(unused)]
    fn remove_edge_path(&mut self, h_idx: usize, graph: &Graph, state: &State, param: &Param) {
        for edge_index in self.to_source_paths[h_idx].iter() {
            if self.edge_used[*edge_index] == 1 {
                self.score -= graph.edge_weight(*edge_index, param.c, state);
            }
            self.edge_used[*edge_index] -= 1;
        }
    }
}

pub fn solve(state: &mut State, input: &Input, interactor: &Interactor, param: &Param) {
    let (xs, ys) = create_grid_axis(&input, param.p_grid_size);
    let graph = Graph::new(&xs, &ys, state, &param.p_test_power, interactor);
    let mut annealing_state = AnnealingState::new(input, &graph, state, param.c);

    // 繋ぐ辺を最適化する
    for _ in 0..100 {
        for (i, edge) in graph.edges.iter().enumerate() {
            if annealing_state.edge_used[i] == 0 {
                continue;
            }
            for v in [edge.u, edge.v] {
                crack_point(state, &graph.points[v], &param.p_test_power2, interactor);
            }
        }
        annealing_state.recalculate_all(input, &graph, state, param.c);
    }

    // 辺の間を繋げる
    let mut edges = vec![];
    for edge_path in &mut annealing_state.to_source_paths {
        edges.append(edge_path);
    }
    create_path(&edges, &graph, state, &param, interactor);
}

//
// 辺の間を繋げる
//

fn create_path(
    edges: &Vec<usize>,
    graph: &Graph,
    state: &mut State,
    param: &Param,
    interactor: &Interactor,
) {
    let mut cells = vec![];

    for edge_index in edges.iter() {
        let mut p = graph.points[graph.edges[*edge_index].u];
        let h = graph.points[graph.edges[*edge_index].v];
        while p != h {
            cells.push(p.clone());
            if p.y < h.y {
                p.y += 1;
            } else if p.y > h.y {
                p.y -= 1;
            } else if p.x < h.x {
                p.x += 1;
            } else if p.x > h.x {
                p.x -= 1;
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
        if let Some(cell) = cells.last() {
            interactor.respond(cell, i64::max(20, param.c), state);
        } else {
            break;
        }
    }
}

//
// ユーティリティ
//

fn crack_point(state: &mut State, pos: &Pos, test_power: &Vec<i64>, interactor: &Interactor) {
    for test_power in test_power.iter() {
        if state.is_broken.get(pos) {
            break;
        }
        let power = test_power - state.damage.get(pos);
        if power <= 0 {
            break;
        }
        interactor.respond(pos, power, state);
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
