from math import sqrt

from PIL import Image, ImageDraw, ImageFont


def visualize_graph(
    grid_file: str, state_file: str, input_file: str, output_file: str
) -> None:
    font = ImageFont.truetype("Arial.ttf", 36)  # noqa
    N = 200
    D = 16
    is_used = [[False] * N for _ in range(N)]
    estimated_s = [[0] * N for _ in range(N)]
    damage = [[0] * N for _ in range(N)]
    s = []
    houses = []
    sources = []

    with open(grid_file, "r") as f:
        for y in range(N):
            v = list(map(int, f.readline().strip().split()))
            for x in range(N):
                if v[x] == 1:
                    is_used[y][x] = True

        for y in range(N):
            v = list(map(int, f.readline().strip().split()))
            for x in range(N):
                estimated_s[y][x] = v[x]

    with open(state_file, "r") as f:
        for y in range(N):
            v = list(map(int, f.readline().strip().split()))
            for x in range(N):
                damage[y][x] = v[x]

    with open(input_file, "r") as f:
        n, w, k, c = map(int, f.readline().strip().split())

        for _ in range(N):
            s.append(list(map(int, f.readline().strip().split())))

        for _ in range(w):
            y, x = map(int, f.readline().strip().split())
            sources.append((y, x))

        for _ in range(k):
            y, x = map(int, f.readline().strip().split())
            houses.append((y, x))

    ds = {}
    ps = {}
    is_join = False
    start_join = "# end optimize"

    with open(output_file, "r") as f:
        for line in f.readlines():
            if len(line) >= 1 and line[:1] == "#":
                if (
                    len(line.strip()) >= len(start_join)
                    and line[: len(start_join)] == start_join
                ):
                    is_join = True
                continue

            y, x, p = map(int, line.split())
            if (y, x) not in ps:
                ps[(y, x)] = 0
            ps[(y, x)] += p

            if is_join:
                if (y, x) not in ds:
                    ds[(y, x)] = 0
                ds[(y, x)] += 1

    im = Image.new("RGB", (N * D, N * D), (255, 255, 255))
    draw = ImageDraw.Draw(im)
    for y in range(N):
        for x in range(N):
            o = 255 - int(sqrt(estimated_s[y][x]) / sqrt(5000) * 255)
            draw.rectangle((x * D, y * D, (x + 1) * D, (y + 1) * D), fill=(o, o, 255))

    for y in range(N):
        for x in range(N):
            if damage[y][x] > 0:
                draw.rectangle(
                    (x * D, y * D, (x + 1) * D, (y + 1) * D), fill=(30, 30, 30)
                )
                draw.text(
                    (x * D, y * D),
                    f"{damage[y][x]} / {s[y][x]}",
                    font=font,
                    fill=(30, 30, 30),
                )

            if is_used[y][x]:
                draw.rectangle(
                    (x * D, y * D, (x + 1) * D, (y + 1) * D), fill=(255, 0, 0)
                )

    for py, px in houses:
        draw.ellipse(
            (
                D * (px - 1.5),
                D * (py - 1.5),
                D * (px + 1.5),
                D * (py + 1.5),
            ),
            fill=(255, 0, 0),
        )

    for py, px in sources:
        draw.ellipse(
            (
                D * (px - 1.5),
                D * (py - 1.5),
                D * (px + 1.5),
                D * (py + 1.5),
            ),
            fill=(0, 0, 255),
        )

    im.show()


if __name__ == "__main__":
    import sys

    case = sys.argv[1]
    for i in range(4):
        visualize_graph(
            f"log/grid_{i}.txt",
            f"log/state_{i}.txt",
            f"tools/in/{case}.txt",
            f"tools/out/{case}.txt",
        )
