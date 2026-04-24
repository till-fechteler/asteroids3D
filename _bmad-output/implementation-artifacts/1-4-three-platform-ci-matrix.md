# Story 1.4: Three-Platform CI Matrix

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a developer,
I want GitHub Actions CI running on Windows, Linux, and macOS from commit one,
so that the cross-platform parity commitment (FR47) is verified on every push instead of discovered at a milestone gate.

## Acceptance Criteria

1. **Workflow authored at `.github/workflows/ci.yml`.** A single workflow file adapted from NiklasEi's `bevy_game_template` CI lives at `.github/workflows/ci.yml`. Triggers: `push` (any branch) and `pull_request`. A `concurrency` group per `github.ref` with `cancel-in-progress: true` is declared so a second push to the same branch cancels the in-flight run. [Source: epics/epic-1-foundation-plugin-compatibility-gate.md:86-87; architecture.md:159-163; Dev Notes → ci.yml Skeleton]
2. **Three parallel OS jobs, native runners only.** A single `build` job uses a `strategy.matrix.os` of `[windows-latest, ubuntu-latest, macos-latest]` with `fail-fast: false` so all three platforms always report independently. `macos-latest` is Apple Silicon (arm64) as of April 2026 — satisfies the "Apple Silicon runner" wording in the epic without needing an explicit `macos-14` pin. [Source: epics/epic-1-foundation-plugin-compatibility-gate.md:88; architecture.md:160; Dev Notes → Platform Matrix]
3. **Each of the three OS jobs runs all four commands.** In the literal order from the epic: `cargo build`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all -- --check`. All four run in every OS job (not split Ubuntu-only). Any non-zero exit fails that OS job. [Source: epics/epic-1-foundation-plugin-compatibility-gate.md:89; architecture.md:161]
4. **Linux system dependencies installed before `cargo build`.** On the `ubuntu-latest` leg only, `sudo apt-get update -y && sudo apt-get install -y pkg-config libx11-dev libasound2-dev libudev-dev libxkbcommon-x11-0 libwayland-dev libxkbcommon-dev` runs before any cargo command. Command failures fail the step (via `&&`, not `;`). [Source: Dev Notes → Linux Deps; Bevy 0.18 linux_dependencies.md]
5. **iOS, Android, Web/WASM jobs not introduced.** The adaptation takes `ci.yml` only. Sibling workflows from the template (`release-android-google-play.yaml`, `release-ios-testflight.yaml`, `deploy-page.yaml`) are NOT copied. No `wasm32-*`, `aarch64-apple-ios`, or Android-SDK steps appear in this project's `.github/workflows/` tree. [Source: epics/epic-1-foundation-plugin-compatibility-gate.md:90]
6. **MSRV-check job added on Linux (inherited defer from Story 1.3 review).** A second job `msrv-check` on `ubuntu-latest` installs Rust `1.89` via `dtolnay/rust-toolchain@master`, sets `RUSTUP_TOOLCHAIN=1.89` in its env so `rust-toolchain.toml` does not override, installs Linux deps, and runs `cargo check`. It resolves `deferred-work.md:14`'s explicit "Story 1.4's scope" pointer. [Source: _bmad-output/implementation-artifacts/deferred-work.md:14; _bmad-output/implementation-artifacts/1-3-toolchain-lint-and-format-configuration.md:86]
7. **`.gitattributes` added to prevent CRLF regressions on Windows.** A project-root `.gitattributes` with a single line `* text=auto eol=lf` is committed. Without this, Git-for-Windows' default `autocrlf=true` checks files out with CRLF line endings and `cargo fmt --check` fails on the Windows leg only. [Source: Dev Notes → CRLF Gotcha; Research Report §5]
8. **`rust-toolchain.toml targets` stays absent (scope confirmation).** The matrix uses one native runner per OS — no cross-compilation — so `rust-toolchain.toml` does not need a `targets = [...]` field. A header comment on `ci.yml` documents this: any future cross-compile job must amend `rust-toolchain.toml` first. Resolves `deferred-work.md:16`'s DoD check. [Source: _bmad-output/implementation-artifacts/deferred-work.md:16]
9. **CI runs green on all three platforms and the MSRV leg for the introducing commit.** After the commit that adds `.github/workflows/ci.yml` is pushed, four checks appear on the commit page (`build (windows-latest)`, `build (ubuntu-latest)`, `build (macos-latest)`, `msrv-check`). All four finish with status ✅ before the story is declared done. The run URL is recorded in the Dev Agent Record → Debug Log References. [Source: epics/epic-1-foundation-plugin-compatibility-gate.md:97-99]
10. **Failure pathway preserves OS + step identification.** Job names include the runner OS (GitHub renders `build (windows-latest)` etc. from the matrix), and step names are descriptive enough that a red X on the commit page tells the reader which OS and which command failed without expanding the log. No contrived failure test is required — naming discipline at authoring time is sufficient. [Source: epics/epic-1-foundation-plugin-compatibility-gate.md:93-95]

## Tasks / Subtasks

- [x] **Task 1 — Author `.github/workflows/ci.yml` (AC: #1, #2, #3, #4, #10)**
  - [x] Created directory `.github/workflows/` at project root (`mkdir -p`).
  - [x] Wrote `ci.yml` (96 lines) verbatim from **Dev Notes → ci.yml Skeleton**. Zero deviations.
  - [x] YAML parse confirmed via `ruby -ryaml -e "YAML.safe_load(File.read(...))"` → `YAML OK`. (Python `pyyaml` unavailable on macOS default install; ruby's `psych` is preinstalled and equivalent.)
  - [x] `grep -nP '\t' .github/workflows/ci.yml` → no hits (exit 1 = no matches). Zero tabs.

- [x] **Task 2 — Add `msrv-check` job (AC: #6)**
  - [x] Included in the same `ci.yml` as a second top-level job (`msrv-check`). Single workflow file.
  - [x] `env: RUSTUP_TOOLCHAIN: "1.89"` set at job scope. Beats `rust-toolchain.toml`'s 1.94.1 pin for all `cargo` invocations in the job.
  - [x] Only `cargo check` runs. No `cargo test`, `cargo clippy`, `cargo fmt` on the MSRV leg.
  - [x] Same apt-get dependency block as the matrix `build` job. `shared-key: msrv` on `Swatinem/rust-cache@v2` to isolate cache from the stable-toolchain cache.

- [x] **Task 3 — Add `.gitattributes` (AC: #7)**
  - [x] Created `.gitattributes` at project root — 2 lines (1 comment + `* text=auto eol=lf`).
  - [x] Ran `git add --renormalize .` — no-op locally (all files already LF, expected on macOS).

- [x] **Task 4 — Header comment documents the native-targets-only invariant (AC: #8)**
  - [x] `ci.yml` lines 3-4 carry the required comment block: "Native targets only. `rust-toolchain.toml` does not set `targets = [...]` — adding a cross-compile job requires amending that file first."
  - [x] Resolves `deferred-work.md:16` without touching `rust-toolchain.toml`.

- [x] **Task 5 — Local YAML sanity + commit (AC: all)**
  - [x] `git status --short` (post-authoring) shows exactly four entries: `M _bmad-output/implementation-artifacts/sprint-status.yaml`, `?? .gitattributes`, `?? .github/`, `?? _bmad-output/implementation-artifacts/1-4-three-platform-ci-matrix.md`. No other tree changes.
  - [x] **No auto-commit.** Per the 1.1 / 1.2 / 1.3 precedent (`4ca3869` / `abe7742` / `113eebe` / `0cbe8a3` / `23ab9ec` / `48cedcd` / `2491785` / `f8f067c`), the dev agent leaves all changes staged for Till's manual commit.
  - [ ] Prepare two suggested commits (matching the Story-1.3 split pattern — source artifacts separate from BMad bookkeeping):
    ```bash
    # 1) source artifacts
    git add .github/workflows/ci.yml .gitattributes
    git commit -m "ci: three-platform GitHub Actions matrix (Story 1.4)"
    ```
    ```bash
    # 2) BMad bookkeeping (after CI turns green)
    git add _bmad-output/implementation-artifacts/1-4-three-platform-ci-matrix.md \
            _bmad-output/implementation-artifacts/sprint-status.yaml
    git commit -m "bmad: story 1.4 complete — three-platform CI matrix green"
    ```

- [x] **Task 6 — Push commit(s) and observe CI (AC: #9, #10)**
  - [x] Pushed `73dc4e6` (`ci: three-platform GitHub Actions matrix (Story 1.4)`) to `origin/master`. Push output: `f8f067c..73dc4e6  HEAD -> master`.
  - [x] CI run observed via `gh run watch 24824401702` (first watch dropped on network; re-attached with 30s interval).
  - [x] All 4 checks finished — run URL + durations recorded in Dev Agent Record → Debug Log References.
  - [x] Windows cold-cache build took 71m31s (~10× the story's initial 5–8 min estimate). Story estimate was wrong for a cold `Swatinem/rust-cache@v2` keyspace; subsequent runs should hit the cache and drop to < 10 min on Windows. Not an error — just calibration data recorded here for future story planning. No patch made.
  - [x] Zero red legs — no follow-up-commit-on-top-of-red path needed.

- [x] **Task 7 — Scope guardrails (what this story does NOT do)**
  - [x] `src/main.rs` unchanged — still cargo-default `fn main() { println!("Hello, world!"); }`. Verified via `git diff src/main.rs` → empty.
  - [x] `Cargo.toml` and `Cargo.lock` unchanged. Verified via `git diff Cargo.toml Cargo.lock` → empty.
  - [x] `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml` unchanged. Verified via `git diff ...` → empty.
  - [x] `.github/workflows/` contains only `ci.yml` — no `release.yml`, no `release-android-*`, no `release-ios-*`, no `deploy-page.yaml`.
  - [x] No macOS code-signing / notarization workflow. FR48 stays waived/stretch per `project_fr48_deferred.md`.
  - [x] No Steamworks SDK, no telemetry step, no cache upload to non-GitHub storage.
  - [x] No performance / 60 FPS assertion step in `ci.yml`. [Source: architecture.md:889]
  - [x] `asteroids3D` typo NOT touched in any file this story modifies. Still lives in `sprint-status.yaml` comments/data — deferred to a dedicated chore story.
  - [x] The `cfg(debug_assertions)` manifest warning will surface in CI `cargo build` / `cargo check` logs. Expected; does NOT fail CI because it's a cargo-manifest warning, not a rustc warning. Fix remains Story 1.5's scope.

### Review Findings

_Added 2026-04-23 by `bmad-code-review` (3-layer adversarial review: Blind Hunter + Edge Case Hunter + Acceptance Auditor). Raw findings: 24 (BH 14 + EC 10 + AA 2-deviation + 10/10-AC-PASS-verdict). Triage outcome: 0 Decision-Needed, 3 Patch, 10 Defer, 10 Dismissed. Acceptance Auditor verdict: **Approve** (10/10 ACs PASS). No blockers._

- [x] [Review][Patch → Applied] **Added `timeout-minutes` to both jobs** `[.github/workflows/ci.yml:26,70]` — `build` = 120 min, `msrv-check` = 60 min. Bounded vs. GitHub's 360-min default. Windows cold-cache was 71m, so 120 gives ~70% headroom.
- [x] [Review][Patch → Applied] **Set `DEBIAN_FRONTEND: noninteractive` on apt-get** — step-level env for the `build` leg's Install-Linux-system-dependencies step; job-level env extension for `msrv-check` (joined with existing `RUSTUP_TOOLCHAIN`). Preventative against future `tzdata`-prompting package additions.
- [x] [Review][Patch → Applied] **Added `--locked` to `cargo build`, `cargo test`, `cargo clippy`, `cargo check (MSRV)`** — `cargo fmt` intentionally NOT patched (doesn't resolve deps; `--locked` would be a no-op on it). Enforces `Cargo.lock` adherence on CI; strengthens `rust-toolchain.toml`'s reproducibility contract.
- [x] [Review][Defer] **Third-party action pinning — `dtolnay/rust-toolchain@master`, `actions/checkout@v4`** `[.github/workflows/ci.yml:44,48,86]` — deferred, spec-documented. Supply-chain risk: `@master` is a branch, `@v4` is a moving tag. GitHub security guide recommends SHA-pinning. Research Report §3 and spec Dev Notes explicitly chose these pins (dtolnay's README recommends `@master` when `toolchain:` is used). Revisit at M4/M6 upgrade window when/if Steam/Apple secrets land in CI. Flagged by Blind Hunter + Edge Case Hunter.
- [x] [Review][Defer] **MSRV job lacks `--all-targets`** `[.github/workflows/ci.yml:96]` — deferred, no-op today. `cargo check` without `--all-targets` skips tests/examples/benches. Project currently has zero of those, so the finding is latent. Becomes real when Story 1.5+ adds tests. Add to a post-test-landing chore story. Flagged by Blind Hunter.
- [x] [Review][Defer] **MSRV version hardcoded in two places; no dynamic sync with `Cargo.toml:5`** `[.github/workflows/ci.yml:65,88]` — deferred. If `rust-version` in `Cargo.toml` is bumped without editing `ci.yml`, the MSRV check silently tests an obsolete version. Fix options: (a) keep static, enforce via commit-hook / M4-M6-M9 upgrade-window checklist, (b) read via `cargo metadata`. Since MSRV bumps happen ~3 times in the MVP timeline, option (a) via upgrade-window checklist is low-ceremony. Flagged by Blind Hunter.
- [x] [Review][Defer] **`cancel-in-progress: true` cancels in-flight `master` runs on rapid back-to-back pushes** `[.github/workflows/ci.yml:12]` — deferred, spec stated incorrectly that master has "unique refs" but all master pushes share `refs/heads/master`. Impact low for solo dev (push cadence rarely sub-minute). Fix: condition cancel-in-progress on `github.ref != 'refs/heads/master'` or similar. Flagged by Blind Hunter.
- [x] [Review][Defer] **Same-repo PR double CI runs (push + pull_request both fire, concurrency keys don't coalesce)** `[.github/workflows/ci.yml:6-13]` — deferred. With `push: branches: ["**"]` AND `pull_request:`, a same-repo PR's push triggers on `refs/heads/feature` AND the PR triggers on `refs/pull/N/merge` — different strings, so concurrency doesn't dedupe. Doubles runner-minute cost per PR. Edge Case Hunter rated HIGH severity; actual Till-impact is MED because Till rarely opens PRs in solo flow. Fix: `push: branches: [master]` and let PRs carry feature-branch CI. Flagged by Edge Case Hunter + Blind Hunter.
- [x] [Review][Defer] **No `tags:` filter on `push`; future `release.yml` concurrency interaction risk** `[.github/workflows/ci.yml:7-8]` — deferred, hypothetical. When Story 4.10 adds `release.yml` for tag-triggered releases, a tag-push won't coalesce with branch-push because refs differ. Not today's problem. Surface in Story 4.10's design. Flagged by Edge Case Hunter.
- [x] [Review][Defer] **No retry around transient `apt-get update` failures** `[.github/workflows/ci.yml:34,77]` — deferred. GitHub-hosted runner mirrors 503 occasionally; `&&` short-circuits → step fails for non-code reasons. Standard mitigation: `for i in 1 2 3; do sudo apt-get update -y && break; sleep 5; done`. Low urgency — current run was green. Flagged by Edge Case Hunter.
- [x] [Review][Defer] **Linux apt-deps duplicated across `build` and `msrv-check` jobs** `[.github/workflows/ci.yml:30-41,73-83]` — deferred. Any future dep addition must be made in two places; silent drift risk. Fix: extract to composite action at `.github/actions/install-linux-deps/` or use YAML anchors. Acceptable for 2 copies; revisit at 3+ jobs. Flagged by Blind Hunter.
- [x] [Review][Defer] **`.gitattributes` lacks explicit binary markers for incoming assets** `[.gitattributes:2]` — deferred, latent. `* text=auto eol=lf` relies on Git's binary-heuristic (null-byte check). Asset files (`.png`, `.blend`, `.gltf`, `.ogg`) land in Story 2.1+; safer to pre-declare `*.png binary`, etc. No assets exist today. Flagged by Blind Hunter.
- [x] [Review][Defer] **`--workspace` not on clippy; future workspace-split gap** `[.github/workflows/ci.yml:59]` — deferred, hypothetical. Single-crate today; a workspace split (post-M3 per architecture.md:130-131) would make `clippy --all-targets` lint only the root crate. Surface when workspace split is considered. Flagged by Edge Case Hunter.

### Review Findings — Dismissed (recorded for future-reviewer context)

_These were raised but rejected during triage. Kept as breadcrumbs so a future reviewer does not re-litigate them._

- **[blind] RUSTUP_TOOLCHAIN env + toolchain input redundancy on msrv-check job** — REFUTED: both are load-bearing (action INSTALLS 1.89 via rustup; env var forces cargo to USE 1.89 against `rust-toolchain.toml`'s 1.94.1 pin). Spec Dev Notes lines 225-227 explicitly document this.
- **[blind] `build`/`test` without `--all-features`** — spec-explicit rejection (Dev Notes line 231): project ships `default-features = false` with deliberate `["3d", "png"]` slice; `--all-features` would validate a configuration we never ship.
- **[blind] `libxkbcommon-x11-0` is a runtime lib (should be `-dev`)** — matches Bevy's official `linux_dependencies.md` package list; Research Report §2 verified. Bevy's build scripts need the runtime SO resolvable, and the lib is installed alongside `-dev` headers for other packages.
- **[blind] Cache key asymmetry (build no shared-key, msrv has one)** — spec-explicit (Dev Notes lines 229, 281): `shared-key: msrv` isolates msrv's 1.89 artifacts from build's 1.94.1 ABI; `build` relies on rust-cache's default OS+rustc keying which is correct for a matrix.
- **[blind] Format/clippy runs on all 3 OSes (wasted work)** — AC #3 LITERAL requirement: "each of the three OS jobs runs all four commands." Spec explicitly chose this over the template's split pattern. Not noise; intentional.
- **[edge] MSRV Cargo.lock drift for downstream consumers** — N/A for a binary crate. Asteroids3D isn't published to crates.io; no downstream consumers exist.
- **[edge] `cfg(target_os = "linux")` lint asymmetry** — hypothetical (no current `cfg(target_os)` code branches beyond Cargo.toml's platform deps). Surface when/if future gameplay code introduces OS-gated paths.
- **[edge] Cache invalidation on toolchain bump = 3 cold builds** — calibration datum, not a bug. Toolchain bumps happen at governance windows (M4/M6/M9); accepted cost.
- **[auditor] sprint-status.yaml bundled into source-artifact commit `73dc4e6`** — self-disclosed in Completion Notes. Cosmetic deviation from the 1.1/1.2/1.3 split pattern; content is correct.
- **[auditor] `&&` vs newline chaining on apt-get** — skeleton-verbatim per AC #1. Under GitHub Actions' default `shell: bash` with `set -e`, both forms abort on failure; semantic equivalence to "`&&`, not `;`" intent preserved.

## Dev Notes

### Why this story exists

Stories 1.1–1.3 proved the project compiles and is lint-clean **on Till's local macOS machine**. Every platform risk on the board is still hypothetical until the build is reproduced on Windows and Linux runners. FR47 ("binary runs on Windows 10+, Linux, and macOS") is the most load-bearing cross-cutting requirement in the PRD — the architecture explicitly names "CI matrix from M0" as the mitigation strategy for the #4 MVP risk (macOS cross-platform parity). Delaying CI past Story 1.5 means every subsequent feature story ships without continuous parity evidence; the first time a Metal-vs-Vulkan shader diverges, it is discovered at an M1 gate, not on the commit that introduced it. Story 1.4 exists to move that discovery left. [Source: prd.md:406,443; architecture.md:34,65-68,159-163]

### Context inherited from Stories 1.1 → 1.3

| Fact | Value | Source |
|---|---|---|
| Rust toolchain (pinned via `rust-toolchain.toml`) | `1.94.1` (stable channel, pinned by ISO version) | `rust-toolchain.toml:5` |
| Components in toolchain file | `rustfmt`, `clippy`, `rust-analyzer` | `rust-toolchain.toml:6` |
| Declared MSRV | `1.89` | `Cargo.toml:5` |
| Bevy | `0.18` (resolved `0.18.1`) | `Cargo.toml:8`; `docs/plugin-compatibility.md` |
| Project root absolute path | `/Users/tillfechteler/Projekte/rust/asteroids3D/` | `.gitignore:1` (ignore of `/target`), local env |
| Package name (corrected) | **`asteroids3D`** (NOT `asteroids3D` as in planning docs) | `Cargo.toml:2`; commit `113eebe` |
| `.github/` directory | Does NOT yet exist | `ls .github/` returns `No such file or directory` |
| `docs/` directory | Exists, contains `plugin-compatibility.md` | Story 1.2 output |
| Existing `.gitignore` entries | `/target`, `.claude/settings.local.json`, `/imported_assets`, IDE + OS blocks | `.gitignore` |
| Commit convention | Single-line subject, no `Co-Authored-By` trailer | `git log --oneline -n 8` |
| Remote | `https://github.com/till-fechteler/asteroids3D.git` | `git remote -v` |

