# Epic 1: Foundation & Plugin Compatibility Gate

Project compiles and runs on Windows, Linux, and macOS. `cargo run` opens a window showing "asteroids3D" splash. Plugin compatibility matrix verified and version-pinned. CI matrix green. No gameplay code — this is the compatibility gate per Architecture Starter decision. M-alignment: M0.

## Story 1.1: Bootstrap Cargo Project with Hand-Authored Cargo.toml

As a developer,
I want the project directory initialized with a hand-authored `Cargo.toml` containing all pinned dependencies,
So that every dependency is committed and reproducible from day one, and I internalize the Bevy setup rather than inheriting it from a template.

**Acceptance Criteria:**

**Given** an empty working directory at `~/Projekte/rust/asteroids3D`
**When** `cargo new --bin asteroids3d` is executed
**Then** `src/main.rs` and `Cargo.toml` are created by cargo

**Given** the default `Cargo.toml` is replaced by hand
**When** the edit is saved
**Then** Bevy is pinned at `0.18` with `default-features = false` and features `["3d", "png"]` plus platform-appropriate windowing (`x11`/`wayland` on Linux)
**And** avian3d is pinned at `0.6`
**And** `bevy_mod_outline`, `bevy_kira_audio`, `leafwing-input-manager` are pinned at their latest Bevy-0.18-compatible versions
**And** `bevy_egui` is declared under `[target.'cfg(debug_assertions)'.dependencies]`
**And** `serde`, `serde_json`, `ron`, `thiserror`, `tracing`, `tracing-subscriber`, `directories` are pinned
**And** release profile sets `lto = "fat"`, `codegen-units = 1`, `opt-level = 3`
**And** dev profile sets dependency `opt-level = 1`

**Given** all dependencies are pinned
**When** `cargo check` runs
**Then** resolution succeeds
**And** `Cargo.lock` is committed

## Story 1.2: Plugin Compatibility Verification Gate

As a developer,
I want explicit verification that every pinned plugin has a working Bevy-0.18-compatible release,
So that I discover fork-or-substitute decisions before writing gameplay code, not three weeks into M2.

**Acceptance Criteria:**

**Given** `Cargo.toml` from Story 1.1
**When** `cargo check` is executed on the local machine
**Then** all four third-party plugins (`bevy_mod_outline`, `bevy_kira_audio`, `leafwing-input-manager`, `bevy_egui`) compile without errors

**Given** any plugin fails to compile
**When** the failure is reviewed
**Then** a resolution is documented in `docs/plugin-compatibility.md` with (plugin name, error summary, resolution path)
**And** the resolution path is one of: (a) upstream patch exists → pin updated, (b) fork-and-inline per PRD Tech-Risk strategy, (c) substitute alternative plugin

**Given** all plugins resolve
**When** the verification is complete
**Then** `docs/plugin-compatibility.md` lists verification date, Rust toolchain version, Bevy version, and each plugin version
**And** this story's gate is passed — subsequent stories may proceed

## Story 1.3: Toolchain, Lint, and Format Configuration

As a developer,
I want reproducible toolchain + lint + format configs committed,
So that local dev and CI share the same rules and formatting drift is impossible.

**Acceptance Criteria:**

**Given** the project root has no toolchain config
**When** `rust-toolchain.toml` is added pinning the latest stable Rust channel
**Then** `rustup show` inside the project reports the pinned channel
**And** CI (when added in Story 1.4) uses the same channel deterministically

**Given** no format/lint configs exist
**When** `rustfmt.toml` and `clippy.toml` are added with project style rules
**Then** `cargo fmt --check` passes on all committed code
**And** `cargo clippy -- -D warnings` passes on all committed code

**Given** no ignore rules exist
**When** `.gitignore` is added per Rust + Bevy conventions
**Then** `target/` is excluded
**And** Bevy asset-cache directories are excluded
**And** IDE-local files (`.vscode/`, `.idea/`) and OS artifacts (`.DS_Store`, `Thumbs.db`) are excluded

## Story 1.4: Three-Platform CI Matrix

As a developer,
I want GitHub Actions CI running on Windows, Linux, and macOS from commit one,
So that the cross-platform parity commitment (FR47) is verified on every push instead of discovered at a milestone gate.

**Acceptance Criteria:**

**Given** `.github/workflows/ci.yml` is added, adapted from NiklasEi's `bevy_game_template` CI
**When** a commit is pushed to any branch
**Then** parallel jobs run on `windows-latest`, `ubuntu-latest`, and `macos-latest` (Apple Silicon runner)
**And** each job executes `cargo build`, `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check`
**And** iOS, Android, and Web/WASM jobs from the source template are stripped out

**Given** any of the three OS jobs fails
**When** the CI result is reported
**Then** the pull request / commit status reports red
**And** the failing log identifies which OS and which step failed

