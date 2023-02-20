use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
};

use crate::{def::*, interactor::*, param::*, util::rnd};

pub fn solve(input: &Input, interactor: &Interactor, param: &Param) {
    let mut state = State::new(input.n);
    let (xs, ys) = create_grid_axis(&input, param.p_grid_size);
    for x in xs.iter() {
        for y in ys.iter() {
            let pos = Pos { x: *x, y: *y };
            state.crack_point(&pos, &param.p_test_power, interactor);
        }
    }
    let graph = Graph::new(&xs, &ys, &input);
    let mut annealing_state = AnnealingState::new(graph, state, param.c);

    // 繋ぐ辺を最適化する
    for _ in 0..100 {
        annealing_state.update(&param, &interactor);
    }

    // 辺の間を繋げる
    let mut edges = vec![];
    for edge_path in &mut annealing_state.to_source_paths {
        edges.append(edge_path);
    }

    let mut state = annealing_state.state;
    let graph = annealing_state.graph;
    create_path(&edges, &graph, &mut state, &param, interactor);
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
    points: Vec<Pos>,
    edges: Vec<Edge>,
    adj: Vec<Vec<usize>>,
    pos_index: HashMap<Pos, usize>,
    house: Vec<usize>,
    source: Vec<usize>,
}

impl Graph {
    fn new(xs: &Vec<i64>, ys: &Vec<i64>, input: &Input) -> Graph {
        let mut points = vec![];
        let mut pos_index = HashMap::new();
        for x in xs.iter() {
            for y in ys.iter() {
                let pos = Pos { x: *x, y: *y };
                pos_index.insert(pos, points.len());
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
                let ui = pos_index.get(&u).unwrap();
                let vi = pos_index.get(&v).unwrap();
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
                let ui = pos_index.get(&u).unwrap();
                let vi = pos_index.get(&v).unwrap();
                adj[*ui].push(edges.len());
                adj[*vi].push(edges.len());
                edges.push(Edge { u: *ui, v: *vi });
            }
        }

        let mut graph = Graph {
            points,
            edges,
            adj,
            pos_index,
            house: vec![],
            source: vec![],
        };

        for h in input.house.iter() {
            graph.house.push(graph.pos_to_index(h));
        }
        for src in input.source.iter() {
            graph.source.push(graph.pos_to_index(src));
        }

        graph
    }

    fn pos_to_index(&self, pos: &Pos) -> usize {
        debug_assert!(self.pos_index.contains_key(pos));
        *self.pos_index.get(pos).unwrap()
    }
}

struct AnnealingState {
    edge_used: Vec<i64>,
    to_source_paths: Vec<Vec<usize>>,
    state: State,
    graph: Graph,
    score: i64,
}

impl AnnealingState {
    fn new(graph: Graph, state: State, c: i64) -> AnnealingState {
        let mut annealing_state = AnnealingState {
            edge_used: vec![],
            to_source_paths: vec![],
            state,
            graph,
            score: 0,
        };
        annealing_state.recalculate_all(c);
        annealing_state
    }

    fn find_path_to_source(&self, point_idx: usize, c: i64) -> Vec<usize> {
        let (dist, par_edge) = self.dijkstra(point_idx, c);

        let mut best_source_index = NA;
        // 一番繋げるまでの距離が近い水源を探す
        for src_idx in self.graph.source.iter() {
            if best_source_index == NA || dist[*src_idx] < dist[best_source_index] {
                best_source_index = *src_idx;
            }
        }

        // h -> best_point_indexまでに通る辺を復元する
        let mut cur = best_source_index;
        let mut edge_path = vec![];
        while cur != point_idx {
            let par_edge_index = par_edge[cur];
            edge_path.push(par_edge_index);
            cur = self.graph.edges[par_edge_index].other_point(cur);
        }
        edge_path.reverse();

        edge_path
    }

    fn recalculate_all(&mut self, c: i64) {
        self.edge_used = vec![0; self.graph.edges.len()];
        self.to_source_paths = vec![vec![]; self.graph.house.len()];
        self.score = 0;
        let house_count = self.graph.house.len();
        for i in 0..house_count {
            let h_idx = self.graph.house[i];
            let edge_path = self.find_path_to_source(h_idx, c);
            self.set_edge_path(i, edge_path, c);
        }
    }