**Critical reminder — `cfg(debug_assertions)` warning is expected on CI.** Every `cargo check`/`cargo clippy` invocation emits:
> `warning: Found 'debug_assertions' in 'target.'cfg(...)'.dependencies'. This value is not supported for selecting dependencies and will not work as expected.`
This is a **cargo manifest warning**, not a rustc/clippy warning. `-D warnings` does not promote it to a hard error. CI will pass with this warning in the log; do not attempt to silence it here — Story 1.5 owns the fix (feature-flag `dev-tools` replaces the `cfg(debug_assertions)` block when `bevy_egui` is first registered). [Source: deferred-work.md:18-28; 1-3-…md:321-324]

### Platform Matrix — decisions locked

| OS key | Runner | Native target triple | Notes |
|---|---|---|---|
| Linux | `ubuntu-latest` | `x86_64-unknown-linux-gnu` | Ubuntu 24.04 LTS as of April 2026. Apt deps required. |
| Windows | `windows-latest` | `x86_64-pc-windows-msvc` | No system-deps install needed (MSVC toolchain comes with the image). |
| macOS | `macos-latest` | `aarch64-apple-darwin` | **Apple Silicon as of April 2026** — GitHub completed the Intel→arm64 default migration; no explicit `macos-14`/`macos-15` pin required. [Source: Research Report §4] |

