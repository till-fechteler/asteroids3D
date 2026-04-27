//! Application-wide tracing subscriber + panic-hook-to-file wiring.
//! Installed from main.rs before App::new() per architecture.md:278-281.

use std::{backtrace::Backtrace, fs::OpenOptions, io::Write, panic, path::PathBuf, sync::Mutex};

use directories::BaseDirs;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

const LOG_FILE_NAME: &str = "asteroids3D.log";
const DEFAULT_FILTER: &str = "info";

/// Resolve the per-OS user-log-directory.
///
/// Windows: `%APPDATA%\asteroids3D\logs\`
/// Linux:   `$XDG_STATE_HOME/asteroids3d/logs/` (fallback `~/.local/state/asteroids3d/logs/`)
/// macOS:   `~/Library/Logs/asteroids3D/`
fn resolve_log_dir() -> Option<PathBuf> {
    let base = BaseDirs::new()?;

    #[cfg(target_os = "macos")]
    {
        Some(base.home_dir().join("Library/Logs/asteroids3D"))
    }

    #[cfg(target_os = "linux")]
    {
        let state = base
            .state_dir()
            .map(PathBuf::from)
            .unwrap_or_else(|| base.home_dir().join(".local/state"));
        Some(state.join("asteroids3d").join("logs"))
    }

    #[cfg(target_os = "windows")]
    {
        Some(base.data_dir().join("asteroids3D").join("logs"))
    }
}

/// Initialize the tracing subscriber (stderr + optional file) and panic hook.
/// Returns the log-file path when file logging is active, else `None` (stderr only).
pub fn init_logging() -> Option<PathBuf> {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(DEFAULT_FILTER));

    let stderr_layer = fmt::layer().with_writer(std::io::stderr).with_ansi(true);

    let file_open = resolve_log_dir().and_then(|dir| match std::fs::create_dir_all(&dir) {
        Ok(()) => {
            let path = dir.join(LOG_FILE_NAME);
            match OpenOptions::new().create(true).append(true).open(&path) {
                Ok(file) => Some((path, file)),
                Err(e) => {
                    eprintln!("logging: failed to open log file {}: {e}", path.display());
                    None
                }
            }
        }
        Err(e) => {
            eprintln!("logging: failed to create log dir {}: {e}", dir.display());
            None
        }
    });

    match file_open {
        Some((path, file)) => {
            let file_layer = fmt::layer().with_writer(Mutex::new(file)).with_ansi(false);
            tracing_subscriber::registry()
                .with(filter)
                .with(stderr_layer)
                .with(file_layer)
                .init();
            install_panic_hook(path.clone());
            Some(path)
        }
        None => {
            eprintln!("logging: file logging unavailable; stderr only.");
            tracing_subscriber::registry()
                .with(filter)
                .with(stderr_layer)
                .init();
            None
        }
    }
}

fn install_panic_hook(log_path: PathBuf) {
    let prev = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        if let Ok(mut file) = OpenOptions::new().append(true).open(&log_path) {
            let backtrace = Backtrace::capture();
            let _ = writeln!(file, "PANIC: {info}\nBacktrace:\n{backtrace}");
            let _ = file.flush();
        }
        prev(info);
    }));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_log_dir_yields_expected_suffix() {
        let dir = resolve_log_dir().expect("BaseDirs should resolve on the test host");
        let as_str = dir.to_string_lossy();

        #[cfg(target_os = "macos")]
        assert!(
            as_str.ends_with("Library/Logs/asteroids3D"),
            "macOS log dir mismatch: {as_str}"
        );

        #[cfg(target_os = "linux")]
        assert!(
            as_str.ends_with("asteroids3d/logs"),
            "Linux log dir mismatch: {as_str}"
        );

        #[cfg(target_os = "windows")]
        assert!(
            as_str.ends_with(r"asteroids3D\logs"),
            "Windows log dir mismatch: {as_str}"
        );
    }
}
