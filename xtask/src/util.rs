use std::{
    fmt,
    path::Path,
    process::{Command, ExitStatus},
};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn workspace_root() -> &'static Path {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.parent().expect("workspace root should exist")
}

pub fn run_capture(program: &str, args: &[&str]) -> Option<String> {
    match run_capture_checked(program, args) {
        Ok(stdout) => Some(stdout),
        Err(error) => {
            if std::env::var_os("XTASK_DEBUG").is_some() {
                eprintln!("[xtask] {error}");
            }
            None
        }
    }
}

pub fn run_capture_checked(program: &str, args: &[&str]) -> Result<String> {
    let output =
        Command::new(program)
            .args(args)
            .output()
            .map_err(|error| CommandCaptureError::Spawn {
                program: program.to_owned(),
                source: error,
            })?;

    if !output.status.success() {
        return Err(CommandCaptureError::NonZero {
            program: format_command(program, args),
            status: output.status,
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        }
        .into());
    }

    String::from_utf8(output.stdout)
        .map(|s| s.trim().to_owned())
        .map_err(|error| {
            CommandCaptureError::Decode {
                program: format_command(program, args),
                source: error,
            }
            .into()
        })
}

pub fn slugify(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut start = true;
    let mut prev_dash = false;
    for ch in input.trim().to_lowercase().chars() {
        match ch {
            ' ' | '-' | '_' => {
                if !start && !prev_dash {
                    out.push('-');
                    start = false;
                    prev_dash = true;
                }
            }
            _ => {
                out.push(ch);
                start = false;
                prev_dash = false;
            }
        }
    }
    out
}

fn format_command(program: &str, args: &[&str]) -> String {
    if args.is_empty() {
        return program.to_owned();
    }
    format!("{program} {}", args.join(" "))
}

#[derive(Debug)]
enum CommandCaptureError {
    Spawn {
        program: String,
        source: std::io::Error,
    },
    NonZero {
        program: String,
        status: ExitStatus,
        stderr: String,
    },
    Decode {
        program: String,
        source: std::string::FromUtf8Error,
    },
}

impl fmt::Display for CommandCaptureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Spawn { program, source } => {
                write!(f, "failed to run `{program}`: {source}")
            }
            Self::NonZero {
                program,
                status,
                stderr,
            } => {
                if stderr.is_empty() {
                    write!(f, "command `{program}` failed with status {status}")
                } else {
                    write!(
                        f,
                        "command `{program}` failed with status {status}: {stderr}"
                    )
                }
            }
            Self::Decode { program, source } => {
                write!(f, "failed to decode stdout of `{program}`: {source}")
            }
        }
    }
}

impl std::error::Error for CommandCaptureError {}

#[cfg(test)]
mod tests {
    use super::{run_capture, run_capture_checked, slugify};

    #[test]
    fn slugify_merges_separators() {
        assert_eq!(slugify(" Apple  M1 -- Pro "), "apple-m1-pro");
    }

    #[test]
    fn run_capture_checked_includes_stderr() {
        let err = run_capture_checked("sh", &["-c", "echo oops >&2; exit 7"])
            .expect_err("command should fail");
        let msg = err.to_string();
        assert!(msg.contains("status"), "missing status: {msg}");
        assert!(msg.contains("oops"), "missing stderr: {msg}");
    }

    #[test]
    fn run_capture_returns_none_on_failure() {
        let output = run_capture("sh", &["-c", "exit 3"]);
        assert!(output.is_none());
    }
}
