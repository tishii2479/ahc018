use std::{cmp::Reverse, collections::BinaryHeap};

use crate::{
    def::*,
    grid::*,
    interactor::*,
    util::{rnd, time},
};

struct Change {
    p: Pos,
    prev: bool,
}

pub struct Solver {
    input: Input,
    state: State,
    interactor: Interactor,
}

fn pos_to_grid(y: i64, x: i64) -> Pos {
    let y = if y == N as i64 { y - 1 } else { y };
    let x = if x == N as i64 { x - 1 } else { x };
    Pos { y, x }
}

fn add_damage_to_hardness_if_needed(
    p: &Pos,
    hardness: i64,
    state: &mut State,
    interactor: &mut Interactor,
) -> bool {
    if state.is_broken.get(&p) {
        return false;
    }
    let power = i64::min(S_MAX, hardness) - state.damage.get(&p);
    if power <= 0 {
        return false;
    }
    interactor.add_damage(&p, power, state);
    return true;
}

impl Solver {
    pub fn new() -> Solver {
        let mut interactor = Interactor::new();
        let input = interactor.read_input();
        let state = State::new(input.n);

        Solver {
            input,
            state,
            interactor,
        }
    }

    pub fn solve(&mut self) {
        // 家の地点の頑丈度を調べる
        for h_pos in self.input.house.iter() {
            for h in vec![13, 25, 50, 100, 200, 400, 800, 1600, 3200, 5000] {
                add_damage_to_hardness_if_needed(h_pos, h, &mut self.state, &mut self.interactor);
            }
        }
        let mut sinked_pos = vec![];
        for s_pos in self.input.source.iter() {
            for h in vec![13, 25, 50, 100, 200, 400, 800, 1600, 3200, 5000] {
                add_damage_to_hardness_if_needed(s_pos, h, &mut self.state, &mut self.interactor);
            }
            sinked_pos.push(s_pos.clone());
        }
        let mut log_count = 0;

        // 上限を決めたdfsにより、家から水源までのルートを確保する
        // TODO: 水源までの距離が近い家から探索する
        let house_count = self.input.house.len();
        let mut done = vec![false; house_count];
        let interval = 10; // :param
        for upper in vec![
            100, 200, 300, 400, 500, 600, 700, 800, 900, // :param
        ] {
            for i in 0..house_count {
                if done[i] {
                    continue;
                }
                let h_pos = self.input.house[i];
                if let Some(path_to_source) =
                    self.search(&h_pos, upper, self.input.c, interval, &sinked_pos)
                {
                    // 使用する経路上の点をsinked_posに追加する
                    for p in path_to_source {
                        sinked_pos.push(p);
                    }
                    eprintln!("ok: {:?} {}", h_pos, upper);
                    done[i] = true;
                }
            }
            let estimated_grid = self.generate_estimated_grid(interval);
            estimated_grid.output_grid(format!("log/grid_{}.txt", log_count).as_str());
            self.state
                .output_state(format!("log/state_{}.txt", log_count).as_str());
            log_count += 1;
        }

        let mut estimated_grid = self.generate_estimated_grid(interval);
        self.optimize_route(&mut estimated_grid);
        estimated_grid.output_grid(format!("log/grid_{}.txt", log_count).as_str());
        self.state
            .output_state(format!("log/state_{}.txt", log_count).as_str());

        if cfg!(feature = "local") {
            println!("# end optimize");
            eprintln!(
                "total power before destroy_used_path: {}, total crack cost: {}",
                self.state.total_damage,
                self.state.total_crack * self.input.c,
            );
        }

        // 選択経路に使われている地点を割る
        self.destroy_used_path(&estimated_grid, interval);
    }

