extern crate alloc;

use std::collections::HashMap;
use std::collections::HashSet;

pub(crate) fn detect_cycles() {
    let graph = super::ruff_util::ruff_graph(false, false, None);
    let cycles = detect_cycles_in_graph(&graph);
    for cycle in &cycles {
        println!("{}", cycle.join(" -> "));
    }
    println!("{}", cycles.len())
}

fn detect_cycles_in_graph(graph: &HashMap<String, HashSet<String>>) -> HashSet<Vec<&str>> {
    let mut cycles = HashSet::new();
    for vertex in graph.keys() {
        cycles.extend(get_cycles_from_vertex(&graph, vertex));
    }
    return cycles;
}

/// This is ported from pylint's cycle detection which is rather chaotic,
/// and reasonably does not find all cycles (which is non-polynomial, of course)
/// TODO: since we only care about minimal cycles, we could use Horton's Algorithm
/// to find a minimum cycle basis in O(ve^3)
fn get_cycles_from_vertex<'a>(
    graph: &'a HashMap<String, HashSet<String>>,
    vertex: &'a String,
) -> HashSet<Vec<&'a str>> {
    let mut stack = Vec::new();
    let mut cycles = HashSet::new();
    let mut visited = HashSet::new();
    // path, visited, node to explore
    stack.push((Vec::new(), vertex));

    while !stack.is_empty() {
        let (path, vertex) = stack.pop().unwrap();
        match path.iter().position(|v| v == vertex) {
            Some(vertex_index) => {
                cycles.insert(super::minimize_cycles::minimize_cycle(
                    &graph,
                    super::minimize_cycles::canonical_cycle(&path[vertex_index..]),
                ));
            }
            None => {
                let mut new_path = path.clone();
                new_path.push(vertex);
                if graph.contains_key(vertex) {
                    for node in graph.get(vertex).unwrap() {
                        if !visited.contains(node) {
                            visited.insert(node);
                            stack.push((new_path.clone(), node));
                        }
                    }
                }
            }
        }
    }

    cycles
}

#[cfg(test)]
mod tests {
    use super::*;

    /// only one cycle in the graph
    #[test]
    fn test_detect_cycles_simple() {
        let graph = HashMap::from([
            ("a".to_string(), HashSet::from(["b".to_string()])),
            ("b".to_string(), HashSet::from(["c".to_string()])),
            ("c".to_string(), HashSet::from(["d".to_string()])),
            ("c".to_string(), HashSet::from(["a".to_string()])),
        ]);
        assert_eq!(
            detect_cycles_in_graph(&graph),
            HashSet::from([vec!["a", "b", "c"]])
        );
    }

    /// has many cycles - however, since the algorithm isn't guaranteed to find all
    /// of them, we make sure we find at least a certain number - this is deterministic,
    /// but not reasonable pre-calculable
    #[test]
    fn test_detect_cycles_complex() {
        let graph = HashMap::from([
            (
                "a".to_string(),
                HashSet::from([
                    "b".to_string(),
                    "j".to_string(),
                    "k".to_string(),
                    "n".to_string(),
                    "q".to_string(),
                    "r".to_string(),
                ]),
            ),
            ("b".to_string(), HashSet::from(["a".to_string()])),
            (
                "j".to_string(),
                HashSet::from(["a".to_string(), "k".to_string(), "l".to_string()]),
            ),
            ("k".to_string(), HashSet::from(["j".to_string()])),
            ("l".to_string(), HashSet::from(["a".to_string()])),
            ("n".to_string(), HashSet::from(["l".to_string()])),
        ]);
        // these are 3 cycles we know this finds; any changes to the logic could alter this
        assert_eq!(
            HashSet::from([vec!["a", "n", "l"], vec!["j", "k"], vec!["a", "b"]])
                .difference(&detect_cycles_in_graph(&graph))
                .count(),
            0
        );
    }
}