    fn set_edge_path(&mut self, h_idx: usize, edge_path: Vec<usize>, c: i64) {
        for edge_index in edge_path.iter() {
            if self.edge_used[*edge_index] == 0 {
                self.score += self.edge_weight(*edge_index, c);
            }
            self.edge_used[*edge_index] += 1;
        }
        self.to_source_paths[h_idx] = edge_path;
    }

    #[allow(unused)]
    fn remove_edge_path(&mut self, h_idx: usize, graph: &Graph, state: &State, param: &Param) {
        for edge_index in self.to_source_paths[h_idx].iter() {
            if self.edge_used[*edge_index] == 1 {
                self.score -= self.edge_weight(*edge_index, param.c);
            }
            self.edge_used[*edge_index] -= 1;
        }
    }

    fn dijkstra(&self, start: usize, c: i64) -> (Vec<i64>, Vec<usize>) {
        let mut dist = vec![INF; self.graph.points.len()];
        let mut par_edge = vec![NA; self.graph.points.len()];

        // hから各頂点までの距離を計算する
        let mut heap = BinaryHeap::new();
        dist[start] = 0;
        heap.push((Reverse(0), start));

        // TODO: 枝刈り
        while let Some((Reverse(d), v)) = heap.pop() {
            if dist[v] < d {
                continue;
            }
            for edge_index in self.graph.adj[v].iter() {
                let weight = if self.edge_used[*edge_index] == 0 {
                    // 使われていない辺なら、重みはgraph.edge_weight
                    self.edge_weight(*edge_index, c)
                } else {
                    // すでに使われている辺なら、重みは0
                    0
                };
                let u = self.graph.edges[*edge_index].other_point(v);
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

    fn edge_weight(&self, edge_index: usize, c: i64) -> i64 {
        let estimated_hardness = |v: usize| -> i64 {
            let p = &self.graph.points[v];
            if self.state.is_broken.get(p) {
                self.state.damage.get(p)
            } else {
                // まだ壊れていなかったら、2倍の強度を想定する
                self.state.damage.get(p) * 2 // :param
            }
        };
        let edge = &self.graph.edges[edge_index];
        let dist = self.graph.points[edge.u].dist(&self.graph.points[edge.v]);
        let hard_mean = (estimated_hardness(edge.u) + estimated_hardness(edge.v)) / 2;
        (hard_mean + c) * dist
    }

    fn update(&mut self, param: &Param, interactor: &Interactor) {
        for (i, edge) in self.graph.edges.iter().enumerate() {
            if self.edge_used[i] == 0 {
                continue;
            }
            for v in [edge.u, edge.v] {
                self.state
                    .crack_point(&self.graph.points[v], &param.p_test_power2, interactor);
            }
        }
        self.recalculate_all(param.c);
    }
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

    rnd::shuffle(&mut cells);

    loop {
        while let Some(cell) = cells.last() {
            if state.is_broken.get(cell) {
                cells.pop();
            } else {
                break;
            }
        }
        if let Some(cell) = cells.last() {
            // 周囲の壊れているセルから硬さを予測する
            let mut sum = 0.;
            let mut div = 0.;
            for dx in -5..=5 {
                for dy in -5..=5 {
                    let x = dx + cell.x;
                    let y = dy + cell.y;
                    if x < 0 || y < 0 || x >= 200 || y >= 200 {
                        continue;
                    }
                    let p = Pos { x, y };
                    if !state.is_broken.get(&p) {
                        continue;
                    }
                    let w = 1. / p.dist(cell) as f64;
                    sum += state.damage.get(&p) as f64 * w;
                    div += w;
                }
            }

            let power = if div == 0. {
                i64::max(20, param.c)
            } else {
                let mean = sum / div;
                let estimated = (mean * 0.75) as i64;
                let to_estimated = i64::min(5000, estimated - state.damage.get(&cell));
                if to_estimated <= 0 {
                    i64::max(20, param.c)
                } else {
                    to_estimated
                }
            };

            interactor.respond(cell, power, state);
        } else {
            break;
        }
    }
}

//
// ユーティリティ
//

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