    fn search(
        &mut self,
        start: &Pos,
        upper: i64,
        c: i64,
        interval: i64,
        sinked_pos: &Vec<Pos>,
    ) -> Option<Vec<Pos>> {
        fn to_nearest_sinked_dist(pos: &Pos, sinked_pos: &Vec<Pos>) -> i64 {
            let mut val = INF;
            for sp in sinked_pos.iter() {
                let d = sp.manhattan_dist(pos);
                if d < val {
                    val = d;
                }
            }
            val
        }

        // 評価関数
        fn eval(pos: &Pos, consumed: i64, upper: i64, c: i64, sinked_pos: &Vec<Pos>) -> i64 {
            let dist = to_nearest_sinked_dist(pos, sinked_pos);
            dist * (upper + c * 2) + consumed
        }

        // すでに開拓されている近くの点を探す
        // なければ元の点を返す
        fn find_near_pos(pos: Pos, state: &mut State, interval: i64) -> Pos {
            for dy in -interval + 1..interval {
                for dx in -interval + 1..interval {
                    let p = Pos {
                        y: pos.y + dy,
                        x: pos.x + dx,
                    };
                    if !p.is_valid() {
                        continue;
                    }
                    if state.damage.get(&p) > 0 {
                        return p;
                    }
                }
            }
            pos
        }

        let mut dist = Vec2d::new(N, N, INF);
        let mut par: Vec2d<Option<Pos>> = Vec2d::new(N, N, None);
        let mut heap = BinaryHeap::new();
        dist.set(start, 0);
        heap.push((Reverse(INF), start.clone()));

        let mut best_eval = INF;
        let mut best_sinked_pos = None;
        while let Some((Reverse(_), pos)) = heap.pop() {
            // upperまで掘削し、壊れたら追加する
            // 20刻みとかで調べる
            // 前回の結果を反映する（posの硬さを反映する）
            let parent = par.get(&pos).unwrap_or(pos);
            let start = i64::max(10, (self.state.damage.get(&parent) as f64 * 0.5) as i64); // :param
            let end = i64::min(
                upper,
                i64::max(upper, (self.state.damage.get(&pos) as f64 * 2.) as i64),
            ); // :param
            let step = i64::max(i64::min(20, self.input.c * 2), (end - start) / 2) as usize; // :param
            for p in (start..end).step_by(step) {
                add_damage_to_hardness_if_needed(&pos, p, &mut self.state, &mut self.interactor);
            }
            if !self.state.is_broken.get(&pos) {
                continue;
            }
            let hard_mean = (self.state.damage.get(&parent) + self.state.damage.get(&pos)) / 2;
            let d = (hard_mean + c) * parent.manhattan_dist(&pos);
            dist.set(&pos, d);

            if d > best_eval {
                continue;
            }

            best_eval = i64::min(best_eval, eval(&pos, d, upper, c, sinked_pos));

            if sinked_pos.contains(&pos) {
                if d < best_eval {
                    best_eval = d;
                    best_sinked_pos = Some(pos.clone());
                }
                continue;
            }

            for (dy, dx) in DELTA {
                let next_pos = Pos {
                    y: pos.y + dy * interval,
                    x: pos.x + dx * interval,
                };
                // next_posの近くに使える点があれば、それを使う
                let next_pos = find_near_pos(next_pos, &mut self.state, interval);
                if !next_pos.is_valid() {
                    continue;
                }
                let hard_mean = (self.state.damage.get(&pos) + upper) / 2;
                let consumed = d + pos.manhattan_dist(&next_pos) * (hard_mean + c);
                let next_dist = eval(&next_pos, consumed, upper, c, sinked_pos);
                if next_dist < dist.get(&next_pos) {
                    dist.set(&next_pos, next_dist);
                    par.set(&next_pos, Some(pos.clone()));
                    heap.push((Reverse(next_dist), next_pos));
                }
            }
        }

        if let Some(cur) = best_sinked_pos {
            let mut path = vec![];
            let mut cur = cur;
            // 最も評価値が良い水が通っているところから、
            // startまでparを辿って道を作る
            while let Some(parent) = par.get(&cur) {
                path.push(parent);
                cur = parent;
            }
            Some(path)
        } else {
            None
        }
    }

    fn generate_route(&self, estimated_grid: &mut Grid) {
        for h_pos in self.input.house.iter() {
            let (nearest_source_path, _) = estimated_grid.find_path_to_nearest_source(
                &h_pos,
                INF,
                &self.input.source,
                self.input.c,
            );
            for p in nearest_source_path.iter() {
                estimated_grid.set(p, true);
            }
        }
    }

