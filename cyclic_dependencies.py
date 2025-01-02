import json
import subprocess


def get_cycles(graph: dict[str, set[str]]) -> set[tuple[str, ...]]:
    """Return a list of detected cycles based on an ordered graph (i.e. keys are
    vertices and values are lists of destination vertexs representing edges).
    """
    result: set[tuple[str, ...]] = set()
    for vertex in graph.keys():
        result.update(_get_cycles(graph, (), set(), vertex))
    return result


def _get_cycles(
    graph: dict[str, set[str]],
    path: tuple[str, ...],
    visited: set[str],
    vertex: str,
) -> set[tuple[str, ...]]:
    try:
        vertex_index = path.index(vertex)
    except ValueError:
        pass
    else:
        cycle = path[vertex_index:]
        # make a canonical representation
        start_from = cycle.index(min(cycle))
        cycle = cycle[start_from:] + cycle[0:start_from]
        return {tuple(cycle)}

    cycles = set()
    path += (vertex,)
    for node in graph.get(vertex, set()):
        # if node not in visited:
        cycles.update(_get_cycles(graph, path, visited, node))
        # visited.add(node)

    return cycles


def main():
    graph = {
        k: set(v)
        for k, v in json.loads(
            subprocess.check_output(["../ruff/target/debug/ruff", "analyze", "graph"])
        ).items()
    }
    cycles = [[node] + path for node in graph for path in dfs_cycles(graph, node, node)]
    # cycles = get_cycles(graph)
    for cycle in cycles:
        print(" -> ".join(cycle))


if __name__ == "__main__":
    main()
