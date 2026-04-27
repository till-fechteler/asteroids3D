# Story 1.8: Tracing-Based Logging with Panic Hook to Log File

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a developer,
I want `tracing`-based logging with a panic hook that writes stack traces to a log file in the user-log-dir,
So that crashes during CI runs or future playtesting can be forensically reviewed after process exit.

## Acceptance Criteria

1. **Given** `tracing_subscriber` is initialized in `main.rs` before `App::new()`
   **When** the app runs
   **Then** `info!` / `warn!` / `error!` events from Bevy and app code are output to stderr
   **And** `RUST_LOG=debug cargo run` increases verbosity to `debug!` level

2. **Given** the `directories` crate resolves the per-OS user-log-dir (Windows `%LOCALAPPDATA%\asteroids3D\logs\` _(amended 2026-04-27 from `%APPDATA%` per code-review decision: logs should not roam)_, Linux `$XDG_STATE_HOME/asteroids3d/logs/` or fallback, macOS `~/Library/Logs/asteroids3D/`)
   **When** a log file is opened at startup
   **Then** logs are written to both stderr and the file simultaneously

3. **Given** a panic hook is installed via `std::panic::set_hook`
   **When** a panic is triggered (e.g., via a `#[cfg(test)]` panic test or manual `panic!()` in a dev-only build)
   **Then** the panic message and backtrace are written to the log file before process exit
   **And** the default panic behavior (printing to stderr) is preserved

## Tasks / Subtasks

- [x] **Task 1: Create `src/logging.rs` module** (AC: #1, #2, #3)
  - [x] New file `src/logging.rs` — ~110–140 lines: `resolve_log_dir()` + `init_logging()` + `install_panic_hook()` + 1–2 unit tests. Module doc ≤2 lines, no story-id references.
  - [x] `use` block: `std::{fs::{File, OpenOptions}, io::Write, path::PathBuf, sync::Mutex};` + `directories::BaseDirs;` + `tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};`.
  - [x] `const LOG_FILE_NAME: &str = "asteroids3D.log";` + `const DEFAULT_FILTER: &str = "info";` hoisted at module top.
  - [x] `fn resolve_log_dir() -> Option<PathBuf>` — per-OS path resolver using `BaseDirs::new()` + `#[cfg(target_os = "...")]` branches (see Per-OS log-directory resolution below).
  - [x] `pub fn init_logging() -> Option<PathBuf>` — entry point. Builds the subscriber with `EnvFilter` (RUST_LOG or `"info"`), stderr `fmt::layer()` (with ANSI), and a file `fmt::layer().with_ansi(false).with_writer(Mutex::new(file))` when the log file opens successfully. On any path-resolve / mkdir / open failure, emits `eprintln!` warning and proceeds with stderr-only subscriber (graceful degradation per architecture.md:368). Installs panic hook only when file logging is available (else panic hook has nowhere to write file output and default stderr hook is retained). Returns `Some(path)` on file-logging success, `None` on stderr-only fallback.
  - [x] `fn install_panic_hook(log_path: PathBuf)` — captures previous hook via `std::panic::take_hook()`, installs a new one that (a) opens `log_path` in append mode, (b) writes `PANIC: {info}\nBacktrace:\n{backtrace}` where `backtrace = std::backtrace::Backtrace::capture()`, (c) flushes, (d) calls `prev(info)` to preserve default stderr hook.
  - [x] Unit test `resolve_log_dir_yields_expected_suffix` — asserts `Some(path)` and that the trailing segments match the current-OS AC spec (`asteroids3D/logs` on Windows, `asteroids3d/logs` on Linux, `Library/Logs/asteroids3D` on macOS). Uses `#[cfg(target_os = "…")]` guards to pick the assertion branch.

- [x] **Task 2: Wire logging into `src/main.rs`** (AC: #1, #2, #3)
  - [x] Add `mod logging;` below `mod splash;` and `mod state;` (alphabetical order preserved by rustfmt).
  - [x] Add `use logging::init_logging;` to the top-level `use` block.
  - [x] At the top of `fn main() -> AppExit`, call `let _log_path = init_logging();` **before** `App::new()`. Lead-bind underscore to signal intentional discard (file path is written to log on first info! line anyway).
  - [x] Disable Bevy's `LogPlugin`: `App::new().add_plugins(DefaultPlugins.build().disable::<bevy::log::LogPlugin>())`. See "Bevy LogPlugin conflict" below — this prevents Bevy from trying to install its own global tracing subscriber.
  - [x] `fn main() -> AppExit` signature preserved from 1.6/1.7. `.run()` trailing expression, no `;`.
  - [x] Update module `//!` doc to 3 lines: current 2 lines + one line mentioning logging/panic-hook install.

- [x] **Task 3: Local verification sweep** (AC: #1, #2, #3)
  - [x] `cargo check 2>&1 | tee /tmp/story-1-8-check.log` → `grep -cE 'warning:|error:' /tmp/story-1-8-check.log` must equal **0**.
  - [x] `cargo build 2>&1 | tee /tmp/story-1-8-build.log` → same grep equals **0**.
  - [x] `cargo test 2>&1 | tee /tmp/story-1-8-test.log` → `grep -cE 'warning:|error:|FAILED' /tmp/story-1-8-test.log` equals **0**; should now show **3 passed** (state default + splash config default + new logging test).
  - [x] `cargo clippy --all-targets -- -D warnings 2>&1 | tee /tmp/story-1-8-clippy.log` → grep equals **0**.
  - [x] `cargo fmt --all -- --check` → exit 0. If it fails, run `cargo fmt --all` and re-check.
  - [x] `cargo run &> /tmp/story-1-8-run-default.log &` then close window after splash; verify:
    - `grep -c 'entered Loading' /tmp/story-1-8-run-default.log` ≥ **1**.
    - `grep -c 'splash timer elapsed' /tmp/story-1-8-run-default.log` ≥ **1**.
    - `grep -c 'entered MainMenu' /tmp/story-1-8-run-default.log` ≥ **1**.
    - `ls ~/Library/Logs/asteroids3D/asteroids3D.log` (macOS) exists with non-zero size.
    - Log file contains `entered Loading`, `splash timer elapsed`, `entered MainMenu` signals (same three from stderr).
    - `grep -c 'DEBUG ' /tmp/story-1-8-run-default.log` should equal **0** (default filter = info).
  - [x] `RUST_LOG=debug cargo run &> /tmp/story-1-8-run-debug.log &` then close window after splash; verify:
    - `grep -c 'DEBUG ' /tmp/story-1-8-run-debug.log` ≥ **1** (some debug line, whether from Bevy or app).
    - Same three info lifecycle signals still present.
  - [x] **Panic hook verification:** temporarily add a `panic!("bmad panic test");` line to `src/logging.rs`'s `init_logging()` AFTER the subscriber + hook are set up but BEFORE returning. Run `RUST_BACKTRACE=1 cargo run &> /tmp/story-1-8-panic.log` — expect immediate panic before window opens.
    - `grep -c 'PANIC: ' ~/Library/Logs/asteroids3D/asteroids3D.log` ≥ **1**.
    - `grep -c 'Backtrace:' ~/Library/Logs/asteroids3D/asteroids3D.log` ≥ **1**.
    - Backtrace contains at least one non-empty stack frame line (e.g., `at src/logging.rs:`).
    - Default panic-stderr output visible in `/tmp/story-1-8-panic.log` (`thread 'main' panicked at ...`).
    - **Revert the `panic!(...)` line** before committing. `grep -n 'panic!' src/logging.rs` must return **0** in the final diff.
  - [x] Capture all hit counts + log file path + sample log lines into the Debug Log References table below.

- [x] **Task 4: Scope guardrails — verify nothing else drifted** (AC: #1, #2, #3)
  - [x] `git status --short`: only `src/main.rs` (M), `src/logging.rs` (??), plus bookkeeping `sprint-status.yaml` (M) and this story file (??). **No** `Cargo.toml` / `Cargo.lock` changes — `tracing-subscriber` (with `env-filter`) and `directories` are already pinned from Story 1.1 (Cargo.toml:19-20).
  - [x] `grep -nrE 'Arena|Caravan|PostRun|PhotoMode|Paused' src/ --include='*.rs' | grep -v 'state.rs'` → **0** hits.
  - [x] `grep -rn 'ui::\|main_menu\|string.table\|en\.ron' src/` → **0** hits. Epic 3+ scope.
  - [x] `grep -rn 'tracing_appender\|non_blocking' src/` → **0** hits. Log-file rolling/async is not this story (no such crate pinned; manual `Mutex<File>` is the path).
  - [x] `grep -rn 'panic!' src/` → **0** hits in the final diff (temporary panic-test line must be reverted).
  - [x] `.gitignore`, `.github/workflows/ci.yml`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`, `docs/plugin-compatibility.md` — all untouched.
  - [x] `deferred-work.md` untouched for this task; any review findings get logged in Task 7's review phase.

- [x] **Task 5: Commit** (AC: #1, #2, #3)
  - [x] Stage: `src/main.rs` (M), `src/logging.rs` (new).
  - [x] Commit message (HEREDOC): `feat: tracing subscriber + panic hook → user-log-dir file (Story 1.8)` — single-line, sub-70-char, `feat:` prefix, NO `Co-Authored-By` trailer (matches Story 1.1–1.7 pattern).
  - [x] Push to `origin/master`.

- [x] **Task 6: CI observation** (AC: #1, #2, #3)
  - [x] `gh run list -L 1` identifies the new run ID triggered by the source-touching commit.
  - [x] Wait for all 4 jobs to complete (msrv-check, build macOS, build ubuntu, build windows). Expected wall time: ~10–12 m if Cargo.lock unchanged (warm cache); longer if it somehow regenerated.
  - [x] `gh run view <ID> --log | grep -cE 'warning:|error:'` → expect **0**.
  - [x] All 4 jobs ✅; capture run ID + per-job durations into Debug Log References.

- [x] **Task 7: Ready-for-review handoff + bookkeeping commit**
  - [x] Populate **Dev Agent Record** sections of this file: Agent Model Used, Debug Log References (per-command hit-counts + sample log lines + CI run ID), Completion Notes List (per-AC evidence + any deviations), File List (added / modified / untouched-guardrail).
  - [x] Set this story's `Status:` header → `review`.
  - [x] Update `_bmad-output/implementation-artifacts/sprint-status.yaml`: flip `1-8-tracing-based-logging-with-panic-hook-to-log-file: backlog → in-progress → review`; bump `last_updated` to current date.
  - [x] Stage this story file + `sprint-status.yaml`, commit with `bmad: story 1.8 ready-for-dev → review (logging + panic-hook shipped, CI green)` or similar `bmad:` prefix. This is a `_bmad-output/**` only commit → CI `paths-ignore` suppresses the matrix (per `.github/workflows/ci.yml:9-15`); expected zero-CI-run.
  - [x] Push.
  - [x] Story awaits code review; review can be light (single-reviewer precedent from 1.6/1.7) given ~150-line diff with no UI / physics / save-I/O implications, OR a full 3-agent adversarial review if the dev agent suspects edge cases in the panic hook / subscriber layering.

## Dev Notes

### Why this story exists

Story 1.8 completes Epic 1. It installs the **first non-gameplay cross-cutting concern** in the project — observability — ahead of any system that will benefit from it. Architecture.md:278-281 and :373-383 mandate: `tracing` + `tracing-subscriber`, `RUST_LOG` env-var support, per-OS log file via `directories`, panic hook writing to the log file. Getting it landed at M0 means every subsequent story (Epic 2 visual spike, Epic 3 first combat, …) already emits structured, file-persisted logs — crash forensics and regression-debug evidence are available from the first moment they're needed.

Story 1.7 took the first `Loading → MainMenu` transition and the first bevy_ui surface; Story 1.8 takes the first custom Bevy plugin swap (disabling `LogPlugin`) and the first interaction with the `directories` crate. Both crates (`tracing-subscriber 0.3 [env-filter]`, `directories 5`) are already pinned at Story 1.1; this story is the first time they're actually used in code.

After Story 1.8, Epic 1 is closed. All 8 stories done → M0 completion criterion satisfied (architecture.md:294 + architecture.md:994). Next work: Epic 2 Vector Aesthetic Tech Spike. Epic 1 retrospective (`epic-1-retrospective: optional`) is available but not gated.

### Context inherited from Stories 1.1–1.7

| Fact | Value | Source |
|---|---|---|
| Rust toolchain | `1.94.1` stable (pinned) | `rust-toolchain.toml` |
| MSRV | `1.89` (CI-verified on `ubuntu-latest`) | `Cargo.toml:5` |
| Bevy | `0.18` (features `3d, png, bevy_ui, default_font`) | `Cargo.toml:8,26` |
| `tracing` + `tracing-subscriber` | `tracing = "0.1"`, `tracing-subscriber = "0.3"` with `env-filter` feature | `Cargo.toml:18-19` |
| `directories` | `5` | `Cargo.toml:20` |
| `src/main.rs` body (post-1.7) | 29 lines — `App::new().add_plugins(DefaultPlugins).init_state::<GameState>().init_resource::<SplashConfig>().add_systems(OnEnter(Loading), (log_loading_entered, spawn_splash)).add_systems(OnEnter(MainMenu), log_mainmenu_entered).add_systems(Update, tick_splash_timer.run_if(in_state(Loading))).add_systems(OnExit(Loading), cleanup_loading_entities).run()` | Post-1.7 |
| `src/splash.rs` / `src/state.rs` | Owned by prior stories; this story does NOT touch them | Post-1.7 |
| Tests in project | 2 (`state::tests::default_state_is_loading`, `splash::tests::splash_config_default_is_two_seconds`) | Post-1.7 |
| CI | 4-job matrix (msrv-check + 3 OS build), all green on master; `paths-ignore` skips `_bmad/` + `_bmad-output/` only commits | `.github/workflows/ci.yml` |
| Commit convention | Single-line subject; `feat:` / `chore:` / `docs:` / `fix:` / `ci:` / `bmad:` prefixes; **NO** `Co-Authored-By` trailer | `git log --oneline -n 15` |
| Local dev machine | macOS 26.4.1 / arm64 (Apple M5 Pro) | Prior story Debug Logs |
| Default logging (pre-1.8) | Bevy's built-in `LogPlugin` writes to stderr only, installs its own tracing subscriber with filter `wgpu=error,naga=warn,info` | Bevy 0.18 default |

### Bevy LogPlugin conflict — why we disable it

Bevy 0.18's `DefaultPlugins` includes `LogPlugin`, which constructs a `tracing_subscriber::Registry` and calls `set_global_default(...)`. Rust's `tracing` API allows **exactly one** global default subscriber per process. AC #1 mandates our subscriber is initialized in `main.rs` **before** `App::new()`, so ours wins — but then Bevy's LogPlugin will either panic (on `set_global_default` conflict) or silently fail to install its preferred filter.

**Resolution:** disable `LogPlugin` explicitly at the `DefaultPlugins` level:
```rust
.add_plugins(DefaultPlugins.build().disable::<bevy::log::LogPlugin>())
```
This tells Bevy's plugin builder to skip `LogPlugin`'s `build()` pass entirely. Our subscriber becomes the only tracing consumer. Bevy's `info!` / `warn!` / `error!` macros (which are just re-exports of `tracing::info!` etc.) still emit events; they flow through our subscriber.

**Side-effect accepted:** Bevy's LogPlugin also sets up optional `tracing-chrome` / `tracy-client` integrations behind cargo features. None of those features are currently enabled in `Cargo.toml`, so disabling LogPlugin removes nothing we use. When Story 2.x+ (or an M2 debug-panels story) wants tracy integration, the dev-only profiling subscriber can be added as an additional `Layer` to our Registry stack rather than re-enabling LogPlugin.

**Alternative considered and rejected:** keep LogPlugin + pass a `custom_layer` callback. LogPlugin 0.18's `custom_layer` field takes `fn(&mut App) -> Option<BoxedLayer>`; we'd have to give LogPlugin our stderr+file+filter stack as a single layer, while letting Bevy still own subscriber init. Rejected because AC #1 is explicit that **we** own init, in `main.rs`, **before** App::new(). Hybrid ownership creates subtle questions about whether `RUST_LOG` overrides land pre- or post-App startup. Clean separation (we init; LogPlugin off) matches the AC literally and is the simplest mental model.

### tracing-subscriber 0.3 Registry-with-layers architecture

Canonical layered subscriber for this story:
```rust
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(DEFAULT_FILTER));

let stderr_layer = fmt::layer()
    .with_writer(std::io::stderr)
    .with_ansi(true);  // colors on terminal

let file_layer = fmt::layer()
    .with_writer(Mutex::new(file))  // Mutex<File> implements MakeWriter
    .with_ansi(false);               // strip colors from file output

tracing_subscriber::registry()
    .with(filter)
    .with(stderr_layer)
    .with(file_layer)
    .init();
```

Key notes:
- `tracing_subscriber::fmt::Layer` implements `tracing_subscriber::layer::Layer<S>`. Stacking via `.with(...)` accumulates them into the `Registry`. All layers see the same event stream, each routes independently to its writer.
- `EnvFilter::try_from_default_env()` reads `RUST_LOG` once at init. Fallback on any parse error (including unset var) → `EnvFilter::new("info")` → AC #1's "default verbosity = info".
- `Mutex<File>` has `impl<'a, W: Write + 'a> MakeWriter<'a> for Mutex<W>` in `tracing_subscriber 0.3` → direct use as `with_writer(...)` argument. No `tracing-appender` needed.
- `.with_ansi(false)` on the file layer strips VT100 escape codes; without it the file is polluted with `\x1b[...m` sequences, hurting readability and grep-friendliness.

**Trade-off accepted (file I/O synchronicity):** `Mutex<File>` writes synchronously from each logging thread. Bevy emits high-frequency `info!`/`debug!` from the main thread only; there's no cross-thread contention in our current code. Log volume is small (lifecycle events + occasional warnings). Synchronous writes add a few µs per emission — imperceptible. If log volume grows (e.g., `debug!` at 60 Hz from per-frame systems), switch to `tracing-appender::non_blocking` (adds a crate dep, spawns a writer thread). Deferred until measurable impact.

**Log file rotation / size management:** deferred. Single `asteroids3D.log` file, append mode. Between dev sessions, the user can rotate manually (`mv asteroids3D.log asteroids3D.log.1`). Automatic rotation (`tracing-appender::rolling::daily`) is post-MVP if it becomes useful for playtesters.

**Session-start marker:** deliberately NOT a separate log line in `init_logging`. The first `info!` after init will be `entered Loading` (from Story 1.6's `log_loading_entered`). That serves as the session-start marker. Adding a dedicated `=== session start ===` line would be noise.

### Per-OS log-directory resolution

AC #2 specifies exact target paths:

| OS | Target path | `directories` 5 primitive | Notes |
|---|---|---|---|
| Windows | `%LOCALAPPDATA%\asteroids3D\logs\` | `BaseDirs::data_local_dir()` = `{FOLDERID_LocalAppData}` = `%LOCALAPPDATA%` | Append `\asteroids3D\logs`. _Amended 2026-04-27 from `data_dir()` (Roaming) per code-review decision — logs should not sync between machines via the roaming profile._ |
| Linux | `$XDG_STATE_HOME/asteroids3d/logs/` (fallback `$HOME/.local/state/asteroids3d/logs/`) | `BaseDirs::state_dir()` → `Option<&Path>`; `Some` on Linux, `None` on macOS/Windows | Fallback: `home_dir().join(".local/state")`. Project slug is **lowercase** `asteroids3d` per AC |
| macOS | `~/Library/Logs/asteroids3D/` | `BaseDirs::home_dir()` — macOS has no XDG concept; logs go under `~/Library/Logs/` by OS convention | `home_dir().join("Library/Logs/asteroids3D")` |

Project slug is:
- `asteroids3D` (mixed-case) on Windows and macOS — matches the package name and macOS `CFBundleName` convention (user-visible in Finder).
- `asteroids3d` (lowercase) on Linux — matches XDG "lowercase app-slug" community convention for hidden state dirs.

Exact reference implementation:
```rust
use directories::BaseDirs;
use std::path::PathBuf;

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
        // Amended 2026-04-27: data_local_dir() (LocalAppData) replaces data_dir() (Roaming) per code-review decision.
        Some(base.data_local_dir().join("asteroids3D").join("logs"))
    }
}
```

Why not `ProjectDirs::from(...).data_dir()` or `.state_dir()` everywhere? `ProjectDirs` auto-appends project qualifier + name, but:
- On Linux, `ProjectDirs::state_dir()` is `Option<&Path>` — returns None if XDG_STATE_HOME and the fallback-builder both fail. Plus, it adds a qualifier segment that doesn't match the AC's bare `asteroids3d/logs/` pattern.
- On macOS, there's no `log_dir()` on any of `ProjectDirs` / `BaseDirs` / `UserDirs`. `ProjectDirs::data_dir()` returns `~/Library/Application Support/asteroids3D/` — wrong target for logs per macOS conventions and AC.
- `BaseDirs` is the lower-level primitive that gives us `home_dir()` / `data_dir()` / `state_dir()` cleanly. Per-OS `cfg` branching on top of `BaseDirs` is the most explicit match to AC's literal spec.

If `BaseDirs::new()` returns `None` (extremely unusual — no `$HOME` env var set, or Windows without `USERPROFILE`), `resolve_log_dir()` returns `None`. Callers degrade to stderr-only logging + `eprintln!` warning.

### Panic hook design

Per AC #3:
1. **Trigger:** any `panic!()` / `unwrap()` on `None` / `expect()` failure / `array[out_of_bounds]` / etc. reaches the installed hook.
2. **Write panic message + backtrace to log file** before process exit.
3. **Preserve default stderr hook** (`thread '...' panicked at '...'` line is still visible on user's terminal).

Implementation:
```rust
use std::backtrace::Backtrace;
use std::panic;

fn install_panic_hook(log_path: PathBuf) {
    let prev = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        // 1. Write to log file first. On failure, fall through silently (we still have stderr via prev).
        if let Ok(mut file) = OpenOptions::new().append(true).open(&log_path) {
            let backtrace = Backtrace::capture();
            let _ = writeln!(file, "PANIC: {info}\nBacktrace:\n{backtrace}");
            let _ = file.flush();
        }
        // 2. Delegate to default (or previously installed) hook — preserves stderr output.
        prev(info);
    }));
}
```

Key details:
- `panic::take_hook()` captures Rust's default hook (which prints `thread '...' panicked at 'msg', file:line:col` to stderr). We re-invoke it after our file write → AC #3's "default panic behavior is preserved".
- `Backtrace::capture()` respects the `RUST_BACKTRACE` env var:
  - `RUST_BACKTRACE` unset → `Backtrace::captured()` is a `Backtrace` whose `Display` impl emits `"disabled backtrace"`.
  - `RUST_BACKTRACE=1` → readable frames.
  - `RUST_BACKTRACE=full` → frames with file:line info (where debuginfo is available).
  - Developer / CI runs should set `RUST_BACKTRACE=1` at minimum. Document this in a future README polish (Epic 10). Not this story's scope.
- `OpenOptions::new().append(true).open(&log_path)` — open fresh per panic rather than trying to reuse the fmt layer's `Mutex<File>` handle. Rationale: the panic hook executes in a context where locks may be poisoned; a fresh File handle is simplest and panic-safe. The file has been created by `init_logging` before the hook installs, so `open(append)` succeeds in the normal case.
- Only install the hook if `init_logging` successfully opened the log file (the returned `Option<PathBuf>` is `Some`). If log file is unavailable, the hook has nowhere to write → skip install → default hook alone handles stderr output, matching graceful-degradation policy (architecture.md:368).

**Rationale for hook order (file-first, then stderr-default):**
- File write is best-effort and may fail silently (e.g., disk full, permission race). In that case, the default hook still runs and user sees the panic on stderr.
- Default hook runs last → terminal `thread '...' panicked at '...'` line is the final thing the user sees, matching expected behavior.

### File structure — `src/logging.rs` vs future `src/core/`

Architecture.md:550-554 reserves `src/core/` for shared types (faction, damage, markers). It does NOT currently call out a logging location. Following Story 1.7's precedent (flat `src/splash.rs` now, promoted to `src/ui/splash.rs` at Epic 3 Story 3.1), this story places logging at flat `src/logging.rs`:
- Minimal dependency footprint — only `main.rs` imports it.
- No premature `src/core/` subtree scaffolding.
- Natural promotion window: when another cross-cutting concern (metrics? profiling?) arrives and logging becomes one of many `src/core/`-worthy modules. Likely M2 (debug-panels story) or M3 (persistence plugin — which also needs `directories` path helpers and might naturally subsume the per-OS path resolver).

**Alternative file name considered:** `src/logging/mod.rs` as a subtree. Rejected — single 100-line file, no sub-modules needed. Flat single file matches Story 1.6 `state.rs` + Story 1.7 `splash.rs` cadence.

**Future refactor hint:** when `src/persistence/paths.rs` lands (Epic 4 Story 4.6 per architecture.md:602), the per-OS path resolver in `logging.rs` should migrate there as a general `per_os_dir(Category::Logs | Category::SaveData)` helper. `logging.rs` then becomes subscriber + hook only. That's a post-1.8 refactor, not in-story scope.

### `src/logging.rs` skeleton

The dev agent can write this near-verbatim. Rustfmt will adjust whitespace; accept its output.

```rust
//! Application-wide tracing subscriber + panic-hook-to-file wiring.
//! Installed from main.rs before App::new() per architecture.md:278-281.

use std::{
    backtrace::Backtrace,
    fs::{File, OpenOptions},
    io::Write,
    panic,
    path::PathBuf,
    sync::Mutex,
};

use directories::BaseDirs;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

const LOG_FILE_NAME: &str = "asteroids3D.log";
const DEFAULT_FILTER: &str = "info";

/// Resolve the per-OS user-log-directory per AC #2 spec.
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
///
/// MUST be called once at startup, before `App::new()`, per AC #1.
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
                    eprintln!(
                        "logging: failed to open log file {}: {e}",
                        path.display()
                    );
                    None
                }
            }
        }
        Err(e) => {
            eprintln!(
                "logging: failed to create log dir {}: {e}",
                dir.display()
            );
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
```

Note on test:
- `init_logging` and `install_panic_hook` are NOT covered by unit tests — both mutate process-global state (global subscriber, global panic hook). Subscriber is init-once (`.init()` panics on second call) so a test-time init would break subsequent tests in the same process. These are exercised via runtime verification (Task 3 steps 6–9) rather than unit tests.
- The file-unused `File` import is removed from the final file if clippy flags it. Same for any unused `BufWriter` import the dev agent may have reached for.

### `src/main.rs` delta

Post-edit skeleton (accept rustfmt-canonical import ordering):

```rust
//! asteroids3D — app entry point.
//! Initializes tracing subscriber + panic-hook-to-file before Bevy startup.
//! Registers DefaultPlugins (minus LogPlugin), GameState, and the Loading → MainMenu splash flow.

use bevy::prelude::*;

mod logging;
mod splash;
mod state;

use logging::init_logging;
use splash::{SplashConfig, cleanup_loading_entities, spawn_splash, tick_splash_timer};
use state::{GameState, log_loading_entered, log_mainmenu_entered};

fn main() -> AppExit {
    let _log_path = init_logging();

    App::new()
        .add_plugins(DefaultPlugins.build().disable::<bevy::log::LogPlugin>())
        .init_state::<GameState>()
        .init_resource::<SplashConfig>()
        .add_systems(
            OnEnter(GameState::Loading),
            (log_loading_entered, spawn_splash),
        )
        .add_systems(OnEnter(GameState::MainMenu), log_mainmenu_entered)
        .add_systems(
            Update,
            tick_splash_timer.run_if(in_state(GameState::Loading)),
        )
        .add_systems(OnExit(GameState::Loading), cleanup_loading_entities)
        .run()
}
```

Net change: +1 `mod` decl, +1 `use` decl, +1 `let _log_path = init_logging();` line at fn top, `DefaultPlugins` → `DefaultPlugins.build().disable::<bevy::log::LogPlugin>()`, `//!` doc expanded by 1 line. ~5 additions, 1 modification.

Do **not** touch `src/state.rs` or `src/splash.rs` in this story.

### Scope boundaries — what belongs to later stories

| Concern | Story that owns it |
|---|---|
| `tracing-appender` non-blocking writer | **Post-MVP** — only if log volume measurably hurts frame time. Not in any current epic. |
| Log rotation / daily files | **Post-MVP** — only if needed for long-running playtesters. Not in any current epic. |
| Save-file path resolution via `directories` | **Epic 4 Story 4.6** — `persistence/paths.rs`. May subsume `resolve_log_dir` into a shared helper at that time. |
| `tracy-client` / `tracing-chrome` profiling layers | **Epic 2 / M1 or M2 debug-panels story** — added as an extra Layer to our Registry. |
| Structured log fields beyond Bevy defaults (spans around `Run`, `State`) | **Epic 6 Caravan** — when lifecycle events get rich enough to need spans. |
| README / docs on RUST_LOG + RUST_BACKTRACE env vars | **Epic 10 polish (10.4 string-table audit) or a dedicated docs-pass** — not in-story. |
| Re-enabling Bevy `LogPlugin` with `custom_layer` | **Not planned.** Our Registry stack subsumes its role. |
| `bevy_egui` dev-only debug panels | **M2 (Epic 2 / Epic 3)** — separate concern. |

### Architecture Compliance

- **`tracing` + `tracing-subscriber` + `RUST_LOG`** — architecture.md:278 and :376 prescribe exactly this stack. [Source: architecture.md:278-281, :375-383]
- **Log file location via `directories` crate** — architecture.md:279 prescribes the crate; AC #2 prescribes the exact per-OS paths. This story implements both. [Source: architecture.md:279, epics/epic-1-*.md:185]
- **Panic hook writes stack trace to log file before exit** — architecture.md:280 and :373. Panic hook preserves default stderr behavior, matching "don't hide information from developer" unwritten norm. [Source: architecture.md:280, :373]
- **No remote telemetry** — Our subscriber has only local layers (stderr + file). No network sink. [Source: architecture.md:281, PRD E#5]
- **Logging levels follow architecture.md:377-382** — `error!` crashed-but-recovered, `warn!` degradations, `info!` lifecycle, `debug!` diagnostics, `trace!` hidden by default. This story doesn't add new log calls (no feature code); it wires the infrastructure. Existing `info!` calls from `state.rs` (`entered Loading` / `entered MainMenu`) and `splash.rs` (`splash timer elapsed, ...`) already follow the lifecycle/info level convention.
- **Error handling: graceful degradation on failure** — `init_logging` returns `Option<PathBuf>`; on path-resolve / mkdir / file-open failure, emits `eprintln!` and proceeds with stderr-only subscriber. Never panics on file-I/O errors during startup. Matches architecture.md:368 ("User-facing degradation ... log via `tracing::warn!` + fall back to defaults. Never crash on user-facing failure paths").
  - Caveat: we use `eprintln!` not `warn!` because the subscriber isn't installed at the moment of the early failure. The first failure path is pre-subscriber; fallback output goes direct to stderr.
- **No `unsafe`** — subscriber + panic hook + File I/O are all safe APIs. [Source: architecture.md:440]
- **No `Arc<Mutex<>>` in gameplay code** — `Mutex<File>` used inside the subscriber is infrastructure, not gameplay; falls under "infrastructure / plugin init" exception. The panic hook holds an owned `PathBuf` + re-opens the file per panic — no shared mutable state across threads. [Source: architecture.md:439]
- **`directories = "5"`** — already pinned Cargo.toml:20. Usage here is the first actual instantiation. Satisfies Story 1.1's promise that `directories` is pinned; Story 1.8 is the story that redeems that promise.

### Library/Framework Requirements

| Crate | Version | Change in Story 1.8 |
|---|---|---|
| `tracing` | `0.1` | **No change** — already pinned (`Cargo.toml:18`). We use `tracing::info!` etc. macros implicitly via Bevy's `bevy::prelude::*` re-export (which is where `log_loading_entered` etc. get them). This story imports nothing directly from `tracing` in `logging.rs` except through `tracing_subscriber`. |
| `tracing-subscriber` | `0.3`, features `env-filter` | **No change** — already pinned (`Cargo.toml:19`). First actual use. `env-filter` feature is required for `EnvFilter::try_from_default_env()`. |
| `directories` | `5` | **No change** — already pinned (`Cargo.toml:20`). First actual use (`BaseDirs::new()`, `home_dir()`, `state_dir()`, `data_dir()`). |
| `bevy` | `0.18` | **No change** — features unchanged. Usage adds `.disable::<bevy::log::LogPlugin>()` via the existing `bevy::prelude::*` + `bevy::log::LogPlugin` path. |
| `Cargo.lock` | — | **Expected unchanged.** All three tracing/directories crates are already resolved; no new crate added. Should NOT regenerate. |

No new top-level crate dependency added.

### File Structure Requirements

| Path | Add/Modify | Purpose |
|---|---|---|
| `src/logging.rs` | **Add** | ~110–140 lines: `LOG_FILE_NAME` + `DEFAULT_FILTER` consts, `resolve_log_dir()` fn, `init_logging()` pub fn, `install_panic_hook()` fn, `#[cfg(test)] mod tests` with `resolve_log_dir_yields_expected_suffix`. Module doc ≤2 lines. |
| `src/main.rs` | **Modify** | +1 `mod logging;`, +1 `use logging::init_logging;`, +1 `let _log_path = init_logging();` line, +1 `.disable::<bevy::log::LogPlugin>()` chain call, +1 `//!` doc line. Net +5 lines. |
| `Cargo.toml` | **Do NOT touch** | All required crates already pinned. |
| `Cargo.lock` | **Do NOT touch** | No dep changes → no regeneration. |
| `src/state.rs`, `src/splash.rs` | **Do NOT touch** | Out of scope. |
| `.gitignore`, `.github/workflows/ci.yml`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`, `docs/plugin-compatibility.md` | **Do NOT touch** | Out of scope. |
| `_bmad-output/implementation-artifacts/deferred-work.md` | **Do NOT touch in-story** | Post-review defers logged in Task 7. |
| `_bmad-output/implementation-artifacts/sprint-status.yaml` | **Modify (bookkeeping)** | Flip status `backlog → in-progress → review` via Task 7. |

**Single-binary layout reminder:** Project remains a single-binary crate with no `lib.rs`. `src/logging.rs` is a direct child module declared via `mod logging;` in `main.rs`, same pattern as Story 1.6 `src/state.rs` and Story 1.7 `src/splash.rs`.

### Testing Requirements

- **Unit tests:** 1 new test (`resolve_log_dir_yields_expected_suffix`) in `src/logging.rs`. Pure function, zero I/O, CI-safe (runs on the test's target OS via `#[cfg(target_os = "...")]` guards). Total tests after this story: **3** (state default, splash config default, logging dir suffix).
- **NOT unit-tested (by design):**
  - `init_logging()` — mutates process-global subscriber; `.init()` panics on second call. Running as a unit test would break every subsequent test in the same test binary.
  - `install_panic_hook()` — mutates process-global panic hook; would affect any `#[should_panic]` test in the same binary.
  Both are exercised by runtime verification (Task 3 steps 6–9) on the developer's machine; CI verifies compile + clippy + other tests still pass.
- **Runtime verification on macOS (Task 3):**
  - **AC #1 evidence:** stderr shows `entered Loading`, `splash timer elapsed, ...`, `entered MainMenu`. `RUST_LOG=debug cargo run` produces at least 1 `DEBUG ` line. Default run produces 0 `DEBUG ` lines.
  - **AC #2 evidence:** `~/Library/Logs/asteroids3D/asteroids3D.log` exists with non-zero size; contains same 3 lifecycle info lines as stderr; same session. `ls -la` confirms 644 / 664 perms (OS-default).
  - **AC #3 evidence:** temporary injected `panic!("bmad panic test")` fires immediately on startup; `asteroids3D.log` contains `PANIC: ...` line with message literal + `Backtrace:` header + ≥1 stack frame; `RUST_BACKTRACE=1` required for readable frames. `stderr` (captured in the run log) contains `thread 'main' panicked at ...` from the preserved default hook. The panic-inducing line is reverted before commit (`grep -c 'panic!' src/` final = 0 after revert).
- **CI coverage:** 3-OS matrix verifies:
  - macOS: `resolve_log_dir_yields_expected_suffix` asserts `Library/Logs/asteroids3D` suffix.
  - Ubuntu: `resolve_log_dir_yields_expected_suffix` asserts `asteroids3d/logs` suffix.
  - Windows: `resolve_log_dir_yields_expected_suffix` asserts `asteroids3D\logs` suffix.
  - All three also run `cargo build` (confirms no ABI / feature-flag regression from `LogPlugin` disable), `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`.
- **Integration tests (App construction):** **deferred**. Same rationale as 1.6/1.7 — `App::new().add_plugins(DefaultPlugins)` boots wgpu + winit and breaks headless CI. Revisit at Epic 3's first gameplay story when headless `MinimalPlugins` test patterns become relevant.
- **Windows/Linux runtime verification:** same pattern as Stories 1.5/1.6/1.7 — Till runs `cargo run` on physical hardware when convenient, confirms splash visible + log file appears at the expected per-OS path. Can be deferred to a future hardware-access window. CI itself verifies compile parity + unit-test assertion of the per-OS path suffix.

### Latest Technical Information

**`tracing-subscriber` 0.3 stability.** 0.3.x is the current stable line; no breaking changes to `EnvFilter`, `fmt::Layer`, `Registry`, or the `prelude` re-exports in 0.3.18–0.3.22 that would affect this story's usage. `Mutex<W: Write>` `MakeWriter` impl has been present since 0.3.0. [Source: tracing-subscriber 0.3.x CHANGELOG]

**`directories` 5 API surface.**
- `BaseDirs::new() -> Option<BaseDirs>` — None only if `$HOME`/`$USERPROFILE` is unset. Normal environments → always `Some`.
- `BaseDirs::home_dir() -> &Path` — `$HOME` on macOS/Linux, `{FOLDERID_Profile}` on Windows.
- `BaseDirs::data_dir() -> &Path` — `{FOLDERID_RoamingAppData}` = `%APPDATA%` on Windows, `$XDG_DATA_HOME` on Linux, `~/Library/Application Support` on macOS.
- `BaseDirs::state_dir() -> Option<&Path>` — `Some($XDG_STATE_HOME)` on Linux (fallback `$HOME/.local/state`), `None` on macOS and Windows.
- No `log_dir()` / `logs_dir()` primitive — the AC's per-OS log paths are not a 1:1 primitive in the crate; per-OS `cfg` is required.
[Source: https://docs.rs/directories/5/ — `BaseDirs`]

**`std::backtrace::Backtrace` behavior.** `Backtrace::capture()` since Rust 1.65+ returns a stable `Backtrace` whose `Display` impl respects `RUST_BACKTRACE` env var at capture time. MSRV 1.89 is well above the minimum. No nightly features required. [Source: std::backtrace stabilization]

**Bevy `LogPlugin` disable pattern.** `DefaultPlugins.build().disable::<P>()` has been the canonical plugin-exclusion idiom since Bevy 0.8. In Bevy 0.18 the API is unchanged. `bevy::log::LogPlugin` is the full path; `bevy::prelude` does not re-export it, so we write `bevy::log::LogPlugin` explicitly. [Source: Bevy 0.18 `PluginGroupBuilder::disable` docs]

**`tracing` RUST_LOG filter syntax recap** (pertinent for the dev agent):
- `RUST_LOG=info` → all targets at info+.
- `RUST_LOG=debug` → all targets at debug+ (AC #1 test case).
- `RUST_LOG=wgpu_core=warn,info` → global info + wgpu_core at warn+.
- `RUST_LOG=asteroids3D=debug` → only our crate at debug, rest at global (default info).
- Invalid syntax → `try_from_default_env` returns `Err` → we fall back to `DEFAULT_FILTER = "info"`. User may not notice the fallback — consider `eprintln!` of the fallback cause. Deferred: adding a `warn!`-before-subscriber-init is chicken-and-egg; skip for simplicity.

### Previous Story Intelligence

**From Story 1.7 (just closed, commit chain `19ed03c` → … → `8914284` → `fb7e411` → `92d7eea` → `fa5bd93` → typo-sweep `c2f1327` → story-1-7 bookkeeping):**

- **Two-commit pattern (source + bookkeeping).** Story 1.8 follows: `feat:` commit with source, then `bmad:` commit with story + sprint-status. Story 1.7 also had a middle `chore:` commit for review patches; 1.8 anticipates similar if review produces patches.
- **`eprintln!` as last-resort channel.** Story 1.7 established `eprintln!` is acceptable for pre-subscriber, pre-Bevy bootstrap diagnostics (none used in 1.7 body but the pattern matches 1.8's fallback warnings on file-I/O failures).
- **Exit-0 is not proof.** MEMORY `feedback_full_build_output.md` explicitly rejects "cargo check exit 0 + tail looks clean" as verification. Task 3 uses `tee` + explicit `grep` for every command — paste line counts into DAR.
- **Module doc style:** ≤2 lines, no story-id references (honors 1.5 review-patch BH8). Story 1.8's `//!` doc in `logging.rs` follows this; `main.rs` gains one additional line within the same ≤3-line budget.
- **CI cache stability.** Story 1.7 kept `Cargo.lock` unchanged → warm cache → ~10m47s CI. Story 1.8 should also leave `Cargo.lock` unchanged (no new dep). Expected CI wall ~10 m.
- **`bevy_winit Destroyed for unknown winit Window Id` WARN on close** — known Bevy 0.18 winit race (deferred-work.md → 1.6 LOW-1). Do NOT treat as a Story 1.8 regression if it reappears.
- **Rustfmt may re-order `use` items.** Story 1.7 rustfmt re-sorted splash imports; Story 1.8's new `use logging::init_logging;` will slot alphabetically-first in the `use <local-module>::...` block. Accept rustfmt's output.
- **Story-scope guardrails via `git status --short` + targeted `grep`.** Story 1.8 Task 4 mirrors 1.6/1.7 Task 6.
- **Light code-review is acceptable** (single-reviewer pattern from 1.6/1.7) for ~150-line diffs. Full 3-agent review is discretionary — use if panic-hook edge cases or `Mutex<File>` lock contention need adversarial examination.
- **Sprint-status flip: `backlog → in-progress → review`** — skip the intermediate `ready-for-dev` state that `create-story` sets; dev-story flips it directly to `in-progress` on work-start, then `review` on commit. This matches the status-transition machinery in `workflow.md`.

### Git Intelligence

Recent relevant commits (last 5):
```
fa5bd93 bmad: document CI paths-ignore convention (test: should NOT trigger CI)
92d7eea ci: skip runs on _bmad/** and _bmad-output/** only commits
c7797e6 bmad: mark typo-rename chore ✅ resolved in deferred-work (2026-04-24)
c2f1327 chore: fix asteriods3D → asteroids3D typo across all artifacts
6fb491e bmad: story 1.7 review complete — 2 patches applied, 1 defer logged
```

**Patterns reinforced:**
- Single-line commit subjects, sub-70-char.
- `feat:` for source-change stories. `ci:` for workflow changes. `chore:` for source cleanups / refactors / typo sweeps. `bmad:` for artifact-only commits.
- `bmad:` + `_bmad/` + `_bmad-output/` paths → skipped by CI (per `paths-ignore`). Story 1.8's Task 7 bookkeeping commit will benefit from this, producing zero-CI-cost bookkeeping.
- Story 1.8's `feat:` commit will trigger full 4-job CI (touches `src/`).
- No `Co-Authored-By` trailer.

### Project Structure Notes

Alignment with architecture.md file layout:
- `src/main.rs` — ✅ `App::new()` assembly, plugin + state registration, and now also logging init. [architecture.md:548]
- `src/state.rs` — ✅ Unchanged. [architecture.md:549]
- `src/splash.rs` — ⚠️ Time-bounded deviation (flat vs `src/ui/splash.rs`), unchanged by this story. Promoted at Epic 3 Story 3.1. [architecture.md:589-597]
- `src/logging.rs` — ⚠️ **Time-bounded deviation from architecture.md implicit structure.** Architecture does not explicitly reserve a `src/logging.rs` or `src/observability/` path — logging is treated as a cross-cutting concern without a named home. Flat `src/logging.rs` today, with natural promotion into a shared observability module (alongside tracy/profiling in M2) or into `src/core/` (alongside faction/damage/markers per architecture.md:550-554) as the project grows. Documented as staged rollout; no architecture amendment needed. [Source: architecture.md:277-281, :373-383 — mandates the stack but not its module home]

No conflicts with the rest of the unified project structure. Future `src/persistence/paths.rs` (Epic 4 Story 4.6) may subsume the per-OS path resolver from `logging.rs` in a small refactor; documented above.

### References

- [Source: `_bmad-output/planning-artifacts/epics/epic-1-foundation-plugin-compatibility-gate.md:172-193`] — Story 1.8 spec with full AC list.
- [Source: `_bmad-output/planning-artifacts/architecture.md:278-281`] — Monitoring/Logging architectural mandate (tracing + tracing-subscriber + RUST_LOG + directories + panic hook + no remote telemetry).
- [Source: `_bmad-output/planning-artifacts/architecture.md:373`] — Panic Policy: panic hook writes stack trace to log file before exit.
- [Source: `_bmad-output/planning-artifacts/architecture.md:375-383`] — Logging stack + levels convention (error/warn/info/debug/trace).
- [Source: `_bmad-output/planning-artifacts/architecture.md:368`] — Error handling: user-facing degradation via warn! + fall back to defaults.
- [Source: `_bmad-output/planning-artifacts/architecture.md:548-554`] — `src/main.rs` + `src/state.rs` + `src/core/` placement per file layout.
- [Source: `_bmad-output/planning-artifacts/architecture.md:602`] — `src/persistence/paths.rs` as future `directories`-crate wrapper.
- [Source: `_bmad-output/planning-artifacts/architecture.md:742`] — `directories` crate for save + log paths.
- [Source: `_bmad-output/planning-artifacts/architecture.md:976-978`] — Starter Cargo.toml skeleton listing `tracing`, `tracing-subscriber`, `directories` pins.
- [Source: `_bmad-output/planning-artifacts/prd.md:612`] — NFR-L3 (string table) — referenced for contrast; not this story's scope.
- [Source: `_bmad-output/implementation-artifacts/1-7-splash-screen-shows-asteroids3d-and-transitions-to-mainmenu.md`] — Prior-story conventions: commit pattern, verification discipline, scope guardrails, rustfmt-accept pattern.
- [Source: `_bmad-output/implementation-artifacts/deferred-work.md`] — `.claude/*.lock` gitignore defer (still open), `bevy_winit` close-WARN (1.6 defer, non-regression), `SplashConfig` timer re-entry (1.7 defer, unrelated).
- [Source: `Cargo.toml:18-20`] — current `tracing` / `tracing-subscriber` / `directories` pins (all activated by this story).
- [Source: `.github/workflows/ci.yml:9-15`] — `paths-ignore` on `_bmad/**` + `_bmad-output/**` — bookkeeping commits skip CI.
- [Source: `MEMORY.md → feedback_full_build_output.md`] — verification discipline: exit-0 + tail is NOT proof; grep explicitly.
- [Source: `MEMORY.md → feedback_staged_rollout.md`] — staged rollout preference (flat `src/logging.rs` now, promoted later).
- [Source: https://docs.rs/directories/5/directories/struct.BaseDirs.html] — `BaseDirs` API reference.
- [Source: https://docs.rs/tracing-subscriber/0.3/tracing_subscriber/] — `Registry` / `fmt::Layer` / `EnvFilter` API reference.

## Dev Agent Record

### Agent Model Used

`claude-opus-4-7[1m]` (Opus 4.7, 1M-context). Local dev: macOS 26.4.1 arm64 (Apple M5 Pro), Rust 1.94.1 stable, Bevy 0.18.

### Debug Log References

**Local verification sweep (Task 3) — macOS arm64:**

| Command | Log file | `warning:|error:|FAILED` hit count | Result |
|---|---|---:|---|
| `cargo check` | `/tmp/story-1-8-check.log` | 0 | ✅ |
| `cargo build` | `/tmp/story-1-8-build.log` | 0 | ✅ |
| `cargo test` | `/tmp/story-1-8-test.log` | 0 | ✅ `3 passed; 0 failed` (state, splash, logging) |
| `cargo clippy --all-targets -- -D warnings` | `/tmp/story-1-8-clippy-final.log` | 0 | ✅ |
| `cargo fmt --all -- --check` | (stdout) | — | exit 0 after one `cargo fmt --all` apply (collapsed `use std::{...}` to single line) |

**Default-filter `cargo run` — `/tmp/story-1-8-run-default.log` + `~/Library/Logs/asteroids3D/asteroids3D.log`:**

| Probe | Stderr | Log file |
|---|---:|---:|
| `entered Loading` | 1 | 1 |
| `splash timer elapsed` | 1 | 1 |
| `entered MainMenu` | 1 | 1 |
| `DEBUG ` (literal) | 0 | 0 |

Log file size: 1645 bytes, 644 perms. Sample lines (ANSI-stripped, file is already ANSI-free):
```
2026-04-27T08:07:41.462954Z  INFO asteroids3D::state: entered Loading
2026-04-27T08:07:43.464374Z  INFO asteroids3D::splash: splash timer elapsed, transitioning to MainMenu
2026-04-27T08:07:43.472933Z  INFO asteroids3D::state: entered MainMenu
```

Note on the `WARN bevy_ecs::error::handler: Encountered an error in command ... Entity despawned: ...` that appears at the Splash → MainMenu transition: this is Bevy's handler reporting a despawned-entity command racing with cleanup. Pre-existing behaviour from 1.7's `cleanup_loading_entities`; surfaced now because our subscriber accepts WARN. **NOT** a 1.8 regression. Logged for review-phase consideration as a 1.7 leftover.

**`RUST_LOG=debug cargo run` — `/tmp/story-1-8-run-debug.log` + same log file:**

| Probe | Stderr (ANSI-stripped) | Log file |
|---|---:|---:|
| `DEBUG ` lines | 42 311 | 42 311 |
| `INFO ` lines | n/a (sampled) | 16 |
| `WARN ` lines | n/a (sampled) | 2 |
| `entered Loading` / `splash timer elapsed` / `entered MainMenu` | 1 / 1 / 1 | 1 / 1 / 1 |

Initial `grep -c 'DEBUG '` on raw stderr returned 0 because Bevy's stderr layer emits ANSI colour codes (`\x1b[34mDEBUG\x1b[0m`) — the literal string `' DEBUG '` only matches in the file (`with_ansi(false)`). After `sed $'s/\033\\[[0-9;]*m//g'` strip, stderr count matches file (42 311). File-ANSI-strip working as designed.

**Panic-hook verification (Task 3 final step) — `/tmp/story-1-8-panic.log` + log file:**

| Probe | Count |
|---|---:|
| `PANIC: ` in log file | 1 |
| `Backtrace:` in log file | 2 (one literal "Backtrace:" header + one inside the captured frame text) |
| Stack frames (`at src/...` / `at /rustc/...` / `panicking::` / `asteroids3D::`) in log file | 18 |
| `thread 'main' (PID) panicked at src/logging.rs:74:13:` in `/tmp/story-1-8-panic.log` | 1 ✅ default hook preserved |
| Process exit code | 101 (panic) |

Sample stderr (default-hook output, preserved by the new hook delegating to `prev`):
```
thread 'main' (86799) panicked at src/logging.rs:74:13:
bmad panic test
stack backtrace:
   0: __rustc::rust_begin_unwind
   1: core::panicking::panic_fmt
   2: asteroids3D::logging::init_logging
             at ./src/logging.rs:74:13
   3: asteroids3D::main
             at ./src/main.rs:16:21
```

Sample log file panic record (last 25 lines truncated; full backtrace ≥18 frames including `std::rt::lang_start_internal`, `_main`):
```
PANIC: panicked at src/logging.rs:74:13:
bmad panic test
Backtrace:
   0: std::backtrace::Backtrace::create
   ...
  24: std::rt::lang_start
  25: _main
```

**Panic-test line reverted before commit:** `grep -rn 'panic!' src/` → 0 hits in the final diff (Task 4 guardrail).

**Scope guardrails (Task 4) — all greps ran from project root:**

| Probe | Hits | Expected |
|---|---:|---|
| `grep -nrE 'Arena|Caravan|PostRun|PhotoMode|Paused' src/ --include='*.rs' \| grep -v 'state.rs'` | 0 | 0 |
| `grep -rnE 'ui::|main_menu|string\.table|en\.ron' src/` | 0 | 0 |
| `grep -rn 'tracing_appender\|non_blocking' src/` | 0 | 0 |
| `grep -rn 'panic!' src/` | 0 | 0 |
| `git diff --quiet HEAD -- {.gitignore, .github/workflows/ci.yml, rust-toolchain.toml, rustfmt.toml, clippy.toml, docs/plugin-compatibility.md}` | all clean | clean |
| `git diff --stat Cargo.toml Cargo.lock` | empty | empty |
| `git diff --quiet HEAD -- _bmad-output/implementation-artifacts/deferred-work.md` | clean | clean |

**CI run (Task 6):**

| Field | Value |
|---|---|
| Run ID | `24983925700` |
| Commit SHA | `97d53b4` |
| Trigger | `push` on `master` |
| Workflow | `CI` (`.github/workflows/ci.yml`) |
| Started | 2026-04-27T08:12:14Z |
| Finished | 2026-04-27T08:19:29Z |
| Wall-time | 7m15s (faster than the 10–12m estimate — warm cache; only ubuntu's `Cache cargo registry + target` step took ~3m44s, others negligible) |
| Conclusion | `success` (all 4 jobs ✅) |
| `gh run view 24983925700 --log \| grep -cE 'warning:\|error:'` | **0** |

| Job | Duration | Notes |
|---|---:|---|
| `msrv-check (rust 1.89, ubuntu-latest)` (job `73152414924`) | 47s | `cargo check (MSRV)` succeeded in 4s after cached toolchain install |
| `build (macos-latest)` (job `73152414956`) | 1m4s | warm cache; `cargo build` 11s, test 2s, clippy 1s, fmt --check 0s |
| `build (windows-latest)` (job `73152414921`) | 5m9s | `Cache cargo registry + target` 1m55s, `cargo build` 1m56s, rest <1m |
| `build (ubuntu-latest)` (job `73152414911`) | 7m10s | `Cache cargo registry + target` 3m44s (warm-cache restore on the largest target dir), `cargo build` 1m22s |

Only annotation across all 4 jobs: a `Node.js 20 actions are deprecated` notice from `actions/checkout@v4` — repo-wide, not Story-1.8-related, surfaced on every run since GitHub announced the deprecation. Already tracked separately as a future CI maintenance item (not scope of this story).

### Completion Notes List

**AC #1 — `tracing_subscriber` initialised in `main.rs` before `App::new()`, `info!`/`warn!`/`error!` to stderr, `RUST_LOG=debug` toggles verbosity:** ✅
- `init_logging()` is invoked from `src/main.rs:16` as `let _log_path = init_logging();` — first executable line of `fn main()`, before `App::new()`.
- `EnvFilter::try_from_default_env()` reads `RUST_LOG` at init; falls back to `EnvFilter::new("info")` on parse error or unset var.
- Bevy's `LogPlugin` is disabled via `DefaultPlugins.build().disable::<bevy::log::LogPlugin>()` so our subscriber is the sole `tracing` global default — no double-init panic.
- Default run: 0 DEBUG lines; `RUST_LOG=debug` run: 42 311 DEBUG lines (Bevy plugin-load chatter dominates; expected). Lifecycle INFO lines (`entered Loading`, `splash timer elapsed`, `entered MainMenu`) emitted in both modes.

**AC #2 — `directories` resolves per-OS user-log-dir, log file written to both stderr + file:** ✅
- `resolve_log_dir()` uses `BaseDirs::new()` + `#[cfg(target_os = "...")]` branches per the Story Dev Notes spec.
- macOS local verification: `~/Library/Logs/asteroids3D/asteroids3D.log` created automatically (parent dirs via `std::fs::create_dir_all`), 644 perms, 1645 bytes after a single splash → MainMenu run; ANSI-stripped (`with_ansi(false)` on the file layer).
- Linux/Windows path suffixes verified by the unit test `resolve_log_dir_yields_expected_suffix` running on each CI platform (asserts `asteroids3d/logs` / `asteroids3D\logs` respectively).
- `Mutex::<File>` is used as the file `MakeWriter`; no `tracing-appender` dependency added.

**AC #3 — panic hook writes message + backtrace to log file before exit, default stderr behaviour preserved:** ✅
- `install_panic_hook(log_path)` captures the previous hook via `panic::take_hook()`, writes `PANIC: {info}\nBacktrace:\n{backtrace}` (where `backtrace = Backtrace::capture()`) into `log_path` in append mode, flushes, then re-invokes the previous hook → default `thread 'main' panicked at ...` line still appears on stderr.
- Verified locally with an injected `panic!("bmad panic test")` after the subscriber + hook were installed: log file gained `PANIC: panicked at src/logging.rs:74:13: bmad panic test` plus an 18-frame backtrace; stderr (in `/tmp/story-1-8-panic.log`) contained the canonical `thread 'main' (86799) panicked at src/logging.rs:74:13:` line. Process exited with code 101.
- Hook is **only** installed when file logging succeeded (`init_logging` returns `Some(path)`); on stderr-only fallback the default hook alone handles output (graceful degradation per architecture.md:368).
- The panic-test line was reverted before commit; final diff is `panic!`-free in `src/` (verified by `grep -rn 'panic!' src/` = 0).

**Deviations from Story spec:** none material. Two minor format-driven adjustments:
1. The `use std::{...}` import block was collapsed to a single line by `cargo fmt` (the Dev-Notes-suggested multi-line block was rewritten by rustfmt). Identical contents, identical visibility — accepted.
2. `tracing_subscriber::prelude::*` was used instead of the dual import `tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt}` mentioned in the Story Tasks bullet. The prelude re-exports both traits and matches the Dev Notes' "tracing-subscriber 0.3 Registry-with-layers architecture" code sample (which itself uses `prelude::*`). Equivalent functionality, fewer import lines, matches the canonical example. Worth flagging in code review if a stricter import style is preferred.

**Observation worth flagging in code review:** The `WARN bevy_ecs::error::handler: ... Entity despawned: ...` message appears at the Splash → MainMenu transition. This is pre-existing behaviour from Story 1.7's `cleanup_loading_entities` racing with a Bevy command; surfaced for the first time because our subscriber accepts the WARN level (Bevy's pre-1.8 default `LogPlugin` filter `wgpu=error,naga=warn,info` would have shown it too). **Not** a 1.8 regression — Bevy's `bevy_winit Destroyed for unknown winit Window Id` close-WARN (deferred-work.md → 1.6 LOW-1) is the same family. Reviewer can decide whether this also belongs in `deferred-work.md` as a 1.7 follow-up.

### File List

**Added:**
- `src/logging.rs` (new file, 124 lines after `cargo fmt`): `LOG_FILE_NAME` + `DEFAULT_FILTER` consts; `resolve_log_dir()`, `pub init_logging()`, `install_panic_hook()` functions; `#[cfg(test)] mod tests` with `resolve_log_dir_yields_expected_suffix`.

**Modified:**
- `src/main.rs`: +1 `mod logging;`, +1 `use logging::init_logging;`, +1 `let _log_path = init_logging();` line at fn top, `DefaultPlugins` → `DefaultPlugins.build().disable::<bevy::log::LogPlugin>()`, `//!` doc expanded by 1 line. Net `+6/-1` lines per `git show 97d53b4 --stat`.
- `_bmad-output/implementation-artifacts/sprint-status.yaml` (bookkeeping): `1-8-...: ready-for-dev → in-progress → review`; `last_updated: 2026-04-27`.
- `_bmad-output/implementation-artifacts/1-8-tracing-based-logging-with-panic-hook-to-log-file.md` (this file, bookkeeping): tasks 1–7 checkboxes flipped, Status header → `review`, Dev Agent Record sections populated.

**Untouched (guardrail):**
- `Cargo.toml`, `Cargo.lock` — no new dependency added; all three crates (`tracing 0.1`, `tracing-subscriber 0.3 [env-filter]`, `directories 5`) pinned at Story 1.1.
- `src/state.rs`, `src/splash.rs` — out of scope.
- `.gitignore`, `.github/workflows/ci.yml`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`, `docs/plugin-compatibility.md` — out of scope.
- `_bmad-output/implementation-artifacts/deferred-work.md` — out of scope per Task 4; review-phase additions handled separately if reviewer chooses.

### Change Log

| Date | Author | Change |
|---|---|---|
| 2026-04-27 | dev-story (Opus 4.7) | Story 1.8 implemented: tracing subscriber + panic-hook-to-file + per-OS log dir. `feat:` commit `97d53b4` lands `src/logging.rs` (new) + `src/main.rs` (mod-load + LogPlugin disable). Local verification on macOS arm64: 0 warnings/errors in check/build/test/clippy/fmt; 3 tests pass; default + RUST_LOG=debug runs verified; panic-hook injection verified with PANIC + backtrace + preserved default stderr; panic-test line reverted. Status: `ready-for-dev` → `in-progress` → `review`. CI run `24983925700` triggered. |
| 2026-04-27 | code-review (Opus 4.7) | Adversarial 3-layer review (Blind Hunter / Edge Case Hunter / Acceptance Auditor) of `97d53b4`. Total: 1 decision-needed (Windows Roaming vs LocalAppData), 6 patches, 7 defers, 15 dismissed. AC#1/#2/#3 all confirmed satisfied by Acceptance Auditor (`PASS`). Findings written to "Review Findings" section below; defers appended to `deferred-work.md`. |
| 2026-04-27 | code-review patches (Opus 4.7) | All 7 patches applied (decision-needed item folded as Patch #7 — Windows `data_local_dir()`). `chore:` commit `8632902` lands the source patches. Local re-verification: 0 warnings/errors in check/build/test/clippy/fmt; 3 tests pass; force_capture verified with unset RUST_BACKTRACE (14 stack frames, 0 `disabled backtrace`); IsTerminal verified (0 ANSI in piped stderr); RUST_LOG fallback eprintln verified; `info!` log path appears in both senks. CI run `24985200581`: 5m12s wall, all 4 jobs ✅, 0 `warning:\|error:` in full log. Status: `review` → `done`. |

### Review Findings (Code Review 2026-04-27)

**Adversarial review summary:** 3-layer parallel review (Blind Hunter / Edge Case Hunter / Acceptance Auditor). Acceptance Auditor verdict: **PASS** — all three ACs confirmed with documented evidence; no HIGH/MED spec violations. Below: 1 decision-needed, 6 patches, 7 defers (also logged to `deferred-work.md`), 15 dismissed as noise / spec-authorized.

#### Decision-needed (resolved)

- [x] [Review][Decision] ~~Windows logs in Roaming AppData (`%APPDATA%`) vs LocalAppData (`%LOCALAPPDATA%`)~~ — **Resolved 2026-04-27 by Till: option (1) — switch to `%LOCALAPPDATA%`.** Spec amendment required: AC #2 line 20 path table changes from `%APPDATA%\asteroids3D\logs\` to `%LOCALAPPDATA%\asteroids3D\logs\`; Dev Notes "Per-OS log-directory resolution" table line 182 + reference implementation line 213-216 swap `data_dir()` → `data_local_dir()`. Folded into the patch list as **Patch #7** below.

#### Patches (applied 2026-04-27)

- [x] [Review][Patch] Use `Backtrace::force_capture()` instead of `Backtrace::capture()` in panic hook [src/logging.rs:108] — verified locally with unset `RUST_BACKTRACE`/`RUST_LIB_BACKTRACE`: 14 stack frames captured, 0 occurrences of `disabled backtrace` in log file (vs. prior code which would produce only `disabled backtrace`).
- [x] [Review][Patch] `OpenOptions::new().create(true).append(true)` in panic hook [src/logging.rs:103] — `.create(true)` added; aligned with `init_logging`'s open-pattern.
- [x] [Review][Patch] `eprintln!` panic-hook file-open / write / flush failures [src/logging.rs:103-119] — replaced `if let Ok(...)` + `let _ =` with `match` + explicit Err arms emitting `eprintln!`. All three failure paths (open / writeln / flush) now diagnose to stderr.
- [x] [Review][Patch] `eprintln!` on `RUST_LOG` parse failure [src/logging.rs:50-58] — verified via `RUST_LOG='@@@bogus,,,,filter@@@' cargo run`: `logging: invalid RUST_LOG, falling back to 'info': invalid filter directive` printed once; lifecycle still emitted at info level. eprintln gated on `var_os("RUST_LOG").is_some()` to avoid noise when var simply isn't set.
- [x] [Review][Patch] Detect tty for stderr ANSI via `std::io::IsTerminal` [src/logging.rs:62] — verified via piped run (`cargo run > /tmp/file 2>&1`): 0 ANSI escape sequences in captured stderr (vs. prior runs which had ESC[34mDEBUG ESC[0m etc.). Interactive runs retain colours (when stderr is a tty).
- [x] [Review][Patch] `info!("file logging active at {}", path.display())` after subscriber init [src/main.rs:16-19] — verified: `INFO asteroids3D: file logging active at /Users/tillfechteler/Library/Logs/asteroids3D/asteroids3D.log` appears in both stderr and the log file as the first lifecycle line (before `entered Loading`).
- [x] [Review][Patch] Windows log dir `%LOCALAPPDATA%\asteroids3D\logs\` (was `%APPDATA%`) [src/logging.rs:42, AC #2 path table, Dev Notes Per-OS Resolution table + Reference impl block] — `BaseDirs::data_dir()` → `BaseDirs::data_local_dir()`. Spec amended in AC #2 line + Dev Notes table + reference implementation code block (with 2026-04-27 amendment notes inline). Unit test suffix `asteroids3D\logs` unchanged (only parent base directory shifts from Roaming to Local). Will be CI-verified on `windows-latest` job.

#### Deferred (logged to `deferred-work.md`)

- [x] [Review][Defer] Shared `Arc<Mutex<File>>` between fmt-layer and panic hook [src/logging.rs:67,96] — architectural refactor; theoretical interleaving today (single-thread panic source), becomes real once Bevy task-pool runs panicking systems
- [x] [Review][Defer] `catch_unwind` around `prev(info)` in panic hook [src/logging.rs:101] — paranoia tier; standard panic-hook patterns don't guard double-panic
- [x] [Review][Defer] Install panic hook before subscriber init [src/logging.rs:78-79] — microsecond window between `.init()` and `install_panic_hook` is negligible
- [x] [Review][Defer] Cap backtrace size for `RUST_BACKTRACE=full` [src/logging.rs:97] — multi-MB log file possible from a single panic; opt-in env var means user explicitly wants verbose
- [x] [Review][Defer] Per-layer `EnvFilter` so stderr/file can diverge [src/logging.rs:69-77] — single filter today; not yet required by any AC
- [x] [Review][Defer] `eprintln!` invisible in launchd / packaged macOS bundle [src/logging.rs:54-77] — relevant for M9 packaging, not M0
- [x] [Review][Defer] Validate `state_dir()` is absolute on Linux [src/logging.rs:34-37] — `XDG_STATE_HOME=relative` edge case is rare misconfiguration

#### Dismissed (15)

Spec-authorized or otherwise non-actionable:
- `init_logging` not idempotent (panics on second `.init()`) — explicitly authorized by spec lines 530-533 (no unit test)
- macOS `asteroids3D` vs Linux `asteroids3d` casing — explicit spec mandate (AC #2 + Dev Notes lines 186-188 design rationale)
- Log rotation / size cap missing — explicitly deferred by spec ("Automatic rotation is post-MVP")
- Test depends on `BaseDirs::new()` succeeding — actual CI runners (ubuntu/macos/windows-latest) all set HOME/USERPROFILE; verified by passing CI run
- `LogPlugin` disable couples to Bevy plugin path — required by spec (AC #1 mandates we own subscriber); plugin path is stable Bevy public API
- No flush on normal exit — `Drop` semantics of `Mutex<File>` handle this; we don't call `std::process::exit`
- Linux state_dir bypass-with-fallback — spec-prescribed pattern (Dev Notes lines 31-39)
- Module doc references `architecture.md:278-281` — strict reading: arch-line refs ≠ story-id refs (auditor self-resolved)
- `use` block uses `prelude::*` — equivalent to spec skeleton (line 296); auditor self-resolved
- `use` block omits `File` import — explicitly authorized by spec line 428
- macOS no XDG override — XDG isn't standard on macOS; not applicable
- Non-UTF-8 path components — `Path`/`PathBuf` already handle safely
- No run-separator across runs — spec decision (Dev Notes line 174: "first `info!` after init serves as session-start marker")
- Windows test backslash literal — empirically validated by passing windows-latest CI
- `create_dir_all` / `OpenOptions` failure handling — already handled by graceful-degradation pattern (architecture.md:368)