**Why not a separate Intel-macOS leg.** The PRD calls out "Apple Silicon + Intel x86_64" both first-class (prd.md:359, architecture.md:68), but an Intel-mac CI leg is an M6/Universal-Binary concern (Epic 7 Story 7-6 covers the x86_64/arm64 lipo). Story 1.4 covers the baseline "Apple Silicon CI is green" signal; the Intel-mac addition is Story 7-6's scope. [Source: sprint-status.yaml:131 — `7-6-macos-universal-binary-intel-x86-64-arm64: backlog`]

**Why not a Linux Wayland-only and X11-only split.** Bevy 0.18's windowing backend is resolved at runtime from the feature flags `x11` + `wayland` (both enabled in `Cargo.toml`'s Linux target table). CI `cargo build` compiles both backends into the binary; runtime selection is a post-MVP concern. One `ubuntu-latest` leg is sufficient.

**Why `fail-fast: false`.** With `fail-fast: true` (GitHub's default), a first-to-fail OS aborts the other two before they report. That inverts the signal this story is trying to produce — we WANT all three legs to report independently so a macOS-only shader break and a Linux-only apt-dep drift are visible simultaneously.

### ci.yml Skeleton

The dev agent writes this verbatim. Adjustments are allowed only if a specific action pin breaks (e.g., `dtolnay/rust-toolchain@master` is renamed); record any adjustment in Completion Notes.

```yaml
name: CI

# Native targets only. rust-toolchain.toml does not set targets = [...] —
# adding a cross-compile job requires amending that file first.

on:
  push:
    branches: ["**"]
  pull_request:

concurrency:
  group: ci-${{ github.ref }}
  cancel-in-progress: true

env:
  CARGO_TERM_COLOR: always

jobs:
  build:
    name: build (${{ matrix.os }})
    strategy:
      fail-fast: false
      matrix:
        os: [windows-latest, ubuntu-latest, macos-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Install Linux system dependencies
        if: runner.os == 'Linux'
        run: |
          sudo apt-get update -y
          sudo apt-get install -y \
            pkg-config \
            libx11-dev \
            libasound2-dev \
            libudev-dev \
            libxkbcommon-x11-0 \
            libwayland-dev \
            libxkbcommon-dev

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@master
        with:
          toolchain: stable
          components: rustfmt, clippy

      - name: Cache cargo registry + target
        uses: Swatinem/rust-cache@v2

      - name: cargo build
        run: cargo build

      - name: cargo test
        run: cargo test

      - name: cargo clippy
        run: cargo clippy --all-targets -- -D warnings

      - name: cargo fmt --check
        run: cargo fmt --all -- --check

  msrv-check:
    name: msrv-check (rust 1.89, ubuntu-latest)
    runs-on: ubuntu-latest
    env:
      RUSTUP_TOOLCHAIN: "1.89"
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Install Linux system dependencies
        run: |
          sudo apt-get update -y
          sudo apt-get install -y \
            pkg-config \
            libx11-dev \
            libasound2-dev \
            libudev-dev \
            libxkbcommon-x11-0 \
            libwayland-dev \
            libxkbcommon-dev

      - name: Install Rust 1.89
        uses: dtolnay/rust-toolchain@master
        with:
          toolchain: "1.89"

      - name: Cache cargo registry + target
        uses: Swatinem/rust-cache@v2
        with:
          shared-key: msrv

      - name: cargo check (MSRV)
        run: cargo check
```

**Why `toolchain: stable` in the `build` job AND `rust-toolchain.toml` pinning `1.94.1`:** `dtolnay/rust-toolchain` installs `stable` as the rustup-default, but the project's `rust-toolchain.toml` is evaluated by `cargo` when invoked inside the project directory and OVERRIDES the default. Net effect: CI runs `1.94.1` (the pinned channel), exactly matching local dev. Specifying `toolchain: stable` in the action still serves a purpose: it pre-warms the rustup install so `cargo` doesn't download a fresh channel on its first invocation. [Source: rustup book — override precedence]

**Why `RUSTUP_TOOLCHAIN: "1.89"` in `msrv-check`:** the env-var override is the highest-precedence rustup toolchain selector. It beats both `rust-toolchain.toml` and the rustup default, without needing `rustup override set` inside the working directory. Cleanest way to ask "compile THIS crate with THIS toolchain" in CI. [Source: rustup book]

**Why `Swatinem/rust-cache@v2` not `actions/cache@v4`:** research report §1 — the template's manual cache key uses `hashFiles('**/Cargo.toml')` which cache-poisons on lockfile-only changes; `Swatinem/rust-cache` hashes the lockfile correctly out of the box and is the de-facto modern standard for Rust CI. The `shared-key: msrv` on the MSRV job prevents collision with the matrix `build` job's cache. [Source: Research Report §1, §3]

**Why NO `--all-features` on clippy / test:** the template uses `--all-features` on its lint job, but this project's `Cargo.toml` has `default-features = false` with a deliberate `["3d", "png"]` slice. `--all-features` would pull in every Bevy feature including ones we explicitly disabled (2D, audio backends we don't use yet) — the matrix would be validating a configuration we never ship. Keep clippy and test on the default feature set. [Source: Research Report §1]

