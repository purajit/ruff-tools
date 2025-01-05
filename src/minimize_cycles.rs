use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::vec::Vec;

/// Gives the length of a cycle (number of nodes) if it is shortened
/// using an edge from vertex index i to j
fn cycle_size(c_len: usize, i: usize, j: usize) -> usize {
    if j > i {
        c_len - (j - i - 1)
    } else {
        i - j + 1
    }
}

/// Returns a canonical representation of a cycle, where the first vertex is the smallest
/// (alphabetically)
pub(crate) fn canonical_cycle<'a>(c: &[&'a str]) -> Vec<&'a str> {
    let start_vertex = c.iter().min().unwrap();
    let start_index = c.iter().position(|v| v == start_vertex).unwrap();
    return c[start_index..]
        .iter()
        .chain(&c[..start_index])
        .cloned()
        .collect();
}

/// Get the sub-cycle within c by using an edge from vertex index i to j
fn sub_cycle<'a>(c: &Vec<&'a str>, i: usize, j: usize) -> Vec<&'a str> {
    let new_cycle = if i < j {
        &c[..(i + 1)]
            .iter()
            .chain(&c[j..])
            .cloned()
            .collect::<Vec<&str>>()
    } else {
        &c[j..(i + 1)]
    };

    return canonical_cycle(new_cycle);
}

