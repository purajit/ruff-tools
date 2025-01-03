use regex::Regex;
use std::collections::HashMap;
use std::collections::HashSet;
use std::process::Command;
use std::str;

use serde_json::Value;

pub(crate) fn ruff_graph(as_pkgs: bool, as_dependents: bool) -> HashMap<String, HashSet<String>> {
    let graph_output = Command::new("ruff")
        .args(["analyze", "graph", "--preview"])
        .args(if as_dependents {
            vec![]
        } else {
            vec!["--direction", "dependents"]
        })
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
            if as_pkgs {
                (
                    path_to_module(&k),
                    v.as_array()
                        .unwrap()
                        .into_iter()
                        .map(|i| path_to_module(&i.as_str().unwrap()))
                        .collect::<HashSet<_>>(),
                )
            } else {
                (
                    k,
                    v.as_array()
                        .unwrap()
                        .into_iter()
                        .map(|i| i.as_str().unwrap().to_string())
                        .collect::<HashSet<_>>(),
                )
            }
        })
        .collect::<HashMap<_, _>>();
}

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