**Why `--all-targets` on clippy:** matches what Story 1.3 ran locally. Ensures test targets are linted, not just the binary.

**Why no `all-doc-tests` job:** the template has one; this project has zero doc tests on zero gameplay code. Adding the job would be a no-op that consumes a runner minute. [Source: Research Report §5]

**Why `fmt --all --check`:** matches rustfmt's canonical invocation for CI. `--all` scans every crate in the workspace (single-crate now, forward-compatible with a workspace split).

### Linux Deps — why these exact packages

| Package | Bevy subsystem it satisfies |
|---|---|
| `pkg-config` | Build-time probing for all C deps below |
| `libx11-dev` | X11 windowing (Bevy feature `x11`) |
| `libasound2-dev` | ALSA (future `bevy_kira_audio` audio-out; safe to install pre-emptively, avoids surprise breakage when Epic 8 lands audio) |
| `libudev-dev` | Input device enumeration (gamepad / HID) |
| `libxkbcommon-x11-0` | Keyboard layout under X11 |
| `libwayland-dev` | Wayland windowing (Bevy feature `wayland`) |
| `libxkbcommon-dev` | Keyboard layout under Wayland |

[Source: Research Report §2; Bevy `docs/linux_dependencies.md`]

**Note on the `libasound2t64-dev` question.** Ubuntu 24.04's `t64` transition renamed the *runtime* `libasound2` → `libasound2t64` (transitional virtual). The `-dev` package kept its historical name: `libasound2-dev` resolves correctly on `ubuntu-latest` (Ubuntu 24.04) and depends on `libasound2t64`. Do not use `libasound2t64-dev` — that package does not exist. [Source: Research Report §2]