pub(crate) fn minimize_cycle<'a>(
    graph: &HashMap<String, HashSet<String>>,
    cycle: Vec<&'a str>,
) -> Vec<&'a str> {
    // all the None cases can be simplified with a base case of
    // embiggen = (-1, 0, cycle.len())
    // but this is more explicit
    let mut emsmallen: Option<(usize, usize, usize)> = None;
    for i in 0..cycle.len() {
        for j in 0..cycle.len() {
            if j != i
                && j != (i + 1)
                && graph.contains_key(cycle[i])
                && graph.get(cycle[i]).unwrap().contains(cycle[j])
            {
                let proposed_cycle_size = cycle_size(cycle.len(), i, j);
                if emsmallen.is_none() || proposed_cycle_size < emsmallen.unwrap().2 {
                    emsmallen = Some((i, j, proposed_cycle_size));
                }
            }
        }
    }
    return match emsmallen {
        Some((i, j, _)) => sub_cycle(&cycle, i, j),
        None => cycle,
    };
}
pub(crate) fn minimize_cycles(cycles_results_file: String) {
    let graph = super::ruff_util::ruff_graph(true, false, None);

    let contents =
        fs::read_to_string(cycles_results_file).expect("Should have been able to read the file");
    let mut cycles = contents
        .split("\n")
        .filter_map(|l| match l.find(" -> ") {
            Some(_) => Some(l.split(" -> ").collect()),
            None => None,
        })
        .collect::<Vec<Vec<&str>>>();

    println!("Pre-minimization");
    println!("# cycles          : {}", cycles.len());
    println!(
        "total cycle length: {}",
        cycles.iter().map(|c| c.len()).sum::<usize>()
    );

    println!(
        "longest cycle     : {}",
        cycles.iter().map(|c| c.len()).max().unwrap()
    );

    // sort cycles by length, since larger cycles are likelier to be minimized, and this
    // makes it easier to grok the results and logs
    cycles.sort_by(|a, b| a.len().cmp(&b.len()));

    // println!("GRAPH {:?}", graph);
    // println!("CYCLES {:?}", cycles);

    let mut minimal_cycles = Vec::<Vec<&str>>::new();
    for cycle in cycles {
        minimal_cycles.push(minimize_cycle(&graph, cycle));
    }

    // find number of unique cycles, total length of all cycles
    let unique_minimal_cycles = minimal_cycles.iter().collect::<HashSet<_>>();
    println!("\nPost-minimization");
    println!("# cycles          : {}", unique_minimal_cycles.len());
    println!(
        "total cycle length: {}",
        unique_minimal_cycles.iter().map(|c| c.len()).sum::<usize>()
    );

    println!(
        "longest cycle     : {}",
        unique_minimal_cycles.iter().map(|c| c.len()).max().unwrap()
    );

    for cycle in &unique_minimal_cycles {
        println!("{}", cycle.join(" -> "));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cycle_size() {
        // unchanged cycle
        assert_eq!(cycle_size(10, 9, 0), 10);
        // cycles changed by shortcuts
        assert_eq!(cycle_size(10, 2, 5), 8);
        assert_eq!(cycle_size(10, 0, 9), 2);
        // cycles changed by a contained cycle
        assert_eq!(cycle_size(10, 5, 2), 4);
        assert_eq!(cycle_size(10, 5, 4), 2);
        // not possible in reality, but single-node cycle
        assert_eq!(cycle_size(10, 3, 3), 1);
    }

    #[test]
    fn test_canonical_cycle() {
        assert_eq!(canonical_cycle(&["a", "b"]), ["a", "b"]);
        assert_eq!(canonical_cycle(&["b", "a"]), ["a", "b"]);
        assert_eq!(canonical_cycle(&["b", "c", "a"]), ["a", "b", "c"]);
        assert_eq!(
            canonical_cycle(&["b", "c", "q", "a", "d"]),
            ["a", "d", "b", "c", "q"]
        );
    }

    #[test]
    fn test_sub_cycle() {
        // unchanged cycle
        assert_eq!(sub_cycle(&vec!["a", "b", "c"], 2, 0), ["a", "b", "c"]);
        // unchanged cycle, just canonicalized
        assert_eq!(sub_cycle(&vec!["b", "c", "a"], 2, 0), ["a", "b", "c"]);
        // shortcut
        assert_eq!(sub_cycle(&vec!["a", "b", "c"], 1, 0), ["a", "b"]);
        // should shortcut
        assert_eq!(
            sub_cycle(&vec!["b", "c", "e", "a", "d"], 2, 4),
            ["b", "c", "e", "d"]
        );
        // should shortcut and canonicalize
        assert_eq!(
            sub_cycle(&vec!["b", "a", "c", "e", "d"], 1, 3),
            ["a", "e", "d", "b"]
        );
        // should get contained cycle AND canonicalize
        assert_eq!(sub_cycle(&vec!["b", "c", "a"], 2, 1), ["a", "c"]);
    }

    /// these have only one possible option, and just test reading edges from the graph
    #[test]
    fn test_minimize_cycle_simple() {
        let graph = HashMap::from([("b".to_string(), HashSet::from(["a".to_string()]))]);
        // unchanged cycle
        assert_eq!(minimize_cycle(&graph, vec!["j", "k", "l"]), ["j", "k", "l"]);
        assert_eq!(minimize_cycle(&graph, vec!["a", "j", "b"]), ["a", "j", "b"]);
        // should shortcut
        assert_eq!(minimize_cycle(&graph, vec!["a", "b", "c"]), ["a", "b"]);
        assert_eq!(
            minimize_cycle(&graph, vec!["b", "c", "e", "a", "d"]),
            ["a", "d", "b"]
        );
        // should get contained cycle
        assert_eq!(minimize_cycle(&graph, vec!["c", "a", "b", "d"]), ["a", "b"]);

        // these have multiple options, and should find the best one
    }

    /// these have multiple options, and should find the best one
    #[test]
    fn test_minimize_cycle_complex() {
        let graph = HashMap::from([
            ("a".to_string(), HashSet::from([])),
            ("b".to_string(), HashSet::from(["a".to_string()])),
            (
                "j".to_string(),
                HashSet::from(["a".to_string(), "l".to_string()]),
            ),
            ("k".to_string(), HashSet::from(["j".to_string()])),
            ("n".to_string(), HashSet::from(["l".to_string()])),
        ]);
        // three shortcuts: j -> a (cuts 3), b -> a (cuts 1)
        assert_eq!(
            minimize_cycle(&graph, vec!["a", "j", "k", "b", "l"]),
            ["a", "j"]
        );
        // two shortcuts: k -> j (cuts 1), n -> l (cuts 2)
        assert_eq!(
            minimize_cycle(&graph, vec!["a", "k", "m", "n", "b1", "b2", "l"]),
            ["a", "k", "m", "n", "l"]
        );
        // two contained cycles: l -> r -> j, and a -> b
        assert_eq!(
            minimize_cycle(&graph, vec!["q", "d", "l", "r", "j", "a", "b"]),
            ["a", "b"]
        );
        // two contained cycles: l -> j, and a -> r -> b
        assert_eq!(
            minimize_cycle(&graph, vec!["q", "d", "l", "j", "a", "r", "b"]),
            ["j", "l"]
        );
        // one shortcut: q -> a (cuts 6), one contained cycle: l -> l1 -> l2 -> j (cuts 2)
        assert_eq!(
            minimize_cycle(&graph, vec!["a", "q", "b", "l", "l1", "l2", "j"]),
            ["a", "q", "b"]
        );
        // one shortcut: q -> a (cuts 3), one contained cycle: l -> l1 -> j (cuts 4)
        assert_eq!(
            minimize_cycle(&graph, vec!["a", "q1", "q", "b", "l", "l1", "j"]),
            ["j", "l", "l1"]
        );
    }
}