    fn optimize_route(&self, estimated_grid: &mut Grid) {
        // 初期解の作成
        self.generate_route(estimated_grid);

        let mut current_score = estimated_grid.total_score;

        // 山登りによる最適化
        while time::elapsed_seconds() < 4. {
            // ランダムな家から接続している水源までのパスを消す
            let h_pos = &self.input.house[rnd::gen_range(0, self.input.house.len())];
            let (path_to_source, _) = estimated_grid.find_current_path_to_source(&h_pos).unwrap();

            let mut changes = vec![];
            for p in path_to_source.iter() {
                changes.push(Change {
                    p: p.clone(),
                    prev: estimated_grid.is_used.get(p),
                });
                estimated_grid.set(p, false);
            }

            // 水源に接続されなくなった家を再度接続する
            // TODO: 経路を消した家を最初に再接続する
            // FIXME: 接続されなくなった家から出ている残りを消す
            let mut reconnect_houses = estimated_grid.find_unconnected_houses();
            rnd::shuffle(&mut reconnect_houses);

            for i in reconnect_houses.iter() {
                let (nearest_source_path, _) = estimated_grid.find_path_to_nearest_source(
                    &self.input.house[*i],
                    INF,
                    &self.input.source,
                    self.input.c,
                );
                for p in nearest_source_path.iter() {
                    changes.push(Change {
                        p: p.clone(),
                        prev: estimated_grid.is_used.get(p),
                    });
                    estimated_grid.set(p, true);
                }
            }

            let new_score = estimated_grid.total_score;

            if new_score < current_score {
                // 採用
                eprintln!("{} -> {}", current_score, new_score);
                current_score = new_score;
            } else {
                // ロールバック
                changes.reverse();
                for c in changes.iter() {
                    estimated_grid.set(&c.p, c.prev);
                }
            }
        }
    }

    fn destroy_used_path(&mut self, estimated_grid: &Grid, interval: i64) {
        for y in 0..N as i64 {
            for x in 0..N as i64 {
                let p = Pos { y, x };
                // FIXME: is_usedに常にhouse、sourceが含まれるように修正
                if !estimated_grid.is_used.get(&p)
                    && !self.input.house.contains(&p)
                    && !self.input.source.contains(&p)
                {
                    continue;
                }
                let mut estimated_hardness = i64::max(
                    10,
                    (self.estimate_hardness(&p, interval) as f64 * 0.8) as i64,
                );
                while !self.state.is_broken.get(&p) {
                    add_damage_to_hardness_if_needed(
                        &p,
                        estimated_hardness,
                        &mut self.state,
                        &mut self.interactor,
                    );
                    // param:
                    estimated_hardness = i64::min(S_MAX, (estimated_hardness as f64 * 1.2) as i64);
                }
            }
        }
    }

    fn generate_estimated_grid(&self, interval: i64) -> Grid {
        // TODO: is_usedにhouseとsourceの位置を追加
        let mut estimated_weight = Vec2d::new(N, N, 0);
        for y in 0..N as i64 {
            for x in 0..N as i64 {
                let p = pos_to_grid(y, x);
                let w = self.estimate_hardness(&p, interval) - self.state.damage.get(&p);
                estimated_weight.set(&p, i64::max(0, w));
            }
        }
        let mut is_used = Vec2d::new(N, N, false);
        for p in self.input.house.iter() {
            is_used.set(&p, true);
        }
        for p in self.input.source.iter() {
            is_used.set(&p, true);
        }

        Grid {
            total_score: 0,
            estimated_weight,
            is_used,
            house: self.input.house.clone(),
            source: self.input.source.clone(),
        }
    }

    fn estimate_hardness(&self, pos: &Pos, interval: i64) -> i64 {
        if self.state.is_broken.get(pos) {
            return self.fetch_investigated_hardness(pos).unwrap();
        }

        let mut sum = 0.;
        let mut div = 0.;

        for dy in -interval..=interval {
            for dx in -interval..=interval {
                let p = Pos {
                    y: pos.y + dy,
                    x: pos.x + dx,
                };
                if !p.is_valid() {
                    continue;
                }
                let d = pos.euclid_dist(&p);
                if d > 20. {
                    continue;
                }
                if let Some(h) = self.fetch_investigated_hardness(&p) {
                    let w = 1. / (1. + d);
                    sum += h as f64 * w;
                    div += w;
                }
            }
        }

        if sum == 0. {
            5000
        } else {
            (sum / div) as i64
        }
    }

    fn fetch_investigated_hardness(&self, p: &Pos) -> Option<i64> {
        // まだ壊れていない場合
        if !self.state.is_broken.get(p) {
            return None;
        }

        let damage_before_break = self.state.damage_before_break.get(p);
        let damage = self.state.damage.get(p);

        let a = if damage_before_break == 0 {
            (damage as f64 * 0.5) as i64
        } else {
            (damage_before_break + damage) / 2
        };
        Some(i64::max(10, a))
    }
}
