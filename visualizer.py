from PIL import Image, ImageDraw, ImageFont


def visualize_graph(graph_file: str, input_file: str) -> None:
    font = ImageFont.truetype("Arial.ttf", 36)
    N = 200
    D = 16
    points = []
    edges = []
    paths = []
    s = []
    houses = []
    sources = []

    with open(graph_file, "r") as f:
        n, m = map(int, f.readline().strip().split())
        for _ in range(n):
            y, x = map(int, f.readline().strip().split())
            points.append((y, x))

        for _ in range(m):
            u, v, w = map(int, f.readline().strip().split())
            edges.append((u, v, w))

        for _ in range(n):
            p = list(map(int, f.readline().strip().split()))
            paths.append(p)

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

    im = Image.new("RGB", (N * D, N * D), (255, 255, 255))
    draw = ImageDraw.Draw(im)
    for y in range(N):
        for x in range(N):
            o = 255 - int(s[y][x] / 5000 * 255)
            draw.rectangle((x * D, y * D, (x + 1) * D, (y + 1) * D), fill=(o, o, 255))

    for py, px in points:
        draw.ellipse(
            (
                D * (px - 0.5),
                D * (py - 0.5),
                D * (px + 0.5),
                D * (py + 0.5),
            ),
            fill=(60, 60, 60),
        )

    for u, v, _ in edges:
        pu, pv = points[u], points[v]
        draw.line(
            (D * pu[1], D * pu[0], D * pv[1], D * pv[0]),
            fill=(60, 60, 60),
            width=3,
        )

    for path in paths:
        for e in path:
            u, v, w = edges[e]
            pu, pv = points[u], points[v]
            draw.text(
                ((pu[1] + pv[1]) / 2 * D, (pu[0] + pv[0]) / 2 * D),
                f"{w}",
                fill=(255, 0, 0),
                font=font,
            )
            draw.line(
                (D * pu[1], D * pu[0], D * pv[1], D * pv[0]),
                fill=(255, 0, 0),
                width=5,
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
    visualize_graph("log/graph.txt", f"tools/in/{case}.txt")
