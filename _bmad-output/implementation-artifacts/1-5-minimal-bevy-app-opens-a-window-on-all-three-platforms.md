# Story 1.5: Minimal Bevy App Opens a Window on All Three Platforms

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a first-time observer of the project,
I want `cargo run` to open a window on Windows, Linux, and macOS,
so that the "asteroids3D project exists and runs" signal is demonstrable from day one — the motivation-preservation baseline.

## Acceptance Criteria

1. **`src/main.rs` contains the canonical minimal Bevy app.** The file's body is exactly (whitespace-insensitive): `fn main() { App::new().add_plugins(DefaultPlugins).run(); }` plus the `use bevy::prelude::*;` import. No extra plugins, no custom Window settings, no state registration, no UI nodes, no systems. AC #1 from the epic says "default Bevy title and size" — do not set `WindowPlugin` overrides. [Source: epics/epic-1-foundation-plugin-compatibility-gate.md:109-111]
2. **Native window opens on Windows 10+ (DX12 via wgpu).** When `cargo run` executes on a Till-reachable Windows machine, a native window appears with Bevy's default title ("App" or similar) and default size (1280×720 as of Bevy 0.18). No panics. No `error!` / `warn!` lines in stderr beyond Bevy's own informational startup logs. Till records the dev-box spec + success screenshot (or explicit written confirmation) in the Dev Agent Record. [Source: epics/epic-1-foundation-plugin-compatibility-gate.md:111-115]
3. **Native window opens on Linux (Vulkan via wgpu).** On Ubuntu LTS (or Till's equivalent — Fedora/Arch both acceptable), `cargo run` produces a native window. The backend is Vulkan (verified by grepping the startup log for `AdapterInfo { backend: Vulkan, ... }` or equivalent wgpu info log). X11 and Wayland are both acceptable windowing backends — whichever the session provides. No panics; no unexpected errors. [Source: epics/epic-1-foundation-plugin-compatibility-gate.md:116-119; Cargo.toml:23-26 (x11+wayland features already enabled on Linux target)]
4. **Native window opens on macOS Apple Silicon (Metal via wgpu).** `cargo run` on Till's local macOS dev box (macOS 26.4.1 / M-series) produces a native window. Backend is Metal (verified via the same wgpu `AdapterInfo` log line). Zero panics, zero unexpected errors. This is the single required manual-run validation gate on Till's own hardware — the other two OSes run on CI and on Till's physical Windows/Linux boxes. [Source: epics/epic-1-foundation-plugin-compatibility-gate.md:120-123]
5. **CI matrix (Story 1.4 artifact) stays green on the commit that introduces `App::new()`.** All four checks (`build (windows-latest)`, `build (ubuntu-latest)`, `build (macos-latest)`, `msrv-check`) complete ✅ on the source-artifact commit. CI does **not** actually run the binary — headless GitHub runners cannot open a display — so CI green validates *compile* parity, not *window-opens* parity. Actual window-opens evidence for AC #2, #3, #4 comes from Till's manual runs. The run URL is captured in Dev Agent Record → Debug Log References. [Source: .github/workflows/ci.yml:44-61; 1-4-three-platform-ci-matrix.md:67-69]
6. **The `cfg(debug_assertions)` Cargo-manifest warning is eliminated by removing `bevy_egui` from `Cargo.toml` entirely.** Story 1.1 shipped `[target.'cfg(debug_assertions)'.dependencies] bevy_egui = "0.39"`, which Cargo treats as always-true and emits `warning: Found 'debug_assertions' in target.'cfg(...)'.dependencies`. This story owns the fix per the 1.1 review correction + Story 1.4 Dev Notes. **Chosen fix: delete the broken block and `bevy_egui` with it.** No `[features]` section, no optional dep, no replacement — `bevy_egui` is unused until the M2 debug-panels story first registers an egui plugin, and re-introduction lands with that first use (as a feature-flag dependency at that time). Post-change: plain `cargo check` compiles with zero warnings and zero `Found 'debug_assertions'` lines in output. [Source: deferred-work.md:33-41; 1-4-…md:139; docs/plugin-compatibility.md:32; staged-rollout preference from brainstorming — reduced-MVP + post-MVP expansion over speculative scaffolding]
7. **`docs/plugin-compatibility.md` updated to reflect the removal.** The `bevy_egui` row in the **Third-party plugins (gated)** table is removed (crate is no longer a declared dep — the table's role is "what's pinned and gated in `Cargo.toml`"). A new row or bullet under **Deferred / Planned** documents: "bevy_egui — removed 2026-04-23 by Story 1.5. The `cfg(debug_assertions)` gating scheme was broken (Cargo treats the predicate as always-true). Re-introduction: M2 debug-panels story, as an optional feature-flag dep (`[features] dev-tools = ["dep:bevy_egui"]`) at first actual registration." The **Known Issues / Deferred** section's `cfg(debug_assertions)` bullet is removed (issue resolved by the removal itself). Change Log entry added. [Source: docs/plugin-compatibility.md:24,32]
8. **Scope-guardrail: no features beyond window-opens.** No `state.rs`, no `GameState` enum (Story 1.6's scope), no splash screen (Story 1.7), no logging override (Story 1.8 — Bevy's `LogPlugin` inside `DefaultPlugins` is the default logger for this story). No `VisualPlugin`, `FlightPlugin`, or any other plugin registration. No asset loading. No window title override. No `WindowPlugin` customization. No `[features]` section added (deferred with `bevy_egui` itself — M2 introduces the first feature flag alongside the first feature-gated crate). [Source: epics/epic-1-foundation-plugin-compatibility-gate.md:125-193 (stories 1.6, 1.7, 1.8 scopes)]
9. **`Cargo.lock` shrinks — `bevy_egui` and its transitives are pruned.** Removing the only line that declares `bevy_egui` causes `cargo check` to regenerate `Cargo.lock` without `bevy_egui 0.39.1`, `egui`, `emath`, `epaint`, `ecolor`, `accesskit`, and related transitives. Net lock-file shrinkage: dozens of crate entries. Verify via `cargo tree --depth 1 --edges normal` (no `bevy_egui` hit) and `grep -c '"bevy_egui"' Cargo.lock` (expect 0). No new top-level dep is introduced to replace it. [Source: Cargo.toml:28-30 current; Cargo.lock current]
10. **Local `cargo build`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all -- --check` all exit 0 on Till's macOS with zero `warning:|error:` hits in the full output.** Per MEMORY.md → feedback_full_build_output.md: exit-0 alone is not proof; the dev agent greps the captured full output for `warning:` and `error:` occurrences and confirms the only hit is an unavoidable engine/runtime log that's clearly benign (and even that gets called out in the Completion Notes). Clippy and fmt must produce zero warnings. [Source: MEMORY.md → feedback_full_build_output.md]

## Tasks / Subtasks

- [x] **Task 1 — Rewrite `src/main.rs` to the canonical minimal Bevy app (AC: #1, #8)**
  - [x] Opened `src/main.rs`. Confirmed prior body was the cargo-default `fn main() { println!("Hello, world!"); }`.
  - [x] Replaced the entire file contents with the skeleton from **Dev Notes → `main.rs` Skeleton** verbatim. 7 lines total: 2-line module doc comment, 1 blank, `use bevy::prelude::*;`, 1 blank, 3-line `fn main()`.
  - [x] Ran `cargo fmt -- --check` — exit 0 with no output. The as-written skeleton already matches rustfmt's canonical formatting for this project's `rustfmt.toml` (edition 2024, max_width 100, newline_style Unix).
  - [x] Clippy deferred to Task 4's combined sweep (avoids compiling the build graph twice because Task 2 also modifies `Cargo.toml`). Full clippy result captured in Task 4's Debug Log References.

- [x] **Task 2 — Remove `bevy_egui` + the broken `cfg(debug_assertions)` block from Cargo.toml (AC: #6, #9)**
  - [x] Deleted the 4-line block (blank + `# Dev-only GUI tooling …` comment + `[target.'cfg(debug_assertions)'.dependencies]` header + `bevy_egui = "0.39"` line) from `Cargo.toml`. Net: -4 lines. Zero additions.
  - [x] Ran `cargo check` — exit 0 in 18s (warm cache). Full output captured at `/tmp/asteroids3d-check.log`: 2 lines total, zero `warning:|error:` hits (`grep` exit 1). The `cfg(debug_assertions)` warning is gone.
  - [x] `Cargo.lock` pruned: `grep -c '"bevy_egui"' Cargo.lock` → **0**; `grep -Ec '"egui"|"emath"|"epaint"|"ecolor"' Cargo.lock` → **0**. The 10 remaining `accesskit*` refs are Bevy's own accessibility plugin dependencies (part of `DefaultPlugins`), not `bevy_egui` transitives — expected to remain.
  - [x] `cargo tree --depth 1 --edges normal | grep -i egui` → no match (exit 1). Confirmed `bevy_egui` is gone from the normal dep graph.

- [x] **Task 3 — Update `docs/plugin-compatibility.md` to document the removal + planned M2 re-introduction (AC: #7)**
  - [x] Deleted the `bevy_egui` row from the **Third-party plugins (gated)** table.
  - [x] Added a new **Deferred / Planned re-introduction** section between the plugin table and the existing Resolution Log, documenting the removal rationale + the M2 feature-flag re-introduction template (including a full Cargo.toml snippet for reference).
  - [x] Reset the **Known Issues / Deferred** section to empty (prior `[target.'cfg(debug_assertions)'.dependencies]` bullet removed; replaced with a parenthetical note that the issue was resolved by Story 1.5 and points to the Deferred / Planned section above).
  - [x] Appended a new row to the Change Log: `| 2026-04-23 | Story 1.5: removed bevy_egui from Cargo.toml (the cfg(debug_assertions) gating was broken; Cargo treats that predicate as always-true in dependency tables). Manifest warning eliminated. Re-introduction deferred to M2 debug-panels story as a feature-flag dep. |`

- [x] **Task 4 — Local macOS verification — window opens (AC: #4, #10)**
  - [x] `cargo build` — exit 0 in 1m45s. Full output at `/tmp/asteroids3d-build.log`; zero `warning:|error:` hits (grep exit 1). Pulled full `DefaultPlugins` graph including `bevy_winit 0.18.1`, `bevy_window 0.18.1`, `bevy_render 0.18.1`, `winit 0.30.13`, `wgpu 27.0.1`, `metal 0.32.0`, etc. This **empirically validates Story 1.1's deferred `default-features = false` concern (`deferred-work.md:9`)** — Bevy's `"3d"` feature collection DOES transitively pull the needed renderer/windowing crates on macOS.
  - [x] `cargo run` — window opened. Adapter info captured at `/tmp/asteroids3d-run.log`: `AdapterInfo { name: "Apple M5 Pro", vendor: 0, device: 0, device_type: IntegratedGpu, driver: "", driver_info: "", backend: Metal }`. Window title: `asteroids3D` (derived from package name; default-title scope honored — no `WindowPlugin` override in code). Window creation logged: `Creating new window asteroids3D (0v0)`.
  - [x] Window closed; process exited cleanly with code 0. Zero panics, zero `error!`/`warn!` lines.
  - [x] Debug Log References captured: dev-box spec = `macOS 26.4.1 / Darwin 25.4.0 / Apple M5 Pro / 18 cores / 64 GiB`, backend = `Metal`, exit code = 0.
  - [x] `cargo test` → exit 0, 0 tests passed/filtered (expected). Log at `/tmp/asteroids3d-test.log`, zero warnings.
  - [x] `cargo clippy --all-targets -- -D warnings` → exit 0 in 0.25s (no new workspace code to lint; 1-file crate). Log at `/tmp/asteroids3d-clippy.log`, zero warnings.
  - [x] `cargo fmt --all -- --check` → exit 0, no output.

- [x] **Task 5 — Local Windows + Linux verification — window opens (AC: #2, #3)**
  - [x] Till's physical Windows box: `cargo run` verified OK by Till on 2026-04-23 ("Windows OK"). Window opened; AC #2 runtime-satisfied.
  - [x] Till's physical Linux box: `cargo run` verified OK by Till on 2026-04-23 ("Linux läuft auch"). Window opened; AC #3 runtime-satisfied.
  - [x] All three OS legs runtime-confirmed: **macOS ✅ (dev-agent, Metal), Windows ✅ (Till), Linux ✅ (Till)**. No OS is closing on CI-compile-only evidence — every AC has runtime proof.

- [x] **Task 6 — Push + observe CI — matrix green (AC: #5)** — _executed by dev agent on Till's explicit "Option B" authorization 2026-04-23_
  - [x] Source-artifact commit `03eb7a4` — `feat: minimal Bevy app + remove bevy_egui from Cargo.toml (Story 1.5)`. 4 files changed: +39 / -638 (heavy deletions reflect pruned `Cargo.lock` entries). `.claude/scheduled_tasks.lock` correctly excluded.
  - [x] `git push origin master` — `011f99d..03eb7a4  master -> master`.
  - [x] CI run observed via `gh run view 24842252974` — all four jobs ✅ in 62m 37s total. Cold-cache Windows matches Story 1.4's ~71m pattern (the `Cargo.lock` dep-graph change invalidated `Swatinem/rust-cache@v2` keys across legs).
  - [x] Full CI log grep — `gh run view --log` piped through `grep -c 'error:'` → **0 hits** across 3542-line / 452K log. Same for `grep -c 'warning:'` → **0 hits**. The `cfg(debug_assertions)` manifest warning that appeared 3× in Story 1.4's logs is cross-platform eliminated.
  - [x] All evidence recorded in Debug Log References (CI run URL + per-job durations + grep outcomes).

  **Suggested commit split** (matching the Story 1.3/1.4 split pattern — source artifacts separate from BMad bookkeeping):
  ```bash
  # 1) source artifacts (note: excludes .claude/scheduled_tasks.lock, a runtime artifact)
  git add src/main.rs Cargo.toml Cargo.lock docs/plugin-compatibility.md
  git commit -m "feat: minimal Bevy app + remove bevy_egui from Cargo.toml (Story 1.5)"
  git push origin master
  ```
  ```bash
  # 2) BMad bookkeeping (after CI turns green)
  git add _bmad-output/implementation-artifacts/1-5-minimal-bevy-app-opens-a-window-on-all-three-platforms.md \
          _bmad-output/implementation-artifacts/sprint-status.yaml
  git commit -m "bmad: story 1.5 complete — minimal Bevy window on macOS + CI green"
  git push origin master
  ```

- [x] **Task 7 — Scope guardrails — what this story does NOT do (AC: #8)**
  - [x] `git status --short` shows exactly 5 modifications + 2 untracked: `M Cargo.lock`, `M Cargo.toml`, `M _bmad-output/implementation-artifacts/sprint-status.yaml`, `M docs/plugin-compatibility.md`, `M src/main.rs`, `?? _bmad-output/implementation-artifacts/1-5-minimal-bevy-app-opens-a-window-on-all-three-platforms.md`, `?? .claude/scheduled_tasks.lock`. First 6 are the story-expected set. The 7th (`.claude/scheduled_tasks.lock`) is a Claude Code runtime artifact — NOT story-scope; Till excludes it at `git add` time. No `state.rs`, no `src/<module>/` tree, no asset files, no CI workflow changes, no `rust-toolchain.toml` / `rustfmt.toml` / `clippy.toml` / `.gitignore` / `.gitattributes` edits.
  - [x] `src/main.rs` line count: **7** (2-line module doc, 1 blank, `use bevy::prelude::*;`, 1 blank, 3-line `fn main`). Under the ≤ 10 ceiling.
  - [x] `Cargo.toml` net change: **-4 lines** (blank + comment + `[target.'cfg(debug_assertions)'.dependencies]` header + `bevy_egui = "0.39"`). Zero lines added. No `[features]` section, no new optional dep, no replacement for `bevy_egui`.
  - [x] `grep -r bevy_egui .` across the repo (excluding `target/`) confirms: zero hits in `src/`, zero hits in `Cargo.toml`, zero hits in `Cargo.lock` (pruned); remaining hits are only in doc/story/change-log files documenting the *removal* — expected.

### Review Findings

_Added 2026-04-23 by `bmad-code-review` (3-layer adversarial review: Blind Hunter + Edge Case Hunter + Acceptance Auditor). Raw findings: 30 (BH 17 + EC 9 + AA 4). Triage outcome: 0 Decision-Needed, **2 Patch (both applied)**, 5 Defer, 18 Dismissed (2 merged cross-layer; AA1+AA2 evidence-gap patch P3 skipped by Till → dismissed). Acceptance Auditor verdict: **Approve** (5/5 in-diff ACs PASS; 5/5 out-of-diff ACs confirmed via DAR runtime evidence). No blockers._

- [x] [Review][Patch → Applied] **Removed `Story 1.5 scope.` trailer from `src/main.rs` module doc** `[src/main.rs:2]` — Bevy-idiomatic module doc describes what the file IS, not which story introduced it. Commit-message / story-file records carry the provenance. Source: Blind Hunter BH8.
- [x] [Review][Patch → Applied] **Added ✅ RESOLVED 2026-04-23 by Story 1.5 note to `deferred-work.md`'s `cfg(debug_assertions)` review-correction entry** `[_bmad-output/implementation-artifacts/deferred-work.md:31-33]` — historical context preserved below the note so the audit trail stays intact; readers no longer chase a phantom issue. Source: Blind Hunter BH13.
- [x] [Review][Defer] **Historical doc-drift: planning-artifacts still reference `bevy_egui` as a current pinned dep** `[_bmad-output/planning-artifacts/epics/epic-1-foundation-plugin-compatibility-gate.md:22,42 ; _bmad-output/planning-artifacts/epics.md:128,177,282,451,471 ; _bmad-output/planning-artifacts/epics/epic-list.md:5 ; _bmad-output/planning-artifacts/epics/requirements-inventory.md:108,157]` — deferred, cross-story scope. These are historical Story 1.1 / 1.2 ACs that were accurate at their time; rewriting them is rewriting history. A dedicated doc-sync chore story (or an addendum at the next planning-sweep story) should add 2026-04-23 erratum pointers. Not Story 1.5's scope. Source: Edge Case Hunter EC3 + EC4 + EC5 + EC6.
- [x] [Review][Defer] **`architecture.md` prescribes the broken `cfg(debug_assertions)` pattern and shows `bevy_egui` in the starter Cargo.toml skeleton** `[_bmad-output/planning-artifacts/architecture.md:256,977]` — deferred, explicitly M2-owned per this story's own Dev Notes ("Architecture Compliance" section). Erratum lands with the M2 debug-panels story alongside the first actual feature-flag usage. Flagging so a future M2 implementer doesn't copy the broken pattern. Source: Edge Case Hunter EC7.
- [x] [Review][Defer] **Story 1.2 artifact still shows `bevy_egui` in its gated-plugins matrix + `cfg(debug_assertions)` Known Issue** `[_bmad-output/implementation-artifacts/1-2-plugin-compatibility-verification-gate.md:72,113,123,156,210]` — deferred. Clean fix: append a 2026-04-23 addendum noting removal by Story 1.5; leave historical body intact. Same reasoning as the planning-artifact drift above — historical artifact should not be rewritten, but a pointer-to-current-truth is cheap and safe. Source: Edge Case Hunter EC8.
- [x] [Review][Defer] **`App::run()` return value (`AppExit`) is discarded by `fn main()`** `[src/main.rs:7]` — deferred. Bevy 0.18 `App::run() -> AppExit` is `#[must_use]`-flavored; discarding it means `AppExit::Error(_)` still exits 0 (crashed Bevy loop reported as success). Current clippy config didn't flag it (run passed clean). Fix pattern: `fn main() -> AppExit { App::new().add_plugins(DefaultPlugins).run() }`. Landing point: Story 1.6 is the next story to touch `main.rs` (GameState wiring); land the return-type change there together with the state-registration code. Source: Blind Hunter BH14 + Edge Case Hunter EC1.
- [x] [Review][Defer] **`.claude/scheduled_tasks.lock` is not gitignored** `[.gitignore (future)]` — deferred. This file is a Claude Code runtime artifact (appeared in working tree during the ScheduleWakeup call for CI polling). The existing `.gitignore` excludes `.claude/settings.local.json` (same category) but not arbitrary runtime state. Low urgency. Add `.claude/*.lock` or `.claude/scheduled_tasks.lock` at the next `.gitignore`-touching story. Story 1.5 explicitly forbids touching `.gitignore` per Task 7 scope guardrails. Source: dev-story verification.

### Review Findings — Dismissed (recorded for future-reviewer context)

_These were raised but rejected during triage. Kept as breadcrumbs so a future reviewer does not re-litigate them._

- **[blind] No `WindowPlugin` customization (title / resolution / present-mode)** — REFUTED by AC #1 literal language: "default Bevy title and size" is the spec requirement. The window title `asteroids3D` derives from the package name; that's the default. Splash-text UI belongs to Story 1.7, not a `WindowPlugin` override.
- **[blind] Declared deps (`avian3d`, `bevy_kira_audio`, `bevy_mod_outline`, `leafwing-input-manager`, `directories`) unused by `main.rs`** — by design per Story 1.1 (pinned early so the plugin-compatibility gate Story 1.2 verifies them) + Story 1.5's own "Library/Framework Requirements" table explicitly enumerates "not imported yet" for each.
- **[blind] `DefaultPlugins` pulls audio/asset/winit — headless CI concern** — CI never runs the binary, only compiles. No runtime test exercises `App::new()`. Confirmed via Task 4 CI design in 1-4-…md.
- **[blind] `bevy_egui` removal may break transitive expectation elsewhere** — verified during Task 7 scope guardrail: `grep -r bevy_egui .` (excluding target/_bmad*/.claude/.git) showed the ONLY remaining reference is `docs/plugin-compatibility.md` documenting the removal. Zero code references. False positive for in-diff scope.
- **[blind] "Manifest warning eliminated" doc claim is unverified by diff** — evidenced in companion artifacts: Dev Agent Record has CI log grep `grep -c 'warning:' → 0` across 3542-line combined log. Evidence is in the spec file, not the diff.
- **[blind] Doc prose about cargo `cfg(debug_assertions)` evaluation is imprecise** — wording quibble. "Cargo treats the predicate as always-true" is a reasonable approximation of the observed effect (the crate IS compiled in every configuration, including release). The technical reality (Cargo warns + does not gate) produces the same functional outcome. Not worth a doc edit.
- **[blind] No `[features]` section placeholder for future `dev-tools`** — intentionally NOT scaffolded per `feedback_staged_rollout.md` (reduced-MVP + post-MVP expansion over speculative scaffolding). Discussed explicitly during story creation and reconfirmed by Till.
- **[blind] Cargo.lock churn includes large `objc2 0.6` / `windows 0.60` subsystems** — informational only. `--locked` CI confirmed internal consistency. Supply-chain surface reduction is a net positive.
- **[blind] `image` crate loses `tiff` feature transitively** — no `.tif`/`.tiff` assets exist or are planned for MVP per `architecture.md` asset-type enumeration. Re-enable at the specific story that first introduces a TIFF asset (none currently planned).
- **[blind] `getrandom` loses `js-sys` / `wasm-bindgen`** — WASM target is explicitly out-of-scope per `architecture.md:163` ("iOS, Android, Web/WASM jobs all explicitly out-of-scope") and Story 1.4's CI-matrix scope.
- **[blind] "M2" milestone reference without ID/ticket link** — navigable via `MEMORY.md → project_hobby_cadence` (M0–M9 milestone map) + sharded epics plan (`reference_epics_plan`). Project convention is milestone-name references, not ticket IDs.
- **[blind] No integration/smoke test for "app opens window"** — spec-accepted decision documented in `Testing Requirements`: "wgpu mocking is a disproportionate effort for this story." Bevy-integration tests remain post-M3 per `architecture.md:144-146`.
- **[blind] Change Log table lacks author/PR column** — style minor; existing docs template doesn't include it; out of scope.
- **[blind] Empty `assets/` dir may surface warn log on first run** — does not reproduce: actual cargo run log (`/tmp/asteroids3d-run.log`) captured 9 startup lines, zero asset-dir warnings.
- **[edge] `cargo run` on Linux headless (no DISPLAY)** — user-error class. Running a GUI binary in a non-GUI session is not a supported use case for this project.
- **[edge] `.github/workflows/ci.yml:49` uses `toolchain: stable` ignoring `rust-toolchain.toml` pin `1.94.1`** — INTENTIONAL per Story 1.4 Dev Notes ("Why `toolchain: stable` in the `build` job AND `rust-toolchain.toml` pinning `1.94.1`"): `dtolnay/rust-toolchain@master` installs stable as rustup default, but `rust-toolchain.toml` OVERRIDES it when cargo runs inside the project directory. Net effect: CI runs 1.94.1 (verified via 1.4's CI run URL).
- **[auditor] Task 1's clippy step merged with Task 4's sweep** — DAR-documented rationale under "Deviations from plan". Subtask checked with justification pointing to T4's combined evidence. Not a spec violation.
- **[auditor] AC #2 + AC #3 evidence gaps (no Windows dev-box spec; no Linux Vulkan `AdapterInfo` grep line)** — Till elected to skip retrofitting these evidence fields ("skip" reply to code-review patch P3). The Auditor had already classified both as "within the accepted evidence envelope" (AC #2 allows written confirmation; AC #3's adapter-grep recipe was explicit but the written confirmation "Linux läuft auch" was accepted as equivalent). Runtime window-opens on both OSes is confirmed; the gap is purely record-quality and not AC-failing.

## Dev Notes

### Why this story exists

Stories 1.1–1.4 proved the **build infrastructure** works: Cargo resolves all plugins (1.1, 1.2), lint/format/toolchain are pinned (1.3), CI compiles on three OSes (1.4). Story 1.5 is where the product **stops being a `println!` and starts being a Bevy app** — the first line of gameplay-adjacent code. The FR47 "binary runs on Windows 10+, Linux, macOS" requirement transitions from *hypothetically-verifiable* to *demonstrably-verifiable* when a native window opens on each platform. This is also the motivation-preservation baseline (brainstorming Phase 3, M0 completion criterion): every subsequent story ships on top of "the game opens a window," which is psychologically enormous even though technically it's five lines of Rust. [Source: prd.md:406,564; architecture.md:986-993 "M0 completion criterion"]

Additionally, Story 1.5 **inherits one residual fix** from Story 1.1's review correction: the `cfg(debug_assertions)` Cargo-manifest warning that has surfaced in every `cargo check` / `cargo build` run since day one. Both Story 1.1's review-correction note (`deferred-work.md:33-41`) and Story 1.4's Dev Notes (`1-4-…md:139`) explicitly assign the fix to this story. Since the original plan ("defer to Story 1.5 because that's when `bevy_egui` is first registered") turned out wrong — Story 1.5 does NOT register `bevy_egui` (that's M2 per `architecture.md:256`) — the lighter fix (matching `feedback_staged_rollout.md`) is to **remove `bevy_egui` from `Cargo.toml` entirely** and defer re-introduction to the M2 story that actually uses it. See AC #6 + Task 2.

### Context inherited from Stories 1.1–1.4

| Fact | Value | Source |
|---|---|---|
| Rust toolchain | `1.94.1` (stable, pinned) | `rust-toolchain.toml:5` |
| MSRV | `1.89` (CI-verified via msrv-check job) | `Cargo.toml:5`; `.github/workflows/ci.yml` |
| Bevy | `0.18` (resolved `0.18.1`) | `Cargo.toml:8`; `docs/plugin-compatibility.md` |
| Package name (corrected from planning typo) | **`asteroids3D`** | `Cargo.toml:2` (commit `113eebe`) |
| Current `src/main.rs` body | `fn main() { println!("Hello, world!"); }` (cargo-default, unchanged since bootstrap) | `src/main.rs` |
| CI workflow | `.github/workflows/ci.yml` — 3-OS build matrix + msrv-check leg, all four checks ✅ on `011f99d` | `.github/workflows/ci.yml` |
| `cfg(debug_assertions)` manifest warning | Present in all current CI logs; this story eliminates it | `Cargo.toml:28-30`; `1-4-…md:492` |
| Commit convention | Single-line subject, no `Co-Authored-By` trailer; `feat:` / `chore:` / `docs:` / `fix:` / `ci:` / `bmad:` prefixes observed | `git log --oneline -n 12` |
| Remote | `https://github.com/till-fechteler/asteroids3D.git` | `git remote -v` |
| Local dev machine | macOS 26.4.1 / arm64 (Apple Silicon, M-series) | Prior stories' Debug Log References |

### Platform backend expectations

When `cargo run` starts, Bevy's `RenderPlugin` logs an `AdapterInfo` line via `wgpu` identifying the backend chosen at runtime. Expected values on a default install:

| OS | Expected `backend` | Alternate backends acceptable? |
|---|---|---|
| Windows 10+ | `Dx12` | `Vulkan` is acceptable if DX12 somehow fails and Bevy falls back; log the observed backend verbatim. |
| Linux | `Vulkan` | `Gl` is acceptable for ancient-hardware fallback but unexpected on Till's modern Linux boxes; log anyway. |
| macOS | `Metal` | No alternative on macOS (`Vulkan` via MoltenVK exists but Bevy does not use it by default). |

**Why these expectations matter:** AC #3 + #4 text names "Vulkan" and "Metal" explicitly as backend selection evidence. Documenting the expected log line gives the dev agent a precise grep target rather than "look for the right-sounding phrase."

**Grep pattern for the adapter info line (cross-OS portable):**
```bash
cargo run 2>&1 | grep -E 'AdapterInfo|backend:'
```

### `main.rs` Skeleton

The dev agent writes this verbatim. Adjustments allowed only for rustfmt's canonical formatting (trailing newline, etc.).

```rust
//! asteroids3D — minimal Bevy app entry point.
//! Opens a default native window via DefaultPlugins. Story 1.5 scope.

use bevy::prelude::*;

fn main() {
    App::new().add_plugins(DefaultPlugins).run();
}
```

**Why a module doc comment (`//!`)** — Rust clippy's `missing_docs` lint is NOT active on this project (check `clippy.toml` — no `missing_docs_in_private_items` / similar threshold; verify before writing). The `//!` is for human-readability only: it documents the file's purpose for a future reader scanning `src/main.rs` without opening git blame. Keep it to 2 lines max.

**Why `use bevy::prelude::*;`** — Bevy's idiomatic convention. Brings `App`, `DefaultPlugins`, and most commonly-used types into scope. Story 1.6+ will build on this.

**Why no custom window title / size** — AC #1 from the epic literally states "default Bevy title and size." Setting a custom title here (e.g., `"asteroids3D"`) would:
- Conflict with the epic's explicit AC language.
- Duplicate Story 1.7's splash-screen role (which does the first UI rendering of `"asteroids3D"` as a `bevy_ui` text node inside the window content, not as an OS-window title).
- Require a `WindowPlugin` configuration via `DefaultPlugins.set(WindowPlugin { ... })` — more code, more things to review, zero value for this story's scope.

**Why no `tracing_subscriber::fmt().init()`** — Bevy 0.18's `DefaultPlugins` includes `bevy::log::LogPlugin`, which wires `tracing` → stderr automatically. Story 1.8 is where custom tracing setup lands (user-log-dir file output + panic hook). For Story 1.5, Bevy's built-in is sufficient for window-startup visibility. Adding our own init here would clash with `LogPlugin` at runtime (the crate `tracing_subscriber::fmt().try_init()` returns `Err` if a global subscriber is already installed).

**Why no `#[bevy_main]` macro** — that macro is for `bevy_mobile` iOS/Android entrypoints, which are out-of-scope per architecture.md:163. Plain `fn main()` is correct for desktop.

### Cargo.toml Diff

Current `Cargo.toml` lines 27–30 (the broken block + its preceding comment):
```toml
# Dev-only GUI tooling (egui panels for FPS, entity inspector, tuning) — stripped from release.
[target.'cfg(debug_assertions)'.dependencies]
bevy_egui = "0.39"
```

(Line 27 is the blank line between the Linux target block and the `#` comment — count as part of the removed group if your editor does. Inspect locally to confirm exact offsets; line numbers are indicative.)

**Post-fix — simply delete those four lines.** No replacement. No `[features]` section. No optional dep. `bevy_egui` is gone from the manifest until the M2 debug-panels story re-introduces it with its first actual usage.

**Leave the `[target.'cfg(target_os = "linux")'.dependencies.bevy]` block (current lines 22–26) unchanged.** That block uses `cfg(target_os = "linux")`, which IS a Cargo-supported predicate in dependency tables — only `cfg(debug_assertions)` is not. The Linux block is load-bearing for Linux windowing backends. [Source: Cargo Reference — "Platform specific dependencies"]

**Why remove rather than migrate to a feature flag:** this project's scope-focus preference ("reduced-MVP + post-MVP expansion over speculative scaffolding" — memory `feedback_staged_rollout.md`) points at the lighter path. `bevy_egui` is unused today and will remain unused until the M2 debug-panels story. Declaring it as an optional feature now would add ~150 KB of Cargo.lock entries (egui + emath + epaint + ecolor + accesskit + transitives) for zero present benefit; re-introducing it cleanly at first use is strictly less churn than maintaining it in a "declared but never exercised" state across Epics 1–2. The feature-flag migration is the right shape — it just lands with the story that needs it. [Source: MEMORY.md → feedback_staged_rollout.md]

**What the M2 debug-panels story will eventually do** (documented here so the convention is pre-established):
```toml
# M2 debug-panels story will add:
[dependencies]
bevy_egui = { version = "0.39", optional = true }  # version re-verified at M2 against then-current Bevy pin

[features]
dev-tools = ["dep:bevy_egui"]
```
Plus `#[cfg(feature = "dev-tools")]`-gated registration in the relevant plugin. The `dep:` prefix (Cargo 1.60+) prevents an auto-mirrored `bevy_egui` feature. [Source: Cargo RFC 3143]

### Verification Recipe (local, macOS, pre-commit)

Run in order. Each command must exit 0 before the next is run.

```bash
# 1. Clean slate (optional; not required, but clarifies logs).
cargo clean

# 2. Default (only) feature set — bevy_egui removed, no cfg(debug_assertions) warning.
cargo check 2>&1 | tee /tmp/asteroids3d-check.log
grep -E 'warning:|error:' /tmp/asteroids3d-check.log
# Expect: no hits at all. The cfg(debug_assertions) warning that used to appear here must be gone.

# 3. Lock-file pruning verification.
grep -c '"bevy_egui"' Cargo.lock            # expect: 0
grep -Ec '"egui"|"emath"|"epaint"|"ecolor"|"accesskit"' Cargo.lock   # expect: 0
cargo tree --depth 1 --edges normal | grep bevy_egui
# Expect: empty (grep exits 1, no match).

# 4. Full local pre-commit sweep.
cargo build 2>&1 | tee /tmp/asteroids3d-build.log; grep -E 'warning:|error:' /tmp/asteroids3d-build.log
cargo test 2>&1 | tee /tmp/asteroids3d-test.log; grep -E 'warning:|error:' /tmp/asteroids3d-test.log
cargo clippy --all-targets -- -D warnings 2>&1 | tee /tmp/asteroids3d-clippy.log; grep -E 'warning:|error:' /tmp/asteroids3d-clippy.log
cargo fmt --all -- --check  # exit 0, no output

# 5. Actual window opens on macOS (AC #4).
cargo run 2>&1 | tee /tmp/asteroids3d-run.log &
# Wait ~3s for window to appear. Close it manually (Cmd-Q).
grep -E 'AdapterInfo|backend:' /tmp/asteroids3d-run.log
# Expect: one line with `backend: Metal`.
```

### CI Matrix interaction

The existing CI workflow (`.github/workflows/ci.yml`, Story 1.4 artifact) runs `cargo build`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all -- --check` — none of those enable `--features dev-tools`. So CI validates the **default feature set** only: bevy_egui is NOT compiled on any of the four green CI legs post-Story-1.5. This is correct — the architecture explicitly wants dev-only tools stripped from release/default builds.

If a future story (M2 debug panels) requires CI to validate the `dev-tools` feature path, add a fifth CI job with `cargo check --features dev-tools` at that time. Not in Story 1.5's scope.

**Headless-runner caveat.** GitHub Actions runners have no display. `cargo run` would fail if executed in CI with a wgpu adapter-init error. CI does NOT run `cargo run` — only `cargo build` / `cargo test`. So AC #2 + AC #3 ("window opens on Windows/Linux") are **not validated by CI** — they are validated by Till running the binary on his physical Windows/Linux boxes. The CI green signal is **compile-parity evidence**, necessary but not sufficient; Till's manual `cargo run` is the sufficient signal. Documented so the checklist review does not flag this as a gap.

### Compile-failure pathway (feature-collection gap)

Bevy 0.18's `"3d"` feature collection (architecture.md:92, introduced in 0.18 to simplify compile-time feature selection) is advertised as "pulls everything needed for 3D rendering." Empirically the compile graph includes `bevy_render`, `bevy_core_pipeline`, `bevy_pbr`, `bevy_gltf`, `bevy_scene`, `bevy_animation`, `bevy_text`, `bevy_ui`. **What is NOT guaranteed by the collection name:** `bevy_winit` (window-creation / event-loop) and `bevy_window` (window-component types). On a `default-features = false` manifest, if the chosen collection does not transitively enable these two, `DefaultPlugins` references non-existent types (`WindowPlugin`, `WinitPlugin`) and the `cargo build` step fails at the *first* invocation — before `cargo run` gets a chance to panic at runtime. This is the compile-time failure path; the runtime path is "binary runs but wgpu adapter init fails because no windowing subsystem." Both are captured by Task 4 (macOS local build first, then run) and Task 6 (CI matrix catches if any of Windows/Linux fails to build).

**If compile fails on any leg:**

1. Read the error verbatim from the log. It will name the missing type (e.g., `cannot find type WinitPlugin in crate bevy`) or the missing crate feature.
2. Patch `Cargo.toml`'s `[dependencies].bevy.features` list — typical missing features are `bevy_winit`, `bevy_window`, `bevy_render` (direct), or `x11` / `wayland` / `webgpu` for specific backends. On Linux the `x11` + `wayland` features are already enabled via the existing `[target.'cfg(target_os = "linux")'.dependencies.bevy]` block; Windows/macOS rely on the base feature list only.
3. Add the missing feature to the BASE `[dependencies].bevy.features` array (not the Linux-target override — the error would be on Windows/macOS and those use the base list).
4. Re-run `cargo build` locally → CI. Record the feature addition in Completion Notes as: "Bevy `0.18` `3d` collection needed explicit `bevy_winit` (or similar) feature; patched."

This closes Story 1.1's deferred empirical-validation concern (`deferred-work.md:9`) regardless of outcome.

### Scope boundaries — what belongs to later stories

| Concern | Story that owns it |
|---|---|
| `GameState` enum + `Bevy States` registration | **Story 1.6** — `state.rs` with `Loading`/`MainMenu`/`Arena`/`Caravan`/`PostRun`/`PhotoMode`/`Paused`. |
| `"asteroids3D"` text displayed in the window | **Story 1.7** — `bevy_ui` text node, `LoadingStateEntity` marker, splash-duration timer. |
| Custom `tracing_subscriber` init + panic hook + user-log-dir file output | **Story 1.8** — full logging architecture. Story 1.5 relies on Bevy's `LogPlugin` inside `DefaultPlugins`. |
| `bevy_egui` actual usage / panel registration | **M2 debug-panels story** (post-Epic-1) — architecture.md:256 assigns debug UI to M2. |
| `src/<feature>/` module scaffolding (flight, combat, etc.) | **Epic 3+** — first feature plugin is `src/flight/` at Epic 3. |
| Window title customization | Not planned; "asteroids3D" text lives in-window (Story 1.7), not as OS-window title. |
| Release profile tuning (LTO, codegen-units) | Already complete in Story 1.1 (`Cargo.toml:32-35`). Not touched here. |

### Architecture Compliance

- **`src/main.rs`** assembles `App::new()` and registers DefaultPlugins — matches architecture.md:149: "`src/main.rs`: `App::new()` assembly, plugin registration, top-level config." [Source: architecture.md:149]
- **No gameplay code in `main.rs`** — future plugin registrations go through `app.add_plugins(FlightPlugin).add_plugins(CombatPlugin)...` pattern. None of those plugins exist yet, so main is pristine. [Source: architecture.md:151]
- **Debug UI (`bevy_egui`) deferred to M2** — matches architecture.md:256 "Debug UI: `bevy_egui` behind `cfg(debug_assertions)`. ... Stripped from release builds (zero binary-size cost). Available from M2 onward when gameplay tuning starts." The letter of the spec says `cfg(debug_assertions)`; the spirit is "dev-only, strip from release, available from M2." This story removes the broken premature-scaffolding attempt; the M2 debug-panels story will land the crate + the correct feature-flag gating + the first actual usage together (documented in `docs/plugin-compatibility.md`'s Deferred / Planned section after Task 3). The architecture doc will want an eventual amendment noting that `features = { dev-tools = [...] }` is the canonical mechanism, not the manifest-level `cfg(debug_assertions)` that was specified in prose — that amendment lands with the M2 story, not this one. [Source: architecture.md:256; pattern-deviation policy at architecture.md:453-455]
- **Panic policy honored** — `App::new().run()` does not panic on normal startup. If wgpu adapter init fails (e.g., no GPU), Bevy panics — that's the "programmer invariant violations that prove the engine/OS is broken" bucket per architecture.md:371. Acceptable.
- **CI matrix from M0 mitigation strategy (for Tech-Risk #4, macOS cross-platform parity)** — already in place since Story 1.4. Story 1.5 exercises it: the first non-trivial wgpu-using code lands, and CI verifies it compiles on all three platforms. [Source: prd.md:443; architecture.md:34]

### Library/Framework Requirements

No new crates added to `[dependencies]`. Moving `bevy_egui` from one dep table to another + making it optional. All current crate versions:

| Crate | Version | Status after Story 1.5 |
|---|---|---|
| `bevy` | `0.18` | unchanged; provides `DefaultPlugins`, `App`, `Update`, window handling |
| `avian3d` | `0.6` | unchanged; not imported yet (Epic 3+) |
| `bevy_mod_outline` | `0.12` | unchanged; not imported yet (Epic 2) |
| `bevy_kira_audio` | `0.25` | unchanged; not imported yet (Epic 8) |
| `leafwing-input-manager` | `0.20` | unchanged; not imported yet (Epic 3) |
| `bevy_egui` | ~~`0.39`~~ | **REMOVED** — was declared in `[target.'cfg(debug_assertions)'.dependencies]` (broken: Cargo evaluates predicate as always-true). Re-introduction deferred to M2 debug-panels story as an optional feature-flag dep. |
| `serde`, `serde_json`, `ron`, `thiserror`, `tracing`, `tracing-subscriber`, `directories` | pinned per 1.1 | unchanged; not imported yet |

**No `[[bin]]` section added.** The implicit `[[bin]]` from `[package].name = "asteroids3D"` remains in place. Release-binary naming (lowercase, no `3D` capitalization) is Story 4.10 / Epic 10 packaging work per `deferred-work.md:10`.

### File Structure Requirements

Files added/modified by this story, all paths relative to project root:

| Path | Add/Modify | Purpose |
|---|---|---|
| `src/main.rs` | Modify | Replace `println!` with `App::new().add_plugins(DefaultPlugins).run()`. ~6 lines total. |
| `Cargo.toml` | Modify | Delete the `[target.'cfg(debug_assertions)'.dependencies]` block (including its preceding comment line). Net line-count change: **-4 lines**. No additions. |
| `Cargo.lock` | Modify (auto-regenerated by `cargo check`) | `bevy_egui 0.39.1` + transitives (`egui`, `emath`, `epaint`, `ecolor`, `accesskit`, …) pruned. Net shrinkage: dozens of crate entries. |
| `docs/plugin-compatibility.md` | Modify | Remove `bevy_egui` row from the "Third-party plugins (gated)" table; add a new "Deferred / Planned" bullet documenting removal + M2 re-introduction plan; delete the Known Issues `cfg(debug_assertions)` bullet (issue resolved); add Change Log entry. |

Files explicitly **not** touched by this story:

- `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml` — Story 1.3's artifacts.
- `.gitignore`, `.gitattributes` — Story 1.3 / 1.4 artifacts.
- `.github/workflows/ci.yml` — Story 1.4's artifact. Story 1.5 does not need a new CI job.
- `src/state.rs` — **does not exist yet.** Story 1.6 creates it.
- Any `src/<module>/` tree — none exist; first module lands at Epic 3.

### Testing Requirements

- No `#[test]`s are added. `App::new().add_plugins(DefaultPlugins).run()` is a runtime entrypoint — it's not unit-testable without mocking out wgpu + winit, which is a disproportionate effort for this story.
- `cargo test` runs, compiles, finds zero tests, exits 0. That's the success signal.
- **Manual run validation** on each of Till's three OSes (macOS ✅ required for the story; Windows + Linux best-effort — see Task 5) is the primary *functional* verification.
- CI continues to exercise `cargo build` + `cargo test` + `cargo clippy` + `cargo fmt --check` on all three OSes — compile-parity proof.
- Full-build-output rule honored: [Source: MEMORY.md → feedback_full_build_output.md] every `cargo …` invocation is piped to a log file and then grep'd for `warning:|error:`. Exit-0 alone is not proof — the grep is the proof.

### Latest Technical Information

- **Bevy 0.18 `DefaultPlugins`** composition (April 2026) — bundles `LogPlugin`, `TaskPoolPlugin`, `TypeRegistrationPlugin`, `FrameCountPlugin`, `TimePlugin`, `TransformPlugin`, `HierarchyPlugin`, `DiagnosticsPlugin`, `InputPlugin`, `WindowPlugin` (creates the window), `AccessibilityPlugin`, `WinitPlugin`, `RenderPlugin` (wgpu adapter init), `ImagePlugin`, `PipelinedRenderingPlugin`, `CorePipelinePlugin`, `AssetPlugin`, `ScenePlugin`, `TextPlugin`, `UiPlugin`, `PbrPlugin`, `GltfPlugin`, `AudioPlugin`, `GilrsPlugin` (gamepad), `AnimationPlugin`, `StatesPlugin`. The `3d` feature collection we have enabled includes the renderer + windowing; `2d`-specific plugins are excluded. [Source: Bevy 0.18 `crates/bevy_internal/src/default_plugins.rs`]
- **wgpu 26.x backend auto-selection** (shipped with Bevy 0.18) — prefers Dx12 on Windows, Metal on macOS, Vulkan on Linux. Falls back to Gl only if the preferred backend lacks an adapter. [Source: wgpu docs — `Instance::request_adapter`]
- **Cargo features `dep:` syntax** stabilized in Cargo 1.60 (April 2022). Toolchain `1.94.1` supports it cleanly. No concerns. [Source: Cargo RFC 3143]
- **`bevy::log::LogPlugin` default level** is `info` for Bevy crates and `info` for app crates; `wgpu` is gated at `error`. `RUST_LOG` env-var override works as expected. [Source: Bevy 0.18 `crates/bevy_log/src/lib.rs`]

### Previous Story Intelligence

**From Story 1.1 (Cargo.toml bootstrap):**
- The `[target.'cfg(debug_assertions)'.dependencies]` block was introduced by 1.1. The Blind Hunter + Edge Case Hunter both raised the "this doesn't actually gate `bevy_egui` to debug builds" finding during the 1.1 review; both were **incorrectly dismissed** under "the compile succeeded, so the semantics are fine." The 2026-04-22 review correction (logged to `deferred-work.md:33-41` and committed as `0cbe8a3 docs: log review correction for cfg(debug_assertions) finding`) confirmed the dismissal was wrong. Story 1.5 corrects it. [Source: deferred-work.md:33-41; commit `0cbe8a3`]
- The `default-features = false` concern (could Windows/macOS miss winit/wgpu-Metal?) was deferred from 1.1 review to "empirically validated when Story 1.5 opens the first window on each platform." **Story 1.5 IS that validation gate.** If `cargo run` on Windows or macOS surfaces a "missing adapter / no windowing backend" panic, patch the needed Bevy feature back into `Cargo.toml` and report in Completion Notes. Otherwise, the 1.1 concern is officially resolved. [Source: deferred-work.md:9]

**From Story 1.2 (plugin compat gate):**
- All four plugins compiled cleanly on macOS. Story 1.5 brings the first one (`bevy` itself) into actual usage via `DefaultPlugins`. The other three (`bevy_mod_outline`, `bevy_kira_audio`, `leafwing-input-manager`) remain unused until their respective feature stories in Epic 2, Epic 3, Epic 8. [Source: docs/plugin-compatibility.md]

**From Story 1.3 (toolchain/lint):**
- `rustfmt.toml` sets `newline_style = "Unix"` and `edition = "2024"`. The `src/main.rs` skeleton in **Dev Notes → `main.rs` Skeleton** respects both (LF line endings, no edition-sensitive constructs). `cargo fmt --check` should pass without reformat.
- `clippy.toml` sets only thresholds (cognitive-complexity 30, too-many-arguments 8, type-complexity 500). The minimal `main.rs` triggers zero lints. [Source: clippy.toml]

**From Story 1.4 (CI matrix):**
- CI is green on the pre-1.5 HEAD (`011f99d`). Story 1.5's push is the first commit to exercise CI with non-trivial compile work (wgpu + winit + full `DefaultPlugins` tree). Cold-cache Windows took 71m on 1.4's first push; warm-cache should drop to < 10 min. Story 1.5's introducing commit may NOT be warm-cache (the `Cargo.toml` dep-graph change invalidates `Swatinem/rust-cache`'s key in some configurations). Expect Windows 10–40 min, macOS 15–25 min, Linux 10–20 min — calibration datum, not a blocker. [Source: 1-4-…md:487-488]
- Story 1.4's patched `ci.yml` has `timeout-minutes: 120` on `build`, `60` on `msrv-check`. Any single leg exceeding that is a real bug, not a hung runner. [Source: 1-4-…md:86]
- Three `cfg(debug_assertions)` warnings appeared in Story 1.4's CI logs (one each on Ubuntu `cargo build`, Windows `cargo build`, Windows `cargo test`). Story 1.5's CI run should have **zero** — that's the cross-platform verification that removing the broken block eliminated the warning on every OS, not just Till's local macOS. [Source: 1-4-…md:492]

**Commit-type convention:** `feat:` is the natural prefix for Story 1.5's source-artifact commit (first non-infra code). `docs:` is acceptable for the plugin-compatibility.md update if done as a separate commit. Suggested splits in Task 6.

### Git Intelligence

Recent commits (newest first, 12 total on `master`):

| SHA | Subject | Relevance to 1.5 |
|---|---|---|
| `011f99d` | `bmad: story 1.4 review complete — 3 patches applied, 10 defers logged` | Latest HEAD. Story 1.5 builds on this commit. |
| `3f3d5f2` | `ci: add timeout + DEBIAN_FRONTEND + --locked (Story 1.4 review patches)` | ci.yml hardening. Relevant: `--locked` now in effect on CI `cargo` calls; Cargo.lock changes in 1.5 MUST be committed. |
| `7b99af6` | `bmad: story 1.4 complete — three-platform CI matrix green` | 1.4 bookkeeping. |
| `73dc4e6` | `ci: three-platform GitHub Actions matrix (Story 1.4)` | ci.yml first commit. |
| `f8f067c` | `bmad: story 1.3 complete — toolchain, lint, format configuration` | Toolchain bookkeeping. |
| `2491785` | `chore: toolchain, lint, and format configuration (Story 1.3)` | `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml` land. Relevant for the local `cargo fmt` / `cargo clippy` invocations in Task 1. |
| `48cedcd` | `bmad: story 1.2 complete — plugin compatibility gate passed` | Irrelevant. |
| `23ab9ec` | `docs: add plugin compatibility verification gate (Story 1.2)` | Creates `docs/plugin-compatibility.md` — Task 3 edits this file. |
| `0cbe8a3` | `docs: log review correction for cfg(debug_assertions) finding` | **Critical context** — logged the retraction of the original "dismiss" verdict and explicitly scheduled the fix for Story 1.5. Task 2 fulfills this commitment. |
| `113eebe` | `fix: correct package name typo asteriods3D -> asteroids3D` | Package name is `asteroids3D`. Do not re-rename. |
| `abe7742` | `planning: import BMad artifacts` | Irrelevant. |
| `4ca3869` | `chore: bootstrap Cargo project (Story 1.1)` | Original `src/main.rs` = `println!("Hello, world!");`. This story replaces that body. |

**`--locked` gotcha:** Story 1.4's review patches added `--locked` to `cargo build/test/clippy` invocations on CI. When Task 2 modifies `Cargo.toml` dep layout, the regenerated `Cargo.lock` MUST be committed in the source-artifact commit, otherwise the `--locked` CI jobs will fail with `error: the lock file Cargo.lock needs to be updated but --locked was passed to prevent this`. Task 6's commit must include both `Cargo.toml` AND `Cargo.lock`.

### What this story explicitly does NOT fix

Enumerated here so the review step does not flag them as "missed":

1. **No `src/state.rs` / `GameState` enum** — Story 1.6's exclusive scope.
2. **No splash-screen UI text** — Story 1.7's exclusive scope.
3. **No custom `tracing_subscriber` init, log-file output, or panic hook** — Story 1.8's exclusive scope. Bevy's `LogPlugin` provides stderr logging out-of-box.
4. **No `bevy_egui` dep declaration OR plugin registration** — the crate is removed from `Cargo.toml` entirely. Re-introduction (as a feature-flag dep) + first registration both belong to the M2 debug-panels story.
5. **No `src/<feature>/` module tree** — first feature module (`src/flight/`) is Epic 3's job.
6. **No Intel-macOS CI leg** — Story 7-6's scope (macOS universal binary).
7. **No `release.yml` workflow** — Story 4.10's scope.
8. **No macOS code-signing / notarization** — waived stretch per `project_fr48_deferred.md`.
9. **No 60 FPS performance gate** — architecture.md:889 says not CI-enforceable; playtest-only.
10. **No re-introduction of `[target.'cfg(debug_assertions)'.dependencies]` under any plugin** — when a dev-only tool is eventually needed, it goes in via the feature-flag pattern (`[features] dev-tools = ["dep:<crate>"]`, `optional = true`) at the story that first uses it.
11. **The `asteriods3D` typo in BMad artifacts** — Story 1.3 Review Findings deferred this to a dedicated chore story. Not touched here.
12. **`[profile.dev.build-override] opt-level = 0`** — re-deferred in Story 1.3 review to an M4 upgrade window. Not touched here.

### Project Structure Notes

- **`src/main.rs` is the only non-scaffolding code file after Story 1.5.** `src/` has just one file. Module scaffolding starts at Epic 3.
- **`Cargo.toml` has no `[features]` section yet.** The first feature flag (`dev-tools`) will land with the M2 debug-panels story, alongside the first optional dep (`bevy_egui`). Architecturally, every future "dev-only tooling" crate (e.g., `tracy-client` per architecture.md:260) should gate behind that same `dev-tools` feature, not a new one, unless there's a strong reason to split. Keeps the user-facing CLI simple: `cargo run` (release path) vs `cargo run --features dev-tools` (dev path — to exist from M2 onward).
- **Package name is `asteroids3D`** (from `Cargo.toml:2`). Any future reference to the binary name uses `asteroids3D` (mixed case). Release-binary renaming to `asteroids3d` (lowercase) is deferred to Story 4.10.
- **Remote URL** is `https://github.com/till-fechteler/asteroids3D.git`. Actions UI: `https://github.com/till-fechteler/asteroids3D/actions`.

### References

- [Source: _bmad-output/planning-artifacts/epics/epic-1-foundation-plugin-compatibility-gate.md#Story-1.5 (lines 101-123)]
- [Source: _bmad-output/planning-artifacts/prd.md#FR47 (line 564)]
- [Source: _bmad-output/planning-artifacts/prd.md#Tech-Risk-4-macOS-cross-platform-parity (line 443)]
- [Source: _bmad-output/planning-artifacts/architecture.md#Architectural-Decisions-Provided-by-Starter-Choice (lines 129-165)]
- [Source: _bmad-output/planning-artifacts/architecture.md#Complete-Project-Directory-Structure (lines 534-639)]
- [Source: _bmad-output/planning-artifacts/architecture.md#Debug-UI (line 256) — bevy_egui intent]
- [Source: _bmad-output/planning-artifacts/architecture.md#First-Implementation-Priority (lines 961-993) — M0 completion criterion]
- [Source: _bmad-output/implementation-artifacts/1-1-bootstrap-cargo-project-with-hand-authored-cargo-toml.md (Cargo.toml skeleton inherited as-is)]
- [Source: _bmad-output/implementation-artifacts/1-2-plugin-compatibility-verification-gate.md (plugin compat baseline)]
- [Source: _bmad-output/implementation-artifacts/1-3-toolchain-lint-and-format-configuration.md (lint/fmt/toolchain baseline)]
- [Source: _bmad-output/implementation-artifacts/1-4-three-platform-ci-matrix.md (CI matrix + --locked gotcha)]
- [Source: _bmad-output/implementation-artifacts/deferred-work.md (lines 9-10, 33-41 — inherited defers + cfg(debug_assertions) correction)]
- [Source: docs/plugin-compatibility.md (gate-passed artifact; Task 3 edits)]
- [Source: Cargo RFC 3143 — `dep:` syntax in feature specs]
- [Source: Bevy 0.18 `crates/bevy_internal/src/default_plugins.rs` — DefaultPlugins composition]
- [Source: Cargo Reference — Platform specific dependencies (`cfg()` predicates in dep tables)]

## Dev Agent Record

### Agent Model Used

claude-opus-4-7 (Claude Code) — 1M-context configuration.

### Debug Log References

Tasks T1–T4, T7 executed on Till's local macOS (26.4.1 / Darwin 25.4.0 / arm64 / Apple M5 Pro / 18 cores / 64 GiB). Toolchain `1.94.1-aarch64-apple-darwin` active via `rust-toolchain.toml`.

| Command | Exit | Log | Notes |
|---|---|---|---|
| `cargo fmt -- --check` (post-T1) | 0 | — | Zero output. 7-line `src/main.rs` already matches canonical rustfmt (edition 2024, max_width 100, newline_style Unix). |
| `cargo check` (post-T2) | 0 | `/tmp/asteroids3d-check.log` | 2-line output: `Blocking waiting for file lock on build directory` (transient), `Finished 'dev' profile ... in 18.07s`. Zero `warning:|error:` hits (grep exit 1). **The `cfg(debug_assertions)` manifest warning is gone.** |
| `grep -c '"bevy_egui"' Cargo.lock` | 1 (no match) | — | Output: `0`. Lock file pruned. |
| `grep -Ec '"egui"\|"emath"\|"epaint"\|"ecolor"' Cargo.lock` | 1 (no match) | — | Output: `0`. All egui-proper transitives pruned. 10 remaining `accesskit*` refs are Bevy's own accessibility-plugin deps (part of `DefaultPlugins`), not bevy_egui transitives. |
| `cargo tree --depth 1 --edges normal \| grep -i egui` | 1 (no match) | — | Zero egui-family edges in the normal dep graph. |
| `cargo build` (T4) | 0 | `/tmp/asteroids3d-build.log` | 1m 45s. Full DefaultPlugins graph compiled including `bevy_winit 0.18.1`, `bevy_window 0.18.1`, `bevy_render 0.18.1`, `winit 0.30.13`, `wgpu 27.0.1`, `metal 0.32.0`, `wgpu-hal 27.0.4`, `wgpu-core 27.0.3`. Zero `warning:|error:` hits. |
| `cargo run` (T4) | 0 (after window close) | `/tmp/asteroids3d-run.log` | Full startup log (9 lines): `SystemInfo { os: "macOS 26.4.1", kernel: "25.4.0", cpu: "Apple M5 Pro", core_count: "18", memory: "64.0 GiB" }` → `AdapterInfo { name: "Apple M5 Pro", vendor: 0, device: 0, device_type: IntegratedGpu, driver: "", driver_info: "", backend: Metal }` → `GPU preprocessing is fully supported on this device.` → `Creating new window asteroids3D (0v0)`. Zero panics. Process exited cleanly after window close. |
| `cargo test` (T4) | 0 | `/tmp/asteroids3d-test.log` | `running 0 tests` / `test result: ok. 0 passed; 0 failed`. Zero warnings. |
| `cargo clippy --all-targets -- -D warnings` (T4) | 0 | `/tmp/asteroids3d-clippy.log` | 0.25s (warm). Zero warnings. |
| `cargo fmt --all -- --check` (T4) | 0 | — | Zero output. |
| `git status --short` (T7) | 0 | — | 5 modifications + 2 untracked. All expected scope + 1 Claude-Code runtime file (`.claude/scheduled_tasks.lock`) which Till excludes at `git add` time. |
| `grep -rl bevy_egui .` (T7, excluding target/.git/_bmad/_bmad-output/.claude) | 0 | — | Only `docs/plugin-compatibility.md` matches — and it documents the removal. `src/*.rs` + `Cargo.toml` + `Cargo.lock` all clean. |

**CI run URL (AC #5):** https://github.com/till-fechteler/asteroids3D/actions/runs/24842252974 — triggered by source-artifact commit `03eb7a4`, started 2026-04-23T14:58:25Z, finished 2026-04-23T16:01:02Z (62m 37s total).

Per-leg outcomes (all ✅ green):

| Job | Runner | Duration | Outcome |
|---|---|---|---|
| `build (ubuntu-latest)` | `ubuntu-latest` / x86_64-unknown-linux-gnu | 29m 13s | ✅ build, test, clippy (`-D warnings`), fmt --check |
| `build (windows-latest)` | `windows-latest` / x86_64-pc-windows-msvc | 62m 30s | ✅ build, test, clippy (`-D warnings`), fmt --check (cold `Swatinem/rust-cache` — Cargo.lock dep-graph change invalidated keyspace; warm-cache for next run) |
| `build (macos-latest)` | `macos-latest` / aarch64-apple-darwin | 20m 48s | ✅ build, test, clippy (`-D warnings`), fmt --check |
| `msrv-check (rust 1.89, ubuntu-latest)` | `ubuntu-latest` / x86_64-unknown-linux-gnu | 4m 19s | ✅ `cargo check --locked` on Rust 1.89 |

**Full-log grep for warnings/errors (per MEMORY.md → feedback_full_build_output.md):** downloaded the combined log via `gh run view --log` → 3542 lines / 452K. Results:
- `grep -c 'error:'` → **0** across every job / every step.
- `grep -c 'warning:'` → **0** across every job / every step.
- The 3× `cfg(debug_assertions)` manifest warning that appeared in Story 1.4's logs (Ubuntu `cargo build` + Windows `cargo build` + Windows `cargo test`) is **cross-platform eliminated**. Fix verified on Windows + Linux, not just Till's macOS.

**FR47 baseline verified for commit `03eb7a4`:** the minimal Bevy binary compiles + tests + lints cleanly on all three platforms plus the MSRV leg. The architecture's "CI matrix from M0" risk-mitigation (architecture.md:34) continues to operate as a real, running safety net now that the first non-trivial wgpu-consuming code has landed.

**Window-opens evidence (AC #2, #3, #4) — all three OS runtime-confirmed:**

| OS | Runtime verification | Evidence |
|---|---|---|
| macOS 26.4.1 / arm64 / Apple M5 Pro | ✅ (dev-agent) | `/tmp/asteroids3d-run.log` — `AdapterInfo { ..., backend: Metal }`, window created, exit 0 on close. |
| Windows (Till's physical box) | ✅ (Till) | "Windows OK" confirmation 2026-04-23; window opened, no adapter-line grep captured. |
| Linux (Till's physical box) | ✅ (Till) | "Linux läuft auch" confirmation 2026-04-23; window opened. |

### Completion Notes List

**Status: ✅ Story 1.5 complete.** All 10 ACs satisfied, all 7 tasks checked. CI green on Windows + Linux + macOS + MSRV leg for the introducing commit `03eb7a4`. Three OS runtimes confirmed: macOS (dev-agent, Metal adapter) + Windows (Till) + Linux (Till). Zero warnings / zero errors across the entire CI log (3542 lines). The inherited `cfg(debug_assertions)` manifest warning is cross-platform eliminated.

**What was actually implemented + verified:**

1. **`src/main.rs` (7 lines)** — canonical minimal Bevy app: `App::new().add_plugins(DefaultPlugins).run()`. Window opens on macOS with Metal backend.
2. **`Cargo.toml` (-4 lines)** — deleted the broken `[target.'cfg(debug_assertions)'.dependencies]` block + `bevy_egui = "0.39"`. No `[features]` section, no replacement. `cfg(debug_assertions)` manifest warning eliminated.
3. **`Cargo.lock`** — pruned: `bevy_egui` + `egui` + `emath` + `epaint` + `ecolor` and their transitives all gone.
4. **`docs/plugin-compatibility.md`** — `bevy_egui` row removed; Deferred/Planned section added with the M2 re-introduction template (feature-flag pattern); Known Issues cfg(debug_assertions) bullet retired; Change Log entry appended.

**Three inherited concerns resolved by this story:**

- **Story 1.1's `default-features = false` windowing-backend validation (`deferred-work.md:9`)** → RESOLVED. Bevy `0.18`'s `"3d"` feature collection transitively pulls `bevy_winit` + `bevy_window` + `bevy_render` on macOS. `cargo run` opens a Metal-backed window with zero Cargo.toml patches.
- **Story 1.1's `cfg(debug_assertions)` review correction (`deferred-work.md:33-41`)** → RESOLVED. Removing the broken block eliminates the manifest warning; re-introduction plan is documented in `docs/plugin-compatibility.md`'s new Deferred / Planned section.
- **Story 1.4's "Story 1.5 owns the fix" Dev Notes reference (`1-4-…md:139`)** → RESOLVED.

**Follow-up work surfaced (NOT story-1.5 scope):**

1. **`.claude/scheduled_tasks.lock` should be added to `.gitignore`.** This file is a Claude Code runtime artifact; the existing `.gitignore` already excludes `.claude/settings.local.json` (same category). A follow-up chore story should either add `.claude/*.lock` or `.claude/scheduled_tasks.lock` to `.gitignore`. Not done here because Story 1.5's scope-guardrail Task 7 explicitly forbids touching `.gitignore`.
2. **CI run URL + per-job durations + full-log grep outcomes** captured in Task 6 once Till pushes.
3. **Windows + Linux runtime window-opens verification** captured in Task 5 if Till reaches physical hardware sessions before review.

**Deviations from plan:**

- **Task 1's `cargo clippy` step was merged into Task 4's sweep** rather than running twice. Rationale: Task 2 modifies `Cargo.toml`, so running clippy in Task 1 (pre-Cargo.toml-edit) and again in Task 4 (post-edit) would compile the build graph twice for the same semantic assertion. Merging saves ~1m 45s. The Task 4 clippy run exit-0'd, which is a strictly stronger signal than separate runs would have been.
- **`cargo check` (T2) completed in 18s** rather than the expected cold-ish time. This is because rust-analyzer's `flycheck0` target directory had pre-populated cache for most of the Bevy graph during the story-writing phase. The first true "cold" compile was the subsequent `cargo build` in T4 (1m 45s for the full debug-binary link).

### File List

All paths relative to project root.

**Modified (source artifacts — will land in commit `feat: minimal Bevy app + remove bevy_egui from Cargo.toml (Story 1.5)`):**
- `src/main.rs` — replaced cargo-default `println!` body with `App::new().add_plugins(DefaultPlugins).run()` + module doc comment + `use bevy::prelude::*;` import. 7 lines total.
- `Cargo.toml` — removed 4 lines: blank + `# Dev-only GUI tooling …` comment + `[target.'cfg(debug_assertions)'.dependencies]` block. Zero additions.
- `Cargo.lock` — regenerated by `cargo check`; `bevy_egui 0.39.1` + `egui` / `emath` / `epaint` / `ecolor` and their transitives pruned.
- `docs/plugin-compatibility.md` — `bevy_egui` row removed from gated-plugins table; new Deferred / Planned re-introduction section added with M2 feature-flag template; Known Issues section reset; Change Log appended.

**Modified (BMad bookkeeping — will land in follow-up `bmad:` commit after CI green):**
- `_bmad-output/implementation-artifacts/1-5-minimal-bevy-app-opens-a-window-on-all-three-platforms.md` — this file: Task 1/2/3/4/7 checkboxes [x]; Dev Agent Record populated; Change Log entries added; status will flip `ready-for-dev` → `in-progress` → (eventually) `review`.
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — `development_status[1-5-…]` flipped `ready-for-dev` → `in-progress` (will flip `in-progress` → `review` at story close in Step 9).

**Unchanged (scope guardrails honored — verified via `git status --short` + targeted `git diff` checks):**
- `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`, `.gitignore`, `.gitattributes`, `.github/workflows/ci.yml`.
- `docs/plugin-compatibility.md` is modified, but `Cargo.lock` was regenerated by Cargo — the only "source" manifest edit is `Cargo.toml`.
- No `state.rs`, no `src/<module>/` scaffolding, no asset files.

## Change Log

| Date | Author | Change |
|---|---|---|
| 2026-04-23 | claude-opus-4-7 (create-story) | Story 1.5 drafted. Scope: `src/main.rs` = `App::new().add_plugins(DefaultPlugins).run()` + remove `bevy_egui` from `Cargo.toml` entirely (delete the broken `[target.'cfg(debug_assertions)'.dependencies]` block). Re-introduction as a feature-flag dep is deferred to the M2 debug-panels story, per Till's staged-rollout preference. Three inherited concerns resolved: (a) Story 1.1's `default-features = false` Windows/macOS-windowing-backend validation gate (deferred-work.md:9), (b) Story 1.1's `cfg(debug_assertions)` review correction (deferred-work.md:33-41), (c) Story 1.4's "Story 1.5 owns the fix" note (1-4-…md:139). Window-open evidence on Windows + Linux depends on Till's physical hardware access; CI is compile-parity only. Status: ready-for-dev. |
| 2026-04-23 | claude-opus-4-7 (create-story, Till-directed revision) | Switched the `cfg(debug_assertions)` fix from "feature-flag migration" (feature flag + optional dep scaffolding landing in this story) to "remove entirely" (delete 4 lines; defer re-introduction to M2 with the first actual usage). Aligns with `feedback_staged_rollout.md` — reduced-MVP + post-MVP expansion over speculative scaffolding. Status unchanged: ready-for-dev. |
| 2026-04-23 | claude-opus-4-7 (dev-story, T1–T4 + T7 on local macOS) | Implemented the minimal Bevy app: `src/main.rs` = `App::new().add_plugins(DefaultPlugins).run()` (7 lines); removed 4 lines from `Cargo.toml` (cfg(debug_assertions) block + bevy_egui); pruned bevy_egui + egui family transitives from `Cargo.lock`; updated `docs/plugin-compatibility.md` (table row removed, Deferred / Planned section added, Known Issues cleared, Change Log appended). Local macOS full sweep: `cargo build` (1m 45s, zero warnings), `cargo test` (0 tests, exit 0), `cargo clippy --all-targets -- -D warnings` (0.25s warm, exit 0), `cargo fmt --all --check` (exit 0), `cargo run` (window opened, `AdapterInfo { backend: Metal, … "Apple M5 Pro" }`, exit 0 on window close). Three inherited defers resolved: deferred-work.md:9 (default-features = false → DefaultPlugins resolves cleanly on macOS), deferred-work.md:33-41 (cfg(debug_assertions) warning gone), 1-4-…md:139 ("Story 1.5 owns the fix"). Status: ready-for-dev → in-progress. Tasks 5 (Windows/Linux runtime) + 6 (push + CI observation) pending Till's manual action. |
| 2026-04-23 | claude-opus-4-7 (dev-story, T5 + T6 finalize on Till's Option-B authorization) | Commit `03eb7a4` (`feat: minimal Bevy app + remove bevy_egui from Cargo.toml (Story 1.5)`) pushed to `origin/master`. CI run `24842252974` observed — all 4 jobs ✅ in 62m 37s total: build(ubuntu) 29m13s, build(macos) 20m48s, build(windows) 62m30s (cold cache — Cargo.lock dep-graph change invalidated Swatinem/rust-cache keys), msrv-check 4m19s. Full CI log (3542 lines / 452K via `gh run view --log`): **zero `error:` hits, zero `warning:` hits** — the `cfg(debug_assertions)` manifest warning is cross-platform eliminated (was 3× in Story 1.4's logs). All three OS runtimes runtime-confirmed: macOS ✅ (dev-agent, Metal adapter) + Windows ✅ (Till: "Windows OK") + Linux ✅ (Till: "Linux läuft auch"). Status: in-progress → review. |
| 2026-04-23 | claude-opus-4-7 (code-review) | 3-layer adversarial review (Blind Hunter 17 + Edge Case Hunter 9 + Acceptance Auditor 4). Acceptance Auditor verdict: **Approve** (5/5 in-diff ACs PASS; 5/5 out-of-diff ACs satisfied via DAR runtime evidence). Triage: 0 Decision-Needed, 2 Patch (both applied), 5 Defer, 18 Dismissed. Patches applied: (P1) removed `Story 1.5 scope.` trailer from `src/main.rs:2` module doc; (P2) added `✅ RESOLVED 2026-04-23 by Story 1.5` note to `deferred-work.md`'s cfg(debug_assertions) review-correction entry (historical body preserved). Patch P3 (evidence-gap fill for Windows dev-box spec + Linux Vulkan adapter grep) skipped per Till's "skip" directive; gaps were already within the Auditor's accepted envelope and moved to Dismissed. 5 defers appended to `deferred-work.md` under "Deferred from: code review of 1-5-… (2026-04-23)": (a) historical doc-drift across planning-artifacts, (b) architecture.md erratum (M2-owned), (c) Story 1.2 artifact addendum, (d) `App::run() -> AppExit` discard in main.rs (land with Story 1.6), (e) `.claude/scheduled_tasks.lock` gitignore entry. Status: review → done. |
