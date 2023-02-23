struct Grid {
    total_score: i64,
    estimated_hardness: Vec2d,
    is_used: Vec2dBool,
}

impl Grid {
    fn dijkstra(&self, start: &Pos, upper: i64) -> (Vec<Vec<i64>>, Vec<Vec<Pos>>) {}

    fn find_path_to_nearest_source(&self, h: &Pos) -> (Vec<Pos>, i64) {}

    fn find_current_path_to_source(&self, h: &Pos) -> Option<Vec<Pos>> {}

    fn find_unconnected_houses(&self) -> Vec<usize> {
        let mut v = vec![];
        for (i, h_pos) in self.houses.iter().enumerate() {
            if estimated_grid
                .find_current_path_to_source(h_pos)
                .is_none()
            {
                v.push(i);
            }
        }
        v
    }

    fn set(&mut self, p: &Pos, v: bool) -> bool {
        if is_used[p.y][p.x] == v {
            return false;
        }
        if is_used[p.y][p.x] {
            total_score -= estimate_hardness[p.y][p.x];
        } else {
            total_score += estimate_hardness[p.y][p.x];
        }
        is_used[p.y][p.x] = v;
        return true;
    }
}

struct Change {
    p: Pos,
    prev: bool,
}

struct Solver {
    input: Input,
    state: State,
    interactor: Interactor
}

fn pos_to_grid(y: i64, x: i64) -> Pos {
    let y = if y == N { N - 1 } else { y };
    let x = if x == N { N - 1 } else { x };
    Pos { y, x };
}

fn add_damage_to_hardness_if_needed(hardness: i64, state: &mut State, interactor: &Interactor) -> bool {
    let power = hardness - state.damage.get(&p);
    if power <= 0 {
        return false;
    }
    interactor.add_damage(&mut state, &p, power);
    return true;
}

impl Solver {
    fn solve(&mut self) {
        // グリッド上にあらかじめ掘削し、頑丈度を調べる
        for y in (0..N).step_by(20) {
            for x in (0..N).step_by(20) {
                let p = pos_to_grid(y, x);
                self.investigate(&p);
            }
        }

        for d in vec![10, 5] {
            // 頑丈度を予測したグリッドを作成する
            let mut estimated_grid = self.generate_estimated_grid();

            // 山登りによる選択経路の最適化
            // TODO: 複数回やって、多様性を出す
            self.optimize_route(&mut estimated_grid);

            // 選択経路の周りを探索する
            self.investigate_around_used_path(&estimated_grid, d);
        }

        let mut estimated_grid = self.generate_estimated_grid();
        self.optimize_route(&mut estimated_grid);

        // 選択経路に使われている地点を割る
        self.destroy_used_path(&estimated_grid);
    }

    fn optimize_route(&self, esimated_grid: &mut Grid) {
        // 初期解の作成
        for h_pos in self.input.house.iter() {
            let (nearest_source_path, dist) = estimated_grid.find_path_to_nearest_source(&h_pos);
            for p in nearest_source_path.iter() {
                estimated_grid.set(p, true);
            }
        }

        let mut current_score = estimated_grid.total_score();

        // 山登りによる最適化
        for _ in 0..1000 {
            // ランダムな家から接続している水源までのパスを消す
            let h_idx = rnd::gen_range(0, self.input.house.len());
            let h_pos = &self.input.house[h_idx];
            let path_to_source = estimated_grid.find_current_path_to_source(&h_pos).unwrap();

            let mut changes = vec![];

            for p in path_to_source.iter() {
                changes.push(Change { p, estimated_grid.is_used[p.y][p.x] });
                estimated_grid.set(p, false);
            }

            // 水源に接続されなくなった家を再度接続する
            let mut reconnect_houses = estimated_grid.find_unconnected_houses();
            rnd::shuffle(reconnect_houses);
            for h_idx in reconnect_houses.iter() {
                let (nearest_source_path, dist) = estimated_grid.find_path_to_nearest_source(&h_pos);
                for p in nearest_source_path.iter() {
                    changes.push(Change { p, estimated_grid.is_used[p.y][p.x] });
                    estimated_grid.set(p, true);
                }
            }

            let new_score = estimated_grid.total_score;

            if new_score < current_score {
                // 採用
            } else {
                // ロールバック
                changes.reverse();
                for c in changes.iter() {
                    estimated_grid.set(c.p, c.prev);
                }
            }
        }
    }

