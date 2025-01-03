#! .venv/bin/python
import json
import re
import subprocess
import sys


def _path_to_module(path: str) -> str:
    # this by no means fully PEP-compliant, and does not work for flat layouts or custom
    # package organization; it works for default src-layouts only
    path = path.replace("/__init__.py", "").replace("/", ".")
    path = re.sub(r"\.py$", "", path)
    if "src." in path:
        start_index = path.index("src.") + len("src.")
        path = path[start_index:]
    return path


def _ruff_graph_pkgs(repo_root: str) -> dict[str, set[str]]:
    # once https://github.com/astral-sh/ruff/issues/13431 is implemented, we don't need
    # path_to_module anymore
    return {
        _path_to_module(k): set({_path_to_module(m) for m in v})
        for k, v in json.loads(
            subprocess.check_output(
                ["ruff", "analyze", "graph", "--preview"], cwd=repo_root
            )
        ).items()
    }


def _cycle_size(c_len: int, i: int, j: int) -> int:
    """Gives the length of a cycle if it is shortened using an edge from vertex index i to j"""
    return c_len - (j - i + 1) if j > i else (i - j + 1)


def _sub_cycle(c: tuple[str, ...], i: int, j: int) -> tuple[str, ...]:
    """Get the sub-cycle within c by using an edge from vertex index i to j"""
    if i < j:
        return c[: i + 1] + c[j:]

    new_cycle = c[j : i + 1]
    # make a canonical representation
    start_from = new_cycle.index(min(new_cycle))
    return new_cycle[start_from:] + new_cycle[0:start_from]


def main(repo_root: str, cycle_results_file: str):
    graph = _ruff_graph_pkgs(repo_root)

    with open(cycle_results_file) as f:
        cycles = [
            tuple(line.split(" -> ")) for line in f.read().split("\n") if " -> " in line
        ]

    # sort cycles by length, since larger cycles are likelier to be minimized, and this
    # makes it easier to grok the results and logs
    cycles = sorted(cycles, key=lambda c: len(c))
    print("Pre-minimization")
    print("# cycles          :", len(cycles))
    print("total cycle length:", sum(len(cycle) for cycle in cycles))
    print("longest cycle     :", max(len(cycle) for cycle in cycles))

    minimal_cycles = []
    for cycle in cycles:
        # all the None cases can be simplified with a base case of
        # embiggen = (-1, 0, len(cycle))
        # but this is more explicit
        emsmallen = None
        for i in range(len(cycle)):
            for j in range(len(cycle)):
                if j != i and j != (i + 1) and cycle[j] in graph[cycle[i]]:
                    proposed_cycle_size = _cycle_size(len(cycle), i, j)
                    if not emsmallen or proposed_cycle_size < emsmallen[2]:
                        emsmallen = (i, j, proposed_cycle_size)
        if emsmallen:
            i, j, _ = emsmallen
            minimal_cycles.append(_sub_cycle(cycle, i, j))
        else:
            minimal_cycles.append(cycle)

    # find number of unique cycles, total length of all cycles
    unique_minimal_cycles = set(minimal_cycles)
    print("\nPost-minimization")
    print("# cycles          :", len(unique_minimal_cycles))
    print("total cycle length:", sum(len(cycle) for cycle in unique_minimal_cycles))
    print("longest cycle     :", max(len(cycle) for cycle in unique_minimal_cycles))

    # print potentially most problematic edges (which show up in many cycles)
    # breaking these edges _might_ help resolve many cycles at once
    edge_frequencies = {}
    for cycle in unique_minimal_cycles:
        for i in range(len(cycle)):
            edge = (cycle[i], cycle[(i + 1) % len(cycle)])
            edge_frequencies.setdefault(edge, 0)
            edge_frequencies[edge] += 1

    for t in sorted(edge_frequencies.items(), key=lambda t: -t[1])[:10]:
        print(t[1], t[0])


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
