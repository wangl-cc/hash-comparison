mod bench;
mod cli;
mod collect;
mod environment;
mod results;
mod scope;
mod util;

use clap::Parser;

use crate::{cli::Command, util::Result};

fn run() -> Result<()> {
    let cmd = Command::parse();
    match cmd {
        Command::Run(args) => bench::run_benchmarks(&args),
        Command::Collect(args) => collect::collect_results(args),
        Command::Aggregate => results::aggregate_results(),
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
