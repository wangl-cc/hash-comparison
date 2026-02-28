use std::{io, path::Path, process::Command};

use crate::{
    cli::{BenchOpts, RunOpts},
    scope::Scope,
    util::{Result, workspace_root},
};

pub fn run_benchmarks(args: &RunOpts) -> Result<()> {
    let workspace_root = workspace_root();
    for &scope in args.scope.to_scopes() {
        run_benchmark(scope, &args.bench_args, workspace_root)?;
    }
    Ok(())
}

pub fn run_benchmark(scope: Scope, bench_args: &BenchOpts, workspace_root: &Path) -> Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.current_dir(workspace_root)
        .arg("bench")
        .args(["-p", scope.bench_crate()])
        .args(["--bench", scope.bench_file()])
        .arg("--")
        .args(["--plotting-backend", "plotters"]);

    if bench_args.quick {
        cmd.args(["--warm-up-time", "1"]);
        cmd.args(["--measurement-time", "1"]);
    }

    cmd.args(&bench_args.bench_extra_args);

    println!("running {:?}", cmd);

    let status = cmd.status()?;
    if !status.success() {
        return Err(format!("benchmark command failed for scope: {scope}").into());
    }
    Ok(())
}

pub fn ensure_chart_artifacts(scope: Scope, workspace_root: &Path) -> Result<()> {
    for chart in scope.charts() {
        let source = workspace_root.join(chart.src_path);
        if !source.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!(
                    "missing artifact: {}\nrun with --run-bench or run benchmark manually first",
                    source.display()
                ),
            )
            .into());
        }
    }
    Ok(())
}
