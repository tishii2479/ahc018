use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
    fs::File,
    io::Write,
};

use crate::{
    def::*,
    interactor::*,
    param::*,
    util::{rnd, UnionFind},
};

fn add_point(
    pos: &Pos,
    state: &mut State,
    graph: &mut Graph,
    param: &Param,
    interactor: &Interactor,
) -> bool {
    if !pos.is_valid() || !graph.should_add_point(&pos) {
        return false;
    }
    state.crack_point(&pos, &param.p_test_power, interactor);
    graph.add_point(&pos);
    return true;
}

pub fn solve(input: &Input, interactor: &Interactor, param: &Param) {
    let mut state = State::new(N);
    let mut graph = Graph::new();
    for x in (param.p_grid_size / 2..N).step_by(param.p_grid_size) {
        for y in (param.p_grid_size / 2..N).step_by(param.p_grid_size) {
            let pos = Pos {
                x: x as i64,
                y: y as i64,
            };
            add_point(&pos, &mut state, &mut graph, param, interactor);
        }
    }
    let mut annealing_state = AnnealingState::new(graph, state, input);
    annealing_state.recalculate_all(param.c);

    // 繋ぐ辺を最適化する
    for t in 0..100 {
        annealing_state.update(&param, interactor, t);
    }

    println!("# end optimize");
    eprintln!("used power: {}", annealing_state.state.total_damage);
    annealing_state.output_graph(param.c);

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

impl PartialEq for Edge {
    fn eq(&self, other: &Self) -> bool {
        (self.u == other.u && self.v == other.v) || (self.u == other.v && self.v == other.u)
    }
}

struct Graph {
    points: Vec<Pos>,
    edges: Vec<Edge>,
    adj: Vec<Vec<usize>>,
    pos_index: HashMap<Pos, usize>,
}

impl Graph {
    fn new() -> Graph {
        Graph {
            points: vec![],
            edges: vec![],
            adj: vec![],
            pos_index: HashMap::new(),
        }
    }

    fn add_point(&mut self, pos: &Pos) -> usize {
        let p = pos.clone();
        let p_idx = self.points.len();

        self.points.push(p);
        self.pos_index.insert(p, p_idx);

        // 点のインデックスを返す
        p_idx
    }

    fn recalculate_edges(&mut self) {
        self.edges = vec![];
        self.adj = vec![vec![]; self.points.len()];
        for (i, s) in self.points.iter().enumerate() {
            let mut near_pos = vec![i];

            // 近くの頂点を列挙
            for (j, p) in self.points.iter().enumerate() {
                if i == j {
                    continue;
                }
                if s.dist(p) <= 30 {
                    near_pos.push(j);
                }
            }

            let mut dist = vec![];

            for u in near_pos.iter() {
                for v in near_pos.iter() {
                    if u == v {
                        continue;
                    }
                    let (pu, pv) = (self.points[*u], self.points[*v]);
                    dist.push((pu.dist(&pv), *u, *v));
                }
            }

            dist.sort_by(|a, b| a.0.cmp(&b.0));

            let mut uf = UnionFind::new(near_pos.len());
            let mut mp = HashMap::new();
            for (i, v) in near_pos.iter().enumerate() {
                mp.insert(v, i);
            }

            for (_, u, v) in dist.iter() {
                if uf.same(mp[u], mp[v]) {
                    continue;
                }
                uf.unite(mp[u], mp[v]);

                let edge = Edge { u: *u, v: *v };
                // すでに追加されている辺なら追加しない
                if self.edges.contains(&edge) {
                    continue;
                }
                let edge_index = self.edges.len();
                self.adj[*u].push(edge_index);
                self.adj[*v].push(edge_index);
                self.edges.push(edge);
            }
        }
    }

    fn dijkstra<T>(&self, start: usize, edge_weight: T) -> (Vec<i64>, Vec<usize>)
    where
        T: Fn(usize) -> i64,
    {
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
                let weight = edge_weight(*edge_index);
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

    fn should_add_point(&self, pos: &Pos) -> bool {
        // 距離が一定以下の場所には点を打たなくて良い
        for p in self.points.iter() {
            if p.dist(pos) < 5 {
                return false;
            }
        }
        return true;
    }

    #[allow(unused)]
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
    house: Vec<usize>,
    source: Vec<usize>,
    score: i64,
}

impl AnnealingState {
    fn new(graph: Graph, state: State, input: &Input) -> AnnealingState {
        let mut house = vec![];
        let mut graph = graph;
        for h in input.house.iter() {
            let h_idx = graph.add_point(&h);
            house.push(h_idx);
        }
        let mut source = vec![];
        for src in input.source.iter() {
            let src_idx = graph.add_point(&src);
            source.push(src_idx);
        }
        AnnealingState {
            edge_used: vec![],
            to_source_paths: vec![],
            state,
            graph,
            house,
            source,
            score: 0,
        }
    }

    fn find_path_to_source(&self, point_idx: usize, c: i64) -> Vec<usize> {
        let edge_weight = {
            |edge_index: usize| {
                if self.edge_used[edge_index] == 0 {
                    // 使われていない辺なら、重みはgraph.edge_weight
                    self.edge_weight(edge_index, c)
                } else {
                    // すでに使われている辺なら、重みは0
                    0
                }
            }
        };
        let (dist, par_edge) = self.graph.dijkstra(point_idx, edge_weight);

        let mut best_source_index = NA;
        // 一番繋げるまでの距離が近い水源を探す
        for src_idx in self.source.iter() {
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
        self.graph.recalculate_edges();

        self.edge_used = vec![0; self.graph.edges.len()];
        self.to_source_paths = vec![vec![]; self.house.len()];
        self.score = 0;
        let house_count = self.house.len();
        for i in 0..house_count {
            let h_idx = self.house[i];
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

    fn edge_weight(&self, edge_index: usize, c: i64) -> i64 {
        let estimated_hardness = |v: usize| -> i64 {
            let p = &self.graph.points[v];
            if self.state.is_broken.get(p) {
                self.state.damage.get(p)
            } else {
                // まだ壊れていなかったら、5倍の強度を想定する
                i64::min(S_MAX, self.state.damage.get(p) * 5) // :param
            }
        };
        let edge = &self.graph.edges[edge_index];
        let dist = self.graph.points[edge.u].dist(&self.graph.points[edge.v]);
        let hard_mean = (estimated_hardness(edge.u) + estimated_hardness(edge.v)) / 2;
        (hard_mean + c) * dist
    }

    fn update(&mut self, param: &Param, interactor: &Interactor, _iteration: usize) {
        let mut add_pos = vec![];
        for (i, edge) in self.graph.edges.iter().enumerate() {
            if self.edge_used[i] == 0 {
                continue;
            }
            for v in [edge.u, edge.v] {
                let p = &self.graph.points[v];
                self.state.crack_point(&p, &param.p_test_power2, interactor);
            }
            let (u, v) = (self.graph.points[edge.u], self.graph.points[edge.v]);
            let c = Pos {
                x: (u.x + v.x) / 2,
                y: (u.y + v.y) / 2,
            };
            add_pos.push(c);
        }
        if _iteration > 10 {
            for p in add_pos.iter() {
                add_point(p, &mut self.state, &mut self.graph, param, interactor);
            }
        }
        self.recalculate_all(param.c);
    }

    fn output_graph(&self, c: i64) {
        let mut file = File::create("log/graph.txt").unwrap();
        writeln!(
            file,
            "{} {}",
            self.graph.points.len(),
            self.graph.edges.len()
        )
        .unwrap();

        for p in self.graph.points.iter() {
            writeln!(file, "{} {}", p.y, p.x).unwrap();
        }
        for (i, e) in self.graph.edges.iter().enumerate() {
            writeln!(file, "{} {} {}", e.u, e.v, self.edge_weight(i, c)).unwrap();
        }
        for path in self.to_source_paths.iter() {
            for e in path.iter() {
                write!(file, "{} ", e).unwrap();
            }
            writeln!(file).unwrap();
        }
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
            if i64::abs(p.y - h.y) > i64::abs(p.x - h.x) {
                if p.y < h.y {
                    p.y += 1;
                } else if p.y > h.y {
                    p.y -= 1;
                }
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
                    let p = Pos {
                        x: dx + cell.x,
                        y: dy + cell.y,
                    };
                    if !p.is_valid() {
                        continue;
                    }
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
                let to_estimated = i64::min(S_MAX, estimated - state.damage.get(&cell));
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
