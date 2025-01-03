extern crate alloc;

use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::process::Command;
use std::str;

use alloc::vec::Vec;
use regex::Regex;
use serde_json::Value;

fn path_to_module(path: &str) -> String {
    // this by no means fully PEP-compliant, and does not work for flat layouts or custom
    // package organization; it works for default src-layouts only
    let _module_path_with_extensions = path.replace("/__init__.py", "").replace("/", ".");
    let full_module_path: String = Regex::new(r"\.py$")
        .unwrap()
        .replace(&_module_path_with_extensions, "")
        .into();
    return match full_module_path.find("src.") {
        Some(src_index) => {
            let start_index = src_index + 4; // "src.".len()
            full_module_path[start_index..].to_string()
        }
        None => full_module_path,
    };
}

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
fn canonical_cycle<'a>(c: &[&'a str]) -> Vec<&'a str> {
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

fn ruff_graph_pkgs(repo_root: &String) -> HashMap<String, HashSet<String>> {
    let graph_output = Command::new("ruff")
        .args(["analyze", "graph", "--preview"])
        .current_dir(repo_root)
        .output()
        .expect("failed to execute process");

    let j: Value =
        serde_json::from_str::<Value>(str::from_utf8(&graph_output.stdout).unwrap()).unwrap();

    return j
        .as_object()
        .unwrap()
        .clone()
        .into_iter()
        // once https://github.com/astral-sh/ruff/issues/13431 is implemented, we don't need
        // path_to_module anymore
        .map(|(k, v)| {
            (
                path_to_module(&k),
                v.as_array()
                    .unwrap()
                    .clone()
                    .iter()
                    .map(|i| path_to_module(&i.as_str().unwrap()))
                    .collect::<HashSet<_>>(),
            )
        })
        .collect::<HashMap<_, _>>();
}

pub(crate) fn minimize_cycles(repo_root: String, cycles_results_file: String) {
    let graph = ruff_graph_pkgs(&repo_root);

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
        if emsmallen.is_some() {
            let (i, j, _) = emsmallen.unwrap();
            minimal_cycles.push(sub_cycle(&cycle, i, j));
        } else {
            minimal_cycles.push(cycle);
        }
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
}
