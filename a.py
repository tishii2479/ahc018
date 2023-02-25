s = []
N = 200

c = int(input())

with open(f"tools/in/{c:04}.txt", "r") as f:
    n, w, k, c = map(int, f.readline().strip().split())

    for _ in range(N):
        s.append(list(map(int, f.readline().strip().split())))

print(sum([sum(e) for e in s]) / N / N)
