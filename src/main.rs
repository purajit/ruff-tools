mod minimal_cycles;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct MinimizeCyclesOptions {
    #[structopt(short, long)]
    repo_root: String,

    #[structopt(short, long)]
    cycle_results_file: String,
}

#[derive(StructOpt, Debug)]
enum RuffTools {
    #[structopt(name = "minimize-cycles")]
    MinimizeCycles(MinimizeCyclesOptions),
    ImportLinter {},
    CycleDetection {},
}

fn main() {
    let options = RuffTools::from_args();
    match options {
        RuffTools::MinimizeCycles(cmd) => {
            minimal_cycles::minimize_cycles(cmd.repo_root, cmd.cycle_results_file)
        }
        _ => (),
    }
}
