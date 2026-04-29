# Story 2.6: Go/Fallback Decision Document

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As the project author,
I want a committed M1 go/fallback decision authored from the Story 2.5 parity-report evidence and saved at `docs/tech-spike/m1-decision.md`,
So that M1 closes with explicit, auditable scope resolution for M2 — `ToonMaterial` + `bevy_mod_outline` are either ratified as the production stack (Story 2.7 skipped) or rejected in favor of flat-shaded + rim-light (Story 2.7 unblocked).

## Acceptance Criteria

1. **Given** the parity evidence at `docs/tech-spike/m1-backends/parity-report.md` (which closes with `RECOMMEND GO toon`)
   **When** `docs/tech-spike/m1-decision.md` is authored
   **Then** the document contains exactly five top-level `##` sections, in this order: `Decision`, `Rationale`, `Risks Accepted`, `Fallback Trigger Criteria`, `M2 Impact`
   **And** the `Decision` section opens with a single line that is exactly one of these three verbatim strings (no surrounding quotes, no trailing punctuation): `GO toon`, `GO toon with scope reduction`, `FALLBACK flat+rim-light`
   **And** the document includes a `Date:` and `Decision Owner:` header line at the top, plus an explicit hyperlink to `parity-report.md` as the source evidence

2. **Given** the parity-report recommendation is `RECOMMEND GO toon` (verified by reading the recommendation section verbatim before authoring)
   **When** the `Decision` line is written
   **Then** the chosen value is `GO toon` (the unconditional variant — matching the recommendation, not the scope-reduced variant)
   **And** the `Rationale` section cites the parity report's six qualitative-equivalence checks (banding count, rim-light, tint colors, outline continuity, outline width, swatch palette) and the three RMSE_normalized values (0.000423 / 0.006960 / 0.006955) with their AC #4 threshold (0.05) margin
   **And** the `Rationale` section explicitly addresses PRD risk R#2 ("WGSL shader complexity for a beginner on three graphics backends") and states that the M1 spike resolves it
   **And** any deviation from the parity report's recommendation (i.e. choosing a different decision than the recommendation) carries a documented justification paragraph — for this dispatch the recommendation is accepted, so this clause is satisfied vacuously

3. **Given** the chosen decision is `GO toon`
   **When** the consequences are bookkept
   **Then** `_bmad-output/implementation-artifacts/sprint-status.yaml` flips `2-7-fallback-material-scaffold-conditional-on-story-2-6` from `backlog` to `not-needed` (a new status value introduced by this story)
   **And** the `# Story Status:` comment block in `sprint-status.yaml` documents the new status: `not-needed: Story is conditionally specified in epics and the trigger condition was not met; no implementation work required`
   **And** `_bmad-output/implementation-artifacts/sprint-status.yaml`'s `2-6-go-fallback-decision-document` entry flips `backlog → ready-for-dev → in-progress → review` over the course of this story (final state at hand-off: `review`)
   **And** the `last_updated` field is bumped to today's date with a one-line annotation matching the precedent of prior stories' updates

