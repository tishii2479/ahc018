fn solve() {
    // 家の地点の頑丈度を調べる
    for h_pos in self.input.house.iter() {
        for p in vec![13, 25, 50, 100, 200, 400, 800, 1600, 3200, 5000] {
            add_damage_to_hardness_if_needed(h_pos, p);
        }
    }

    // 上限を決めたdfsにより、家から水源までのルートを確保する
    // TODO: 水源までの距離が近い家から探索する
    let mut done = vec![false; self.input.house.len()];
    for upper in vec![15, 25, 50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 1000] {
        for (i, h_pos) in self.input.house.iter().enumerate() {
            if done[i] {
                continue;
            }
            if dfs(&h_pos, upper) {
                // 使用する経路上の点をsinked_posに追加する
                done[i] = true;
            }
        }
    }

    // TODO: 繋げることで、スコアが改善する可能性がある道を探索する

    // 焼きなましにより、採用するルートを最適化する
    // TODO: 探索されていない場所の重みの計算方法を考える
    let mut grid = self.generate_estimated_grid();
    self.optimize_route(&mut grid);

    // 採用するルート上の通路を全て壊す
    self.destroy_edge_path(&grid);
}

fn dfs(start: &Pos, upper: i64) -> Option<Vec<Pos>> {
    let mut dist = Vec2d<i64>::new(N, N, INF);
    let mut par = Vec2d<Option<None>>::new(N, N, None);
    let mut heap: BinaryHeap<i64, Pos> = BinaryHeap::new();
    dist.set(start, 0);

    let mut best_eval = INF;
    while let Some((d, pos)) = heap.pop() {
        // upperまで掘削し、壊れたら追加する
        // 20刻みとかで調べる
        // 前回の結果を反映する（posの硬さを反映する）
        let par = par.get(&pos);
        let start = i64::max(10, state.damage.get(&par) * 0.5); // :param
        let end = i64::min(upper, i64::max(20, state.damage.get(&pos) + 100)); // :param
        let step = i64::max(i64::min(20, self.input.c * 2), (end - start) / 5); // :param
        for p in (start..end).step_by(step) {
            add_damage_to_hardness_if_needed(next_pos, p);
        }
        if !state.is_broken.get(&pos) {
            continue;
        }
        let hard_mean = (state.damage.get(&par) + state.damage.get(&pos)) / 2;
        let d = (hard_mean + c) * par.manhattan_dist(&pos);
        dist.set(&pos, d);

        if d > best_eval {
            continue;
        }

        if sinked_pos.contains(pos) {
            best_eval = d;
            continue;
        }

        for next_pos in pos.next() {
            // next_posの近くに使える点があれば、それを使う
            let next_pos = find_near_pos(next_pos);
            let hard_mean = (state.damage.get(&pos) + upper) / 2;
            let consumed = d + pos.dist(&next_pos) * (hard_mean + c);
            let next_dist = eval(next_pos, consumed);
            if next_dist < dist.get(&next_pos) {
                heap.push(next_dist);
                dist.set(&next_pos, next_dist);
                par.set(&next_pos, pos.clone());
            }
        }
    }

    if best_eval < INF {
        let mut path = vec![];
        // 最も評価値が良い水が通っているところから、
        // startまでparを辿って道を作る
        Some(path)
    } else {
        None
    }
}