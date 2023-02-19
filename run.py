import multiprocessing
import subprocess

import pandas as pd

# mypy: ignore-errors

TL = 110.0


def execute_case(seed):
    input_file_path = f"tools/in/{seed:04}.txt"
    output_file_path = f"tools/out/{seed:04}.txt"

    tester_cmd = "./tools/target/release/tester"
    solver_cmd = "./target/release/ahc018"

    with open(input_file_path, "r") as f:
        N, W, K, C = map(int, f.readline().split())

    cmd = f"{tester_cmd} {solver_cmd} < {input_file_path} > {output_file_path}"
    while True:
        proc = subprocess.run(cmd, stderr=subprocess.PIPE, timeout=TL, shell=True)
        stderr = proc.stderr.decode("utf8")
        start = stderr.find("`")
        end = stderr.find("`", start + 1)
        if stderr[start + 1 : end].isnumeric() is False:
            print(f"failed at seed: {seed} {stderr}")
            continue
        score = int(stderr[start + 1 : end])
        assert score != -1, f"Failed at seed: {seed}"

        return seed, score, N, W, K, C


def run(case_num: int):
    subprocess.run("cargo build --release", shell=True)

    scores = []
    count = 0
    total = 0

    with multiprocessing.Pool() as pool:
        for seed, score, N, W, K, C in pool.imap_unordered(
            execute_case, range(case_num)
        ):
            count += 1

            try:
                scores.append((int(score), f"{seed:04}", N, W, K, C))
                total += scores[-1][0]
            except ValueError:
                print(seed, "ValueError", flush=True)
                print(score, flush=True)
                exit()
            except IndexError:
                print(seed, "IndexError", flush=True)
                print(f"error: {score}", flush=True)
                exit()

            print(
                f"case {seed:3}: (score: {scores[-1][0]:>13,}, current ave: "
                + f"{total / count:>15,.2f}, "
                + f"N = {N:3}, W = {W}, K = {K:2}, C = {C:3})",
                flush=True,
            )

    print("=" * 100)
    scores.sort()
    ave = total / count
    print(f"ave: {ave:,.2f}")

    df = pd.DataFrame(scores, columns=["score", "case", "n", "w", "k", "c"])

    return df


if __name__ == "__main__":
    score_df = run(1000)
    score_df.to_csv("log/score.csv", index=False)

    score_df = score_df.set_index("case")
    score_df = score_df[["score"]]

    bench_df = pd.read_csv("log/bench.csv", index_col="case", dtype={"case": str})
    bench_df = bench_df.rename(columns={"score": "bench_score"})

    sample_bench_df = pd.read_csv(
        "log/sample_bench.csv", index_col="case", dtype={"case": str}
    )
    sample_bench_df = sample_bench_df.rename(columns={"score": "sample_bench_score"})

    df = pd.merge(bench_df, score_df, how="inner", on="case")
    df = pd.merge(df, sample_bench_df, how="inner", on="case")
    df["relative_score"] = df.bench_score / df.score
    df["relative_score_sample"] = df.sample_bench_score / df.score
    print(f"Relative_score:         {df.relative_score.mean():.4f}")
    print(f"Relative_score_sample:  {df.relative_score_sample.mean():.4f}")

    for col in ["c_x", "w_x", "k_x"]:
        print(df.groupby(col).mean()["relative_score"])