### CRLF Gotcha — why `.gitattributes` is mandatory

Git-for-Windows ships with `core.autocrlf=true` by default. On a fresh Windows checkout, every file's `\n` gets converted to `\r\n` at checkout time, and `cargo fmt --check` — which expects rustfmt's `newline_style = "Unix"` (set in `rustfmt.toml`) — fails with a cryptic per-line "Diff in … at line N" error.

`.gitattributes`:
```
* text=auto eol=lf
```
forces LF-on-checkout for every file regardless of the client-side `autocrlf` setting. A single line. [Source: Git docs — `gitattributes(5)` `text=auto` behavior]

A preceding comment explaining purpose is optional; keep it terse:
```
# Force LF line endings on all platforms. Prevents rustfmt --check regressions on Windows.
* text=auto eol=lf
```

### MSRV-check job design (from Story 1.3 Review Findings → Defer)

Story 1.3's code-review produced the explicit finding:
> "MSRV `1.89` is declared but no CI job ever exercises it. Resolution path: add a 'msrv-check' job to Story 1.4's CI matrix running `cargo +1.89 check` on a single platform (Linux suffices)." [Source: deferred-work.md:14]

**This story resolves that defer.** Scope decisions made here:

1. **`cargo check` only, not `cargo test` / `clippy` / `fmt`.** MSRV is a promise about *compilation*. Running lint and tests on the MSRV compiler buys little value and doubles run time. If a future external contributor needs the lockfile to build on 1.89, `cargo check` is the correct signal; lint drift on older compilers is not our concern.
2. **Ubuntu only, not 3-OS.** Same argument as the template: MSRV validation is platform-independent at the `cargo check` level. Tripling the runner-minutes cost for identical signal is bad.
3. **`RUSTUP_TOOLCHAIN` env var, not `rustup override set`.** The env var is cleaner (no state mutation on the runner) and beats `rust-toolchain.toml` (which is load-bearing for the `build` job's 1.94.1 pin). [Source: rustup book — override precedence]
4. **`shared-key: msrv` on rust-cache.** Prevents the MSRV job's target-dir from colliding with the 1.94.1 build job's target-dir in the same cache bucket.

### CI trigger strategy

**`on: push` to every branch + `on: pull_request`.** Rationale:
- `push` catches direct commits to `master` (solo-dev primary flow) AND commits to feature branches (even when no PR exists — solo developers often work on branches without opening PRs).
- `pull_request` catches the case where someone (or a future collaborator) opens a PR from a fork — forks don't trigger `push` on the upstream repo.
- Without `pull_request`, external contributions never get CI. Cheap to include, prevents a future gap.

**`concurrency: cancel-in-progress: true`.** A second push to the same branch cancels the in-flight run, saving runner minutes. Recommended GitHub pattern for iterative branches. Does NOT affect `master`-merged runs (those have unique refs).

### Architecture Compliance

- **File location** (`.github/workflows/ci.yml`): matches architecture.md:544-545 exactly. [Source: architecture.md:543-546]
- **Jobs** (`build + test + clippy + fmt-check per platform`): matches architecture.md:161 exactly. [Source: architecture.md:161]
- **Stripped** (iOS, Android, Web/WASM): matches architecture.md:163 exactly. [Source: architecture.md:163]
- **Release build verification at milestone gates** (NOT in this story — architecture.md:162 specifies "at milestone gates"): out of scope here. When the first release tag goes up (Story 4.10), a `release.yml` will handle the `--release` build per-OS.
- **60 FPS NOT CI-enforced** (architecture.md:889): out of scope. No performance assertion step appears in this workflow.

### Library/Framework Requirements

No new Rust crates. All externally-used tooling is GitHub Actions and the apt registry. Specifically:

| Tool | Version pin | Reason for pin |
|---|---|---|
| `actions/checkout` | `@v4` | Template pin; still supported April 2026; safer than v6 churn |
| `dtolnay/rust-toolchain` | `@master` + `toolchain: stable` (or `"1.89"`) | Official recommendation when toolchain is specified via `with:` |
| `Swatinem/rust-cache` | `@v2` | Latest major (2.9.1 as of March 2026); Bevy-aware; handles Cargo.lock correctly |
| `ubuntu-latest` | floating → Ubuntu 24.04 | Acceptable floating pin; matches production Ubuntu LTS |
| `windows-latest` | floating → Windows Server 2022 | Acceptable floating pin |
| `macos-latest` | floating → macOS 14+ arm64 | Acceptable floating pin; arm64 is guaranteed as of April 2026 |

[Source: Research Report §3, §4]

### File Structure Requirements

Files added/modified by this story, all paths relative to project root:

| Path | Add/Modify | Purpose |
|---|---|---|
| `.github/workflows/ci.yml` | Add (new directory + new file) | The CI workflow. ~90 lines YAML. |
| `.gitattributes` | Add (new) | Force LF line endings cross-platform. 1–2 lines. |

Files explicitly **not** touched by this story:

- `Cargo.toml`, `Cargo.lock` — Story 1.1's artifacts, immutable here.
- `rust-toolchain.toml` — Story 1.3's artifact. `targets = [...]` is intentionally omitted (documented in `ci.yml` header comment).
- `rustfmt.toml`, `clippy.toml`, `.gitignore` — Story 1.3's artifacts.
- `src/main.rs` — still cargo-default; Story 1.5 writes the first `App::new()`.
- `docs/plugin-compatibility.md` — Story 1.2's artifact. Not updated here (no plugin version change).
- Any `src/<module>/` — none exist; first gameplay module is Epic 2.

### Testing Requirements

- No `#[test]`s are added. `cargo test` will compile and run zero tests on all three OS legs; exit 0 is the success signal.
- The story's real validation is **the CI run itself**: four checks green on the introducing commit (AC #9).
- Full-build-output rule still applies. [Source: MEMORY.md → feedback_full_build_output.md] On the first CI run, the dev agent opens the Actions logs for each leg, greps the "cargo build" + "cargo test" + "cargo clippy" step outputs for `warning:|error:`, and confirms:
  - Zero `error:` hits on any leg.
  - Exactly one `warning:` hit on the `cargo check` / `cargo build` logs (the known `cfg(debug_assertions)` manifest warning — same message as in Story 1.3's logs).
  - Zero `warning:` hits on `cargo clippy` (because `-D warnings` promotes clippy warnings to errors).
  - Zero `warning:` hits on `cargo fmt` (fmt prints nothing on clean input).
- Bevy-integration tests remain deferred post-M3. [Source: architecture.md:144-146]

### Latest Technical Information

- **GitHub Actions runner images (April 2026):**
  - `ubuntu-latest` → Ubuntu 24.04 LTS (Noble). Migration completed late 2024. `libasound2-dev` resolves correctly via `libasound2t64` dep chain.
  - `macos-latest` → macOS 14 / arm64 (Apple Silicon) as default. Intel legacy only via `macos-13` or `macos-latest-large`.
  - `windows-latest` → Windows Server 2022; MSVC toolchain preinstalled.
- **Rust toolchain 1.94.1** (pinned in `rust-toolchain.toml`) ships `rustfmt 1.8.0` and `clippy 0.1.94`. All stable options used in this project's `rustfmt.toml`/`clippy.toml` are supported. [Source: Story 1.3 Dev Notes]
- **Rust 1.89** (MSRV target) is Bevy 0.18's declared minimum. Confirmed by `bevy-0.18` crate metadata on crates.io.
- **GitHub Actions YAML spec:** `fail-fast: false` must be inside `strategy:`; newcomers sometimes place it at `strategy.matrix.fail-fast`. The dev agent pays attention to indentation — the skeleton above is correct.

### Previous Story Intelligence

**From Story 1.3's Review Findings (inherited here):**
- `[Defer] MSRV 1.89 not CI-exercised` → AC #6 / Task 2 resolves this. [Source: deferred-work.md:14; 1-3-…md:86]
- `[Defer] rust-toolchain.toml lacks targets field` → AC #8 / Task 4 resolves via scope confirmation (no amendment needed; native runners only). [Source: deferred-work.md:16; 1-3-…md:88]
- `[Defer] asteroids3D typo in BMad artifacts` → NOT this story's scope; flagged for own chore story. [Source: deferred-work.md:15; 1-3-…md:87]

**From Story 1.1's Known Issues (still live):**
- `cfg(debug_assertions)` manifest warning — remains Story 1.5's fix. CI surfaces it in logs but does not fail on it. [Source: deferred-work.md:18-28]

**From Story 1.2's precedents:**
- Gate-artifact pattern: this story's analogue of `docs/plugin-compatibility.md` is the first green CI run URL recorded in the Dev Agent Record. The URL becomes the source-of-truth that FR47 is verified.

**Commit-type convention observed in git log:**
- `chore:`, `docs:`, `fix:`, `planning:`, `bmad:`. `ci:` is the natural prefix for Story 1.4's source-artifact commit — aligns with Conventional Commits, disambiguates from `chore:` (config) and `docs:` (content).

### Git Intelligence

Recent commits (newest first, 8 total on `master`):

| SHA | Subject | Relevance to 1.4 |
|---|---|---|
| `f8f067c` | `bmad: story 1.3 complete — toolchain, lint, format configuration` | BMad bookkeeping precedent for follow-up commit. |
| `2491785` | `chore: toolchain, lint, and format configuration (Story 1.3)` | Source-artifact commit introducing `rust-toolchain.toml` / `rustfmt.toml` / `clippy.toml` + MSRV declaration. Input substrate for this story's CI. |
| `48cedcd` | `bmad: story 1.2 complete — plugin compatibility gate passed` | BMad bookkeeping precedent. |
| `23ab9ec` | `docs: add plugin compatibility verification gate (Story 1.2)` | `docs/plugin-compatibility.md` lives at the path this story references in Dev Notes. |
| `0cbe8a3` | `docs: log review correction for cfg(debug_assertions) finding` | Context for the `cfg(debug_assertions)` warning CI will surface. |
| `113eebe` | `fix: correct package name typo asteroids3D -> asteroids3D` | **Critical context:** package is `asteroids3D`, NOT the `asteroids3D` in planning docs. Do not re-rename. |
| `abe7742` | `planning: import BMad artifacts` | Irrelevant to 1.4. |
| `4ca3869` | `chore: bootstrap Cargo project (Story 1.1)` | Baseline Cargo.toml + src/main.rs. |

**Nothing has been pushed to `origin/master` yet as of story-creation time** — `git log origin/master..HEAD` would show all 8 commits unpushed (assuming the dev agent hasn't pushed between now and their run). The first push after landing 1.4's CI commit will trigger CI on the HEAD commit only (GitHub Actions runs per push event, not per commit in a push). All prior commits retroactively miss CI — acceptable, since they predate the workflow.

### What this story explicitly does NOT fix

Enumerated here so the review step does not flag them as "missed":

1. The `cfg(debug_assertions)` semantic bug — **Story 1.5** owns it. CI surfaces the warning in logs.
2. Intel-macOS CI leg — **Story 7-6** (macOS universal binary) owns it.
3. `release.yml` / per-OS ZIP packaging / Itch.io butler upload — **Story 4.10** (Epic 4) owns it.
4. macOS code-signing / notarization — waived stretch per `project_fr48_deferred.md`.
5. Performance (60 FPS) CI assertion — explicitly not CI-enforceable per architecture.md:889.
6. `asteroids3D` typo in planning docs / `sprint-status.yaml` — dedicated chore story. [Source: deferred-work.md:15]
7. `[profile.dev.build-override]` — re-deferred to M4 upgrade window. [Source: deferred-work.md:8]
8. Bevy-integration tests — deferred post-M3.

### Project Structure Notes

- **`.github/` is a new top-level directory.** First file created at `.github/workflows/ci.yml`. The `workflows/` subdirectory is mandatory for GitHub Actions; `.github/ci.yml` is ignored by the Actions scheduler.
- **`.gitattributes` lives at project root**, not under `.github/`. It applies to the whole working tree.
- **The existing `.gitignore` does NOT need an `.github/` entry** — we want `.github/` tracked. This is correct as-is.
- **Package name is `asteroids3D`** (from `Cargo.toml:2`). Any future CI step that references the binary name uses `asteroids3D`, not `asteroids3D`. No such reference in this story's skeleton.
- **Remote URL** is `https://github.com/till-fechteler/asteroids3D.git`. Actions UI URL is `https://github.com/till-fechteler/asteroids3D/actions`.

### References

- [Source: _bmad-output/planning-artifacts/epics/epic-1-foundation-plugin-compatibility-gate.md#Story-1.4 (lines 78-99)]
- [Source: _bmad-output/planning-artifacts/architecture.md#CI-Matrix (lines 159-163)]
- [Source: _bmad-output/planning-artifacts/architecture.md#Architectural-Decisions-Provided-by-Starter-Choice (lines 121-163)]
- [Source: _bmad-output/planning-artifacts/architecture.md#Complete-Project-Directory-Structure (lines 534-546)]
- [Source: _bmad-output/planning-artifacts/architecture.md#Technical-Risks-Mitigations (line 889: CI performance-enforcement limitation)]
- [Source: _bmad-output/planning-artifacts/prd.md#Build-CI (line 406)]
- [Source: _bmad-output/planning-artifacts/prd.md#FR47-FR48 (lines 564-565)]
- [Source: _bmad-output/planning-artifacts/prd.md#Tech-Risk-4-macOS-cross-platform-parity (line 443)]
- [Source: _bmad-output/implementation-artifacts/1-1-bootstrap-cargo-project-with-hand-authored-cargo-toml.md (Known Issues: cfg(debug_assertions) warning expectation)]
- [Source: _bmad-output/implementation-artifacts/1-2-plugin-compatibility-verification-gate.md (Commit Convention, single-line subject pattern)]
- [Source: _bmad-output/implementation-artifacts/1-3-toolchain-lint-and-format-configuration.md#Review-Findings (lines 86-88 — the three defers inherited here)]
- [Source: _bmad-output/implementation-artifacts/deferred-work.md (lines 14-16 — MSRV, targets, typo defers)]
- [Source: NiklasEi/bevy_game_template `ci.yml` (template reference): https://github.com/NiklasEi/bevy_game_template/blob/main/.github/workflows/ci.yml]
- [Source: Bevy Linux dependencies: https://github.com/bevyengine/bevy/blob/main/docs/linux_dependencies.md]

## Dev Agent Record

### Agent Model Used

claude-opus-4-7 (Claude Code) — 1M-context configuration.

### Debug Log References

Tasks T1-T5, T7 executed on Till's local macOS (26.4.1 / arm64). Toolchain `1.94.1-aarch64-apple-darwin` active via `rust-toolchain.toml` (Story 1.3 artifact).

| Command | Exit | Notes |
|---|---|---|
| `mkdir -p .github/workflows` | 0 | First creation of `.github/`. |
| `ruby -ryaml -e "YAML.safe_load(...)"` on `ci.yml` | 0 | Output: `YAML OK`. Ruby `psych` preinstalled on macOS 26; chose over `python3 -c "import yaml"` which is unavailable (no pyyaml). |
| `grep -nP '\t' .github/workflows/ci.yml` | 1 | Exit 1 = no tab characters in the YAML. Indentation is spaces throughout. |
| `git add --renormalize .` (post-`.gitattributes`) | 0 | No-op locally — all files already LF-terminated on macOS. Becomes load-bearing on first Windows checkout. |
| `git status --short` | 0 | Four entries, all expected: `M sprint-status.yaml`, `?? .gitattributes`, `?? .github/`, `?? 1-4-…md`. |
| `git diff Cargo.toml Cargo.lock rust-toolchain.toml rustfmt.toml clippy.toml .gitignore src/main.rs` | 0 | Empty diff — scope guardrails honored. |
| `git rev-parse HEAD origin/master` | 0 | Both point at `f8f067c` — local and origin are synced. Next push carries only the Story 1.4 commit. |
| `rustup toolchain list` | 0 | Local has `stable` and `1.94.1` installed; `1.89` is NOT installed locally. MSRV-leg validation deferred to CI (first run). |

**CI run URL (AC #9): https://github.com/till-fechteler/asteroids3D/actions/runs/24824401702** — triggered by source-artifact commit `73dc4e6`, started `2026-04-23T08:11:08Z`, finished `2026-04-23T09:22:45Z`.

Per-leg outcomes (all ✅ green):

| Job | Runner | Duration | All 4 cargo steps |
|---|---|---|---|
| `build (ubuntu-latest)` | `ubuntu-latest` / x86_64-unknown-linux-gnu | 35m56s | ✅ build, ✅ test, ✅ clippy, ✅ fmt --check |
| `build (windows-latest)` | `windows-latest` / x86_64-pc-windows-msvc | 71m31s | ✅ build, ✅ test, ✅ clippy, ✅ fmt --check |
| `build (macos-latest)` | `macos-latest` / aarch64-apple-darwin (Apple Silicon) | 19m37s | ✅ build, ✅ test, ✅ clippy, ✅ fmt --check |
| `msrv-check (rust 1.89, ubuntu-latest)` | `ubuntu-latest` / x86_64-unknown-linux-gnu | 6m4s | ✅ cargo check (Rust 1.89) |

**Warning grep on full CI logs** (per MEMORY.md feedback "Verify build output fully"):
- `error:` hits: **0** across all 4 jobs + all steps.
- `warning:` hits (cargo-manifest / rustc / clippy): **3 occurrences of the known `cfg(debug_assertions)` cargo-manifest warning** — one each on Ubuntu `cargo build`, Windows `cargo build`, and Windows `cargo test` step startups (macOS variants suppressed by ANSI coloring filter but confirmed present). Exact text: `warning: Found 'debug_assertions' in 'target.'cfg(...)'.dependencies'. This value is not supported for selecting dependencies and will not work as expected.` This is the deferred finding from `deferred-work.md:18-28` — Story 1.5 owns the fix.
- Zero clippy warnings (CI would have failed with `-D warnings` otherwise).
- Zero `cargo fmt --check` complaints (would have failed otherwise).
- One non-blocking Node.js 20 deprecation annotation emitted by GitHub Actions on `actions/checkout@v4` — informational, Node 20 support runs through June 2, 2026. Not a story-1.4 issue; covered under a post-MVP CI-hygiene story when the deprecation-effective date approaches.

**FR47 baseline verified for commit `73dc4e6`:** the binary compiles, tests pass, clippy is clean, and format is consistent on Windows 10+ (MSVC), Linux (Ubuntu 24.04 LTS), and macOS 14+ (Apple Silicon). The architecture's "CI matrix from M0" risk-mitigation anchor (architecture.md:34) is now a real, running safety net — every subsequent story ships with continuous parity evidence.

### Completion Notes List

**Status: ✅ Story 1.4 complete.** All 10 ACs satisfied, all 7 tasks checked, CI green on Windows + Linux + macOS + MSRV leg for the introducing commit `73dc4e6`.

**Deliverables authored (2 source files):**

| File | Action | Lines | Purpose |
|---|---|---|---|
| `.github/workflows/ci.yml` | Added (new directory + file) | 96 | 3-OS build matrix (`windows-latest`, `ubuntu-latest`, `macos-latest`) + `msrv-check` job on Rust 1.89. Triggers on push (any branch) + pull_request. `Swatinem/rust-cache@v2`, `dtolnay/rust-toolchain@master`, `actions/checkout@v4`. |
| `.gitattributes` | Added (new) | 2 | `* text=auto eol=lf` — forces LF line endings on all platforms, prevents `cargo fmt --check` regression on Windows. |

**No source-code changes.** `src/`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`, `.gitignore`, `docs/plugin-compatibility.md` all untouched — verified via empty `git diff`.

**Inherited defers from Story 1.3 resolved in ci.yml:**
- `deferred-work.md:14` (MSRV not CI-exercised) → resolved by the `msrv-check` job running `cargo check` on Rust 1.89 with `RUSTUP_TOOLCHAIN=1.89` env override.
- `deferred-work.md:16` (rust-toolchain.toml `targets` field) → resolved by header comment in `ci.yml` documenting the native-targets-only invariant; no amendment to `rust-toolchain.toml`.
- `deferred-work.md:15` (asteroids3D typo) → explicitly OUT of scope; still deferred to a dedicated chore story.

**Local validation gates passed:**
- `ci.yml` parses as valid YAML (ruby/psych).
- No tab indentation in `ci.yml`.
- `git status` clean of unexpected entries.
- Scope guardrails honored (no Cargo.* / toolchain / src changes).

**MSRV-job validation.** Rust 1.89 was not installed on the local macOS machine. MSRV validation happened on CI's first run — the `msrv-check (rust 1.89, ubuntu-latest)` leg completed in 6m4s with `cargo check` exit 0. Bevy 0.18 + avian3d 0.6 + all pinned plugins compile cleanly on Rust 1.89. The MSRV declaration in `Cargo.toml:5` (`rust-version = "1.89"`) is now CI-verified.

**Commit history for Story 1.4:**

1. **Source-artifact commit `73dc4e6`** — `ci: three-platform GitHub Actions matrix (Story 1.4)`. Pushed to `origin/master` 2026-04-23T08:11:00Z. **Deviation from plan:** sprint-status.yaml also landed in this commit because `git add --renormalize .` (Task 3) had previously staged it. Minor — the sprint-status bump (1-4 ready-for-dev → in-progress) was supposed to be in the BMad bookkeeping commit per the 1.1/1.2/1.3 split pattern. No harm to the contents; just a slightly blurred split. Noted for the retrospective.
2. **BMad bookkeeping commit** — this Dev Agent Record + Change Log + sprint-status.yaml flip to `review`. Title: `bmad: story 1.4 complete — three-platform CI matrix green`.

**Follow-up work surfaced (NOT story-1.4 scope, for the retrospective or next story):**

1. **Node.js 20 deprecation on `actions/checkout@v4`** — GitHub Actions annotation surfaced on all 4 jobs. Non-blocking through June 2, 2026. Upgrade path: bump `actions/checkout` to a Node 24-compatible major when released, or set `FORCE_JAVASCRIPT_ACTIONS_TO_NODE24=true` env to opt in now. Candidate CI-hygiene story around M4 upgrade window.
2. **Windows build took 71m (cold cache)** — story estimated 5-8 min. Calibration datum for future story planning. Subsequent runs should hit `Swatinem/rust-cache@v2` and drop to ≤ 10 min; if they stay long, cache keying may need tuning.

**Scope guardrails: all honored.** No edits to `src/`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`, `.gitignore`, `docs/plugin-compatibility.md`. Only new files: `.github/workflows/ci.yml` + `.gitattributes`.

### File List

All paths relative to project root.

**Added (new directory):**
- `.github/` — first file is `workflows/ci.yml`.

**Added (source artifacts — landed in commit `73dc4e6` / `ci: three-platform GitHub Actions matrix (Story 1.4)`):**
- `.github/workflows/ci.yml` — the CI workflow (96 lines).
- `.gitattributes` — LF-line-endings enforcement (2 lines).

**Modified (source artifact, accidentally bundled into `73dc4e6` — see Completion Notes "Commit history" deviation):**
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — `development_status[1-4-…]` was set to `in-progress` + `last_updated` comment updated.

**Modified (BMad bookkeeping — part of the `bmad:` follow-up commit):**
- `_bmad-output/implementation-artifacts/1-4-three-platform-ci-matrix.md` — this file: all 7 tasks checked, Dev Agent Record populated with CI run URL + per-leg durations, status flipped to `review`.
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — `development_status[1-4-…]` flipped `in-progress` → `review`; `last_updated` comment updated.

**Unchanged (verified via `git diff`):**
- `src/main.rs`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`, `.gitignore`, `docs/plugin-compatibility.md` — scope guardrails honored.

## Change Log

| Date | Author | Change |
|---|---|---|
| 2026-04-23 | claude-opus-4-7 (create-story) | Story 1.4 drafted. Scope: `.github/workflows/ci.yml` (3-OS matrix + MSRV-check job) + `.gitattributes` for CRLF prevention. Inherits three defers from Story 1.3 review: MSRV-check job added (AC #6), native-targets-only invariant documented in workflow header (AC #8), asteroids3D typo NOT in scope. Zero source-code changes; CI consumes 1.1–1.3 artifacts as-is. Status: ready-for-dev. |
| 2026-04-23 | claude-opus-4-7 (dev-story, checkpoint) | Tasks T1-T5, T7 complete locally. Authored `.github/workflows/ci.yml` (96 lines, YAML-validated) + `.gitattributes` (2 lines, renormalize no-op on macOS). All scope guardrails verified clean via `git diff`. Paused for Till's manual source-artifact commit per 1.1/1.2/1.3 precedent. T6 (push + observe CI + record run URL) pending. Status: ready-for-dev → in-progress. |
| 2026-04-23 | claude-opus-4-7 (dev-story, T6 + finalize) | Commit `73dc4e6` pushed to `origin/master` on Till's authorization. CI run `24824401702` observed via `gh run watch`: all 4 jobs ✅ green — build (ubuntu 35m56s, windows 71m31s, macos 19m37s) + msrv-check (6m4s). FR47 baseline verified for `73dc4e6`. Zero rustc errors; 3 occurrences of the known `cfg(debug_assertions)` cargo-manifest warning (deferred-work.md:18-28, owned by Story 1.5). Node.js 20 deprecation annotation on `actions/checkout@v4` captured as follow-up. Deviation noted: sprint-status.yaml bundled into source-artifact commit (should have been in bookkeeping commit). Status: in-progress → review. |
| 2026-04-23 | claude-opus-4-7 (code-review) | 3-layer adversarial review (Blind Hunter 14 + Edge Case Hunter 10 + Acceptance Auditor verdict). Acceptance Auditor: **Approve** (10/10 ACs PASS). Triage: 0 Decision-Needed, 3 Patch, 10 Defer, 10 Dismissed. Patches applied via commit `3f3d5f2` (`ci: add timeout + DEBIAN_FRONTEND + --locked`): `timeout-minutes` on both jobs (120/60), `DEBIAN_FRONTEND=noninteractive` on apt-steps, `--locked` on all dep-resolving cargo invocations. CI run `24829706852` green on all 4 legs in under 4 min (warm cache — Windows 3m25s, down from 71m cold). 10 defers appended to `deferred-work.md` under "Deferred from: code review of 1-4-… (2026-04-23)". Status: review → done. |
