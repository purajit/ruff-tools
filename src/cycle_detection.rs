extern crate alloc;

use std::collections::HashMap;
use std::collections::HashSet;

pub(crate) fn detect_cycles() {
    let graph = super::ruff_util::ruff_graph(false, false);
    let mut cycles = HashSet::new();
    for vertex in graph.keys() {
        cycles.extend(get_cycles_from_vertex(&graph, vertex));
    }

    for cycle in &cycles {
        println!("{}", cycle.join(" -> "));
    }

    println!("{}", cycles.len())
}

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