    fn destroy_used_path(&mut self, estimated_grid: &Grid) {
        for y in 0..N {
            for x in 0..N {
                let p = Pos { y, x };
                if !estimated_grid.is_used.get(&p) {
                    continue;
                }
                let mut estimated_hardness = self.estimate_hardness(&p);
                while !self.state.is_broken.get(&p) {
                    add_damage_to_hardness_if_needed(estimate_hardness, &mut self.state, &self.interactor);
                    estimate_hardness = i64::min(S_MAX, (estimate_hardness as f64 * 1.2) as i64);
                }
            }
        }
    }

    fn investigate_around_used_path(&mut self, estimated_grid: &Grid, d: i64) {
        let mut investigate_pos = vec![];

        for y in 0..N {
            for x in 0..N {
                if !estimated_grid.is_used[y][x] {
                    continue;
                }
                // 探索箇所を加える
                for py in vec![y / d * d, (y / d + 1) * d] {
                    for px in vec![x / d * d, (x / d + 1) * d] {
                        let p = Pos { py, px };
                        if investigate_pos.contains(&p) {
                            continue;
                        }
                        investigate_pos.push(p);
                    }
                }
            }
        }

        for p in investigate_pos.iter() {
            self.investigate(&p);
        }
    }

    fn investigate(&mut self, p: &Pos) -> bool {
        if self.state.is_broken.get(p) {
            return true;
        }

        let estimated_hardness = self.estimate_hardness(p).unwrap_or(10);
        // TODO: inject dp
        for dp in vec![8, 16, 32, 64, 128, 256, 512, 1024] {
            add_damage_to_hardness_if_needed(estimate_hardness + dp, &mut self.state, &self.interactor);
        }
        return false;
    }

    fn generate_estimated_grid(&self) -> Grid {
        let mut estimated_hardness = vec![vec![0; N]; N];
        for y in 0..N {
            for x in 0..N {
                let p = Pos { y, x };
                estimated_hardness[y][x] = self.estimate_hardness(&p).unwrap();
            }
        }

        Grid {
            total_score: 0,
            estimate_hardness,
            is_used: vec![vec![false; N]],
        }
    }

    fn estimate_hardness(&self, pos: &Pos) -> Option<i64> {
        if self.state.is_broken.get(pos) {
            return self.fetch_investigated_hardness(pos).unwrap();
        }

        const D: i64 = 5;
        let (tx, ty) = ((pos.x - (D as f64 * 1.5) as i64) / D, (pos.y - (D as f64 * 1.5) as i64) / D);
        let mut sum = 0.;
        let mut div = 0.;

        // TODO: 開拓が進んでいない時は20x20の区画を見る

        for dx in 0..4 {
            for dy in 0..4 {
                let p = Pos { y: dy * D + ty, x: dx * D + tx };
                if !p.is_vaild() {
                    continue;
                }
                if let Some(h) = self.fetch_investigated_hardness(&p) {
                    let w = 1. / pos.dist(&p) as f64;
                    sum += h as f64 * w;
                    div += w;
                }
            }
        }

        Some((sum / div).round() as i64)
    }

    fn fetch_investigated_hardness(&self, p: &Pos) -> Option<i64> {
        if self.state.is_broken.get(p) {
            return None;
        }

        let damage_before_break = self.state.damage_before_break.get(p);
        let damage = self.state.damage.get(p);

        let a = if damage_before_break == 0 {
            (damage as f64 * 0.9) as i64
        } else {
            (damage_before_break + damage) / 2
        };
        Some(a)
    }
}