4. **Given** the post-2.5 source baseline
   **When** local verification runs at story end
   **Then** `cargo check`, `cargo build`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo build --release` all produce **0** lines matching `grep -cE 'warning:|error:'` (modulo the documented `Free disk space`-action `set -x` ambient noise that does not occur on local runs)
   **And** `cargo test` reports exactly **14 passed, 0 failed** — unchanged from post-2.5 (this story adds no source code, no new tests, removes no tests)
   **And** `git diff --stat src/` returns `0 files changed` (this story is documentation-only — no `src/` mutation is permitted)
   **And** `Cargo.toml`, `Cargo.lock`, `.github/workflows/*.yml`, `assets/**`, and all `src/**` files are byte-identical to the post-2.5 state

5. **Given** Story 2.7's status is now `not-needed`
   **When** Epic 2 closure is recorded
   **Then** `sprint-status.yaml`'s `epic-2` entry flips from `in-progress` to `done` — Story 2.6 is the closing story of Epic 2, and with 2.7 skipped, all 7 stories are resolved (5 done + 1 review→done [this story] + 1 not-needed)
   **And** `epic-2-retrospective` remains `optional` (no change — retrospective is opt-in, not gated by epic closure)
   **And** the `deferred-work.md` file gains exactly one new entry under a new `## Deferred from: 2-6-go-fallback-decision-document (YYYY-MM-DD)` heading documenting the M2-impact carryovers (toon material confirmed; outline integration confirmed; capture-mode cleanup at Story 3.1 — already deferred from 2.5, cross-link for traceability)

## Tasks / Subtasks

- [x] **Task 1: Read the parity-report recommendation verbatim and confirm the decision** (AC: #2)
  - [x] Read `docs/tech-spike/m1-backends/parity-report.md` end-to-end. Locate the `## Recommendation for Story 2.6` section.
  - [x] Verify the recommendation line is exactly: `> **RECOMMEND GO toon**` (with the surrounding blockquote markdown and bold). If the line reads `RECOMMEND GO toon with scope reduction` or `RECOMMEND FALLBACK flat+rim-light`, **HALT** — the dev session was opened on a different evidentiary baseline than this story's author expected, and the story spec assumes `RECOMMEND GO toon`. Re-open this story with corrected ACs in that case.
  - [x] Capture the exact text of the recommendation's justification paragraph for verbatim reuse in `m1-decision.md`'s Rationale section. Direct quoting (with attribution) is preferable to paraphrase — the parity report IS the evidence; the decision document IS the ratification.
  - [x] **Independent sanity-check (the dev's own read, not the report's claim):** open the three PNGs at `docs/tech-spike/m1-backends/{metal,vulkan,dx12}.png` in any image viewer and confirm by eye that all six qualitative checks the report enumerates actually hold. The dev is the M1 decision owner; the parity report is evidence, not a sign-off oracle. If the dev's own visual inspection disagrees with the report on any of the six checks, document the disagreement and either (a) override to `GO toon with scope reduction` or `FALLBACK flat+rim-light` with explicit rationale, or (b) re-run the parity capture with `gh workflow run parity-capture.yml --ref master` and update the report. **Expected outcome on a clean run:** dev confirms all six checks; decision stays `GO toon`.

- [x] **Task 2: Author `docs/tech-spike/m1-decision.md`** (AC: #1, #2)
  - [x] Create new file `docs/tech-spike/m1-decision.md`. Path is mandated by the epic spec at `epics/epic-2-vector-aesthetic-tech-spike.md:158`. **Do NOT** put it in `docs/tech-spike/m1-backends/` — `m1-decision.md` is the milestone-gate decision, parallel to `m1-backends/parity-report.md` (the evidence) and the per-spike sub-folders (`m1-palette/`, `m1-outline/`, `m1-toon/`). The epic spec's path is `docs/tech-spike/m1-decision.md`, top level of `tech-spike/`.
  - [x] Document body — strict structural template:
    ```markdown
    # M1 Vector Aesthetic Tech-Spike — Go/Fallback Decision

    **Date:** <YYYY-MM-DD of dev-story execution>
    **Decision Owner:** Till Fechteler (project author)
    **Source evidence:** [docs/tech-spike/m1-backends/parity-report.md](./m1-backends/parity-report.md)
    **Stories closing:** Story 2.6 (this document); Story 2.7 marked `not-needed`.
    **Milestone:** M1 — Vector Aesthetic Tech Spike (closing).

    ## Decision

    GO toon

    ## Rationale

    *(2–4 paragraphs covering: parity report's six qualitative checks all pass; quantitative
    RMSE values 7×–118× under AC #4 threshold; PRD R#2 risk resolved; toon material is the
    primary M1 learning artifact and the parity gate confirms it ships portfolio-quality
    on all three backends. Cite parity-report.md inline.)*

    ## Risks Accepted

    *(Bulleted list of residual risks the GO toon decision accepts, each with a one-line
    mitigation or "no mitigation; accepted as-is".)*

    ## Fallback Trigger Criteria

    *(Numbered list of empirically observable conditions that would re-open the GO/FALLBACK
    question post-M1, requiring this decision to be revisited or reversed. These are NOT
    pre-emptive escape hatches — they are tripwires. If hit, a new decision document at
    `docs/tech-spike/m1-decision-revisit-<date>.md` overrides this one.)*

    ## M2 Impact

    *(Concrete consequences for the M2 plan: which files become production code, which
    files become deletion candidates at Story 3.1, what Story 2.7 means now, what
    follow-up cleanups are tracked in deferred-work.md.)*
    ```
  - [x] **Why a top-level `docs/tech-spike/m1-decision.md` instead of nesting under `m1-backends/`:** The epic spec mandates this path. Conceptually, the decision document is the milestone-gate artifact, not a sub-spike artifact — it sits at the same level as the four sub-spike folders (`m1-palette/`, `m1-toon/`, `m1-outline/`, `m1-backends/`) which each gathered their own evidence. The decision integrates evidence from all four. The path layout reflects this hierarchy.
  - [x] **Section content guidance — `Decision`:** First line of the section body, no preamble, no prefix: literally `GO toon`. The verbatim string is the AC #1 anchor — automated readers (this dev agent in future stories, retrospective tooling, BMAD plan parsers) will grep for this exact line. **Do NOT** wrap in code fence, blockquote, or backticks. **Do NOT** append `.` or any other punctuation. Format example:
    ```markdown
    ## Decision

    GO toon
    ```
    Nothing else in this section. Two newlines and on to `## Rationale`.
  - [x] **Section content guidance — `Rationale`:** 3 paragraphs (target 200–400 words total):
    1. **Parity-evidence summary.** Reference the six qualitative-equivalence checks from the parity report (banding count, rim-light at silhouette, per-entity tint colors, outline continuity, outline width proportion, swatch palette colors). State that all six pass on Metal (hardware), Vulkan (Mesa lavapipe software), and DX12 (WARP software). Cite the three RMSE_normalized values (Metal↔Vulkan: 0.000423 = 118× under threshold; Metal↔DX12: 0.006960 = 7× under; Vulkan↔DX12: 0.006955 = 7× under). Threshold is AC #4 (Story 2.5) at 0.05.
    2. **Risk-resolution paragraph.** Explicitly address PRD R#2 ("WGSL shader complexity for a beginner on three graphics backends, Metal/Vulkan/DX12") at `prd.md:441`. State that the M1 tech-spike's three-backend validation gate (Story 2.5) resolves this risk: the custom WGSL toon material (Story 2.3) and `bevy_mod_outline` integration (Story 2.4) translate correctly through Naga to SPIR-V (Vulkan) and HLSL/DXIL (DX12) with no observed regressions vs hardware Metal as the reference. Cross-reference `prd.md:347` (M#10 fallback condition resolved) and `architecture.md:295` (M1 — Vector Spike completion gate satisfied).
    3. **Decision-owner ratification paragraph.** State that Till (project author + decision owner) has independently inspected the three captured PNGs side-by-side and concurs with the parity report's recommendation. The toon material is confirmed as the M2 production shader (`assets/shaders/toon.wgsl` + `src/visual/toon_material.rs` graduate from M1-tech-spike status to M2-production status; reference scene + capture mode are M1-only and slated for removal at Story 3.1).
  - [x] **Section content guidance — `Risks Accepted`:** bulleted list of 4 residual risks, each one-line:
    1. Software-rasterizer-noise residual: ImageMagick RMSE on M↔V and V↔D pairs is dominated by sub-pixel AA jitter and software-vs-hardware rasterizer rounding (lavapipe and WARP are both software). Real-hardware Vulkan and DX12 testing has NOT happened — only software rasterizers were exercised in CI. Risk: a real-hardware GPU may exhibit a divergence the software path masked. Mitigation: deferred to first user-reported render artifact + reference-hardware playtest at Epic 10's Story 10.12 ("3-platform 60-FPS zero-crash playtest").
    2. WARP outline-thinning observation: DX12's WARP renders silhouette outlines approximately one pixel-row thinner than hardware Metal. Within visual tolerance per Story 2.5's qualitative check #5 (outline width proportion). Risk: a UI-/HUD-style ultra-fine line-weight choice in Epic 10 polish could cross the threshold where this delta becomes noticeable. Mitigation: HUD/UI uses `bevy_ui` (screen-space), not `bevy_mod_outline` — only world-space mesh silhouettes are affected.
    3. Bevy version-bump risk (M4 / M6): A future Bevy or Naga upgrade could regress WGSL→backend translation. Mitigation: the `parity-capture.yml` workflow stays in place through Story 2.5's review-pass-done, and any Bevy bump's PR description must note re-running the parity capture as a step. (Capture mode itself is removed at Story 3.1, but the workflow file removal is part of that same cleanup — re-introduction at the version-bump window is a 1-day chore if it's needed; the deferred-work.md entry tracks both directions.)
    4. Hardware coverage gap: Apple Silicon M5 Pro (capture host) ≠ the PRD's NFR-P1 baseline of "Apple M1." Metal capture used Till's actual development hardware; M1 baseline parity is implicitly assumed from the M5 Pro evidence (newer-and-faster hardware). Risk: Apple's GPU family generations sometimes have shader-precision quirks. Mitigation: deferred to Apple-Silicon-M1-class playtest in M3 / Story 4.10 readiness or M9 / Story 10.12 if no M1 hardware reaches Till before then.
  - [x] **Section content guidance — `Fallback Trigger Criteria`:** numbered list of 4 tripwire conditions; each ~2 sentences:
    1. **Cross-backend regression observed.** Any of the six qualitative-equivalence checks fails on a fresh `parity-capture.yml` dispatch (different RMSE_normalized values are NOT a trigger; missing rim-light, mismatched band count, swizzled tint channel, broken outline continuity, lost swatch color ARE).
    2. **Real-hardware GPU divergence.** When the parity report is re-run on real GPUs (e.g. Till's GTX 1060 / RX 580 reference hardware at Story 10.12), if the qualitative checks fail on hardware in a way the software-rasterizer path masked, this decision flips to `FALLBACK flat+rim-light` and Story 2.7 work is opened.
    3. **NFR-P1 60-FPS regression attributable to toon shader.** Profiling at Story 10.1 reveals the toon material is the dominant cost preventing 60 FPS at 1080p on the GTX 1060 / RX 580 / Apple M1 baseline. (Performance trigger, not correctness trigger.) Mitigation path is `GO toon with scope reduction` (drop rim-light, reduce step count) before flipping to `FALLBACK`.
    4. **Bevy / Naga version-bump regression.** A future M4 / M6 Bevy upgrade introduces WGSL→backend translation issues that the upstream maintainers do not fix within ~30 days. Pin the prior Bevy version OR flip to fallback if the prior pin is no longer viable for security / dependency-graph reasons.
  - [x] **Section content guidance — `M2 Impact`:** 4 bullet groups, each one-line claim + one-line consequence:
    1. **Production code:** `assets/shaders/toon.wgsl` + `src/visual/{toon_material.rs, outline.rs, palette.rs}` are confirmed M2-production code. They graduate from "M1-tech-spike artifact" to "permanent project code" status.
    2. **Story 2.7 disposition:** Marked `not-needed` (new sprint-status.yaml status value; see Task 3 below). No fallback material scaffold work; no `src/visual/flat_rim_material.rs`; no `[deprecated]` attribute on `ToonMaterial`. The `_fallback`-suffixed paths in `epics/epic-2-vector-aesthetic-tech-spike.md:191-194` will not exist.
    3. **Cleanup at Story 3.1:** `src/visual/capture.rs`, `pub mod capture;` line in `src/visual/mod.rs`, env-var lookup + conditional `WindowPlugin` + conditional `CapturePlugin` in `src/main.rs`, AND `.github/workflows/parity-capture.yml` are removed when the Arena state replaces the cfg-gated reference scene. Deferred-work entry already exists from Story 2.5 (deferred-work.md:99–102); cross-linked from this decision document.
    4. **Audit trail:** `docs/tech-spike/m1-backends/{metal,vulkan,dx12}.png` + `parity-report.md` + the three diff heatmaps + `m1-decision.md` (this file) STAY — they are the auditable evidence that the M1 gate was satisfied. They do not get cleaned up at Story 3.1.

- [x] **Task 3: Sprint-status.yaml updates (Story 2.7 → `not-needed`, Epic 2 → `done`)** (AC: #3, #5)
  - [x] Edit `_bmad-output/implementation-artifacts/sprint-status.yaml`:
    - Add the new status definition under the `# Story Status:` comment block (between the existing `done:` line and the blank line that precedes `# Story Status Transitions:` — match the precedent indentation of two-space `#   - <name>: <description>`):
      ```yaml
      #   - not-needed: Story is conditionally specified in epics and the trigger condition was not met; no implementation work required (e.g., 2.7 fallback skipped after 2.6's GO toon decision)
      ```
    - Flip `2-7-fallback-material-scaffold-conditional-on-story-2-6: backlog` → `2-7-fallback-material-scaffold-conditional-on-story-2-6: not-needed`.
    - Flip `epic-2: in-progress` → `epic-2: done`. Justification: with 2.6 review-pass-done and 2.7 not-needed, all 7 stories in Epic 2 are resolved (`done` ∪ `not-needed` ∪ `review`-which-becomes-done-via-code-review = full coverage).
    - Update `last_updated:` line to today's date with a one-line annotation: `last_updated: <YYYY-MM-DD> (Story 2.6 done — M1 GO toon ratified; Story 2.7 → not-needed; epic-2 → done)`.
  - [x] **Important: do NOT flip `2-6-go-fallback-decision-document` to `done` directly.** This story follows the same `backlog → ready-for-dev → in-progress → review → done` workflow as prior stories. The dev story flips `ready-for-dev → in-progress → review`; the code-review pass flips `review → done` (per `bmad-code-review` skill convention). At hand-off this story's status is `review`. Two ways to handle epic closure cleanly:
    - **Option A (preferred):** flip `epic-2 → done` in this story's bookkeeping commit (epic closure trails the LAST story's review-pass, but with 2.7 not-needed and only one story remaining at "review" status, the epic transition is unambiguous). Rationale: matches the precedent that epic-2 → in-progress was set at Story 2.1 ready-for-dev creation, before any code review — same forward-looking logic in reverse.
    - **Option B (defensive):** leave `epic-2: in-progress` and flip to `done` only in the next story's bookkeeping after 2.6's code review passes (i.e. the 3-1 story bookkeeping does the epic flip). Rationale: tighter coupling between status and lived state.
    - **Recommended: Option A.** The epic's full sweep (5 done + 2.6 review + 2.7 not-needed) leaves no work to do; flipping in the same commit preserves single-source-of-truth atomicity.
  - [x] **YAML formatting discipline:** preserve all comments and blank lines. Do NOT reorder keys. Do NOT add new top-level keys. The status comment block (`# STATUS DEFINITIONS:` etc.) is the canonical contract — adding `not-needed` to that block is the operationally-meaningful change; the per-story line flip is the metadata.
  - [x] **Verification:** after edits, run `python3 -c "import yaml; yaml.safe_load(open('_bmad-output/implementation-artifacts/sprint-status.yaml'))"` → exit 0 (file parses as valid YAML). If parse fails, the comment-block edit broke YAML syntax — re-do with stricter indentation check.

- [x] **Task 4: Update `deferred-work.md` with Story 2.6 closure entries** (AC: #5)
  - [x] Append to `_bmad-output/implementation-artifacts/deferred-work.md`:
    ```markdown
    ## Deferred from: 2-6-go-fallback-decision-document (<YYYY-MM-DD>)

    - **Story 2.7 fallback material scaffold — explicitly NOT NEEDED** — `epic-2-vector-aesthetic-tech-spike.md:172-194`. Story 2.6's GO toon decision skips Story 2.7. This is NOT a deferral — it's a definitive disposition. Sprint status: `not-needed`. Recorded here for cross-reference traceability so a future plan-sweep doesn't reflexively re-add Story 2.7 to backlog. **Resolution path:** if any of the four `Fallback Trigger Criteria` from `docs/tech-spike/m1-decision.md` later fire, open a new decision document `docs/tech-spike/m1-decision-revisit-<date>.md`, flip 2-7 from `not-needed` to `backlog`, and re-create-story.
    - **Cross-link to Story 2.5's Story 3.1 cleanup entry** — see deferred-work.md (Story 2.5 deferral block above): capture mode + parity-capture workflow removal is tracked there. The M1 decision document (this story) STAYS at `docs/tech-spike/m1-decision.md` — it is the audit trail for the M1 gate, NOT a tech-spike artifact slated for cleanup.
    - **Cross-link to PRD risk R#2 resolution** — `_bmad-output/planning-artifacts/prd.md:441`. WGSL-on-three-backends risk officially resolved as of M1 closure. Future PRD/architecture revisions may want to update R#2's status from "MITIGATED" to "RESOLVED" — flagged here so a future planning-sweep story (or M4 Bevy-bump readiness review) doesn't have to rediscover the resolution.
    ```
  - [x] Replace `<YYYY-MM-DD>` with today's date.
  - [x] **Why these entries and not more:** the GO toon decision generates very little new deferred work — it ratifies existing scaffolding rather than adding new code. The three entries above are cross-link traceability, not new technical debt. The Story 3.1 capture-mode cleanup is the one substantial follow-up item; it was already captured in Story 2.5's deferred-work block (entries at lines 99–108 of `deferred-work.md`). Cross-linking is enough.

- [x] **Task 5: Local verification sweep — confirm zero source impact** (AC: #4)
  - [x] **Source-impact pre-check.** Before opening any local terminal:
    ```bash
    git status --short
    ```
    Expected staged-or-unstaged set (after Tasks 2–4 complete): `docs/tech-spike/m1-decision.md` (??), `_bmad-output/implementation-artifacts/sprint-status.yaml` (M), `_bmad-output/implementation-artifacts/deferred-work.md` (M), `_bmad-output/implementation-artifacts/2-6-go-fallback-decision-document.md` (M, this file). **NO** entries under `src/`, `Cargo.toml`, `Cargo.lock`, `.github/`, `assets/`, or any other source-controlled location. If you see one of those: revert it before continuing.
  - [x] **`cargo check`:**
    ```bash
    cargo check 2>&1 | tee /tmp/story-2-6-check.log
    grep -cE 'warning:|error:' /tmp/story-2-6-check.log
    ```
    Expected: `0`. If non-zero: source did get touched — investigate; revert; re-run.
  - [x] **`cargo build`:**
    ```bash
    cargo build 2>&1 | tee /tmp/story-2-6-build.log
    grep -cE 'warning:|error:' /tmp/story-2-6-build.log
    ```
    Expected: `0`.
  - [x] **`cargo test`:**
    ```bash
    cargo test 2>&1 | tee /tmp/story-2-6-test.log
    grep -cE 'warning:|error:|FAILED' /tmp/story-2-6-test.log
    ```
    Expected: `0`. The summary line must read `test result: ok. 14 passed; 0 failed; 0 ignored` — exact match with post-2.5 state. Any drift = source got touched.
  - [x] **`cargo clippy --all-targets -- -D warnings`:**
    ```bash
    cargo clippy --all-targets -- -D warnings 2>&1 | tee /tmp/story-2-6-clippy.log
    grep -cE 'warning:|error:' /tmp/story-2-6-clippy.log
    ```
    Expected: `0`.
  - [x] **`cargo fmt --all -- --check`:**
    ```bash
    cargo fmt --all -- --check
    echo $?
    ```
    Expected exit code: `0`.
  - [x] **`cargo build --release`:**
    ```bash
    cargo build --release 2>&1 | tee /tmp/story-2-6-release.log
    grep -cE 'warning:|error:' /tmp/story-2-6-release.log
    ```
    Expected: `0`.
  - [x] **Why all six:** even though no source changes are intended, the verification sweep proves that's the case. A passing sweep is the load-bearing AC #4 evidence. **Don't skip these on a "but it's docs-only" basis** — Till's feedback memory `feedback_full_build_output` explicitly demands the full grep-for-`warning:|error:` discipline regardless of expected outcome.
  - [x] **`git diff --stat src/`:**
    ```bash
    git diff --stat src/
    ```
    Expected: empty output (no files listed). If any file appears: revert with `git checkout src/<file>`.
  - [x] **YAML parse check:**
    ```bash
    python3 -c "import yaml; yaml.safe_load(open('_bmad-output/implementation-artifacts/sprint-status.yaml')); print('OK')"
    ```
    Expected: `OK`. If parse error: re-edit `sprint-status.yaml` carefully.
  - [x] **No-runtime-smoke for this story.** Unlike Story 2.5, there is NO `cargo run` invocation for this story — there is no runtime behavior to verify. The decision document is consumed by humans (and by the next-story create-story workflow, which is a static-text reader). Skipping the runtime smoke is correct.

- [x] **Task 6: Scope guardrails — verify nothing else drifted** (AC: #4)
  - [x] `git status --short` final inspection. Expected file set:
    - `docs/tech-spike/m1-decision.md` (??) — new, Task 2.
    - `_bmad-output/implementation-artifacts/sprint-status.yaml` (M) — Task 3.
    - `_bmad-output/implementation-artifacts/deferred-work.md` (M) — Task 4.
    - `_bmad-output/implementation-artifacts/2-6-go-fallback-decision-document.md` (M) — this file, status flips + Dev Agent Record population at Task 7.
    - **NO** `src/**` files. **NO** `Cargo.{toml,lock}`. **NO** `.github/`. **NO** `assets/`. **NO** other `docs/` paths (the `m1-backends/` evidence is already committed by Story 2.5 and untouched here).
  - [x] `grep -nrE 'GO toon|FALLBACK flat\+rim-light|m1-decision' src/ --include='*.rs'` → expected: **0 hits** (decision document is docs-only, source code does not reference it).
  - [x] `grep -nrE 'CapturePlugin|ASTEROIDS3D_CAPTURE_PNG' src/ --include='*.rs'` → expected: **same 5–10 hits as post-2.5** (no change). Capture mode is M1-only and stays in place through Story 3.1's cleanup; this story does not remove it.
  - [x] **Files NOT touched (and must NOT be touched by this story):** all `src/**`, `Cargo.toml`, `Cargo.lock`, `.gitignore`, `.gitattributes`, `.github/workflows/{ci,parity-capture}.yml`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`, `assets/**`, `docs/tech-spike/m1-backends/**` (Story 2.5 audit-trail evidence — read-only from here), `docs/tech-spike/m1-{palette,toon,outline}/**`, `_bmad-output/planning-artifacts/**` (PRD / architecture / epics — out of scope; if a planning-sweep story later updates R#2's status to RESOLVED, that's a separate dedicated story).

- [x] **Task 7: Ready-for-review handoff + bookkeeping commit**
  - [x] Populate this file's **Dev Agent Record**: Agent Model Used, Debug Log References (per-command grep counts + log paths + YAML parse OK), Completion Notes (per-AC evidence + any deviations from spec), File List (added / modified — short list this time).
  - [x] Set this story's `Status:` header → `review`.
  - [x] **Commit 1 (decision document — triggers CI):** stage `docs/tech-spike/m1-decision.md`. **NO** other files in this commit.
    - HEREDOC commit message subject: `docs: M1 GO toon decision (Story 2.6)`. Single-line, under 70 chars. Match Till's precedent: `docs: M1 three-backend parity evidence + GO toon recommendation (Story 2.5)` (commit `66c41fd`).
    - Push to `origin/master`. Triggers full 4-job `ci.yml` matrix because `docs/**` is NOT in the workflow's `paths-ignore` list. **Expected CI outcome:** all 4 jobs ✓ (cache warm from Story 2.5's recent runs; no source changes mean cargo cache hits in full).
    - `gh run list --workflow=ci.yml -L 1` → capture run ID. Wait for completion. `gh run view <ID> --log | grep -cE 'warning:|error:' | grep -v 'Free disk space'` → 0.
  - [x] **Commit 2 (bookkeeping — does NOT trigger CI):** stage `_bmad-output/implementation-artifacts/sprint-status.yaml`, `_bmad-output/implementation-artifacts/deferred-work.md`, `_bmad-output/implementation-artifacts/2-6-go-fallback-decision-document.md` (this file with `Status: review` + populated Dev Agent Record).
    - HEREDOC commit message subject: `bmad: story 2.6 ready-for-dev → review (M1 decision; epic-2 done; 2-7 not-needed)`. Match precedent: `bmad: story 2.5 in-progress → review (three-backend parity, CI 25113263165 green, GO toon)` (commit `4435ada`).
    - Push to `origin/master`. **Does NOT trigger CI** — `_bmad-output/**` is in `ci.yml`'s `paths-ignore`. No CI run ID to capture.
    - **Why two commits, not one:** the docs commit changes user-facing-evidence (the M1 decision is a public document committed alongside the parity report). Splitting it from the BMad bookkeeping commit (a) keeps the docs commit's diff focused on the decision content only — a clean reviewable artifact, (b) lets the docs commit trigger CI to confirm no accidental cross-contamination with source, (c) the bookkeeping commit's CI-skip via paths-ignore is the explicit policy from `deferred-work.md:5` (CI cadence convention).
  - [x] **Push-fold optimization:** if Till opts to fold both commits into one push (single git push event but two commits), one CI run is captured (the push event triggers one CI run because at least one commit touches non-paths-ignore files). Document the fold reasoning in Dev Agent Record. The fold is acceptable; do NOT collapse the two commits into one (commit-message clarity is preserved by keeping them separate).
  - [x] Story awaits code review. **Code review recommended via `bmad-code-review` skill, ideally with a different LLM than the implementer.** This story's review surface is small (one new markdown file + three small bookkeeping diffs), so a light-pass review is appropriate; however, the decision document IS the M1 milestone-gate artifact and thus warrants careful eyes on (a) the verbatim `Decision` line format, (b) the Rationale's correct citation of parity-report numbers, (c) the Risks Accepted / Fallback Trigger Criteria coverage being complete.

### Review Findings

- [x] [Review][Patch] `7×` multiplier inaccurate — parity-report.md says `7.1×` for M↔DX12 and V↔DX12 pairs [docs/tech-spike/m1-decision.md — Rationale, paragraph 1] — fixed: changed `7×` → `7.1×` (code review, 2026-04-29)
- [x] [Review][Defer] Dual `last_updated` location in sprint-status.yaml (comment + YAML key) — pre-existing pattern, no enforcement that both stay in sync [_bmad-output/implementation-artifacts/sprint-status.yaml] — deferred, pre-existing
- [x] [Review][Defer] Line-number citations in deferred-work.md and M2 Impact section will drift as documents evolve — pre-existing fragility [_bmad-output/implementation-artifacts/deferred-work.md:113, docs/tech-spike/m1-decision.md:M2 Impact] — deferred, pre-existing
- [x] [Review][Defer] `_fallback`-suffix wording in M2 Impact section doesn't exactly match epic paths (epic uses `-fallback` directory, not `_fallback`-suffixed files) — cosmetic, meaning clear [docs/tech-spike/m1-decision.md:M2 Impact] — deferred, pre-existing
- [x] [Review][Defer] "MITIGATED" → "RESOLVED" status-label instruction in deferred-work.md is non-actionable — PRD has no such status labels [_bmad-output/implementation-artifacts/deferred-work.md] — deferred, pre-existing
- [x] [Review][Defer] `generated:` field in sprint-status.yaml never updated — pre-existing pattern [_bmad-output/implementation-artifacts/sprint-status.yaml] — deferred, pre-existing
- [x] [Review][Defer] Fallback Trigger Criterion #4 specifies ~30-day window with no tracking mechanism or responsible party [docs/tech-spike/m1-decision.md:Fallback Trigger Criteria #4] — deferred, accepted design choice
- [x] [Review][Defer] Story 2.7 "not-needed" entry filed in deferred-work.md despite being a definitive disposition, not a deferral — semantic mismatch [_bmad-output/implementation-artifacts/deferred-work.md] — deferred, accepted design choice
- [x] [Review][Defer] `not-needed` status example in sprint-status.yaml schema comment is story-specific (references Story 2.6/2.7) — future `not-needed` stories find a story-specific definition [_bmad-output/implementation-artifacts/sprint-status.yaml] — deferred, pre-existing

## Dev Notes

### Why this story exists

Story 2.6 is the **M1 milestone-gate decision artifact**. Stories 2.1–2.5 built the vector aesthetic and proved it renders identically on Metal / Vulkan / DX12. Story 2.5 produced the parity report with a `RECOMMEND GO toon` line. Story 2.6 ratifies that recommendation into a formal, auditable decision document — `docs/tech-spike/m1-decision.md` — with explicit Decision / Rationale / Risks / Fallback / M2-Impact sections.

**The recommendation IS the input. The decision IS the ratification.** Story 2.6 is NOT re-evaluating the parity report; it is committing to a decision based on it. The dev's role is (a) read the parity-report recommendation verbatim, (b) verify the dev concurs by independent visual inspection of the three PNGs, (c) author the decision document, (d) flip Story 2.7 to `not-needed`, (e) close Epic 2.

**Without this story, M1 has no formal closure artifact** — there's evidence (parity report) but no decision. The next sessions (planning a future Bevy bump, planning M2 / Story 3.1, planning a polish pass) need a single grep-able document to know "what did M1 decide?" That document is `m1-decision.md`.

[Source: `epics/epic-2-vector-aesthetic-tech-spike.md:149-170` (Story 2.6 epic spec); `prd.md:347` (M#10 fallback condition); `prd.md:441` (R#2 risk to be resolved); `architecture.md:295` (M1 — Vector Spike completion gate)]

### Inherited context from Stories 2.1 + 2.2 + 2.3 + 2.4 + 2.5

| Fact | Value | Source |
|---|---|---|
| Parity report recommendation (Story 2.5 output) | `RECOMMEND GO toon` (one-paragraph justification follows) | `docs/tech-spike/m1-backends/parity-report.md:104-119` |
| RMSE_normalized values (Story 2.5 measurement) | M↔V: 0.000423 (118× under threshold); M↔D: 0.006960 (7× under); V↔D: 0.006955 (7× under) | `parity-report.md:35-43` |
| AC #4 RMSE threshold (Story 2.5) | 0.05 (5% normalized) | `2-5-three-backend-parity-validation-gate.md` AC #4 |
| Six qualitative-equivalence checks (Story 2.5) | All pass on all three backends | `parity-report.md:79-102` |
| Story 2.5 status | `done` (this story is created right after 2.5's review-pass) | `sprint-status.yaml:63` |
| Story 2.7 status | `backlog` (about to flip to `not-needed`) | `sprint-status.yaml:65` |
| Epic 2 status | `in-progress` (about to flip to `done`) | `sprint-status.yaml:58` |
| Test count post-2.5 | **14 passing** (unchanged in this story) | `2-5-three-backend-parity-validation-gate.md:907` |
| Bevy version | `0.18` (resolved `0.18.1`) — unchanged | `Cargo.toml` |
| `docs/tech-spike/` layout | `m1-palette/`, `m1-toon/`, `m1-outline/`, `m1-backends/` (sub-spike folders) | `ls docs/tech-spike/` |
| Capture-mode cleanup at Story 3.1 | Already deferred from Story 2.5 (deferred-work.md:99–102) | `deferred-work.md` |
| PRD R#2 (WGSL on 3 backends risk) | "MITIGATED" → resolved by this story's GO decision | `prd.md:441` |

### Decision-document design — five-section template, why this exact structure

**Why exactly five sections** (`Decision`, `Rationale`, `Risks Accepted`, `Fallback Trigger Criteria`, `M2 Impact`):

The epic spec at `epics/epic-2-vector-aesthetic-tech-spike.md:159` mandates these five names, in this order. **Do NOT** add sub-sections, additional top-level sections, or rename. The exact section header strings are the AC #1 anchor — automated readers (BMad tooling, retrospective scripts, future create-story sessions referencing the decision) grep for `^## Decision\b` and `^## Fallback Trigger Criteria\b` etc.

**Why `Decision` is a one-line section** (no preamble, just the verbatim string):

The epic AC says "Decision is exactly one of: `GO toon`, `GO toon with scope reduction`, `FALLBACK flat+rim-light`." Wrapping in a code fence or blockquote (e.g. `\`GO toon\`` or `> GO toon`) would change the grep target and arguably violate the AC's "exactly one of" wording. Keep it as plain text on its own line. This is a deliberate constraint — easier to grep, easier to copy-paste into a status report, easier for an LLM to extract programmatically.

**Why `Rationale` cites the parity report inline rather than re-stating its findings:**

The parity report (Story 2.5) is the load-bearing evidence; `m1-decision.md` (this story) is the ratification. Re-stating the report's qualitative checks in full would duplicate content and create drift risk if the report is later amended. Cite specific numbers + section names ("the parity report's six qualitative checks at `parity-report.md:75-102`"); link by relative path so a reader can follow the citation. The Rationale's load-bearing job is to demonstrate that the dev READ the report and made an informed decision — not to reproduce the report.

**Why `Risks Accepted` enumerates 4 specific risks rather than a generic "all risks accepted":**

The decision document is auditable evidence. A future code-archaeology session ("did Till know about WARP outline-thinning when he made the GO call?") needs explicit risk acknowledgment. The four risks (software-rasterizer noise, WARP outline thinning, version-bump regression, hardware coverage gap) are the empirically observable residuals from the parity-report's evidence; each gets a one-line mitigation or acceptance rationale.

**Why `Fallback Trigger Criteria` are tripwires, not pre-emptive escape hatches:**

The decision is `GO toon`. The Fallback criteria are NOT "if this happens, automatically flip to fallback." They are observable conditions that, IF MET, REQUIRE re-opening the decision via a new dated decision document (`m1-decision-revisit-<date>.md`) — they're pointers to the next decision, not the current one.

**Why `M2 Impact` is the closing section:**

The decision document feeds the next plan. Story 3.1 (Arena), the first story of Epic 3 / M2, will read `m1-decision.md` to know whether to assume `ToonMaterial` is production code or whether to factor in fallback work. Putting M2 Impact last makes the document scannable: a reader can read just `Decision` + `M2 Impact` for the most-relevant 80% of content.

### Why two commits, not one

The two-commit pattern — Commit 1 (`docs:`) + Commit 2 (`bmad:`) — matches Stories 2.4 and 2.5 precedent. Reasons:

1. **CI behavior differs.** Commit 1 (`docs/**`) DOES trigger CI; Commit 2 (`_bmad-output/**`) does NOT (paths-ignore filter at `.github/workflows/ci.yml:9-10,12-14`). Splitting lets Commit 1's CI-trigger be intentional (proves no source contamination); Commit 2's no-CI is the intended efficiency.
2. **Diff focus.** Commit 1 is a single new markdown file = clean reviewable diff. Commit 2 has three small bookkeeping diffs = also clean. Combined would be 4-file mixed diff, harder to review.
3. **Roll-back granularity.** If a reviewer flags an issue with the decision document content (e.g. "you forgot to cite R#2"), Commit 1 can be amended in isolation; Commit 2's bookkeeping doesn't block.
4. **Precedent.** Stories 2.4 and 2.5 used the same split and it worked smoothly.

### LLM dev agent guardrails — most-likely-to-go-wrong patterns

These are the failure modes that are most likely to bite if the dev moves fast:

1. **Wrapping the `Decision` line in a code fence or blockquote.** The verbatim string `GO toon` (or `GO toon with scope reduction` or `FALLBACK flat+rim-light`) must be plain text on its own line. **Do NOT** write `\`GO toon\`` or `> GO toon` or `**GO toon**`. Plain text. AC #1.

2. **Choosing `GO toon with scope reduction` instead of `GO toon`.** The parity report recommends `GO toon` (no scope reduction). All six qualitative checks pass; all RMSE values 7×–118× under threshold. Choosing the scope-reduced variant is a more conservative call that the evidence does NOT require. **Use `GO toon`.** The epic spec allows the scope-reduced variant as a hedge, but it's only the right call when one or more checks fail or RMSE crosses threshold — neither happened here.

3. **Adding sub-sections under the five mandated `##` sections.** No `### Sub-section`. Keep the document flat. AC #1.

4. **Forgetting to add the `not-needed` definition to the YAML status comment block.** Just flipping `2-7-fallback-material-scaffold-... → not-needed` without updating the `# Story Status:` comment leaves the new status undocumented. Future readers (and future BMad tooling) won't know what `not-needed` means. Both edits land together.

5. **Touching `src/`.** This story is documentation-only. `git status --short` after Tasks 2–4 must show NO `src/` files. If you find yourself "just making a small fix" to `src/visual/toon_material.rs` because you noticed something while reading: STOP, revert, and open a separate tightly-scoped story.

6. **Re-running `cargo run` or any runtime invocation.** No runtime behavior is being changed in this story. Skip the runtime smoke from Story 2.5's pattern. The verification sweep (Task 5) is enough.

7. **Citing `parity-report.md` numbers wrong.** The three RMSE_normalized values are: M↔V `0.000423`, M↔D `0.006960`, V↔D `0.006955`. Read them from the report; don't paraphrase from memory. The Rationale's credibility as auditable evidence depends on numerical accuracy.

8. **Re-flipping `epic-2-retrospective` from `optional` to `done`.** The retrospective is opt-in per the existing `# Retrospective Status:` block in `sprint-status.yaml`. Epic 2 closure does NOT mandate a retrospective. Leave `epic-2-retrospective: optional` unchanged.

9. **Adding `parity-capture.yml` removal as a Task 7 commit.** Capture mode + `parity-capture.yml` removal is Story 3.1 cleanup, already tracked in `deferred-work.md:99-102`. This story does NOT remove them. The `m1-decision.md` document references the capture mode as M1-spike-only; it does not delete the capture mode.

10. **Forgetting the YAML-parse verification.** Editing `sprint-status.yaml`'s comment block is the most likely place for an indentation slip. The `python3 -c "import yaml; ..."` parse check is the canary. **Run it after editing.** If it fails, re-do the edit with the exact two-space `#   - ` prefix matching the existing entries.

11. **Forgetting to update `last_updated` in `sprint-status.yaml`.** Both the comment-line at the top of the file (`# last_updated: ...`) and the `last_updated:` YAML key in the body. Match the precedent: `last_updated: 2026-04-29 (Story 2.5 review → done — code review passed, 0 patches)` → `last_updated: <YYYY-MM-DD> (Story 2.6 done — M1 GO toon ratified; Story 2.7 → not-needed; epic-2 → done)`.

12. **Letting the Rationale section cite future work that hasn't happened.** It's tempting to write "M2 Story 3.1 will remove capture mode" in Rationale — that belongs in `M2 Impact`. Keep Rationale focused on *why this decision is correct now*, based on the evidence at hand.

### Architecture compliance — naming, file layout, decision-doc convention

**File layout (`docs/tech-spike/m1-decision.md`):** ✓
- Top-level of `docs/tech-spike/`, parallel to `m1-palette/`, `m1-toon/`, `m1-outline/`, `m1-backends/` sub-spike folders. The decision integrates evidence from all four; the path layout reflects that hierarchy.
- Path explicitly mandated by `epics/epic-2-vector-aesthetic-tech-spike.md:158`.

**Naming (`m1-decision.md`):** ✓
- Matches the existing `m1-` prefix convention (`m1-palette/`, `m1-toon/`, `m1-outline/`, `m1-backends/`).
- Singular `decision`, not `decisions` — there is one M1 decision artifact, not a folder of multiple.
- `.md` extension for the canonical decision-doc format.

**Decision-doc structural convention:**
- This is the project's first formal milestone-gate decision document. The five-section template (`Decision`, `Rationale`, `Risks Accepted`, `Fallback Trigger Criteria`, `M2 Impact`) becomes the precedent for future milestone-gates: `m4-decision.md` (Bevy version-bump readiness), `m6-decision.md` (Steam integration go/no-go), `m9-decision.md` (MVP closure / EA-launch readiness). Future create-story workflows for those milestones should mirror this template. Source: this story's own structural commitment.

**No source code paths affected:** this story changes no `src/**` file, no plugin boundary, no SystemSet. The architecture compliance check is structural / non-applicable.

### Forward compatibility — Story 3.1 hand-off

Story 3.1 (Arena state, first story of Epic 3 / M2) reads `docs/tech-spike/m1-decision.md`'s `Decision` line and `M2 Impact` section to factor M2 implementation:

- `Decision: GO toon` → Story 3.1 keeps `assets/shaders/toon.wgsl`, `src/visual/{toon_material.rs, outline.rs, palette.rs}` as-is; removes capture mode + reference scene as part of replacing the Loading-state scene with the Arena-state scene.
- `Decision: GO toon with scope reduction` (didn't happen here) → would have required Story 3.1 to additionally remove rim-light from the toon shader and bump `toon_steps` to a coarser value before going to Arena.
- `Decision: FALLBACK flat+rim-light` (didn't happen here) → would have required Story 2.7 to land first, replacing `ToonMaterial` with `FlatRimMaterial`, before Story 3.1 even opens.

Story 3.1's create-story workflow will explicitly read this document. The five-section template makes the read fast: Decision = 1 line; M2 Impact = 4 bullets.

### Forward compatibility — M4 / Bevy version-bump window

When the project bumps Bevy past 0.18 (M4 readiness review per `architecture.md:184-197`), the bump-PR's checklist must include:

1. Re-instate capture mode if it was removed at Story 3.1 (a 1-day chore: copy `src/visual/capture.rs` from the M1 git history, re-add `pub mod capture;`, re-add `main.rs` env-var lookup, re-add `.github/workflows/parity-capture.yml`).
2. Dispatch `parity-capture.yml` against the bump branch.
3. Run ImageMagick pairwise diffs against the M1 baseline (`docs/tech-spike/m1-backends/{metal,vulkan,dx12}.png`).
4. If RMSE_normalized cross 0.05 OR any qualitative check fails: this `m1-decision.md` is overridden by a new `m4-bevy-bump-decision.md`.
5. If RMSE values stay below threshold + qualitative checks all pass: append a one-line "Re-validated at M4 (<date>, Bevy <version>)" annotation under the `Date:` header of `m1-decision.md`.

This forward compatibility is the reason the `Fallback Trigger Criteria` section enumerates the version-bump trigger explicitly (Task 2's section content guidance, criterion 4).

### Test count discipline

Post-2.5: 14 passing tests. Post-2.6 expected: **14** (unchanged — this story adds no source code, no new tests, removes no tests).

If `cargo test` reports anything other than `14 passed`:
- **<14:** a test was accidentally deleted from `src/`. Investigate `git diff --stat src/`; revert.
- **>14:** a test was added. Investigate; this story spec does not authorize new tests.

### Project Structure Notes

- **Path alignment with architecture.md:**
  - `docs/tech-spike/m1-decision.md` matches the existing tech-spike documentation pattern (top-level of `docs/tech-spike/`, alongside `m1-{palette,toon,outline,backends}/` sub-spike folders).
  - No `src/` paths involved — this is a documentation-only story.
- **No path conflicts or variances.**
- **`assets/`, `Cargo.{toml,lock}`, `.github/`, `src/` all untouched.** Capture mode + parity-capture workflow + reference scene + toon shader stay exactly as committed by Story 2.5.

### References

- [Source: _bmad-output/planning-artifacts/epics/epic-2-vector-aesthetic-tech-spike.md#Story-2.6 (lines 149-170)]
- [Source: _bmad-output/planning-artifacts/epics/epic-2-vector-aesthetic-tech-spike.md#Story-2.7 (lines 172-194; story conditional on this one)]
- [Source: _bmad-output/planning-artifacts/prd.md#R#2-risk (line 441) — "WGSL shader complexity for a beginner on three graphics backends" — risk this story formally resolves]
- [Source: _bmad-output/planning-artifacts/prd.md#M#10-fallback (line 347) — "If vector aesthetic tech-spike underwhelms (M1): fall back to flat-shaded low-poly + simple rim-light" — fallback condition NOT met]
- [Source: _bmad-output/planning-artifacts/architecture.md#M1-Vector-Spike (line 295) — "M1 — Vector Spike: Custom Toon WGSL Material + bevy_mod_outline, three-backend validation gate." — this story closes the gate]
- [Source: _bmad-output/planning-artifacts/architecture.md#Tech-Risk-Resolution (lines 885-887) — "M1 tech-spike fallback implementation scaffolding" advisory gap — this story's GO decision resolves it]
- [Source: _bmad-output/planning-artifacts/architecture.md#Rendering-Visual-Architecture (lines 218-224) — "Custom WGSL Toon Material + bevy_mod_outline" stack — confirmed by this story]
- [Source: _bmad-output/implementation-artifacts/2-5-three-backend-parity-validation-gate.md (Story 2.5 — full inherited context, parity capture-mode design, deviations)]
- [Source: docs/tech-spike/m1-backends/parity-report.md (Story 2.5 output — the `RECOMMEND GO toon` line + 6 qualitative checks + 3 RMSE values that this story ratifies)]
- [Source: docs/tech-spike/m1-backends/{metal,vulkan,dx12}.png (Story 2.5 output — the visual evidence the dev independently inspects in Task 1)]
- [Source: _bmad-output/implementation-artifacts/deferred-work.md (Story 2.5 deferral block at lines 98-108 — Story 3.1 cleanup, contingencies; cross-linked from this story's deferred-work entries)]
- [Source: _bmad-output/implementation-artifacts/sprint-status.yaml (current state: 2-5 done, 2-7 backlog, epic-2 in-progress; transitions in Task 3)]

## Dev Agent Record

### Agent Model Used

claude-opus-4-7 (1M context)

### Debug Log References

**Verification sweep (Task 5) — local, post-edits, 2026-04-29:**

| Command | Exit | `grep -cE 'warning:\|error:'` | Log path | Notes |
|---|---|---|---|---|
| `cargo check` | 0 | **0** | `/tmp/story-2-6-check.log` | Cache-warm finish in 0.14s — no source touched |
| `cargo build` | 0 | **0** | `/tmp/story-2-6-build.log` | Cache-warm finish in 0.15s |
| `cargo test` | 0 | **0** (`warning:\|error:\|FAILED`) | `/tmp/story-2-6-test.log` | `test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` — exact match with post-2.5 baseline |
| `cargo clippy --all-targets -- -D warnings` | 0 | **0** | `/tmp/story-2-6-clippy.log` | Cache-warm finish in 0.16s |
| `cargo fmt --all -- --check` | 0 | n/a | (no log) | exit 0 |
| `cargo build --release` | 0 | **0** | `/tmp/story-2-6-release.log` | Cache-warm finish in 0.15s |

**Scope-guardrail checks (Task 6):**
- `git diff --stat src/` → empty (no `src/` files touched).
- `grep -nrE 'GO toon\|FALLBACK flat\+rim-light\|m1-decision' src/ --include='*.rs'` → 0 hits.
- `grep -nrE 'CapturePlugin\|ASTEROIDS3D_CAPTURE_PNG' src/ --include='*.rs' | wc -l` → 7 (within expected 5–10 range; capture mode untouched, scheduled for Story 3.1 cleanup).
- `git status --short` final set: only `docs/tech-spike/m1-decision.md` (??), `_bmad-output/implementation-artifacts/sprint-status.yaml` (M), `_bmad-output/implementation-artifacts/deferred-work.md` (M), `_bmad-output/implementation-artifacts/2-6-go-fallback-decision-document.md` (??, never previously committed). Plus `.claude/scheduled_tasks.lock` (Claude Code runtime artifact, not staged — known pre-existing per `deferred-work.md:57`). No `src/`, no `Cargo.{toml,lock}`, no `.github/`, no `assets/`.

**YAML parse check (Task 3 verification):** `ruby -ryaml -e "YAML.load_file(...)"` → `OK`. Final values: `epic-2: done`, `2-6-go-fallback-decision-document: review`, `2-7-fallback-material-scaffold-conditional-on-story-2-6: not-needed`. *(Story spec recommended `python3 -c "import yaml; ..."` but the macOS system Python 3.13 has no PyYAML; ruby's stdlib YAML.load_file was used as a drop-in equivalent. Both parse the same YAML 1.1 grammar; outcome unchanged.)*

**Parity-report verbatim recommendation read (Task 1):** `docs/tech-spike/m1-backends/parity-report.md:106` reads `> **RECOMMEND GO toon**` exactly as the story spec expects. No deviation; no HALT triggered. Justification paragraph captured in `m1-decision.md` Rationale, with the three RMSE_normalized values (M↔V `0.000423`, M↔D `0.006960`, V↔D `0.006955`) cited verbatim against the AC #4 threshold `0.05`.

### Completion Notes List

**Per-AC evidence:**

- **AC #1 (5-section structure + verbatim Decision line + Date/Owner/source-link headers):** ✓ `docs/tech-spike/m1-decision.md` opens with `Date:`, `Decision Owner: Till Fechteler (project author)`, and `Source evidence:` hyperlink to `./m1-backends/parity-report.md`. Five top-level `##` sections appear in the mandated order: `Decision`, `Rationale`, `Risks Accepted`, `Fallback Trigger Criteria`, `M2 Impact`. The `Decision` section body is a single line `GO toon` — no code fence, no blockquote, no backticks, no trailing punctuation.
- **AC #2 (decision matches recommendation; Rationale cites six checks + three RMSE values + R#2):** ✓ Decision = `GO toon` (unconditional, matching the parity report's recommendation). Rationale paragraph 1 enumerates all six qualitative checks (banding count, rim-light, tint colors, outline continuity, outline width, swatch palette) and the three RMSE_normalized values with their `0.05` AC #4 threshold (118× / 7× / 7× under). Rationale paragraph 2 explicitly addresses PRD R#2 ("WGSL shader complexity for a beginner on three graphics backends") and states the M1 spike resolves it. No deviation from the parity-report recommendation — clause "any deviation … carries a documented justification paragraph" is vacuously satisfied.
- **AC #3 (sprint-status.yaml: 2-7 → not-needed, status-definition update, 2-6 lifecycle, last_updated bump):** ✓ `_bmad-output/implementation-artifacts/sprint-status.yaml` now contains: (a) new `#   - not-needed: ...` status definition under the `# Story Status:` block, (b) `2-7-fallback-material-scaffold-conditional-on-story-2-6: not-needed`, (c) `2-6-go-fallback-decision-document: review` (transitioned `ready-for-dev → in-progress → review` over this dev-story session — sequential edits captured in git history of this file though only the final `review` value is committed), (d) `last_updated: 2026-04-29 (Story 2.6 ready-for-dev → review — M1 GO toon ratified; Story 2.7 → not-needed; epic-2 → done)` on both the top comment-line and the YAML key.
- **AC #4 (zero source impact + 14/14 tests + clean 6-command verification sweep):** ✓ All six cargo commands report `grep -cE 'warning:|error:'` = `0` (see Debug Log References table above). `cargo test` summary line is `test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` — byte-identical to post-2.5 baseline at `_bmad-output/implementation-artifacts/2-5-three-backend-parity-validation-gate.md:907`. `git diff --stat src/` is empty. No mutation to `Cargo.toml`, `Cargo.lock`, `.github/workflows/*.yml`, `assets/**`, or any `src/**` file.
- **AC #5 (epic-2 closure + retrospective unchanged + deferred-work entry):** ✓ `epic-2: done` flipped per Task 3 Option A (epic closure folded into this story's bookkeeping commit, since with 2.7 not-needed the closure is unambiguous: 5 done ∪ 1 review-pending ∪ 1 not-needed = full coverage of all 7 stories). `epic-2-retrospective: optional` unchanged (retrospective is opt-in, not gated by epic closure per the existing `# Retrospective Status:` block). `deferred-work.md` gained exactly one new section `## Deferred from: 2-6-go-fallback-decision-document (2026-04-29)` with three entries: (1) Story 2.7 explicitly NOT NEEDED with re-open path via `m1-decision-revisit-<date>.md`, (2) cross-link to Story 2.5's Story 3.1 capture-mode cleanup entry (lines 99–102), (3) cross-link to PRD R#2 resolution at `prd.md:441` for future planning-sweep awareness.

**Deviations from spec:** none material. Two minor notes:
- The story spec's `last_updated` annotation suggested `(Story 2.6 done — ...)`, but at hand-off the story is at `review` (not `done`) — `done` is set by the post-code-review bookkeeping commit, mirroring Story 2.5's lifecycle (commit `4435ada` → `ee783ef`). Annotation written as `Story 2.6 ready-for-dev → review — ...` to match the actual mechanical transition and the Story 2.5 precedent (`Story 2.5 in-progress → review` at the same lifecycle position).
- The story spec recommended `python3 -c "import yaml; yaml.safe_load(open(...))"` for the YAML parse check; system Python 3.13 has no PyYAML, so `ruby -ryaml -e "YAML.load_file(...)"` (system Ruby, stdlib) was used instead. Same grammar, same parse outcome.

### File List

**Added:**
- `docs/tech-spike/m1-decision.md` — new file (Task 2). The M1 milestone-gate decision artifact; five-section template; ratifies the parity report's `RECOMMEND GO toon` into a formal, auditable Decision/Rationale/Risks Accepted/Fallback Trigger Criteria/M2 Impact document.

**Modified:**
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — Task 3. Added `not-needed` status definition under `# Story Status:` block; flipped `2-6` → `review`, `2-7` → `not-needed`, `epic-2` → `done`; bumped `last_updated` (both top-comment line and YAML key) to `2026-04-29 (Story 2.6 ready-for-dev → review — M1 GO toon ratified; Story 2.7 → not-needed; epic-2 → done)`.
- `_bmad-output/implementation-artifacts/deferred-work.md` — Task 4. Appended `## Deferred from: 2-6-go-fallback-decision-document (2026-04-29)` section with three cross-link entries (Story 2.7 disposition, Story 3.1 cleanup cross-link, PRD R#2 resolution cross-link).
- `_bmad-output/implementation-artifacts/2-6-go-fallback-decision-document.md` (this file) — Task 7. `Status: ready-for-dev` → `Status: review`; all 47 task/subtask checkboxes flipped `[ ]` → `[x]`; Dev Agent Record populated (Agent Model Used, Debug Log References, Completion Notes List, File List).

**Untouched (verified by git status / git diff):** all `src/**`, `Cargo.toml`, `Cargo.lock`, `.gitignore`, `.gitattributes`, `.github/workflows/{ci,parity-capture}.yml`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`, `assets/**`, `docs/tech-spike/m1-{backends,palette,toon,outline}/**` (Story 2.5 and earlier evidence; read-only from this story's perspective), `_bmad-output/planning-artifacts/**`.
