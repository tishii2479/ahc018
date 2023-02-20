from PIL import Image, ImageDraw


def visualize_graph(graph_file: str, input_file: str) -> None:
    N = 200
    D = 16
    points = []
    edges = []
    paths = []
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

    # with open(input_file: str) -> None:

    im = Image.new("RGB", (N * D, N * D), (255, 255, 255))
    draw = ImageDraw.Draw(im)

    for py, px in points:
        draw.ellipse(
            (
                D * px - D / 2,
                D * py - D / 2,
                D * (px + 1) - D / 2,
                D * (py + 1) - D / 2,
            ),
            fill=(255, 0, 0),
        )

    for u, v, w in edges:
        pu, pv = points[u], points[v]
        draw.line(
            (D * pu[1], D * pu[0], D * pv[1], D * pv[0]),
            fill=(180, 0, 0),
            width=3,
        )

    for path in paths:
        for e in path:
            u, v, _ = edges[e]
            pu, pv = points[u], points[v]
            draw.line(
                (D * pu[1], D * pu[0], D * pv[1], D * pv[0]),
                fill=(255, 0, 0),
                width=10,
            )

    im.show()


if __name__ == "__main__":
    visualize_graph("log/graph.txt", "tools/in/0000.txt")
