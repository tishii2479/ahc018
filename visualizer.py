import os
import sys

import matplotlib.pyplot as plt  # noqa
from PIL import Image, ImageDraw, ImageFont


def visualize_graph(
    grid_file: str,
    state_file: str,
    input_file: str,
    output_file: str,
    real_s: bool = False,
) -> Image:
    font = ImageFont.truetype("Arial.ttf", 10)  # noqa
    N = 200
    D = 4
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
            e = s[y][x] if real_s else estimated_s[y][x]
            o = 255 - int(e / 5000 * 255)
            draw.rectangle((x * D, y * D, (x + 1) * D, (y + 1) * D), fill=(o, 255, o))

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

    return im


def analyze_damage_efficency(input_file: str, output_file: str) -> None:
    N = 200

    s = []

    with open(input_file, "r") as f:
        n, w, k, c = map(int, f.readline().strip().split())

        for _ in range(N):
            s.append(list(map(int, f.readline().strip().split())))

    is_join = False
    start_join = "# end optimize"

    ds = {}
    ps = {}

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
    xs = []
    ys = []

    for ((y, x), count) in ds.items():
        expected = s[y][x]
        actual = ps[(y, x)]

        xs.append(expected)
        ys.append(actual - expected)

    # plt.scatter(xs, ys)
    # plt.show()

    print(f"average exceed damage: {sum(ys) / len(ys)}, total exceed damage: {sum(ys)}")

    xs = []
    ys = []

    for ((y, x), count) in ds.items():
        xs.append(s[y][x])
        ys.append(count)

    # plt.scatter(xs, ys)
    # plt.show()

    print(
        f"average damage count: {sum(ys) / len(ys)}, total count: {sum(ys)}, "
        + f"exceed damage count: {sum(ys) - len(ys)}"
    )


if __name__ == "__main__":
    case = sys.argv[1]
    images = []
    n = 0

    while True:
        grid_file = f"log/grid_{n}.txt"
        if os.path.exists(grid_file) is False:
            break
        im = visualize_graph(
            grid_file,
            f"log/state_{n}.txt",
            f"tools/in/{case}.txt",
            f"tools/out/{case}.txt",
        )
        im.save(f"log/vis_{n}.png")
        images.append(im)
        n += 1

    images.append(
        visualize_graph(
            f"log/grid_{n - 1}.txt",
            f"log/state_{n - 1}.txt",
            f"tools/in/{case}.txt",
            f"tools/out/{case}.txt",
            True,
        )
    )

    images[0].save(
        "log/vis_movie.gif",
        save_all=True,
        append_images=images[1:],
        duration=2000,
        loop=0,
    )

    analyze_damage_efficency(
        f"tools/in/{case}.txt",
        f"tools/out/{case}.txt",
    )
