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
    let power = hardness - state.damage.get(&p);
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
        // TODO: 必要な箇所だけを、house、sourceの位置をもとに計算する
        // グリッド上にあらかじめ掘削し、頑丈度を調べる
        for y in (0..N as i64).step_by(20) {
            for x in (0..N as i64).step_by(20) {
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

    fn optimize_route(&self, estimated_grid: &mut Grid) {
        // 初期解の作成
        for h_pos in self.input.house.iter() {
            let (nearest_source_path, dist) =
                estimated_grid.find_path_to_nearest_source(&h_pos, INF, &self.input.source);
            for p in nearest_source_path.iter() {
                estimated_grid.set(p, true);
            }
        }

        let mut current_score = estimated_grid.total_score;

        // 山登りによる最適化
        for _ in 0..1000 {
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
            // TODO: 経路を消した家を優先的に再接続する
            let mut reconnect_houses = estimated_grid.find_unconnected_houses(&self.input.house);
            rnd::shuffle(&mut reconnect_houses);

            for i in reconnect_houses.iter() {
                let (nearest_source_path, dist) = estimated_grid.find_path_to_nearest_source(
                    &self.input.house[*i],
                    INF,
                    &self.input.source,
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
                if !estimated_grid.is_used.get(&p) {
                    continue;
                }
                let mut estimated_hardness = self.estimate_hardness(&p).unwrap();
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

    fn investigate_around_used_path(&mut self, estimated_grid: &Grid, d: i64) {
        let mut investigate_pos = vec![];

        for y in 0..N as i64 {
            for x in 0..N as i64 {
                if !estimated_grid.is_used.get(&Pos { y, x }) {
                    continue;
                }
                // 探索箇所を加える
                for py in vec![y / d * d, (y / d + 1) * d] {
                    for px in vec![x / d * d, (x / d + 1) * d] {
                        let p = Pos { y: py, x: px };
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
            add_damage_to_hardness_if_needed(
                p,
                estimated_hardness + dp,
                &mut self.state,
                &mut self.interactor,
            );
        }
        return false;
    }

    fn generate_estimated_grid(&self) -> Grid {
        // TODO: is_usedにhouseとsourceの位置を追加
        let mut estimated_hardness = Vec2d::new(N, N, 0);
        for y in 0..N as i64 {
            for x in 0..N as i64 {
                let p = Pos { y, x };
                estimated_hardness.set(&p, self.estimate_hardness(&p).unwrap());
            }
        }

        Grid {
            total_score: 0,
            estimated_hardness,
            is_used: Vec2d::new(N, N, false),
        }
    }

    fn estimate_hardness(&self, pos: &Pos) -> Option<i64> {
        if self.state.is_broken.get(pos) {
            return self.fetch_investigated_hardness(pos);
        }

        const D: i64 = 5;
        let (tx, ty) = (
            (pos.x - (D as f64 * 1.5) as i64) / D,
            (pos.y - (D as f64 * 1.5) as i64) / D,
        );
        let mut sum = 0.;
        let mut div = 0.;

        // TODO: 開拓が進んでいない時は20x20の区画を見る

        for dx in 0..4 {
            for dy in 0..4 {
                let p = Pos {
                    y: dy * D + ty,
                    x: dx * D + tx,
                };
                if !p.is_valid() {
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
