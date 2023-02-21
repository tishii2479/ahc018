pub mod def {
    use crate::interactor::Interactor;

    pub const INF: i64 = 100_000_000_000_000;
    pub const NA: usize = 100_000_000_000_000;
    pub const N: usize = 200;
    pub const S_MAX: i64 = 5000;

    pub struct Input {
        pub n: usize,
        pub w: usize,
        pub k: usize,
        pub c: i64,
        pub source: Vec<Pos>,
        pub house: Vec<Pos>,
    }

    pub struct State {
        pub is_broken: Vec2dBool,
        pub damage: Vec2d,
        pub total_damage: i64,
    }

    impl State {
        pub fn new(n: usize) -> State {
            State {
                is_broken: Vec2dBool::new(n, n),
                damage: Vec2d::new(n, n),
                total_damage: 0,
            }
        }

        pub fn crack_point(
            &mut self,
            pos: &Pos,
            test_power: &Vec<i64>,
            interactor: &mut Interactor,
        ) {
            for test_power in test_power.iter() {
                if self.is_broken.get(pos) {
                    break;
                }
                let power = test_power - self.damage.get(pos);
                if power <= 0 {
                    break;
                }
                self.total_damage += power;
                interactor.respond(pos, power, self);
            }
        }
    }

    #[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
    pub struct Pos {
        pub y: i64,
        pub x: i64,
    }

    impl Pos {
        pub fn dist(&self, to: &Pos) -> i64 {
            i64::abs(to.y - self.y) + i64::abs(to.x - self.x)
        }

        pub fn is_valid(&self) -> bool {
            self.x >= 0 && self.y >= 0 && self.x < 200 && self.y < 200
        }
    }

    #[allow(unused)]
    pub struct Vec2d {
        vec: Vec<i64>,
        n: usize,
        m: usize,
    }

    #[allow(unused)]
    impl Vec2d {
        pub fn new(n: usize, m: usize) -> Vec2d {
            Vec2d {
                vec: vec![0; n * m],
                n,
                m,
            }
        }

        pub fn get(&self, pos: &Pos) -> i64 {
            self.vec[pos.y as usize * self.m + pos.x as usize]
        }

        pub fn set(&mut self, pos: &Pos, val: i64) {
            self.vec[pos.y as usize * self.m + pos.x as usize] = val
        }

        pub fn add(&mut self, pos: &Pos, val: i64) {
            self.vec[pos.y as usize * self.m + pos.x as usize] += val
        }
    }

    #[allow(unused)]
    pub struct Vec2dBool {
        vec: Vec<bool>,
        n: usize,
        m: usize,
    }

    #[allow(unused)]
    impl Vec2dBool {
        pub fn new(n: usize, m: usize) -> Vec2dBool {
            Vec2dBool {
                vec: vec![false; n * m],
                n,
                m,
            }
        }

        pub fn get(&self, pos: &Pos) -> bool {
            self.vec[pos.y as usize * self.m + pos.x as usize]
        }

        pub fn set(&mut self, pos: &Pos, val: bool) {
            self.vec[pos.y as usize * self.m + pos.x as usize] = val
        }
    }
}
pub mod interactor {
    use proconio::*;

    use crate::{def::*, util::time};
    use std::io::{Stdin, Write};

    pub struct Interactor {
        source: proconio::source::line::LineSource<std::io::BufReader<Stdin>>,
    }

    impl Interactor {
        pub fn new() -> Interactor {
            Interactor {
                source: proconio::source::line::LineSource::new(std::io::BufReader::new(
                    std::io::stdin(),
                )),
            }
        }

