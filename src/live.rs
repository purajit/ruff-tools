use notify::event::{CreateKind, ModifyKind, RemoveKind};
use notify::EventKind::{Create, Modify, Remove};
use notify::{Event, RecursiveMode, Result as WatcherResult, Watcher};
use std::collections::{HashMap, HashSet, VecDeque};
use std::env;
use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc;

pub(crate) fn run_watcher(
    cmd: Vec<String>,
    paths_glob: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let paths: Vec<_> = paths_glob.split(",").collect();
    let cmd_name = OsStr::new(cmd.first().expect("Command must be provided"));
    let cmd_args = &cmd[1..];

    let cwd = env::current_dir()?.to_string_lossy().into_owned() + "/";

    let (tx, rx) = mpsc::channel::<WatcherResult<Event>>();
    let mut watcher = notify::recommended_watcher(tx)?;

    // maintaining both a dependents graph and dependency graph since:
    // * the dependents graph directly powers the basic functionality
    // * the dependency graph allows us to monitor which edges were removed in a
    //   file change without traversing the entire graph
    println!("\x1b[93mConstructing initial graph ...\x1b[0m");
    let mut import_map_dependents = super::ruff_util::ruff_graph(false, false, None);
    let mut import_map_dependencies = super::ruff_util::ruff_graph(false, true, None);

    watcher.watch(Path::new("."), RecursiveMode::Recursive)?;
    println!("\x1b[93mListening! Ctrl-C to quit.\x1b[0m");
    for res in rx {
        match res {
            Ok(event) => match event.kind {
                Modify(ModifyKind::Name(_))
                | Modify(ModifyKind::Data(_))
                | Create(CreateKind::File)
                | Remove(RemoveKind::File) => {
                    // we only want to rerun analyze on files that changed, specifically either
                    // files already tracked by the import map, or if they're python files, or
                    // if it's a project/ruff configuration
                    let changed_paths = event
                        .paths
                        .into_iter()
                        .filter_map(|p| {
                            let sp = p.to_str().unwrap();
                            // a non-python file might be a dependent explicitly declared
                            // `include-dependencies`; if so, we want to track its changes
                            if import_map_dependents.contains_key(sp)
                                // there might be a new python file
                                || sp.ends_with(".py")
                                // or a change to the config itself
                                || sp.ends_with("ruff.toml")
                                || sp.ends_with(".ruff.toml")
                                || sp.ends_with("pyproject.toml")
                            {
                                return Some(sp.strip_prefix(&cwd).unwrap().to_string());
                            }
                            None
                        })
                        .collect::<Vec<String>>();

                    if changed_paths.is_empty() {
                        continue;
                    }

                    println!("Changed paths: {}", changed_paths.join(", "));

                    // if a file has been removed, first find the impacted files before changing the
                    // import map and losing that information; otherwise, we update the graph first -
                    // even if there are removed edges, we can still evaluate with the updated graph
                    // because for a file to be impacted by it, there must be some file in its path
                    // (possibly itself) that was modified, which will still trigger it
                    if event.kind != Remove(RemoveKind::Any) {
                        // TODO: if config file changed, reconstruct entire graph; this could be
                        // optimized by just adding new edges from include-dependencies, but
                        // in pathological cases, `src` and such might be modified as well
                        let import_map_dependencies_update =
                            super::ruff_util::ruff_graph(false, false, Some(changed_paths.clone()));

                        for (path, new_dependencies) in import_map_dependencies_update.iter() {
                            let old_dependencies = import_map_dependencies
                                .insert(path.clone(), new_dependencies.clone());
                            // handle removed edges
                            if old_dependencies.is_some() {
                                for m in old_dependencies.unwrap().difference(new_dependencies) {
                                    if import_map_dependents.contains_key(m) {
                                        import_map_dependents.entry(m.clone()).and_modify(|curr| {
                                            curr.remove(path);
                                        });
                                    }
                                }
                            }
                            // add new edges
                            for m in new_dependencies.iter() {
                                let values = import_map_dependents.entry(m.clone()).or_default();
                                values.insert(path.clone());
                            }
                        }
                    }

                    let affected_files =
                        get_affected_files(&changed_paths, import_map_dependents.clone())
                            .into_iter()
                            .filter(|p| {
                                import_map_dependents.contains_key(p)
                                    && paths.iter().any(|args_path| p.starts_with(args_path))
                            })
                            .collect::<Vec<String>>();

                    if event.kind == Remove(RemoveKind::File) {
                        // remove node and all edges to it in both graphs
                        for p in changed_paths.into_iter() {
                            let _ = import_map_dependents.remove(&p);
                            let old_dependencies = import_map_dependencies.remove(&p);
                            if old_dependencies.is_some() {
                                for m in old_dependencies.unwrap().iter() {
                                    import_map_dependents.entry(m.clone()).and_modify(|curr| {
                                        curr.remove(&p);
                                    });
                                }
                            }
                        }
                    }

                    if affected_files.is_empty() {
                        println!("\x1b[93mNothing to do!\x1b[0m");
                        continue;
                    }

                    println!("Transitively affected files: {}", affected_files.join(", "));
                    println!();
                    println!("\x1b[93mRUNNING COMMAND!\x1b[0m");
                    println!();
                    Command::new(cmd_name)
                        .args(cmd_args)
                        .args(affected_files)
                        .status()
                        .expect("failed to execute process");
                    println!();
                    println!("\x1b[93mCOMPLETED RUN!\x1b[0m");
                    println!();
                }
                _ => continue,
            },
            Err(_) => continue,
        };
    }
    Ok(())
}

fn get_affected_files(
    modified_files: &[String],
    import_map_dependents: HashMap<String, HashSet<String>>,
) -> HashSet<String> {
    // run a plain BFS of the dependents graph; all visited nodes are affected files
    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<String> = VecDeque::new();
    visited.extend(modified_files.to_owned());
    queue.extend(modified_files.to_owned());
    while let Some(file) = queue.pop_front() {
        match import_map_dependents.get(&file) {
            Some(mi) => {
                for dependent_file in mi.iter() {
                    if visited.insert(dependent_file.clone()) {
                        queue.push_back(dependent_file.clone());
                    }
                }
            }
            None => continue,
        }
    }

    visited
}
