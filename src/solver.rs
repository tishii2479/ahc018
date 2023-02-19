use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
};

use crate::{def::*, interactor::*, param::*, util::rnd};

struct Point {
    pos: Pos,
    hard: i64,
}

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
    points: Vec<Point>,
    edges: Vec<Edge>,
    adj: Vec<Vec<usize>>,
    point_index: HashMap<Pos, usize>,
}

impl Graph {
    fn new(
        xs: &Vec<i64>,
        ys: &Vec<i64>,
        state: &mut State,
        param: &Param,
        interactor: &Interactor,
    ) -> Graph {
        let mut points = vec![];
        let mut point_index = HashMap::new();
        for x in xs.iter() {
            for y in ys.iter() {
                let pos = Pos { x: *x, y: *y };
                let hard = Graph::create_point(state, &pos, &param, interactor);
                point_index.insert(pos, points.len());
                points.push(Point { pos, hard });
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

    fn edge_weight(&self, edge_index: usize, c: i64) -> i64 {
        let edge = &self.edges[edge_index];
        let dist = self.points[edge.u].pos.dist(&self.points[edge.v].pos);
        let hard_mean = (self.points[edge.u].hard + self.points[edge.v].hard) / 2;
        (hard_mean + c) * dist
    }

    fn create_point(state: &mut State, pos: &Pos, param: &Param, interactor: &Interactor) -> i64 {
        for test_power in param.p_test_power.iter() {
            if state.is_broken.get(pos) {
                break;
            }
            let power = test_power - state.damage.get(pos);
            if power <= 0 {
                continue;
            }
            interactor.respond(pos, power, state);
        }
        if state.is_broken.get(pos) {
            state.damage.get(pos)
        } else {
            param.p_hard_max
        }
    }

    fn pos_to_index(&self, pos: &Pos) -> usize {
        debug_assert!(self.point_index.contains_key(pos));
        *self.point_index.get(pos).unwrap()
    }
}

struct AnnealingState {
    edge_used: Vec<i64>,
    to_source_paths: Vec<Vec<usize>>,
    score: i64,
}

impl AnnealingState {
    fn new(input: &Input, graph: &Graph, param: &Param) -> AnnealingState {
        let mut state = AnnealingState {
            edge_used: vec![0; graph.edges.len()],
            to_source_paths: vec![vec![]; input.house.len()],
            score: 0,
        };

        // TODO: 初期解を作る
        for (i, h_pos) in input.house.iter().enumerate() {
            let point_idx = graph.pos_to_index(h_pos);
            let edge_path = state.find_path_to_source(point_idx, input, graph, param);
            state.set_edge_path(i, edge_path, graph, param);
        }

        state
    }

    fn find_path_to_source(
        &self,
        point_idx: usize,
        input: &Input,
        graph: &Graph,
        param: &Param,
    ) -> Vec<usize> {
        let mut dist = vec![INF; graph.points.len()];
        let mut par_edge = vec![NA; graph.points.len()];

        // hから各頂点までの距離を計算する
        let mut heap = BinaryHeap::new();
        dist[point_idx] = 0;
        heap.push((Reverse(0), point_idx));

        while let Some((Reverse(d), v)) = heap.pop() {
            if dist[v] < d {
                continue;
            }
            for edge_index in graph.adj[v].iter() {
                let weight = if self.edge_used[*edge_index] == 0 {
                    // 使われていない辺なら、重みはgraph.edge_weight
                    graph.edge_weight(*edge_index, param.c)
                } else {
                    // すでに使われている辺なら、重みは0
                    0
                };
                let u = graph.edges[*edge_index].other_point(v);
                if dist[u] <= dist[v] + weight {
                    continue;
                }
                par_edge[u] = *edge_index;
                dist[u] = dist[v] + weight;
                heap.push((Reverse(dist[u]), u));
            }
        }

        // 一番繋げるまでの距離が近い水源を探す
        let mut best_source_index = NA;
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

    fn set_edge_path(&mut self, h_idx: usize, edge_path: Vec<usize>, graph: &Graph, param: &Param) {
        for edge_index in edge_path.iter() {
            if self.edge_used[*edge_index] == 0 {
                self.score += graph.edge_weight(*edge_index, param.c);
            }
            self.edge_used[*edge_index] += 1;
        }
        self.to_source_paths[h_idx] = edge_path;
    }

    fn evaluate_remove_edge_path(&mut self, h_idx: usize, graph: &Graph, param: &Param) {
        for edge_index in self.to_source_paths[h_idx].iter() {
            if self.edge_used[*edge_index] == 1 {
                self.score -= graph.edge_weight(*edge_index, param.c);
            }
            self.edge_used[*edge_index] -= 1;
        }
    }
}

pub fn solve(state: &mut State, input: &Input, interactor: &Interactor, param: &Param) {
    let (xs, ys) = create_grid_axis(&input, param.p_grid_size);
    let graph = Graph::new(&xs, &ys, state, &param, interactor);
    let mut annealing_state = AnnealingState::new(input, &graph, param);

    for _ in 0..1000 {
        let h_idx = rnd::gen_range(0, input.k);
        let current_score = annealing_state.score;
        let current_edge_path = annealing_state.to_source_paths[h_idx].clone();
        annealing_state.evaluate_remove_edge_path(h_idx, &graph, param);
        let edge_path = annealing_state.find_path_to_source(
            graph.pos_to_index(&input.house[h_idx]),
            input,
            &graph,
            param,
        );
        annealing_state.set_edge_path(h_idx, edge_path, &graph, param);
        let new_score = annealing_state.score;
        if new_score < current_score {
            // 採用
        } else {
            // ロールバック
            annealing_state.set_edge_path(h_idx, current_edge_path, &graph, param);
        }
    }

    // 壊す
    let mut cells = vec![];

    for edge_path in annealing_state.to_source_paths.iter() {
        for edge_index in edge_path.iter() {
            let mut p = graph.points[graph.edges[*edge_index].u].pos;
            let h = graph.points[graph.edges[*edge_index].v].pos;
            while p != h {
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
            interactor.respond(cell, 10, state);
        } else {
            break;
        }
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
            new_ps.push(ps.first().unwrap() + p_grid_size)
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