        pub fn read_input(&mut self) -> Input {
            input! {
                from &mut self.source,
                n: usize,
                w: usize,
                k: usize,
                c: i64,
            }
            let mut source = vec![];
            for _ in 0..w {
                input! {
                    from &mut self.source,
                    y: i64,
                    x: i64
                }
                source.push(Pos { y, x });
            }
            let mut house = vec![];
            for _ in 0..k {
                input! {
                    from &mut self.source,
                    y: i64,
                    x: i64
                }
                house.push(Pos { y, x });
            }
            Input {
                n,
                w,
                k,
                c,
                source,
                house,
            }
        }

        pub fn respond(&mut self, pos: &Pos, power: i64, state: &mut State) -> bool {
            println!("{} {} {}", pos.y, pos.x, power);
            std::io::stdout().flush().unwrap();
            state.damage.add(pos, power);
            input! {
                from &mut self.source,
                r: i64,
            }

            if r == 0 {
                return false;
            } else if r == 1 {
                state.is_broken.set(pos, true);
                return true;
            } else if r == 2 {
                // 終了する
                eprintln!("elapsed seconds: {:.4}", time::elapsed_seconds());
                std::process::exit(0);
            } else if r == -1 {
                panic!("Invalid operation");
            } else {
                panic!("Invalid result output");
            }
        }
    }
}
pub mod param {
    pub struct Param {
        pub c: i64,
        pub p_grid_size: usize,
        pub p_test_power: Vec<i64>,
        pub p_test_power2: Vec<i64>,
    }

    impl Param {
        pub fn new(c: i64) -> Param {
            Param {
                c,
                p_grid_size: 20,
                p_test_power: vec![20, 50, 80, 110],
                p_test_power2: vec![200, 300, 500, 700, 900],
            }
        }
    }
}
pub mod solver {
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

    fn add_point_if_needed(
        pos: &Pos,
        state: &mut State,
        graph: &mut Graph,
        param: &Param,
        interactor: &mut Interactor,
    ) -> bool {
        if !pos.is_valid() || !graph.should_add_point(&pos) {
            return false;
        }
        state.crack_point(&pos, &param.p_test_power, interactor);
        graph.add_point(&pos);
        return true;
    }