**Given** all three OS jobs pass
**When** CI completes
**Then** FR47 baseline (cross-platform binary) is verified for the current commit

## Story 1.5: Minimal Bevy App Opens a Window on All Three Platforms

As a first-time observer of the project,
I want `cargo run` to open a window on Windows, Linux, and macOS,
So that the "asteroids3D project exists and runs" signal is demonstrable from day one — the motivation-preservation baseline.

**Acceptance Criteria:**

**Given** `src/main.rs` contains `App::new().add_plugins(DefaultPlugins).run()`
**When** `cargo run` is invoked on Windows 10+
**Then** a native window opens with default Bevy title and size
**And** no panics or unexpected error logs are emitted

**Given** the same `src/main.rs`
**When** `cargo run` is invoked on a Linux desktop (Ubuntu LTS or equivalent)
**Then** a native window opens using the Vulkan backend via wgpu
**And** no panics or unexpected error logs are emitted

**Given** the same `src/main.rs`
**When** `cargo run` is invoked on macOS (Apple Silicon)
**Then** a native window opens using the Metal backend via wgpu
**And** no panics or unexpected error logs are emitted

## Story 1.6: GameState Enum with Bevy States Skeleton

As a developer,
I want a `GameState` enum registered with Bevy's `States` API,
So that future plugins can hook `OnEnter`/`OnExit`/`in_state()` scheduling from M1 onward without retrofit.

**Acceptance Criteria:**

**Given** `src/state.rs` is created
**When** `GameState` is defined with variants `Loading`, `MainMenu`, `Arena`, `Caravan`, `PostRun`, `PhotoMode`, `Paused`
**Then** it derives `States`, `Default` (default = `Loading`), `Debug`, `Clone`, `Eq`, `PartialEq`, `Hash`

**Given** `App::init_state::<GameState>()` is called in `main.rs`
**When** the app starts
**Then** `State<GameState>::get()` returns `GameState::Loading` on first frame

**Given** a debug system registered on `OnEnter(GameState::Loading)` emits an `info!` log
**When** the app starts
**Then** the log contains the expected "entered Loading" line
**And** no further state transitions happen automatically in this story (the transition to `MainMenu` is Story 1.7)

## Story 1.7: Splash Screen Shows "asteroids3D" and Transitions to MainMenu

As a player launching the game,
I want to see "asteroids3D" displayed when the app opens,
So that I immediately know the app launched and I'm in the right program.

**Acceptance Criteria:**

**Given** the app is in `GameState::Loading`
**When** `OnEnter(GameState::Loading)` runs
**Then** a `bevy_ui` text Node is spawned with content `"asteroids3D"`
**And** the Node uses centered flexbox layout that scales to window size
**And** the text entity carries a `LoadingStateEntity` marker component

**Given** the splash is visible
**When** a configurable splash-duration elapses (duration loaded from a `SplashConfig` resource, default 2.0 seconds)
**Then** the app mutates `NextState<GameState>` to `MainMenu`

**Given** the state transitions from `Loading` to `MainMenu`
**When** `OnExit(GameState::Loading)` runs
**Then** all entities tagged with `LoadingStateEntity` are despawned
**And** no orphaned splash text remains in the hierarchy

**Given** the app is now in `GameState::MainMenu`
**When** the window is inspected visually
**Then** the splash text is gone (MainMenu UI is a later epic's responsibility — this story ends at the transition)

## Story 1.8: Tracing-Based Logging with Panic Hook to Log File

As a developer,
I want `tracing`-based logging with a panic hook that writes stack traces to a log file in the user-log-dir,
So that crashes during CI runs or future playtesting can be forensically reviewed after process exit.

**Acceptance Criteria:**

**Given** `tracing_subscriber` is initialized in `main.rs` before `App::new()`
**When** the app runs
**Then** `info!` / `warn!` / `error!` events from Bevy and app code are output to stderr
**And** `RUST_LOG=debug cargo run` increases verbosity to `debug!` level

**Given** the `directories` crate resolves the per-OS user-log-dir (Windows `%APPDATA%\asteroids3D\logs\`, Linux `$XDG_STATE_HOME/asteroids3d/logs/` or fallback, macOS `~/Library/Logs/asteroids3D/`)
**When** a log file is opened at startup
**Then** logs are written to both stderr and the file simultaneously

**Given** a panic hook is installed via `std::panic::set_hook`
**When** a panic is triggered (e.g., via a `#[cfg(test)]` panic test or manual `panic!()` in a dev-only build)
**Then** the panic message and backtrace are written to the log file before process exit
**And** the default panic behavior (printing to stderr) is preserved

<!-- Epic 1 complete — 8 stories cover M0 completion criterion. Next epic to decompose: Epic 2 (Vector Aesthetic Tech Spike / M1). -->
