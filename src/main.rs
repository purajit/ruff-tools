mod cycle_detection;
mod live;
mod minimize_cycles;
mod ruff_util;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct MinimizeCyclesOptions {
    #[structopt(short, long)]
    cycle_results_file: String,
}

#[derive(StructOpt, Debug)]
pub struct CycleDetectionOptions {}

#[derive(StructOpt, Debug)]
pub struct LiveOptions {
    cmd: Vec<String>,
    #[structopt(default_value = "", short, long)]
    paths: String,
}

#[derive(StructOpt, Debug)]
enum RuffTools {
    #[structopt(name = "detect-cycles")]
    CycleDetection(CycleDetectionOptions),
    #[structopt(name = "minimize-cycles")]
    MinimizeCycles(MinimizeCyclesOptions),
    #[structopt(name = "lint-imports")]
    ImportLinter {},
    #[structopt(name = "live")]
    Live(LiveOptions),
}

// #[structopt(short, long)]
// detect_string_imports: bool,

fn main() {
    let options = RuffTools::from_args();
    match options {
        RuffTools::MinimizeCycles(cmd) => minimize_cycles::minimize_cycles(cmd.cycle_results_file),
        RuffTools::CycleDetection(_) => cycle_detection::detect_cycles(),
        RuffTools::Live(cmd) => {
            let _ = live::run_watcher(cmd.cmd, cmd.paths);
        }
        _ => (),
    }
}