    pub fn solve(input: &Input, interactor: &mut Interactor, param: &Param) {
        let mut state = State::new(N);
        let mut graph = Graph::new();
        let (xs, ys) = create_grid_axis(input, param.p_grid_size as i64);
        for x in xs.iter() {
            for y in ys.iter() {
                let pos = Pos {
                    x: *x as i64,
                    y: *y as i64,
                };
                add_point_if_needed(&pos, &mut state, &mut graph, param, interactor);
            }
        }
        let mut annealing_state = AnnealingState::new(graph, state, input);
        annealing_state.recalculate_all(param.c);

        // 繋ぐ辺を最適化する
        for t in 0..100 {
            annealing_state.update(&param, interactor, t);
        }

        if cfg!(feature = "local") {
            println!("# end optimize");
            eprintln!("used power: {}", annealing_state.state.total_damage);
            annealing_state.output_graph(param.c);
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

            let mut add_edges = vec![];

            for (i, s) in self.points.iter().enumerate() {
                let mut near_pos = vec![i];

                // 近くの頂点を列挙
                for (j, p) in self.points.iter().enumerate() {
                    if i == j {
                        continue;
                    }
                    if s.dist(p) <= 40 {
                        near_pos.push(j);
                    }
                    if s.dist(p) <= 15 {
                        add_edges.push(Edge { u: i, v: j });
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
                    add_edges.push(Edge { u: *u, v: *v });
                }
            }

            for edge in add_edges {
                // すでに追加されている辺なら追加しない
                if self.edges.contains(&edge) {
                    continue;
                }
                let edge_index = self.edges.len();
                self.adj[edge.u].push(edge_index);
                self.adj[edge.v].push(edge_index);
                self.edges.push(edge);
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
        edge_used: Vec<bool>,
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
                    if self.edge_used[edge_index] {
                        // すでに使われている辺なら、重みは0
                        0
                    } else {
                        // 使われていない辺なら、重みはgraph.edge_weight
                        // 多様性を持たせるために、ランダムな値を足す
                        let w = self.edge_weight(edge_index, c);
                        w + rnd::gen_range(0, w as usize / 10 + 1) as i64
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

            self.edge_used = vec![false; self.graph.edges.len()];
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
                if !self.edge_used[*edge_index] {
                    self.score += self.edge_weight(*edge_index, c);
                }
                self.edge_used[*edge_index] = true;
            }
            self.to_source_paths[h_idx] = edge_path;
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

        fn update(&mut self, param: &Param, interactor: &mut Interactor, _iteration: usize) {
            let mut add_pos = vec![];
            for (i, edge) in self.graph.edges.iter().enumerate() {
                if !self.edge_used[i] {
                    continue;
                }
                for v in vec![edge.u, edge.v] {
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
                    add_point_if_needed(p, &mut self.state, &mut self.graph, param, interactor);
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
        interactor: &mut Interactor,
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

        let new_xs = create_equally_spaced_axis(&xs, p_grid_size, N);
        let new_ys = create_equally_spaced_axis(&ys, p_grid_size, N);

        (new_xs, new_ys)
    }
}
pub mod util {
    #[allow(unused_features)]

    pub mod rnd {
        #[allow(unused)]
        static mut S: usize = 88172645463325252;

        #[allow(unused)]
        #[inline]
        pub fn next() -> usize {
            unsafe {
                S = S ^ S << 7;
                S = S ^ S >> 9;
                S
            }
        }

        #[allow(unused)]
        #[inline]
        pub fn nextf() -> f64 {
            (next() & 4294967295) as f64 / 4294967296.
        }

        #[allow(unused)]
        #[inline]
        pub fn gen_range(low: usize, high: usize) -> usize {
            (next() % (high - low)) + low
        }

        #[allow(unused)]
        pub fn shuffle<I>(vec: &mut Vec<I>) {
            for i in 0..vec.len() {
                let j = gen_range(0, vec.len());
                vec.swap(i, j);
            }
        }
    }

    pub mod time {
        static mut START: f64 = -1.;
        #[allow(unused)]
        pub fn start_clock() {
            let _ = elapsed_seconds();
        }

        #[allow(unused)]
        #[inline]
        pub fn elapsed_seconds() -> f64 {
            let t = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64();
            unsafe {
                if START < 0. {
                    START = t;
                }
                t - START
            }
        }
    }

    #[allow(unused)]
    pub fn min_index<I>(vec: &Vec<I>) -> usize
    where
        I: Ord,
    {
        let mut ret = 0;
        for i in 0..vec.len() {
            if vec[i] < vec[ret] {
                ret = i;
            }
        }
        return ret;
    }

    pub struct UnionFind {
        par: Vec<usize>,
        size: Vec<usize>,
    }

    impl UnionFind {
        pub fn new(n: usize) -> Self {
            UnionFind {
                par: (0..n).into_iter().collect(),
                size: vec![1; n],
            }
        }

        pub fn find(&mut self, x: usize) -> usize {
            if self.par[x] == x {
                x
            } else {
                self.par[x] = self.find(self.par[x]);
                self.par[x]
            }
        }

        pub fn unite(&mut self, x: usize, y: usize) {
            let mut x = self.find(x);
            let mut y = self.find(y);
            if self.size[x] < self.size[y] {
                ::std::mem::swap(&mut x, &mut y);
            }
            if x != y {
                self.size[x] += self.size[y];
                self.par[y] = x;
            }
        }

        pub fn same(&mut self, x: usize, y: usize) -> bool {
            self.find(x) == self.find(y)
        }
    }
}

use param::*;
use util::*;

fn main() {
    time::start_clock();

    let mut interactor = interactor::Interactor::new();
    let input = interactor.read_input();
    let param = Param::new(input.c);

    solver::solve(&input, &mut interactor, &param);
}
