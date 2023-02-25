use std::vec;

use crate::{def::*, grid::*, interactor::*, util::rnd};

struct Change {
    p: Pos,
    prev: bool,
}

pub struct Solver {
    input: Input,
    state: State,
    interactor: Interactor,
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
        static mut log_i: usize = 0;
        fn log(grid: &Grid, state: &State) {
            unsafe {
                if cfg!(feature = "local") {
                    grid.output_grid(format!("log/grid_{}.txt", log_i).as_str());
                    state.output_state(format!("log/state_{}.txt", log_i).as_str());
                }
                log_i += 1;
            }
        }

        fn to_near_pos(pos: Pos, state: &State, interval: i64) -> Pos {
            // :param
            for dy in -interval / 2..=interval / 2 {
                for dx in -interval / 2..=interval / 2 {
                    let p = Pos {
                        y: pos.y + dy,
                        x: pos.x + dx,
                    };
                    if !p.is_valid() {
                        continue;
                    }
                    if pos.euclid_dist(&p) >= interval as f64 / 2. {
                        continue;
                    }
                    if state.damage.get(&p) > 0 {
                        return p;
                    }
                }
            }
            pos
        }

        let mut best = vec![INF; self.input.house.len()];

        for i in 0..8 {
            let interval = 20 - i * 2;
            for (j, upper_p) in (500..=500 * (i + 1)).step_by(500).enumerate() {
                // 頑丈度を予測したグリッドを作成する
                let mut estimated_grid = self.generate_estimated_grid(false);

                let house_count = self.input.house.len();
                // 選択経路の周りを探索する
                for h_idx in 0..house_count {
                    // 枝刈り
                    if best[h_idx] < upper_p {
                        continue;
                    }
                    let (mut nearest_source_path, _) = estimated_grid.find_path_to_nearest_source(
                        &self.input.house[h_idx],
                        INF,
                        &self.input.source,
                        self.input.c,
                    );
                    nearest_source_path.reverse();
                    let mut ok = true;
                    for p in nearest_source_path {
                        // 近くの点を探す
                        estimated_grid.set(&p, true);

                        if best[h_idx] < upper_p || !ok {
                            continue;
                        }

                        let p = to_near_pos(p, &self.state, interval);

                        // upper_pまで硬さを調べる
                        let mut h = 30;
                        while h < upper_p {
                            // :param
                            add_damage_to_hardness_if_needed(
                                &p,
                                h,
                                &mut self.state,
                                &mut self.interactor,
                            );
                            h = h * 3 / 2;
                        }
                        add_damage_to_hardness_if_needed(
                            &p,
                            upper_p,
                            &mut self.state,
                            &mut self.interactor,
                        );
                        if !self.state.is_broken.get(&p) {
                            ok = false;
                        }
                    }
                    if ok && upper_p < best[h_idx] {
                        eprintln!("ok: {:?} at: {}, {}", &self.input.house[h_idx], upper_p, j);
                        best[h_idx] = upper_p;
                    }
                }
                log(&estimated_grid, &self.state);
            }
        }

        let mut estimated_grid = self.generate_estimated_grid(true);
        self.optimize_route(&mut estimated_grid);

        log(&estimated_grid, &self.state);

        if cfg!(feature = "local") {
            println!("# end optimize");
        }

        eprintln!(
            "total power before destroy_used_path: {} {}",
            self.state.total_damage,
            self.state.total_crack * self.input.c
        );

        // 選択経路に使われている地点を割る
        self.destroy_used_path(&estimated_grid);
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
        for t in 0..100 {
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
                eprintln!("{} -> {}, at: {}", current_score, new_score, t);
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

    fn destroy_used_path(&mut self, estimated_grid: &Grid) {
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
                // :param
                let mut estimated_hardness =
                    i64::max(10, (self.estimate_hardness(&p) as f64 * 0.8) as i64);
                while !self.state.is_broken.get(&p) {
                    add_damage_to_hardness_if_needed(
                        &p,
                        estimated_hardness,
                        &mut self.state,
                        &mut self.interactor,
                    );
                    // :param
                    estimated_hardness = i64::min(S_MAX, (estimated_hardness as f64 * 1.2) as i64);
                }
            }
        }
    }

    fn generate_estimated_grid(&self, is_final: bool) -> Grid {
        let mut estimated_weight = Vec2d::new(N, N, 0);
        for y in 0..N as i64 {
            for x in 0..N as i64 {
                let p = Pos { y, x };
                let w = {
                    let h = if is_final {
                        self.estimate_hardness_for_crack(&p)
                    } else {
                        self.estimate_hardness(&p)
                    };
                    if is_final {
                        i64::max(0, h - self.state.damage.get(&p))
                    } else {
                        h
                    }
                };
                estimated_weight.set(&p, w);
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

    fn estimate_hardness_for_crack(&self, pos: &Pos) -> i64 {
        if self.state.is_broken.get(pos) {
            return self.fetch_investigated_hardness(pos).unwrap();
        }
        let not_investigate_hard = S_MAX as f64;
        let mut ret = not_investigate_hard;
        let mut max_w = 0.;
        let r = 10;

        for x in pos.x - r..=pos.x + r {
            for y in pos.y - r..=pos.y + r {
                let p = Pos { y, x };
                if !p.is_valid() || pos == &p {
                    continue;
                }
                if let Some(h) = self.fetch_investigated_hardness(&p) {
                    let d = pos.euclid_dist(&p);
                    let w = (10. - d) / 10.;
                    if w > max_w {
                        ret = w * h as f64 + (1. - w) * not_investigate_hard;
                        max_w = w;
                    }
                }
            }
        }
        ret as i64
    }

    fn estimate_hardness(&self, pos: &Pos) -> i64 {
        if self.state.is_broken.get(pos) {
            return self.fetch_investigated_hardness(pos).unwrap();
        }
        let mut sum = 0.;
        let mut div = 0.;
        let not_investigate_hard = 500;
        let r = 20;

        for x in pos.x - r..=pos.x + r {
            for y in pos.y - r..=pos.y + r {
                let p = Pos { y, x };
                if !p.is_valid() || pos == &p {
                    continue;
                }
                if let Some(h) = self.fetch_investigated_hardness(&p) {
                    let d = pos.euclid_dist(&p);
                    let radius = f64::abs((h - not_investigate_hard) as f64).sqrt() / 4. + 1.;
                    if d >= radius {
                        continue;
                    }
                    let w = 1. - (d / radius);
                    sum += h as f64 * w;
                    div += w;
                }
            }
        }
        if div == 0. {
            not_investigate_hard
        } else if div < 1. {
            (sum + not_investigate_hard as f64 * (1. - div)).round() as i64
        } else {
            (sum / div).round() as i64
        }
    }

    fn fetch_investigated_hardness(&self, p: &Pos) -> Option<i64> {
        if !self.state.is_broken.get(p) {
            if self.state.damage.get(p) > 0 {
                return Some(i64::min(S_MAX, self.state.damage.get(p) * 2));
            }
            return None;
        }

        let damage_before_break = self.state.damage_before_break.get(p);
        let damage = self.state.damage.get(p);

        let a = if damage_before_break == 0 {
            (damage as f64 * 0.5) as i64
        } else {
            (damage_before_break + damage) / 2
        };
        Some(a)
    }
}
