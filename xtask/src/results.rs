use std::{fmt::Write as _, fs, io, path::Path};

use crate::{
    environment::read_cpu_from_metadata,
    scope::{ChartSpec, Scope},
    util::{Result, workspace_root},
};

pub fn aggregate_results() -> Result<()> {
    let workspace_root = workspace_root();
    for &scope in Scope::all() {
        write_scope_results_markdown(workspace_root, scope)?;
    }
    Ok(())
}

fn write_scope_results_markdown(workspace_root: &Path, scope: Scope) -> Result<()> {
    let path = scope.crate_results_readme(workspace_root);
    let hosts = list_hosts(workspace_root)?;

    let mut content = String::new();
    writeln!(&mut content, "# Benchmark Results")?;

    let mut has_any_chart = false;
    for chart in scope.charts() {
        let matching_hosts = list_hosts_with_chart(workspace_root, &hosts, chart)?;
        if matching_hosts.is_empty() {
            continue;
        }
        has_any_chart = true;

        writeln!(&mut content)?;
        writeln!(&mut content, "## {}", chart.title)?;
        for host in matching_hosts {
            writeln!(&mut content)?;
            writeln!(
                &mut content,
                "### [{}]({})",
                host.title,
                host_readme_markdown_path(&host.id)
            )?;
            writeln!(&mut content)?;
            let alt_text = format!("{} ({})", chart.title, host.title);
            writeln!(
                &mut content,
                "![{}]({})",
                alt_text,
                chart_markdown_path(&host.id, chart.dest_path)
            )?;
        }
    }

    if !has_any_chart {
        writeln!(&mut content)?;
        writeln!(&mut content, "_No results found yet._")?;
    }

    fs::write(path, content)?;
    Ok(())
}

#[derive(Debug, Clone)]
struct HostInfo {
    id: String,
    title: String,
}

fn list_hosts(workspace_root: &Path) -> io::Result<Vec<HostInfo>> {
    let results_root = workspace_root.join("results");
    if !results_root.is_dir() {
        return Ok(Vec::new());
    }

    let mut hosts = Vec::new();
    for entry in fs::read_dir(results_root)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        if !path.join("charts").is_dir() {
            continue;
        }
        let id = entry.file_name().to_string_lossy().into_owned();
        let title = read_host_title(&path).unwrap_or_else(|| id.clone());
        hosts.push(HostInfo { id, title });
    }

    hosts.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(hosts)
}

fn list_hosts_with_chart(
    workspace_root: &Path,
    hosts: &[HostInfo],
    chart: &ChartSpec,
) -> io::Result<Vec<HostInfo>> {
    let mut matched = Vec::new();
    for host in hosts {
        let chart_path = workspace_root
            .join("results")
            .join(&host.id)
            .join("charts")
            .join(chart.dest_path);
        if chart_path.is_file() {
            matched.push(host.clone());
        }
    }
    Ok(matched)
}

fn read_host_title(host_result_dir: &Path) -> Option<String> {
    if let Some(cpu) = read_cpu_from_metadata(host_result_dir) {
        return Some(cpu);
    }

    let readme = fs::read_to_string(host_result_dir.join("README.md")).ok()?;
    for line in readme.lines() {
        if let Some(value) = line.strip_prefix("- CPU:") {
            let value = value.trim();
            if !value.is_empty() {
                return Some(value.to_owned());
            }
        }
    }
    None
}

fn chart_markdown_path(host: &str, chart_file: &str) -> String {
    format!("../results/{host}/charts/{chart_file}")
}

fn host_readme_markdown_path(host: &str) -> String {
    format!("../results/{host}/README.md")
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::write_scope_results_markdown;
    use crate::scope::Scope;

    #[test]
    fn aggregate_uses_host_readme_link_and_alt_text() {
        let root = temp_dir("aggregate");
        fs::create_dir_all(root.join("results/apple-m1/charts")).expect("create charts dir");
        fs::create_dir_all(root.join("bench_hash")).expect("create bench_hash");

        let readme = "\
# Benchmark Results

## Environment

- CPU: Apple M1
";
        fs::write(root.join("results/apple-m1/README.md"), readme).expect("write readme");
        fs::write(
            root.join("results/apple-m1/environment.ini"),
            "[environment]\ncpu = Apple M1\n",
        )
        .expect("write meta");
        fs::write(
            root.join("results/apple-m1/charts/non_cryptographic_hash_lines_throughput.svg"),
            "<svg/>",
        )
        .expect("write chart");
        fs::write(
            root.join("results/apple-m1/charts/cryptographic_hash_lines_throughput.svg"),
            "<svg/>",
        )
        .expect("write chart");

        write_scope_results_markdown(&root, Scope::Hash).expect("aggregate should succeed");

        let output = fs::read_to_string(root.join("bench_hash/RESULTS.md")).expect("read output");
        assert!(output.contains("### [Apple M1](../results/apple-m1/README.md)"));
        assert!(output.contains("![Non-Cryptographic Hash Throughput (Apple M1)]"));
        assert!(output.contains("![Cryptographic Hash Throughput (Apple M1)]"));

        fs::remove_dir_all(root).expect("cleanup");
    }

    fn temp_dir(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos();
        std::env::temp_dir().join(format!("xtask-{prefix}-{nanos}"))
    }
}
