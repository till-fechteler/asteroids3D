# Story 2.5: Three-Backend Parity Validation Gate

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As the project author,
I want the reference scene (toon shading + outlines) rendered on Metal (macOS), Vulkan (Linux), and DX12 (Windows) at a fixed deterministic camera transform, captured as 1920×1080 PNGs and committed alongside a pairwise-diff parity report,
So that M1's completion criterion has objective, reviewable evidence and any cross-backend WGSL-translation divergence is documented before Story 2.6's go/fallback decision.

## Acceptance Criteria

1. **Given** an opt-in capture mode entered via the `ASTEROIDS3D_CAPTURE_PNG=<path>` environment variable
   **When** `cargo run` (debug build) is executed with that env var set
   **Then** a 1920×1080 PNG is written to `<path>` with the camera at the deterministic transform `Transform::from_xyz(0.0, 1.5, 6.0).looking_at(Vec3::ZERO, Vec3::Y)` (matching the reference scene's existing default at `src/visual/reference_scene.rs:51`)
   **And** the binary exits with `AppExit::Success` after the screenshot finishes writing
   **And** when the env var is **unset**, the binary's behavior is byte-identical to the post-2.4 baseline (no capture path runs, no extra plugins registered, no extra log lines emitted)

2. **Given** the macOS dev machine (Apple Silicon, Metal backend)
   **When** `ASTEROIDS3D_CAPTURE_PNG=docs/tech-spike/m1-backends/metal.png cargo run` is executed at the project root
   **Then** the resulting PNG is committed to `docs/tech-spike/m1-backends/metal.png`
   **And** the file is exactly 1920×1080 verified via `sips -g pixelWidth -g pixelHeight <path>` (macOS) or `file <path>` showing `PNG image data, 1920 x 1080`
   **And** the screenshot shows all three placeholders (asteroid icosphere, ship cuboid, projectile UV-sphere) with toon shading + silhouette outlines, plus the 5-swatch palette bar and splash text overlay (whichever GameState is active at capture frame)

3. **Given** the `parity-capture.yml` GitHub Actions workflow (manual `workflow_dispatch` trigger)
   **When** the workflow is dispatched against `master`
   **Then** the workflow's `linux-vulkan` job (`ubuntu-latest` + Mesa lavapipe + xvfb) produces `vulkan.png` as a 1920×1080 artifact
   **And** the workflow's `windows-dx12` job (`windows-latest` + WARP fallback for missing GPU) produces `dx12.png` as a 1920×1080 artifact
   **And** both artifacts are downloadable from the workflow run page

4. **Given** the three artifacts/files (`metal.png`, `vulkan.png`, `dx12.png`) exist under `docs/tech-spike/m1-backends/`
   **When** `docs/tech-spike/m1-backends/parity-report.md` is authored
   **Then** the report includes a pairwise diff table for the three pairs (Metal↔Vulkan, Metal↔DX12, Vulkan↔DX12) with: (a) `compare -metric AE` absolute pixel-difference count from ImageMagick, (b) `compare -metric RMSE` root-mean-square error, (c) a one-sentence root-cause hypothesis for any divergence
   **And** any pair with `RMSE > 5%` (i.e. `> 0.05` on the normalized 0..1 scale) carries an explicit annotation explaining the divergence (AA jitter / software-rasterizer color delta / shader-translation drift / etc.)
   **And** the report closes with a single explicit recommendation line — exactly one of: `RECOMMEND GO toon`, `RECOMMEND GO toon with scope reduction`, or `RECOMMEND FALLBACK flat+rim-light` — plus a one-paragraph justification feeding Story 2.6's decision

5. **Given** Story 2.4's post-2.4 test count of 13 passing tests
   **When** `cargo test` is executed at story end
   **Then** the test count is **14 passed, 0 failed** — Story 2.5 adds exactly one new unit test asserting that `CapturePlugin::is_active()` (or equivalent name-resolution check) returns `false` when `ASTEROIDS3D_CAPTURE_PNG` is unset, demonstrating the no-side-effects-when-disabled invariant from AC #1

## Tasks / Subtasks

- [x] **Task 1: Author `src/visual/capture.rs`** (AC: #1)
  - [x] Create new file `src/visual/capture.rs`. Architecture mandates `src/visual/` is the home for visual-presentation modules per `architecture.md:603-607`; capture is presentation-adjacent (composes Camera3d + window-resolution + screenshot save), so `src/visual/capture.rs` is the right home. **Do NOT** put it under `src/dev/` or `src/tools/` — those directories don't exist and architecture.md doesn't sanction them.
  - [x] File contents skeleton (full implementation in subsequent subtasks):
    ```rust
    //! M1 tech-spike screenshot capture (Story 2.5).
    //! Activated by setting ASTEROIDS3D_CAPTURE_PNG=<path> in the environment.
    //! When the env var is unset, this plugin's `build()` is a no-op — the binary
    //! behaves byte-identically to the post-2.4 baseline.
    //!
    //! Removal note: this entire module + its main.rs registration become dead
    //! once Story 3.1 replaces the reference scene with the Arena state. The
    //! capture path is M1-spike-only; no gameplay code may consume it.

    use bevy::prelude::*;
    use bevy::render::view::screenshot::{Screenshot, save_to_disk, ScreenshotCaptured};
    use bevy::window::PrimaryWindow;
    use std::path::PathBuf;

    pub const CAPTURE_ENV_VAR: &str = "ASTEROIDS3D_CAPTURE_PNG";

    /// Returns Some(path) if capture is requested, None otherwise.
    pub fn requested_capture_path() -> Option<PathBuf> {
        std::env::var_os(CAPTURE_ENV_VAR).map(PathBuf::from)
    }

    pub struct CapturePlugin {
        pub output_path: PathBuf,
    }

    #[derive(Resource)]
    struct CaptureState {
        output_path: PathBuf,
        frames_observed: u32,
        capture_triggered: bool,
        capture_completed: bool,
    }

    impl Plugin for CapturePlugin {
        fn build(&self, app: &mut App) {
            app.insert_resource(CaptureState {
                output_path: self.output_path.clone(),
                frames_observed: 0,
                capture_triggered: false,
                capture_completed: false,
            })
            .add_systems(Update, drive_capture);
        }
    }
    ```
    The `drive_capture` system body is specified in subtasks below.
  - [x] **Why an env var, not a CLI arg:** `clap` and friends are not in `Cargo.toml`; adding a CLI parser for one tech-spike codepath is YAGNI. `std::env::var_os` is in std. Env var is also CI-friendly — GitHub Actions sets env via `env:` keys without needing shell-escaping for paths.
  - [x] **Why `var_os` not `var`:** `var_os` returns `Option<OsString>` and tolerates non-UTF-8 paths (Windows file paths can contain non-UTF-8 bytes via UNC). `var` would silently fail on such paths. This is the same idiom Bevy itself uses in its asset-server path resolution. [Source: std docs `env::var_os`]
  - [x] **`drive_capture` system body:**
    ```rust
    fn drive_capture(
        mut commands: Commands,
        mut state: ResMut<CaptureState>,
        primary_window: Query<Entity, With<PrimaryWindow>>,
        mut app_exit: MessageWriter<AppExit>,
    ) {
        if state.capture_completed {
            // Wait one extra frame after the on-disk save observer fires so wgpu's
            // command queue has flushed; then exit.
            app_exit.write(AppExit::Success);
            return;
        }
        state.frames_observed += 1;
        if state.capture_triggered {
            return;
        }
        // Frame budget: 60 frames at default ~60 Hz = ~1 second. Long enough for
        // tuning.ron to load (asset-loader thread) → AssetEvent::Added → outline
        // propagator → outlines reflect RON values. Splash flow has its own 2-second
        // timer, but capture starts the camera in MainMenu via the splash bypass
        // (subtask below) so 60 frames after Startup is post-MainMenu-entry + asset
        // settle.
        const CAPTURE_FRAME: u32 = 60;
        if state.frames_observed < CAPTURE_FRAME {
            return;
        }
        let Ok(_window) = primary_window.single() else {
            // No primary window yet — defer until next frame. Possible on the very
            // first Update before bevy_window has spawned the PrimaryWindow entity.
            return;
        };
        let path = state.output_path.clone();
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path));
        state.capture_triggered = true;
        // capture_completed is set by the observer below.
    }

    fn on_screenshot_captured(
        _trigger: On<ScreenshotCaptured>,
        mut state: ResMut<CaptureState>,
    ) {
        state.capture_completed = true;
        info!(
            "Screenshot capture finished; exiting on next frame. Path: {}",
            state.output_path.display()
        );
    }
    ```
    The `on_screenshot_captured` is registered as a global observer in `build()` via `app.add_observer(on_screenshot_captured);` — Bevy 0.18's renamed global-observer API. **Do NOT** call `.observe(on_screenshot_captured)` on the `Screenshot` entity; that observer fires only for that entity's events, but `ScreenshotCaptured` is triggered globally per the source at `bevy_render-0.18.1/src/view/window/screenshot.rs:198-206`. **However**, for the `save_to_disk` chain, the file-write observer attached via `.observe(save_to_disk(path))` IS entity-scoped and that's correct — it only writes for that capture's event.
  - [x] **Why `MessageWriter<AppExit>` not `EventWriter<AppExit>`:** Bevy 0.18 renamed the cross-system message primitive (per Story 2.3's deviation note 3). Use `MessageWriter`. AppExit is itself a `Message` in 0.18, not an `Event`. [Source: `bevy_app-0.18.1/src/app.rs` AppExit definition]
  - [x] **Why a two-frame exit (capture_triggered → capture_completed → AppExit):** the on-disk write happens asynchronously on a render-world thread. Sending AppExit immediately after spawning the Screenshot can race with the file write, producing 0-byte or truncated PNGs. The state machine (frames count → trigger → captured-observer → exit-on-next-frame) sequences write completion before exit. **Don't** hand-write a `sleep` or a frame-counter-after-trigger — the `ScreenshotCaptured` event is the upstream-provided synchronization primitive; use it. [Source: `bevy_render-0.18.1/src/view/window/screenshot.rs:53-58` doc-comment "Screenshots are captured asynchronously and may not be available immediately after the frame they were spawned in"]
  - [x] **One unit test in `#[cfg(test)] mod tests`** (AC #5 enforcement):
    ```rust
    #[test]
    fn capture_disabled_when_env_var_unset() {
        // Safety: tests run single-threaded under cargo test --test-threads=1 OR cargo
        // test default (parallel) — env var mutation is a global; we read-only here.
        // SAFETY of env::remove_var: this test only READS the env var via requested_capture_path;
        // it does not mutate, so no cross-test interference.
        // If a developer later adds a test that SETS this env var, they MUST gate the test on
        // #[serial] (serial_test crate, not currently in deps) or use a child-process test harness.
        let prior = std::env::var_os(CAPTURE_ENV_VAR);
        // Snapshot — restore at end.
        // SAFETY: assumes no concurrent test mutates this env var.
        unsafe { std::env::remove_var(CAPTURE_ENV_VAR); }
        let result = requested_capture_path();
        // Restore prior value if any.
        if let Some(value) = prior {
            unsafe { std::env::set_var(CAPTURE_ENV_VAR, value); }
        }
        assert!(result.is_none(), "capture must be inert when env var unset");
    }
    ```
    **Why `unsafe`:** Rust 1.81+ marked `std::env::set_var` and `std::env::remove_var` as `unsafe` because env mutation is racy across threads (POSIX `setenv`/`getenv` are not thread-safe). Our MSRV is 1.89 (per `Cargo.toml:5`), so the `unsafe` block is required. The `# Safety` comment must justify single-threaded access. [Source: Rust 1.81 release notes; `std::env::set_var` doc]
  - [x] **Test coverage gap acknowledged:** runtime behavior of `drive_capture` (frame counting, screenshot spawn, AppExit emission) requires `App::new()` with the render plugins, which is integration-test territory. Architecture.md:354 defers integration tests post-M3. Verification of capture-mode runtime behavior is via Tasks 4+5 (real captures + parity-report inspection), not via `cargo test`.

- [x] **Task 2: Wire `CapturePlugin` + window resolution override into `main.rs`** (AC: #1, #2)
  - [x] Edit `src/main.rs`:
    - Add at top of `main()` after `init_logging()` and before `App::new()`:
      ```rust
      let capture_path = visual::capture::requested_capture_path();
      ```
    - Modify the `DefaultPlugins.build()` chain to add a `WindowPlugin` override when capture is active:
      ```rust
      let default_plugins = DefaultPlugins
          .build()
          .disable::<bevy::log::LogPlugin>()
          .set(AssetPlugin {
              watch_for_changes_override: cfg!(debug_assertions).then_some(true),
              ..default()
          });
      let default_plugins = if capture_path.is_some() {
          default_plugins.set(WindowPlugin {
              primary_window: Some(Window {
                  resolution: bevy::window::WindowResolution::new(1920.0, 1080.0),
                  resizable: false,
                  decorations: false,
                  title: String::from("asteroids3D capture"),
                  ..default()
              }),
              ..default()
          })
      } else {
          default_plugins
      };
      ```
    - Then `App::new().add_plugins(default_plugins)` continues as before.
    - After `.add_plugins(VisualPlugin)`, conditionally add the capture plugin:
      ```rust
      if let Some(path) = capture_path {
          app.add_plugins(visual::capture::CapturePlugin { output_path: path });
      }
      ```
      This means the capture plugin attaches to the live `App` only when capture is requested. `app` is the `App` value from `App::new().add_plugins(...)…`; the existing `main.rs` chains methods on a temp value — restructure to a `let mut app = App::new();` binding so we can conditionally add the plugin AND keep the final `app.run()` return value. Final structure:
      ```rust
      fn main() -> AppExit {
          let log_path = init_logging();
          if let Some(path) = &log_path {
              info!("file logging active at {}", path.display());
          }
          let capture_path = visual::capture::requested_capture_path();

          let default_plugins = /* … as above … */;

          let mut app = App::new();
          app.add_plugins(default_plugins)
              .init_state::<GameState>()
              .add_plugins(TuningPlugin)
              .add_plugins(VisualPlugin)
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
              .add_systems(OnExit(GameState::Loading), cleanup_loading_entities);

          if let Some(path) = capture_path {
              app.add_plugins(visual::capture::CapturePlugin { output_path: path });
          }
          app.run()
      }
      ```
  - [x] **Why decorations: false:** capture mode runs in CI under xvfb where window decorations may be drawn by the WM differently across distros. Disabling decorations gives a guaranteed 1920×1080 client-area framebuffer matching the screenshot. (Bevy's `Screenshot::primary_window()` captures the entire surface — typically client area, but consistency matters.)
  - [x] **Why resizable: false:** prevents accidental viewport-size drift if a CI runner's xvfb defaults to a smaller virtual display.
  - [x] **Why title contains "capture":** so a developer running locally can identify the capture window vs a normal dev run if both are open.
  - [x] **Bevy 0.18 deviation watch:** in 0.18 `Window::resolution: WindowResolution` is the field-typed setter; passing a `(f32, f32)` tuple won't work. Use `WindowResolution::new(1920.0, 1080.0)`. [Source: `bevy_window-0.18.1/src/window.rs` Window struct definition]

- [x] **Task 3: Bypass splash + force deterministic camera in capture mode** (AC: #1, #2)
  - [x] **Splash bypass.** The capture happens at frame 60 (~1 second). Splash duration is 2 seconds (`src/splash.rs:8`). At capture time the binary is still in `GameState::Loading` (the splash state), which means the swatch UI hasn't spawned yet (it spawns on `OnEnter(GameState::MainMenu)`). To get a parity capture that includes the **post-MainMenu** scene (which is what Stories 2.6 / 2.7 will compare against), we need to skip splash.

    Add the bypass to `src/visual/capture.rs::CapturePlugin::build`:
    ```rust
    impl Plugin for CapturePlugin {
        fn build(&self, app: &mut App) {
            // … insert_resource as in Task 1 …
            app.add_systems(Startup, force_skip_splash_in_capture);
            app.add_observer(on_screenshot_captured);
            app.add_systems(Update, drive_capture);
        }
    }

    fn force_skip_splash_in_capture(mut config: ResMut<crate::splash::SplashConfig>) {
        // Tick the splash timer past its duration so the very next Update transitions
        // GameState::Loading → MainMenu. This avoids the 2-second wait without modifying
        // splash.rs (which is not capture-aware) and keeps the splash flow intact for
        // non-capture builds.
        config.timer.tick(std::time::Duration::from_secs(10));
    }
    ```
    **Why a Startup system, not a oneshot at App build:** the `SplashConfig` resource is initialized via `init_resource::<SplashConfig>()` AFTER `add_plugins(VisualPlugin)` in main.rs, so attempting to mutate it from `build()` would fail (resource not yet registered). Startup runs after all plugin `build()`s and after `init_resource`, so the resource is guaranteed present. [Source: Bevy startup-schedule ordering]
    **Why `from_secs(10)` not `from_secs(2)`:** tick semantics — the timer fires `just_finished()` only on the tick where elapsed crosses duration. Ticking by 10 seconds well past the 2-second duration is a deliberate margin (the actual elapsed is clamped internally; no risk of drift).
  - [x] **Deterministic camera transform.** The reference scene already spawns Camera3d at `Transform::from_xyz(0.0, 1.5, 6.0).looking_at(Vec3::ZERO, Vec3::Y)` (`reference_scene.rs:51`). This is the deterministic transform AC #1 specifies. **No override is needed** in capture mode — the reference scene's existing default IS the capture transform. This deliberately avoids a "capture-mode-specific transform" diverging from the dev's everyday view; the screenshot reproduces what the dev sees in `cargo run` after splash settles.
    Document this in `capture.rs`:
    ```rust
    // Deterministic camera transform: reference_scene.rs:51 already spawns Camera3d
    // at Transform::from_xyz(0.0, 1.5, 6.0).looking_at(Vec3::ZERO, Vec3::Y). Capture
    // mode reuses this — no separate "capture transform" exists. AC #1 (Story 2.5).
    ```
  - [x] **No `Time<Virtual>::pause()`:** the reference scene has no animation systems (PointLights are static, meshes don't move, no rotators). "Scene time frozen at t=0" (epic AC) is naturally satisfied by the reference scene's design. Adding `pause()` would introduce a dependency on `Time<Virtual>` semantics that's not justified at this scope.

- [x] **Task 4: Wire `pub mod capture;` and run the macOS capture** (AC: #1, #2)
  - [x] Edit `src/visual/mod.rs`:
    - Add `pub mod capture;` after `pub mod outline;` (alphabetical order across the existing 3 sub-modules: `capture`, `outline`, `palette`, `toon_material`).
    - Update top-of-file `//!` doc-comment: append a sixth line: "Story 2.5 adds opt-in screenshot capture (`ASTEROIDS3D_CAPTURE_PNG`) for M1 backend-parity validation."
  - [x] **Local macOS Metal capture:**
    ```bash
    mkdir -p docs/tech-spike/m1-backends
    ASTEROIDS3D_CAPTURE_PNG=docs/tech-spike/m1-backends/metal.png cargo run 2>&1 | tee /tmp/story-2-5-capture-metal.log
    ```
    Wait for the log line `Screenshot capture finished; exiting on next frame.` then verify exit code 0.
  - [x] **Verify dimensions:**
    ```bash
    sips -g pixelWidth -g pixelHeight docs/tech-spike/m1-backends/metal.png
    ```
    Expected: `pixelWidth: 1920`, `pixelHeight: 1080`.
    Also: `file docs/tech-spike/m1-backends/metal.png` → `PNG image data, 1920 x 1080, 8-bit/color RGBA, non-interlaced`.
  - [x] **Visual check (no AC, but self-evident):** open `metal.png`. Expected: 3 placeholders with toon shading + outlines, swatch palette bar at top, dark background, no splash text (we skipped splash → MainMenu transition fired → splash entities cleaned up by `cleanup_loading_entities`). If splash text is visible, the bypass didn't fire — diagnose Task 3.
  - [x] **Cold-start race acknowledgment:** at frame 60 (~1 second), the asset loader may or may not have parsed `tuning.ron` yet — depends on disk speed and loader scheduling. The reference scene's `unwrap_or_default()` fallback (per Story 2.4 Task 5) ensures `metal.png` shows the **default** outline values (`width: 3.0`, `color: srgba(0.05, 0.05, 0.05, 1.0)`) regardless. If you want the RON-driven values, bump `CAPTURE_FRAME` to 120 or 180. For Story 2.5 parity-gate purposes, defaults are fine — the parity comparison cares about backend-translation correctness, not RON-driven configurability (which is Story 2.4's concern, already validated).

- [x] **Task 5: Author `.github/workflows/parity-capture.yml`** (AC: #3)
  - [x] Create new workflow file `.github/workflows/parity-capture.yml`. Manual `workflow_dispatch` only — do NOT trigger on push, this is a deliberate, gated CI cost. `_bmad/**` and `_bmad-output/**` paths-ignore from the main `ci.yml` does NOT apply here; this workflow has its own narrow trigger.
  - [x] Workflow body:
    ```yaml
    name: M1 Parity Capture

    # Manual trigger only — capture jobs are expensive and not needed on every push.
    # Run when Story 2.5 needs fresh PNGs (or when a future shader change warrants
    # re-validation of cross-backend parity).

    on:
      workflow_dispatch:

    env:
      CARGO_TERM_COLOR: always
      ASTEROIDS3D_CAPTURE_PNG: capture.png

    jobs:
      linux-vulkan:
        name: linux-vulkan (Mesa lavapipe)
        runs-on: ubuntu-latest
        timeout-minutes: 60
        steps:
          - uses: actions/checkout@v4

          - name: Free disk space
            uses: jlumbroso/free-disk-space@v1.3.1
            with:
              tool-cache: false
              android: true
              dotnet: true
              haskell: true
              large-packages: false
              docker-images: true
              swap-storage: false

          - name: Install Linux system dependencies
            env:
              DEBIAN_FRONTEND: noninteractive
            run: |
              sudo apt-get update -y
              sudo apt-get install -y \
                pkg-config \
                libx11-dev \
                libasound2-dev \
                libudev-dev \
                libxkbcommon-x11-0 \
                libwayland-dev \
                libxkbcommon-dev \
                xvfb \
                mesa-vulkan-drivers \
                vulkan-tools

          - name: Verify Vulkan adapter is reachable
            run: |
              # vulkaninfo --summary lists adapters; lavapipe should appear as
              # "llvmpipe (LLVM…)" — proves the software ICD is installed before
              # we waste 10+ minutes compiling Bevy against a missing Vulkan stack.
              vulkaninfo --summary || (echo "Vulkan stack missing; aborting" && exit 1)

          - uses: dtolnay/rust-toolchain@master
            with:
              toolchain: stable

          - uses: Swatinem/rust-cache@v2
            with:
              cache-on-failure: true

          - name: Run capture (Vulkan via lavapipe under xvfb)
            env:
              WGPU_BACKEND: vulkan
              # lavapipe is the LLVM software-rasterized Vulkan ICD. Force it as
              # the only adapter so wgpu doesn't pick a hypothetical hardware
              # adapter that may flake on virtualized GitHub Actions hosts.
              VK_ICD_FILENAMES: /usr/share/vulkan/icd.d/lvp_icd.x86_64.json
              ASTEROIDS3D_CAPTURE_PNG: vulkan.png
              # RUST_LOG: "info,wgpu=warn,naga=warn"
            run: xvfb-run -a -s "-screen 0 1920x1080x24" cargo run --locked

          - name: Verify dimensions
            run: |
              file vulkan.png | grep -q "1920 x 1080" || (echo "wrong dimensions" && exit 1)

          - name: Upload artifact
            uses: actions/upload-artifact@v4
            with:
              name: vulkan
              path: vulkan.png
              if-no-files-found: error

      windows-dx12:
        name: windows-dx12 (WARP fallback)
        runs-on: windows-latest
        timeout-minutes: 60
        steps:
          - uses: actions/checkout@v4

          - uses: dtolnay/rust-toolchain@master
            with:
              toolchain: stable

          - uses: Swatinem/rust-cache@v2
            with:
              cache-on-failure: true

          - name: Run capture (DX12 — WARP if no GPU)
            env:
              WGPU_BACKEND: dx12
              # On GitHub Actions windows-latest, no real GPU is exposed; wgpu falls
              # back to the WARP software DX12 adapter. No env override needed —
              # wgpu's default adapter selection picks WARP when it's the only DX12
              # adapter available. Setting WGPU_ADAPTER_NAME explicitly would over-
              # constrain on hosts where a real GPU exists.
              ASTEROIDS3D_CAPTURE_PNG: dx12.png
            run: cargo run --locked

          - name: Verify dimensions
            shell: pwsh
            run: |
              $img = [System.Drawing.Image]::FromFile("$PWD\dx12.png")
              if ($img.Width -ne 1920 -or $img.Height -ne 1080) {
                Write-Error "wrong dimensions: $($img.Width)x$($img.Height)"
                exit 1
              }

          - name: Upload artifact
            uses: actions/upload-artifact@v4
            with:
              name: dx12
              path: dx12.png
              if-no-files-found: error
    ```
  - [x] **Why no macos-vulkan job:** MoltenVK on macOS would validate the SPIR-V emission path (Naga WGSL→SPIR-V→Metal-via-MoltenVK), but the AC asks for "Linux Vulkan", not "macOS-via-MoltenVK Vulkan". The Linux Vulkan job covers SPIR-V validation; adding a macos-vulkan job would be duplicate scope.
  - [x] **Why `if-no-files-found: error` on uploads:** if the capture binary panics or the screenshot save observer never fires, the artifact upload would silently produce a 0-byte payload. This setting forces the workflow to fail loudly. [Source: `actions/upload-artifact@v4` docs]
  - [x] **Why no clippy/fmt steps:** this workflow is capture-only. The main `ci.yml` already runs clippy and fmt on every push. Duplicating here would be wasted CI minutes.
  - [x] **Cost estimate:** Linux job ~8-12 min (5min apt-install + cargo build cold + 30s capture); Windows job ~12-20 min (cold build dominates); total per dispatch ~30-40 CI-minutes. Within hobby budget, manual trigger gates cost.
  - [x] **paths-ignore impact:** this workflow has NO `paths-ignore`. It triggers ONLY on `workflow_dispatch`, so push-time path filtering is N/A.

- [x] **Task 6: Dispatch parity-capture, download artifacts, commit PNGs** (AC: #3, #4)
  - [x] **Dispatch:**
    ```bash
    gh workflow run parity-capture.yml --ref master
    sleep 10
    gh run list --workflow=parity-capture.yml -L 1
    ```
    Capture the run ID from the list output.
  - [x] **Wait for completion:**
    ```bash
    gh run watch <RUN_ID>
    ```
    Both jobs (`linux-vulkan`, `windows-dx12`) must succeed. If either fails, diagnose:
    - **`linux-vulkan` failure modes:** vulkaninfo missing → apt step failed (likely transient mirror issue, retry). xvfb can't start → screen geometry conflict (the `-screen 0 1920x1080x24` arg should avoid this). wgpu picks wrong adapter → check `WGPU_BACKEND=vulkan` env is propagated; check `VK_ICD_FILENAMES` resolves to a real file (`ls -la $VK_ICD_FILENAMES` debug step).
    - **`windows-dx12` failure modes:** WARP not present on runner image → unlikely (WARP ships with Windows since 7), but if it happens, set `WGPU_ADAPTER_NAME="Microsoft Basic Render Driver"`. Build OOM on cold cache → reduce `[profile.dev.package."*"] opt-level = 0` (currently 1) — but don't commit that change; revert after capture. Cargo cache cold (10+ min build) → expected on first run; second dispatch hits Swatinem's cache.
  - [x] **Download artifacts:**
    ```bash
    gh run download <RUN_ID> --dir /tmp/parity-artifacts
    cp /tmp/parity-artifacts/vulkan/vulkan.png docs/tech-spike/m1-backends/vulkan.png
    cp /tmp/parity-artifacts/dx12/dx12.png docs/tech-spike/m1-backends/dx12.png
    ```
  - [x] **Verify all three PNGs exist:**
    ```bash
    ls -la docs/tech-spike/m1-backends/{metal,vulkan,dx12}.png
    file docs/tech-spike/m1-backends/{metal,vulkan,dx12}.png
    ```
    All three must report `PNG image data, 1920 x 1080`.

- [x] **Task 7: Author `docs/tech-spike/m1-backends/parity-report.md`** (AC: #4)
  - [x] **Pre-flight: install ImageMagick if absent.**
    ```bash
    if ! command -v compare &> /dev/null; then brew install imagemagick; fi
    ```
    Note: the `compare` binary in modern ImageMagick (v7) may be invoked as `magick compare`. Test once: `compare -version || magick compare -version`.
  - [x] **Run pairwise diffs.** Three pairs: M↔V, M↔D, V↔D.
    ```bash
    cd docs/tech-spike/m1-backends/

    # AE = absolute pixel count where channels differ. RMSE = root-mean-square
    # error (0..1 normalized). The leading `compare -metric X A B null:` produces
    # ONLY the metric to stderr; we capture via `2>&1 1>/dev/null`.

    ae_mv=$(compare -metric AE metal.png vulkan.png null: 2>&1 1>/dev/null)
    rmse_mv=$(compare -metric RMSE metal.png vulkan.png null: 2>&1 1>/dev/null)
    ae_md=$(compare -metric AE metal.png dx12.png null: 2>&1 1>/dev/null)
    rmse_md=$(compare -metric RMSE metal.png dx12.png null: 2>&1 1>/dev/null)
    ae_vd=$(compare -metric AE vulkan.png dx12.png null: 2>&1 1>/dev/null)
    rmse_vd=$(compare -metric RMSE vulkan.png dx12.png null: 2>&1 1>/dev/null)

    # Optional visual diffs — produce a heatmap PNG per pair.
    compare metal.png vulkan.png diff-metal-vs-vulkan.png
    compare metal.png dx12.png diff-metal-vs-dx12.png
    compare vulkan.png dx12.png diff-vulkan-vs-dx12.png

    echo "M↔V: AE=$ae_mv RMSE=$rmse_mv"
    echo "M↔D: AE=$ae_md RMSE=$rmse_md"
    echo "V↔D: AE=$ae_vd RMSE=$rmse_vd"
    ```
    **Note on RMSE format:** ImageMagick prints RMSE as `<absolute_value> (<normalized_0_to_1>)` — capture both. The `> 5%` threshold from AC #4 maps to the normalized value `> 0.05`.
  - [x] **Author the report** at `docs/tech-spike/m1-backends/parity-report.md`. Template:
    ```markdown
    # M1 Backend Parity Report

    **Date:** <YYYY-MM-DD of capture run>
    **Stories evidenced:** 2.3 (toon material), 2.4 (outline integration), 2.5 (this gate).
    **Reference scene:** asteroid icosphere + ship cuboid + projectile UV-sphere, 3-point lighting,
    deterministic camera at `Transform::from_xyz(0.0, 1.5, 6.0).looking_at(Vec3::ZERO, Vec3::Y)`,
    1920×1080 PNG, captured at frame 60 (~1 second post-Startup).

    ## Capture environment

    | Backend | Platform | Renderer | Capture command |
    |---|---|---|---|
    | Metal | macOS (Apple Silicon, M5 Pro / dev box) | hardware Metal | `ASTEROIDS3D_CAPTURE_PNG=… cargo run` (local) |
    | Vulkan | Linux (ubuntu-latest CI runner) | Mesa lavapipe (LLVM software ICD) | `parity-capture.yml linux-vulkan` job, run <ID> |
    | DX12 | Windows (windows-latest CI runner) | WARP (Microsoft software adapter) | `parity-capture.yml windows-dx12` job, run <ID> |

    **Important context for divergence interpretation:** Vulkan and DX12 here are SOFTWARE-rendered
    on virtualized CI hosts (no GPU passthrough). Pixel-level deltas vs hardware Metal include
    BOTH (a) WGSL-translation correctness — the M1 spike's actual concern — AND (b) software-vs-
    hardware rasterizer rounding/AA differences that say nothing about correctness. The qualitative
    visual-equivalence check (next section) is the load-bearing parity signal; quantitative pixel
    counts are an upper bound that includes irrelevant noise.

    ## Pairwise diffs

    | Pair | AE (absolute pixel diff count) | RMSE (raw / normalized) | Heatmap |
    |---|---|---|---|
    | Metal ↔ Vulkan | <ae_mv> | <rmse_mv_raw> / <rmse_mv_norm> | `diff-metal-vs-vulkan.png` |
    | Metal ↔ DX12 | <ae_md> | <rmse_md_raw> / <rmse_md_norm> | `diff-metal-vs-dx12.png` |
    | Vulkan ↔ DX12 | <ae_vd> | <rmse_vd_raw> / <rmse_vd_norm> | `diff-vulkan-vs-dx12.png` |

    **Total pixels in 1920×1080:** 2,073,600. AE percentages = AE / 2073600 × 100.

    ## Divergence root-cause hypotheses

    *(Author one paragraph per pair where RMSE > 0.05; otherwise note "no divergence above threshold".)*

    ### Metal ↔ Vulkan

    *(Expected: AA jitter at outline edges + software-rasterizer color rounding. The toon shader
    is deterministic, so band boundaries should align; differences should be confined to anti-
    aliased edge pixels and the rim-light gradient where small dot-product deltas accumulate.)*

    ### Metal ↔ DX12

    *(Expected: similar to Metal↔Vulkan plus DX12-specific HLSL-translation artefacts. WARP's
    color rounding may differ from lavapipe's. Outline silhouette positions should align — if
    they don't, that's a bevy_mod_outline backend bug.)*

    ### Vulkan ↔ DX12

    *(Expected: smallest divergence — both software-rendered, both go through Naga WGSL→IR
    translation. Differences here are rasterizer-implementation deltas, not WGSL-translation
    deltas.)*

    ## Qualitative visual equivalence checks (load-bearing)

    Visually compare the three PNGs side-by-side at 100% zoom (Preview / IrfanView / `feh`).
    For each property, note whether all three agree:

    - [x] **Posterized banding count.** All three placeholders should show ~4 visible bands
          (matching `tuning.ron`'s `toon_steps: 4`). Different band counts across backends would
          indicate a uniform-binding mismatch — a critical regression.
    - [x] **Rim-light at asteroid silhouette.** All three should show the brightening at grazing
          angles. Missing rim on one backend = `pow()` instrinsic divergence.
    - [x] **Per-entity tint colors.** Hazard yellow (asteroid), PlayerOwned blue (ship),
          Salvage green (projectile) — all three placeholders show their assigned hex on all
          backends.
    - [x] **Outline silhouette continuity.** All three placeholders fully outlined on all
          backends. Cuboid corners smooth (Story 2.4's `generate_outline_normals` payoff
          applies regardless of backend).
    - [x] **Outline width visual proportion.** Width as fraction of placeholder size matches
          across backends.
    - [x] **Swatch palette colors.** 5-color palette bar at top renders with matching hex on
          all backends.

    ## Recommendation for Story 2.6

    *(Exactly one of these three lines, with a one-paragraph justification.)*

    > **RECOMMEND GO toon** *or* **RECOMMEND GO toon with scope reduction** *or* **RECOMMEND FALLBACK flat+rim-light**

    *(Justification: e.g. "All six qualitative checks pass; quantitative AE is dominated by
    sub-pixel AA jitter (<X% on all pairs); the toon shader and outline plugin behave correctly
    on all three backends. Recommend GO toon for M2 production." OR "Vulkan rim-light is missing;
    Naga WGSL→SPIR-V regression on `pow()` with rim_power=2.0; recommend GO toon with scope
    reduction (drop rim-light term) until upstream Naga fix lands." OR "DX12 produces black
    output; WARP can't compile our shader; recommend FALLBACK flat+rim-light to unblock M2.")*
    ```
  - [x] **The recommendation IS the artifact Story 2.6 ingests.** Story 2.6 will read this section verbatim and either accept the recommendation or override it with documented rationale. Don't hedge — pick one of the three options.
  - [x] **Why we don't auto-generate the report:** the qualitative-visual-equivalence checklist requires human eyeballs. ImageMagick's RMSE alone can't tell you whether the divergence is "rasterizer noise" or "missing rim-light" — those have similar pixel deltas. The dev's qualitative pass is the M1 gate's actual content. Quantitative numbers are anchoring data, not the verdict.

- [x] **Task 8: Local verification sweep — code paths** (AC: #1, #5)
  - [x] `cargo check 2>&1 | tee /tmp/story-2-5-check.log` → `grep -cE 'warning:|error:' /tmp/story-2-5-check.log` must equal **0**.
  - [x] `cargo build 2>&1 | tee /tmp/story-2-5-build.log` → same grep equals **0**.
  - [x] `cargo test 2>&1 | tee /tmp/story-2-5-test.log` → grep `'warning:|error:|FAILED'` equals **0**; test count must read **14 passed, 0 failed** (13 from 2.4 + 1 new `capture_disabled_when_env_var_unset`). If 13: Task 1 unit test was forgotten. If >14: an unintended test was added.
  - [x] `cargo clippy --all-targets -- -D warnings 2>&1 | tee /tmp/story-2-5-clippy.log` → grep equals **0**. Watch for:
    - `clippy::needless_pass_by_value` on `Res<Assets<TuningConfig>>`-style ECS params — false positive; suppress via `#[expect(name, reason = "Bevy ECS system param convention")]`.
    - `clippy::unsafe_derive_deserialize` — N/A (no Deserialize on capture-mode types).
    - `clippy::ref_option` on `&Option<PathBuf>` — N/A (we own a `PathBuf` in `CaptureState`).
    - The `unsafe { std::env::set_var(...) }` block in the unit test will trigger no clippy warning under `-D warnings` provided the `# Safety` comment is present and accurate. If it does fire, document the suppression with `#[expect(clippy::missing_safety_doc, reason = "test-only single-threaded env mutation")]`.
  - [x] `cargo fmt --all -- --check` → exit 0.
  - [x] `cargo build --release 2>&1 | tee /tmp/story-2-5-release.log` → grep equals **0**. (Even though we capture in debug, release-build cleanliness is a sanity check; the capture-plugin code paths are compiled into release too — they're just inert when env var is unset.)
  - [x] **Capture-mode runtime smoke (no env var):**
    ```bash
    cargo run &> /tmp/story-2-5-baseline-run.log &
    PID=$!
    sleep 6
    kill -TERM $PID
    wait $PID 2>/dev/null
    ```
    Then verify:
    - `grep -c 'capture' /tmp/story-2-5-baseline-run.log` → **0** (capture plugin emits no log lines when env var unset).
    - `grep -c 'TuningReloaded' /tmp/story-2-5-baseline-run.log` → ≥ 1 (Story 2.4 cold-start reload still fires).
    - `grep -c 'splash timer elapsed' /tmp/story-2-5-baseline-run.log` → exactly 1 (real 2-second splash, not bypassed).
    This proves AC #1's "byte-identical to post-2.4 baseline when env var unset" — modulo the absence of capture-plugin code paths in the running process state, the observable log output matches.
  - [x] **Capture-mode runtime smoke (with env var):**
    ```bash
    rm -f /tmp/test-capture.png
    ASTEROIDS3D_CAPTURE_PNG=/tmp/test-capture.png cargo run 2>&1 | tee /tmp/story-2-5-capture-run.log
    ```
    Verify:
    - `grep -c 'Screenshot capture finished' /tmp/story-2-5-capture-run.log` → exactly 1.
    - Exit code 0 (`echo $?` immediately after).
    - `file /tmp/test-capture.png` → `PNG image data, 1920 x 1080, ...`.
    - `grep -c 'splash timer elapsed' /tmp/story-2-5-capture-run.log` → 0 (splash bypass active).

- [x] **Task 9: Scope guardrails — verify nothing else drifted** (AC: all)
  - [x] `git status --short`: expected file set:
    - `src/visual/capture.rs` (??) — new
    - `src/visual/mod.rs` (M) — `pub mod capture;` + doc-comment line
    - `src/main.rs` (M) — capture env-var lookup + conditional WindowPlugin override + conditional `CapturePlugin` registration
    - `.github/workflows/parity-capture.yml` (??) — new
    - `docs/tech-spike/m1-backends/metal.png` (??) — new (Task 4)
    - `docs/tech-spike/m1-backends/vulkan.png` (??) — new (Task 6)
    - `docs/tech-spike/m1-backends/dx12.png` (??) — new (Task 6)
    - `docs/tech-spike/m1-backends/parity-report.md` (??) — new (Task 7)
    - `docs/tech-spike/m1-backends/diff-metal-vs-vulkan.png` (??) — new (optional, Task 7)
    - `docs/tech-spike/m1-backends/diff-metal-vs-dx12.png` (??) — new (optional)
    - `docs/tech-spike/m1-backends/diff-vulkan-vs-dx12.png` (??) — new (optional)
    - `Cargo.toml` — **untouched** (no new dep needed)
    - `Cargo.lock` — **untouched**
    - Bookkeeping: this story file (??) + `sprint-status.yaml` (M) — flipped at Task 11.
  - [x] `grep -nrE 'CapturePlugin|ASTEROIDS3D_CAPTURE_PNG|Screenshot::primary_window|save_to_disk|capture\.rs' src/ --include='*.rs'` → expected hits:
    - `src/visual/capture.rs`: own definition (5-10 hits).
    - `src/visual/mod.rs`: `pub mod capture;` (1 hit).
    - `src/main.rs`: `requested_capture_path()` + `CapturePlugin { ... }` (2-3 hits).
    - **No** hits in `src/state.rs`, `src/splash.rs`, `src/logging.rs`, `src/visual/palette.rs`, `src/visual/toon_material.rs`, `src/visual/outline.rs`, `src/visual/reference_scene.rs`, `src/tuning/{mod,config}.rs`. Capture is a presentation-layer addition; gameplay/state/tuning code stays unaware.
  - [x] `grep -nrE 'Arena|Caravan|PostRun|PhotoMode|Paused' src/ --include='*.rs' | grep -v 'state.rs'` → 0 hits (states still not live, same as post-2.4).
  - [x] `grep -rn 'AssetServer::load\b' src/` → expected hits: **1** — same as post-2.4 (only `src/tuning/mod.rs:39`). Capture mode does not load assets.
  - [x] `cargo tree --depth 1 -p asteroids3D | grep -E 'bevy_mod_outline|bevy '` → unchanged from post-2.4: `bevy v0.18.1`, `bevy_mod_outline v0.12.0`.
  - [x] **CI workflow file integrity:** `gh workflow view parity-capture.yml` should succeed (the workflow registers correctly with GitHub). If it fails with a YAML parse error, fix syntax before pushing.
  - [x] **Files NOT touched (and must NOT be touched by this story):** `Cargo.toml`, `Cargo.lock`, `.gitignore`, `.gitattributes`, `.github/workflows/ci.yml`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`, `docs/plugin-compatibility.md`, `src/state.rs`, `src/splash.rs`, `src/logging.rs`, `src/visual/palette.rs`, `src/visual/toon_material.rs`, `src/visual/outline.rs`, `src/visual/reference_scene.rs`, `src/tuning/mod.rs`, `src/tuning/config.rs`, `assets/config/tuning.ron`, `assets/shaders/toon.wgsl`. Capture mode is purely additive.

- [x] **Task 10: Commit + CI observation** (AC: all)
  - [x] **Commit 1 (source + workflow):** stage `src/visual/{mod,capture}.rs`, `src/main.rs`, `.github/workflows/parity-capture.yml`. **No** docs (those go in Commit 2), **no** Cargo files (none changed), **no** screenshots yet.
    - HEREDOC commit message subject: `feat: M1 backend-parity capture mode + parity-capture CI workflow (Story 2.5)`. Single-line, under 100 chars. Match Till's commit-style precedent (`feat: bevy_mod_outline integration + outline hot-reload (Story 2.4)` from `9011739`).
    - Push to `origin/master`. Triggers full 4-job CI matrix (`ci.yml` paths-ignore = `_bmad/**` + `_bmad-output/**`; `src/`, `.github/workflows/` are NOT ignored).
    - `gh run list --workflow=ci.yml -L 1` → identify run ID. Wait for all 4 jobs (build × 3 OSes + msrv-check) to complete.
    - `gh run view <ID> --log | grep -cE 'warning:|error:'` → 0.
    - All 4 jobs ✅; capture run ID + per-job durations.
    - **Note:** the new `parity-capture.yml` workflow does NOT trigger on push (it's `workflow_dispatch` only) — only main `ci.yml` runs on this push.
  - [x] **Commit 2 (docs + screenshots + parity report):** stage `docs/tech-spike/m1-backends/{metal,vulkan,dx12}.png`, `docs/tech-spike/m1-backends/parity-report.md`, optionally `docs/tech-spike/m1-backends/diff-*.png`.
    - HEREDOC commit message subject: `docs: M1 three-backend parity evidence + go/fallback recommendation (Story 2.5)`.
    - **PNG file size sanity check:** each capture should be 200-700 KB at 1920×1080 with our scene's complexity. >5 MB = something is wrong (likely a non-RGBA encoding fluke); <50 KB = mostly-empty frame, capture failed visually. Eyeball before commit.
    - **Push.** Triggers CI (cached, fast). Capture run ID.
    - **Push-fold optimization:** if Till opts to fold both commits into one push (precedent from Stories 2.2, 2.3, 2.4), only one CI run-ID is captured. Document the fold reasoning in Dev Agent Record.
  - [x] **Why two commits, not three:** Tasks 4, 6, 7 produce artifacts (PNGs + report) that belong together. No reason to split docs from screenshots — they're cohesive evidence for the M1 gate. Splitting would just add bookkeeping cost.

- [x] **Task 11: Ready-for-review handoff + bookkeeping commit**
  - [x] Populate **Dev Agent Record**: Agent Model Used, Debug Log References (per-command grep counts + log paths + screenshot dimensions + parity-report metric values + CI run IDs for `ci.yml` push AND `parity-capture.yml` dispatch), Completion Notes (per-AC evidence + any deviations from spec — e.g. AC #1 "release build" deviation to debug build with rationale; if WARP couldn't be reached on windows-latest and required `WGPU_ADAPTER_NAME` override, capture; if RMSE > 0.05 on any pair, capture root-cause), File List (added / modified).
  - [x] Set this story's `Status:` header → `review`.
  - [x] Update `_bmad-output/implementation-artifacts/sprint-status.yaml`: flip `2-5-three-backend-parity-validation-gate: ready-for-dev → in-progress → review`; bump `last_updated`.
  - [x] **Update `deferred-work.md` if new findings emerged** (likely candidates):
    - The Story 2.2 cfg_attr removal stays re-deferred to Story 4.5 (unchanged by 2.5).
    - The Story 2.3 `extensions: &["ron"]` collision concern stays open until Story 4.7 (unchanged).
    - **NEW potential entry:** if WARP is unavailable on windows-latest in the future, the `parity-capture.yml` job will need `WGPU_ADAPTER_NAME` override; capture as a contingency note rather than pre-build.
    - **NEW potential entry:** the `CapturePlugin` + `mod capture` should be removed at Story 3.1 (when Arena replaces the reference scene; capture mode is M1-only). Add a deferred-work entry "Remove `src/visual/capture.rs` + `parity-capture.yml` workflow at Story 3.1" so the cleanup is tracked.
  - [x] Stage story file + `sprint-status.yaml` (and `deferred-work.md` only if edited), commit with `bmad: story 2.5 ready-for-dev → review (three-backend parity, CI <ID> green)`. `_bmad-output/**` paths-ignored → no CI cost.
  - [x] Push.
  - [x] Story awaits code review. **Multi-LLM adversarial review recommended** for this story given the cross-cutting surface (Bevy plugin, env-var trigger, CI workflow YAML, software-rasterizer assumptions, parity-report subjective interpretation). Run `bmad-code-review` ideally with a different LLM than the implementer.

## Dev Notes

### Why this story exists

Story 2.5 is the **M1 completion gate**. Stories 2.1–2.4 built the vector aesthetic (toon material + semantic palette + outlines + hot-reload). Story 2.5 proves the result actually works on all three GPU backends — the project's central de-risking concern per PRD R#2 ("WGSL shader complexity for a beginner on three graphics backends") and architecture.md:441-443. Without 2.5's evidence, Story 2.6's go/fallback decision has no factual basis.

The architectural decision to gate M1 on cross-backend parity is from architecture.md:295: "M1 — Vector Spike: Custom Toon WGSL Material + bevy_mod_outline, three-backend validation gate." This story is that gate. [Source: architecture.md:295, 441-443; prd.md:347, 405; epics/epic-2-vector-aesthetic-tech-spike.md:121-147]

Three artifacts ship from this story:

1. **Capture-mode binary path** (`src/visual/capture.rs` + `src/main.rs` env-var entry) — opt-in, off when env var unset, allowing the same binary to render normally OR to capture a deterministic screenshot and exit.
2. **GitHub Actions workflow** (`.github/workflows/parity-capture.yml`) — manual `workflow_dispatch` to run the capture on `ubuntu-latest` (Vulkan) and `windows-latest` (DX12) with software rasterizers. macOS captures locally on Till's Apple Silicon hardware (real Metal).
3. **Parity report** (`docs/tech-spike/m1-backends/parity-report.md` + 3 PNGs + optional 3 diff heatmaps) — the human-authored verdict combining quantitative ImageMagick metrics with qualitative visual checks, ending in a single explicit recommendation for Story 2.6.

### Inherited context from Stories 2.1 + 2.2 + 2.3 + 2.4

| Fact | Value | Source |
|---|---|---|
| `src/visual/mod.rs` (post-2.4) | `pub mod outline; pub mod palette; pub mod toon_material;`, `VisualPlugin` registers `MaterialPlugin<ToonMaterial>` + `OutlinePlugin` + 2 systems (`apply_tuning_to_toon_materials` + `outline::apply_tuning_to_outlines`) in `TuningSystems::Reload` Update tuple, cfg-gated `mod reference_scene` | `src/visual/mod.rs` post-2.4 |
| `src/visual/reference_scene.rs` (post-2.4) | Spawns Camera3d at `Transform::from_xyz(0.0, 1.5, 6.0).looking_at(Vec3::ZERO, Vec3::Y)` with `order: -1`, 3 toon-shaded placeholders w/ `OutlineVolume`, 3 PointLights, `ReferenceSceneEntity` markers; gated on `cfg(debug_assertions)`. Capture mode reuses this transform — no override needed. | post-2.4 |
| `src/main.rs` (post-2.4) | `fn main() -> AppExit` that builds DefaultPlugins (minus LogPlugin) with custom AssetPlugin, registers GameState, TuningPlugin, VisualPlugin, SplashConfig, splash flow systems. **Reference scene is `cfg(debug_assertions)` — capture works in debug only.** | `src/main.rs` post-2.4 |
| `src/splash.rs` (post-2.4) | `SplashConfig { timer: Timer::from_seconds(2.0, TimerMode::Once) }`, `tick_splash_timer` advances on `Time<...>::delta()`. Capture-mode bypass ticks the timer past 2 seconds in a Startup system. | `src/splash.rs` post-2.4 |
| `assets/config/tuning.ron` (post-2.4) | `TuningConfig(toon_steps: 4, toon_rim_power: 2.0, toon_rim_intensity: 0.3, outline_width: 3.0, outline_color: (0.05, 0.05, 0.05, 1.0))`. Capture renders these values once asset-load completes (or `Default` values on cold-start race per Story 2.4 Task 5). | post-2.4 |
| Test count post-2.4 | **13 passing** | `2-4-bevy-mod-outline-integration-wiring.md:560,581` |
| Bevy version | `0.18` (resolved `0.18.1`), features `["3d", "png", "bevy_ui", "default_font", "file_watcher"]` (+ x11/wayland on Linux) | `Cargo.toml:8,23-26` |
| `bevy::render::view::screenshot` API | `Screenshot::primary_window() -> Self`, `save_to_disk(path) -> impl FnMut(On<ScreenshotCaptured>)` observer, `ScreenshotCaptured` event, `ScreenshotPlugin` (already registered by `DefaultPlugins`) | `bevy_render-0.18.1/src/view/window/screenshot.rs:75,86-93,129-143,402-409` |
| `Screenshot` API surface available transitively | `bevy::render::view::screenshot::*` reexports via `bevy_render` (enabled by our `"3d"` feature transitively per `bevy-0.18.1/Cargo.toml:2322-2349`) | confirmed |
| Story 2.6 dependency | Reads this story's `parity-report.md` and the explicit `RECOMMEND` line. Must not delay Story 2.6 — Till should not need to "interpret" the report; the recommendation IS the input. | `epics/epic-2-vector-aesthetic-tech-spike.md:149-170` |
| Story 2.7 dependency | Triggered ONLY if 2.6's decision is `FALLBACK`. 2.5's recommendation directly drives whether 2.7 ever ships. | `epics/epic-2-vector-aesthetic-tech-spike.md:172-194` |

### Capture-mode design — the chosen approach and the alternatives rejected

**Chosen:** Env-var-gated `CapturePlugin` registered conditionally in `main.rs`. Out-of-process (one binary invocation per backend), captures one frame, exits.

**Why this design:**

- **Out-of-process captures match the reality of cross-backend testing.** Backends are bound to platforms (Metal=macOS, Vulkan=Linux, DX12=Windows); we can't run all three from one process anyway. One invocation per backend is the natural shape.
- **Env-var gating keeps the production code path inert.** When the env var is unset, `CapturePlugin::build` is never called and `requested_capture_path()` returns `None` immediately. AC #1's "byte-identical to post-2.4 baseline when env var unset" is structurally enforced.
- **Reuses existing infrastructure.** Bevy's `bevy_render::view::screenshot` plugin is already registered by `DefaultPlugins` (per `bevy_render-0.18.1/src/view/window/mod.rs:25-32`). We don't add a new plugin to the binary; we use the existing one via `Screenshot::primary_window()`.

**Rejected alternatives:**

- **Cargo feature flag (`features = ["m1-spike-capture"]`).** Architecture.md:781 explicitly says "No feature flags for gameplay. Feature flags reserved for platform-specific code paths and `cfg(debug_assertions)`." A capture-mode feature flag isn't gameplay, but it isn't platform-specific or debug-vs-release either — it's a runtime mode. Env vars are a better fit for runtime modes. Also, feature-flag-gated code requires `cargo run --features m1-spike-capture`, adding a `--features` flag to every CI invocation.
- **Separate binary `cargo run --bin capture`.** Would require a `[[bin]]` section in `Cargo.toml` and duplication of plugin registration. Higher maintenance cost; no benefit.
- **Promote `mod reference_scene` from `cfg(debug_assertions)` to non-gated.** Would re-introduce the dead-code warning resolved in Story 2.2 (per deferred-work.md "Removal-on-graduation" note re-deferred to Story 4.5). Also expands the M1-spike scaffolding's release-build footprint, contrary to architecture.md:887 ("Fallback scope is not pre-built; M1 tech-spike evaluation itself is the trigger"). The binary stays cfg-gated; capture happens in debug build (AC deviation, documented next).

### AC #1 deviation — debug build, not release build

The epic spec at `epics/epic-2-vector-aesthetic-tech-spike.md:127,135,140` says `cargo run --release`. **This story uses `cargo run` (debug build).**

**Reason:** `mod reference_scene` is `#[cfg(debug_assertions)]`-gated (`src/visual/mod.rs:57`). In a release build, the entire reference scene module is compiled out — Camera3d, placeholders, lights, all gone. A release-build screenshot would capture an empty Bevy default-clear-color window, not the reference scene. To make `cargo run --release` show the scene, we'd need to either (a) lift the cfg gate (rejected — re-introduces dead-code warning, expands release footprint), or (b) add a feature flag (rejected — see "Rejected alternatives" above).

**What the deviation costs us:** debug builds use `[profile.dev.package."*"] opt-level = 1` for deps and default `opt-level = 0` for our own code. Our own code is small (~300 LOC of Rust); the WGSL shaders compile through the same Naga path regardless. The shader output bytes — the actual subject of the parity test — are byte-identical between debug and release. **The parity test is unaffected by debug vs release.**

**What we lose:** release-build-specific runtime characteristics (like LTO + codegen-units=1 producing slightly different floating-point ordering in Rust glue code) aren't exercised. But none of those affect WGSL→backend translation — they affect Rust math performed before uniforms are uploaded. Toon material's uniforms are simple primitives (`u32`, `f32`, `[f32; 4]`); no Rust-side math touches them between RON parse and uniform write.

**Documented in:** parity-report.md "Capture environment" section + Dev Agent Record Completion Notes.

### Bevy 0.18 screenshot API — what to know

[Source: `bevy_render-0.18.1/src/view/window/screenshot.rs`, lines cited inline]

**Public API used:**

- `Screenshot` (component, `lib.rs:75`) — wraps a `RenderTarget`. Spawn it as a component on a fresh entity to request a capture.
- `Screenshot::primary_window() -> Self` (`lib.rs:93`) — convenience constructor targeting the primary window. **Use this; don't manually construct `Screenshot(RenderTarget::Window(...))` — the convenience method handles `WindowRef::Primary` resolution correctly.**
- `save_to_disk(path) -> impl FnMut(On<ScreenshotCaptured>)` (`lib.rs:129`) — observer factory; returns a closure that, when wired via `.observe(...)`, writes the captured image to the path on `ScreenshotCaptured` event. Handles PNG / JPG / etc by file extension.
- `ScreenshotCaptured` (event, `lib.rs:44`) — fires when capture finishes; carries `image: Image` and `entity: Entity`.

**Bevy 0.18 idioms:**

- `Screenshot` is a **component**, not a resource. Spawn pattern: `commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path));`. The `.observe(...)` chains an entity-scoped observer that fires for `ScreenshotCaptured` triggered against this entity. [Source: `lib.rs:64-71` doc-comment example]
- The capture is **asynchronous** — pixels are read back from the GPU on a render-world thread. Waiting one frame after the `ScreenshotCaptured` event before sending `AppExit::Success` ensures the file write has flushed. [Source: `lib.rs:53-58` doc-comment]
- `ScreenshotPlugin` is auto-registered by `DefaultPlugins` (via `bevy_render::view::WindowRenderPlugin` per `bevy_render-0.18.1/src/view/window/mod.rs:31`). **Do NOT** manually `app.add_plugins(ScreenshotPlugin)` — it's already there; double-registration causes a panic.

**Common pitfalls:**

- **`save_to_disk` writes synchronously to disk** during the observer callback (it's a closure that calls `image.save_to_disk(path)` internally per `lib.rs:139-145`). On a slow disk + large image, this could block the render world for 10s of milliseconds. Acceptable for one-shot capture; would be a problem for continuous-capture-frame-by-frame scenarios (not us).
- **Path resolution is relative to the current working directory.** `cargo run` sets CWD to the project root, so `docs/tech-spike/m1-backends/metal.png` resolves correctly. CI workflows must `cd` to the checkout root or use absolute paths if they reorganize the working directory.
- **The image format is determined by file extension.** `.png` → PNG. `.jpg` → JPEG. Other extensions may panic or produce unexpected output. Stick to `.png`.

### LLM dev agent guardrails — most-likely-to-go-wrong patterns

These are the failure modes that are most likely to bite if the dev moves fast:

1. **Calling `app.add_plugins(ScreenshotPlugin)` explicitly.** ScreenshotPlugin is registered by `DefaultPlugins` already. Double-registration panics with "plugin already registered". DO NOT add it. Just spawn `Screenshot::primary_window()` — the plugin's render systems pick it up.

2. **Capturing too early — before the asset loader finishes.** If `CAPTURE_FRAME` is too small (< 30 frames), the capture happens before `tuning.ron` parses, before `OutlineVolume` width/color update from RON, possibly before all 3 mesh assets finish loading. Visual symptom: outlines have default values (already true for cold-start race per 2.4 Task 5 fallback), or worse, meshes haven't been rendered yet (white/missing geometry). Frame 60 (~1 second) is the conservative default; bump to 120 if debugging shows asset-load latency on a particular CI runner.

3. **Capturing too late — after window auto-closes.** Bevy 0.18 doesn't auto-close windows during normal operation, so this isn't a real risk at frame 60. But if the dev sets `CAPTURE_FRAME = 1000` (~16 seconds), the splash bypass + immediate-MainMenu + idle scene burns CI minutes for no benefit. Frame 60-120 is the sweet spot.

4. **Using `cargo run --release` per the spec literal.** The release build strips `mod reference_scene` (cfg-gated). Result: empty captures. **AC deviation: use `cargo run` (debug)**. See "AC #1 deviation" Dev Notes section above.

5. **Forgetting the splash bypass.** Without the bypass, capture at frame 60 (~1 second) happens during `GameState::Loading` — splash still up, swatch UI not yet spawned. This is a less-evident scene than post-MainMenu. The bypass (`force_skip_splash_in_capture` in Task 3) ticks the splash timer to fire on the very first Update. **Don't** add a hard `cfg!(...)` skip of `tick_splash_timer` — that would touch `splash.rs`, which is out of scope. The Startup-system bypass is non-invasive.

6. **Using `WGPU_ADAPTER_NAME=Microsoft Basic Render Driver` unconditionally on Windows.** GitHub Actions runners may have a real GPU on some image versions; over-constraining via adapter name forces software rendering even when hardware is available. Default behavior — let wgpu pick the first adapter — is correct. Add the override ONLY if WARP fallback fails empirically.

7. **xvfb screen geometry mismatch.** `xvfb-run -a -s "-screen 0 1920x1080x24"` sets the virtual display to 1920×1080 at 24-bit color, matching our window resolution. If the geometry is omitted (just `xvfb-run -a cargo run`), the default xvfb screen is 1280×1024 at 8-bit, and our 1920×1080 window gets clipped or fails to map. **Always specify `-screen 0 1920x1080x24`** in the xvfb-run invocation.

8. **`VK_ICD_FILENAMES` path varies across Mesa versions.** Ubuntu 22.04+ (which `ubuntu-latest` currently is) installs lavapipe at `/usr/share/vulkan/icd.d/lvp_icd.x86_64.json`. Ubuntu 20.04 had it at `/usr/share/vulkan/icd.d/lvp_icd.json` (no arch suffix). If the path doesn't exist, the Verify-Vulkan-adapter step fails and aborts the workflow before wasting cargo build minutes. The `vulkaninfo --summary` step is the canary.

9. **ImageMagick v7 vs v6 invocation drift.** v6: `compare -metric AE A B null:`. v7: `magick compare -metric AE A B null:` (under "legacy" mode v6 command names also work but aren't guaranteed). On macOS via Homebrew, current ImageMagick is v7; the bare `compare` command may not exist. Use `command -v compare || command -v magick` to detect. Document the resolved binary in parity-report.md.

10. **Committing 0-byte PNGs from a failed capture.** If the screenshot save observer never fires (e.g. window closes before capture finishes due to a panic), `metal.png` may end up 0 bytes. Always run `file metal.png` after capture and verify `PNG image data, 1920 x 1080`. The CI workflow's `if-no-files-found: error` catches the artifact-side miss but does not protect against a corrupt local capture.

11. **Adding clippy `#[allow(...)]` to suppress unsafe-related lints in the unit test.** The `unsafe { std::env::set_var(...) }` block needs a `# Safety` comment justifying single-threaded access. If clippy flags `clippy::missing_safety_doc`, add the comment, don't suppress. Per architecture.md (CI policy) clippy is `-D warnings`. Use `#[expect(name, reason = "...")]` (the explicit form Bevy uses) ONLY if clippy is provably wrong.

12. **Mutating `Cargo.toml` to add `[[bin]] capture`.** No new binary. Capture mode is a runtime branch of the existing `asteroids3D` binary, gated by env var. Leave Cargo.toml untouched.

13. **Adding `paths-ignore` to `parity-capture.yml`.** The workflow has no `push` trigger — only `workflow_dispatch`. `paths-ignore` only filters push-triggered runs. Adding it is harmless but misleading; omit.

14. **Forgetting `if-no-files-found: error` on artifact uploads.** Default behavior is `warn`, which produces a green workflow run with a 0-byte artifact when the capture failed silently. `error` makes the workflow fail loudly. Keep this setting.

15. **Running `parity-capture.yml` against a non-master branch with the expectation of CI green-lighting on master.** The workflow's `--ref` parameter to `gh workflow run` selects the branch the dispatch runs against. Always use `--ref master` for the M1 gate evidence. Running against a feature branch produces artifacts that don't reflect the merged state.

16. **Treating quantitative pixel deltas (RMSE) as the verdict.** Software rasterizers (lavapipe, WARP) produce different pixel values from hardware Metal even for identical correct shader output — this is rasterizer-implementation noise, not a backend-translation bug. The qualitative visual checks (band counts, rim-light presence, tint colors, outline silhouette continuity) ARE the load-bearing parity signal. RMSE is anchoring data, not a decision criterion. The recommendation paragraph in parity-report.md must lead with the qualitative result.

### Architecture compliance — naming, module layout, plugin pattern

**Plugin / SystemSet naming (architecture.md:326-328):** ✓
- `CapturePlugin` follows the `*Plugin` convention.
- No new SystemSet introduced — capture systems run in default `Update` (no other systems care about ordering relative to capture; the capture flow is self-contained).

**Module layout (architecture.md:603-607):** ✓
- `src/visual/capture.rs` is a sibling of `src/visual/{outline,palette,toon_material,reference_scene}.rs`; same flat-feature-module pattern.
- `pub mod capture;` exposes the module via `crate::visual::capture::*` qualified paths from `main.rs`.
- **No** `pub use capture::*;` re-export, consistent with the post-2.4 mod.rs.

**Inter-system communication (architecture.md:243):** ✓
- `drive_capture` reads `Res<CaptureState>`, queries `PrimaryWindow`, writes `MessageWriter<AppExit>` — all standard Bevy-idiomatic ECS patterns. No direct cross-plugin state mutation.

**Plugin boundary table (architecture.md:654, 656):** ✓
- `VisualPlugin` boundary unchanged. `CapturePlugin` is a sibling plugin registered by `main.rs`, not a sub-plugin of `VisualPlugin`. This keeps capture mode visible at the binary's top level (where it conceptually belongs — it's a binary-mode switch, not a visual feature).

**Anti-pattern check (architecture.md:458-468):** ✓
- ❌ God-struct: `CaptureState` is small and single-responsibility (frame counter + path + state-machine flags). ✓
- ❌ Direct cross-plugin state mutation: `drive_capture` mutates only `CaptureState` (capture-internal) and reads framework-provided queries. ✓
- ❌ Magic numbers: `CAPTURE_FRAME = 60` is a `const` with rationale documented in Task 1's drive_capture body comment. ✓
- ❌ `unwrap()` / `expect()`: only the `Ok(_window)` early-return on `single()` query — no unwrap. ✓
- ❌ Scattered `AssetServer::load`: still ONE call (in `load_tuning`). Capture mode loads no assets. ✓
- ❌ `.after(specific_function)` ordering: not used. ✓
- ❌ Feature flags for gameplay (architecture.md:781): not used; runtime env var instead. ✓

### Forward compatibility — Story 3.1 cleanup

Capture mode is M1-spike-only. When Story 3.1 (Arena state) replaces the reference scene with the actual game scene, `src/visual/capture.rs` and `.github/workflows/parity-capture.yml` should be removed. They have no role in M2+ where the Arena state is the binary's normal mode.

The cleanup is a 3-file delete:
1. `rm src/visual/capture.rs`
2. Edit `src/visual/mod.rs`: remove `pub mod capture;` line.
3. Edit `src/main.rs`: remove env-var lookup + conditional WindowPlugin override + conditional CapturePlugin registration.
4. `git rm .github/workflows/parity-capture.yml`

Add a deferred-work entry tracking this cleanup at Task 11 time.

### Forward compatibility — Story 2.6 / 2.7 hand-off

Story 2.6 reads the `RECOMMEND ...` line from `docs/tech-spike/m1-backends/parity-report.md` verbatim. The recommendation IS the input. Story 2.6's own job is:
- Validate the recommendation is reasoned (read the justification paragraph + qualitative checks).
- Either accept (typical case) or override with documented rationale (atypical, only when 2.5's recommendation is provably wrong).
- Author `docs/tech-spike/m1-decision.md` with the formal Decision / Rationale / Risks Accepted sections.

Story 2.7 ships ONLY if 2.6's decision is `FALLBACK flat+rim-light`. The capture-mode binary path stays in place through 2.6/2.7 — fallback work would re-run captures with the fallback material to update parity-report.md's "Recommendation" section.

### Test count discipline

Post-2.4: 13 tests passing. Post-2.5 expected: **14** (13 + 1 new `capture_disabled_when_env_var_unset`).

If `cargo test` reports anything other than `14 passed`:
- **<14:** Task 1's unit test was forgotten; check `src/visual/capture.rs#cfg(test)`.
- **>14:** an unintended test was added; review the diff in Task 9's `git status --short`.

### Integration test deferral

Architecture.md:354 defers integration tests post-M3. Story 2.5's runtime behavior (env-var-gated plugin registration, frame-counted capture, AppExit emission, splash bypass) is verified manually via Tasks 4 + 8 (real captures + log inspection), not via `cargo test`. When M3+ stories introduce integration tests, candidate cases for capture mode:

- `App::new() + CapturePlugin + advance N frames + assert Screenshot entity spawned`
- `set env var, call requested_capture_path, assert Some(_); unset, assert None` (could land sooner — pure state, no `App`)

The first is the meaningful integration test; the second is already in scope as a unit test and lands in this story.

### Project Structure Notes

- **Path alignment with architecture.md:**
  - `src/visual/capture.rs` is in the `src/visual/` feature module per architecture.md:344-349.
  - `.github/workflows/parity-capture.yml` is in the standard GitHub Actions workflow location.
  - `docs/tech-spike/m1-backends/` matches the existing tech-spike documentation pattern (`m1-palette/`, `m1-toon/`, `m1-outline/`).
- **No path conflicts or variances.**
- **`assets/` untouched.** Capture mode adds no assets.

### References

- [Source: _bmad-output/planning-artifacts/epics/epic-2-vector-aesthetic-tech-spike.md#Story-2.5 (lines 121-147)]
- [Source: _bmad-output/planning-artifacts/prd.md#FR49 (line 569), Section "Cross-platform parity" (line 115), R#2 risk (line 441)]
- [Source: _bmad-output/planning-artifacts/architecture.md#M1-Vector-Spike (line 295)]
- [Source: _bmad-output/planning-artifacts/architecture.md#Tech-Risk-Resolution (lines 885-887)]
- [Source: _bmad-output/planning-artifacts/architecture.md#Source-Tree (lines 603-607)]
- [Source: _bmad-output/planning-artifacts/architecture.md#No-Feature-Flags-for-Gameplay (line 781)]
- [Source: _bmad-output/implementation-artifacts/2-4-bevy-mod-outline-integration-wiring.md (Story 2.4 inherited context, Dev Agent Record)]
- [Source: _bmad-output/implementation-artifacts/2-3-wgsl-toon-material-implementation.md (Story 2.3 hot-reload + RON-tuple-vs-array deviation note)]
- [Source: ~/.cargo/registry/src/index.crates.io-*/bevy_render-0.18.1/src/view/window/screenshot.rs (Screenshot, save_to_disk, ScreenshotCaptured, ScreenshotPlugin API)]
- [Source: ~/.cargo/registry/src/index.crates.io-*/bevy_render-0.18.1/src/view/window/mod.rs:25,31 (ScreenshotPlugin auto-registration)]
- [Source: GitHub Actions docs — `actions/upload-artifact@v4`, `workflow_dispatch` trigger, `if-no-files-found: error`]
- [Source: ImageMagick docs — `compare -metric AE`, `compare -metric RMSE`, v6 vs v7 binary invocation]
- [Source: Mesa docs — lavapipe ICD path, `VK_ICD_FILENAMES` env var]
- [Source: Microsoft DirectX docs — WARP (Windows Advanced Rasterization Platform), default DX12 fallback adapter]

## Dev Agent Record

### Agent Model Used

Claude Opus 4.7 (1M context) — `claude-opus-4-7[1m]` via Claude Code (BMad `bmad-dev-story` workflow).

### Debug Log References

**Local verification sweep (Task 8) — all on macOS 26.4.1, Apple M5 Pro, 2026-04-29:**

| Command | Log path | `grep -cE 'warning:\|error:'` | Notes |
|---|---|---|---|
| `cargo check` | `/tmp/story-2-5-check.log` | 0 | clean |
| `cargo build` | `/tmp/story-2-5-build.log` | 0 | clean |
| `cargo test` | `/tmp/story-2-5-test.log` | 0 | **14 passed, 0 failed** (13 from 2.4 + 1 new `capture::tests::capture_disabled_when_env_var_unset`) |
| `cargo clippy --all-targets -- -D warnings` | `/tmp/story-2-5-clippy.log` | 0 | clean under strict mode |
| `cargo fmt --all -- --check` | `/tmp/story-2-5-fmt.log` | (exit 0) | clean |
| `cargo build --release` | `/tmp/story-2-5-release.log` | 0 | clean (capture plugin compiles into release; inert when env var unset) |

**Capture-mode runtime smoke — env var unset (Task 8 baseline):** `/tmp/story-2-5-baseline-run.log`
- `grep -ci 'capture'` → 0 (capture plugin never registered)
- `grep -c 'TuningReloaded'` → 1 (Story 2.4 cold-start reload still fires)
- `grep -c 'splash timer elapsed'` → 1 (real 2-second splash; bypass NOT active)
- `grep -c 'entered MainMenu'` → 1

Confirms AC #1 "byte-identical to post-2.4 baseline when env var unset" at the observable-log level.

**Capture-mode runtime smoke — env var set (Task 8 capture):** `/tmp/story-2-5-capture-run.log`
- Exit code: 0
- `grep -c 'Screenshot capture finished'` → 1
- `grep -c 'splash timer elapsed'` → 0 (splash bypass active)
- `grep -c 'entered MainMenu'` → 1
- `file /tmp/test-capture.png` → `PNG image data, 1920 x 1080, 8-bit/color RGB, non-interlaced`

**Local Metal capture (Task 4):** `/tmp/story-2-5-capture-metal.log`
- Backend: `Apple M5 Pro / Metal` (per `bevy_render::renderer: AdapterInfo` log line)
- Output: `docs/tech-spike/m1-backends/metal.png`
- Dimensions: `sips -g pixelWidth -g pixelHeight` → 1920 × 1080
- File: 123 127 bytes, `PNG image data, 1920 x 1080, 8-bit/color RGB, non-interlaced`

**CI runs:**
- `ci.yml` run [25111612698](https://github.com/till-fechteler/asteroids3D/actions/runs/25111612698) — Commit 1 push (`b99f731`) — all 4 jobs ✓ in `1m45s` / `2m26s` / `5m07s` / `42s` (Linux / macOS / Windows / msrv-check). Full-log `grep -cE 'warning:|error:' | grep -v 'Free disk space'` → 0 (the 9 `Free disk space` hits are pre-existing ambient noise from `jlumbroso/free-disk-space` action's `set -x` echo).
- `parity-capture.yml` run [25111626273](https://github.com/till-fechteler/asteroids3D/actions/runs/25111626273) — first dispatch — `windows-dx12` ✓ in `28m34s` (cold cache; produced `dx12.png` artifact); **`linux-vulkan` ✗ in `15m11s`** with `vkCreateInstance: Found no drivers` panic at `/usr/share/vulkan/icd.d/lvp_icd.x86_64.json`. Root cause: hardcoded `VK_ICD_FILENAMES` path unreachable on `ubuntu-latest` (24.04) despite `vulkaninfo --summary` reporting drivers present. Story Dev Notes pitfall #8 anticipated this Mesa-package-layout drift.
- `ci.yml` run [25113260506](https://github.com/till-fechteler/asteroids3D/actions/runs/25113260506) — workflow-fix push (`84ee887`) — all 4 jobs ✓ in `4m31s` total (cache warm). Filtered grep → 0.
- `parity-capture.yml` run [25113263165](https://github.com/till-fechteler/asteroids3D/actions/runs/25113263165) — re-dispatch after dropping `VK_ICD_FILENAMES` — both jobs ✓: `linux-vulkan` in `2m25s` (auto-discovery + cache warm), `windows-dx12` in `4m31s`. Both artifacts uploaded.
- `ci.yml` run [25113773032](https://github.com/till-fechteler/asteroids3D/actions/runs/25113773032) — Commit 2 push (`66c41fd`, docs+screenshots) — all 4 jobs ✓ in `3m39s` total. Filtered grep → 0.

**Parity-report metrics (Task 7, ImageMagick 7.1.2-21 Q16-HDRI):**

| Pair | AE | RMSE_norm | < AC #4 0.05 threshold? |
|---|---|---|---|
| Metal ↔ Vulkan | 44 787 (2.16%) | 0.000423 | ✓ (118× under) |
| Metal ↔ DX12 | 9 410 (0.45%) | 0.006960 | ✓ (7× under) |
| Vulkan ↔ DX12 | 50 727 (2.45%) | 0.006955 | ✓ (7× under) |

All 6 qualitative visual-equivalence checks pass on all 3 backends (banding count, rim-light, tint
colors, outline continuity, outline width proportion, swatch palette colors). Recommendation:
**GO toon**.

### Completion Notes List

**AC #1 — env-var-gated capture mode**
- ✓ `ASTEROIDS3D_CAPTURE_PNG=<path> cargo run` writes a 1920×1080 PNG to the path.
- ✓ Camera at `Transform::from_xyz(0.0, 1.5, 6.0).looking_at(Vec3::ZERO, Vec3::Y)` — reused unchanged from `src/visual/reference_scene.rs:55-63` (no capture-mode-specific transform).
- ✓ Binary exits with `AppExit::Success` after the screenshot save observer fires (verified by the capture log's `Screenshot capture finished; exiting on next frame.` line + exit code 0).
- ✓ When env var is unset, capture plugin is never registered (conditional in `main.rs`): `grep -ci 'capture' baseline.log` → 0; baseline log byte-equivalent to post-2.4 modulo timestamps and bevy's auto-generated AdapterInfo line.
- **Deviation: `cargo run` (debug) instead of `cargo run --release`**. `mod reference_scene` is `cfg(debug_assertions)`-gated; release strips the entire scene. Documented in `parity-report.md` "AC #1 deviation" section + Dev Notes "AC #1 deviation" section.

**AC #2 — macOS Metal capture**
- ✓ `metal.png` committed to `docs/tech-spike/m1-backends/metal.png` (123 127 bytes).
- ✓ Dimensions verified via `sips -g pixelWidth -g pixelHeight` and `file`: 1920 × 1080.
- ✓ Visible content matches AC #2 specification: 3 placeholders (asteroid icosphere, ship cuboid, projectile UV-sphere) with toon shading + silhouette outlines + 5-swatch palette bar at top (post-MainMenu state). No splash text overlay (splash bypass active in capture mode).

**AC #3 — `parity-capture.yml` workflow**
- ✓ `linux-vulkan` job (Mesa lavapipe + xvfb + auto-discovery) produces 1920×1080 `vulkan.png` artifact (run 25113263165, `2m25s`).
- ✓ `windows-dx12` job (WARP fallback) produces 1920×1080 `dx12.png` artifact (run 25113263165, `4m31s`).
- ✓ Both artifacts downloadable from workflow run page.
- **Deviation: `VK_ICD_FILENAMES` removed.** First dispatch (`25111626273`) failed `linux-vulkan` because the hardcoded `/usr/share/vulkan/icd.d/lvp_icd.x86_64.json` path was unreachable to the Vulkan loader despite `vulkaninfo --summary` reporting drivers present (suggests Mesa-package-layout drift on the current `ubuntu-latest` 24.04 image). Fix: drop the env-var override and rely on the loader's default search path. Auto-discovery succeeded on second dispatch with no other change. ubuntu-latest virtualized hosts have no hardware GPU, so lavapipe is the only adapter wgpu can pick anyway — there's no over-selection risk.

**AC #4 — parity report**
- ✓ `parity-report.md` includes the pairwise diff table with AE absolute count + RMSE raw/normalized for all 3 pairs.
- ✓ All 3 RMSE_normalized values < 0.0070 (AC #4 threshold is 0.05); no pair requires divergence annotation. Brief observations included for completeness.
- ✓ Report closes with explicit `RECOMMEND GO toon` line + one-paragraph justification.
- ✓ Optional 3 diff-heatmap PNGs committed alongside (`diff-metal-vs-vulkan.png`, `diff-metal-vs-dx12.png`, `diff-vulkan-vs-dx12.png`).

**AC #5 — test count**
- ✓ `cargo test` reports `test result: ok. 14 passed; 0 failed; 0 ignored`. New test: `visual::capture::tests::capture_disabled_when_env_var_unset` asserts `requested_capture_path()` returns `None` when `ASTEROIDS3D_CAPTURE_PNG` is unset.

**Spec deviations (capture.rs splash bypass)**
- Spec Task 3 prescribed a Startup system that ticks `SplashConfig.timer` past its 2-second duration. **This does not work** in Bevy 0.18 because `Timer` in `TimerMode::Once` early-returns from `.tick()` once `finished == true`, which means `just_finished()` only returns `true` on the tick that crossed the duration boundary. A Startup pre-tick past duration sets `finished = true` on frame 0; subsequent `tick_splash_timer` calls in `Update` see `finished && Once`, hit the early-return path, and `just_finished()` is permanently `false` — so `tick_splash_timer`'s `next_state.set(MainMenu)` line never fires. **Empirically confirmed** by the first capture attempt (`/tmp/story-2-5-capture-metal.log` from 13:07:38): no `entered MainMenu` log line, no `splash timer elapsed` line, captured PNG showed splash text overlaid on the reference scene with no swatch palette bar.
- **Implemented fix:** replace the timer-tick bypass with a direct `NextState<GameState>::MainMenu` push, run as an `Update` system gated on `in_state(GameState::Loading)`. Robust against this semantics quirk and against future splash-duration changes. Verified: capture at 13:10:51 shows `entered MainMenu` log line and the captured PNG matches AC #2 (swatch bar visible, no splash text).

**Spec deviations (main.rs WindowResolution)**
- Spec Task 2 prescribed `WindowResolution::new(1920.0, 1080.0)` (f32 args). Bevy 0.18.1's actual signature at `bevy_window-0.18.1/src/window.rs:912` is `pub fn new(physical_width: u32, physical_height: u32)`. Compile error E0308. Fix: `WindowResolution::new(1920, 1080)` (integer literals).

**Spec deviations (Task 9 grep pattern)**
- Spec's `grep -rn 'AssetServer::load\b' src/` returned 0 hits, not the spec-expected 1. Reason: the actual call site uses method-call form `asset_server.load(...)` (lowercase, on `Res<AssetServer>`), not the qualified form `AssetServer::load(...)`. Underlying expectation satisfied — `grep -rn 'asset_server.load' src/` returns 1 hit at `src/tuning/mod.rs:39`, identical to post-2.4. No scattered asset loads.

**Cosmetic warning observed (not introduced)**
- Both capture-mode runs and the baseline run emit `WARN bevy_ecs::error::handler: Encountered an error in command ...: Entity despawned: The entity with ID Nv0 is invalid; its index now has generation 1.` during the `OnExit(GameState::Loading)` cleanup. This is the splash-cleanup-iteration race already documented in `deferred-work.md` "Observed during: 2-1-visualplugin-skeleton-reference-scene dev verification (2026-04-27)" — Story 1.7's defensive `LoadingStateEntity` marker on the splash Text child duplicates Bevy's auto-linked-despawn cascade. Behavior is unaffected. Capture mode's near-instant state transition makes this WARN more reliably reproducible. Pre-existing; no fix in scope for Story 2.5.

**Test coverage gap (acknowledged in story spec)**
- `drive_capture` runtime behavior (frame counting, screenshot spawn, `AppExit::Success` emission, splash-bypass `NextState` push) is not covered by `cargo test`. `App::new()` integration testing with render plugins is integration-test territory deferred post-M3 per architecture.md:354. Verification is via Tasks 4, 6, 8 (real captures + log inspection + parity-report inspection).

### File List

**Added (source):**
- `src/visual/capture.rs` — `CapturePlugin`, `requested_capture_path()`, `CaptureState` resource, `drive_capture` system, `on_screenshot_captured` global observer, `force_skip_splash_in_capture` Update system gated on `Loading`, one unit test.

**Added (CI):**
- `.github/workflows/parity-capture.yml` — manual `workflow_dispatch` workflow with `linux-vulkan` (Mesa lavapipe + xvfb) and `windows-dx12` (WARP) jobs producing 1920×1080 PNG artifacts.

**Added (docs):**
- `docs/tech-spike/m1-backends/metal.png` — 1920×1080, 123 127 bytes.
- `docs/tech-spike/m1-backends/vulkan.png` — 1920×1080, 123 525 bytes.
- `docs/tech-spike/m1-backends/dx12.png` — 1920×1080, 114 788 bytes.
- `docs/tech-spike/m1-backends/diff-metal-vs-vulkan.png` — ImageMagick diff heatmap.
- `docs/tech-spike/m1-backends/diff-metal-vs-dx12.png` — ImageMagick diff heatmap.
- `docs/tech-spike/m1-backends/diff-vulkan-vs-dx12.png` — ImageMagick diff heatmap.
- `docs/tech-spike/m1-backends/parity-report.md` — pairwise diff table, qualitative checks, `RECOMMEND GO toon` recommendation for Story 2.6.

**Modified (source):**
- `src/main.rs` — env-var lookup at top of `main()`; conditional `WindowPlugin` override (1920×1080 borderless, no decorations, capture-specific title) when capture is requested; restructured to `let mut app = App::new();` binding for conditional `CapturePlugin` registration.
- `src/visual/mod.rs` — `pub mod capture;` (alphabetical) + Story 2.5 line in module doc-comment.

**Modified (CI — story 2.5 dispatch fix):**
- `.github/workflows/parity-capture.yml` — dropped hardcoded `VK_ICD_FILENAMES` env from `linux-vulkan` Run-capture step; added one-line ICD-directory listing to Verify-Vulkan-adapter step for future diagnostics.

**Modified (bookkeeping, Task 11):**
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — `2-5-three-backend-parity-validation-gate` flipped `ready-for-dev → in-progress → review`; `last_updated` bumped.
- `_bmad-output/implementation-artifacts/2-5-three-backend-parity-validation-gate.md` (this file) — `Status: review`, all task/subtask checkboxes flipped to `[x]`, Dev Agent Record populated.
- `_bmad-output/implementation-artifacts/deferred-work.md` — appended one new entry (Story 3.1 capture-mode cleanup) and one contingency note (`WGPU_ADAPTER_NAME` override if WARP becomes unavailable).

### Change Log

| Date | Change | Reason |
|---|---|---|
| 2026-04-29 | `WindowResolution::new(1920, 1080)` (u32) instead of spec's `(1920.0, 1080.0)` (f32) | Bevy 0.18.1 signature mismatch; story spec was wrong. |
| 2026-04-29 | Splash bypass via direct `NextState<GameState>::MainMenu` push, gated on `in_state(Loading)` | Spec's Startup-timer-tick approach does not transition state — `Timer` in `TimerMode::Once` early-returns from `tick()` once finished, leaving `just_finished()` permanently false in subsequent ticks. |
| 2026-04-29 | `parity-capture.yml`: removed hardcoded `VK_ICD_FILENAMES` env var; rely on Vulkan loader auto-discovery | First dispatch failed `linux-vulkan` with "Failed to open JSON file" at the exact env-var path despite `vulkaninfo --summary` reporting drivers; auto-discovery is more robust against Mesa-package-layout drift on `ubuntu-latest`. |
| 2026-04-29 | Story 2.5 ready-for-dev → review | All 5 ACs satisfied; full local + CI verification clean; parity report recommends `GO toon`. |
