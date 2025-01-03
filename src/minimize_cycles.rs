use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;

use std::vec::Vec;

/// Gives the length of a cycle if it is shortened using an edge from vertex index i to j
fn cycle_size(c_len: usize, i: usize, j: usize) -> usize {
    if j > i {
        c_len - (j - i + 1)
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
    if i < j {
        return c[..(i + 1)].iter().chain(&c[j..]).cloned().collect();
    }

    let new_cycle = &c[j..(i + 1)];
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
    let graph = super::ruff_util::ruff_graph(true, false);

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
