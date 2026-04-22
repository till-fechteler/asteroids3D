# Epic List

## Epic 1: Foundation & Plugin Compatibility Gate

**User outcome (dev-foundational):** Project compiles and runs on Windows, Linux, and macOS. `cargo run` opens a window showing "asteriods3D" splash on all three platforms. Plugin compatibility matrix (Bevy 0.18, Avian 0.6, bevy_mod_outline, bevy_kira_audio, leafwing-input-manager, bevy_egui) verified and version-pinned. CI matrix green. No gameplay — this is the compatibility gate per Architecture Starter decision.

**FRs covered:** FR47 (cross-platform binary baseline)

**Scope:** Hybrid-Manual starter initialization, `Cargo.toml` authored by hand with pinned versions, Bevy-0.18 compatibility verification (fork-and-inline any plugin that lags), CI workflow (Windows/Linux/macOS — strip iOS/Android/Web from NiklasEi template), `rustfmt.toml` + `clippy.toml` + `rust-toolchain.toml`, `App::new()` skeleton with `GameState` enum + `leafwing-input-manager` + bevy_ui splash screen Node.

**M-alignment:** M0

**Completion gate:** `cargo run` opens window with "asteriods3D" splash on Win/Linux/macOS; CI green.

---

## Epic 2: Vector Aesthetic Tech Spike

**User outcome:** Custom WGSL Toon Material + bevy_mod_outline render identically on Metal (macOS), Vulkan (Linux), and DX12 (Windows). M1 go/fallback decision documented. Portfolio-quality shader artifact authored by Till.

**FRs covered:** FR49 (toon shading + outlines), FR50 (semantic accent palette foundation)

**Scope:** Custom WGSL Toon `Material` impl at `src/visual/toon_material.rs`, `bevy_mod_outline` integration wiring, palette primitives (`SemanticAccent` enum + color lookup), three-backend validation gate (render reference scene on all three GPUs and compare visually), go/fallback decision doc. Fallback path (flat-shaded + rim-light) scaffolded only if decision = fallback.

**M-alignment:** M1

**Completion gate:** Reference scene renders with toon + outline on Metal/Vulkan/DX12; decision "go toon" or "fall back" committed.

---

## Epic 3: Arena Flight & First Combat (First Playable)

**User outcome:** Player flies a cockpit ship in the Arena, fires a prefab weapon, destroys asteroids, sees HUD with ship state. No enemies yet. Diegetic learning — no tutorial text.

**FRs covered:** FR1, FR2, FR3, FR5, FR8, FR9, FR12, FR24, FR27, FR28, FR43

**Scope:** FlightPlugin (input → thrust/rotation via leafwing + Avian XPBD in FixedUpdate 60Hz + dampener toggle), cockpit `Camera3d` with wingtip framing, CombatPlugin weapon-firing + projectile ballistics + damage-on-asteroid, HUD screen-space baseline (shields/hull/ammo/salvage placeholders), Arena zone (hand-designed), pause on focus-loss, basic title screen stub.

**M-alignment:** M2

**Completion gate:** Player flies Arena, shoots asteroids, HUD visible, pause works on Alt-Tab.

---

## Epic 4: Enemies Alive & Stop-Ship (Itch.io Prototype)

**User outcome:** The Itch.io-shippable small game. Full combat loop: 3 weapons, 1 enemy type with AI, permadeath on Hull-zero, post-run summary, immediate restart, title screen, settings (volume + sensitivity), saved settings + currency "high-score", signed+notarized macOS binary. The M3 stop-and-ship waypoint.

**FRs covered:** FR10, FR14, FR16, FR36, FR37, FR38, FR39, FR44, FR45, FR46, FR47, FR50 (FR48 deferred to E7 per Till 2026-04-22; macOS ships unsigned for M3)

**Scope:** 2 more weapon archetypes (total 3), enemy AI state machine (detect → pursue → attack), basic single-HP Hull + permadeath flow, title screen (start / settings / credits / quit), settings UI (volume master+SFX, mouse sensitivity), post-run summary screen (cause of death, salvage banked, retry/menu — no "GAME OVER"), restart flow, PersistencePlugin save service (atomic temp+rename, JSON+Serde, versioned schema, `directories` crate per-OS paths), first-launch default save creation, macOS codesign + notarytool workflow (Apple Developer account), release.yml for per-OS ZIPs + butler-to-Itch.io, semantic accent colors wired to enemies + salvage.

**M-alignment:** M3 🏁 (stop-and-ship)

**Completion gate:** Itch.io-ready ZIPs on all 3 platforms (macOS unsigned, right-click-open per M3 decision), zero-crash 3-run Arena playtest passes.

---

## Epic 5: Ship Subsystem State & Formal Save Schema

**User outcome:** Formal Hull + Shields subsystems with regen mechanics. Shields regenerate after cooldown; Hull does not regen mid-run. Ship state readable at-a-glance. Save schema formalized with versioning for post-MVP expansion. Decoupled aim reticle system.

**FRs covered:** FR4, FR15

**NFRs covered:** NFR-R3 (graceful missing-save), NFR-R4 (no between-run meta loss), NFR-U2, NFR-U3 (HUD subsystem at-a-glance)

**Scope:** Formal `HullHP` + `ShieldHP` components (regen_rate, cooldown) per architecture good-pattern example, Shield regen system, Hull-zero → `HullDepleted` event wiring, decoupled-aim reticle overlay (world-space target coords → bevy_ui edge indicator), at-a-glance instrument-panel HUD styling, save schema `version: u32` + migration scaffold, missing-save recovery prompt.

**M-alignment:** M4

**Completion gate:** Shield regen tunable in `tuning.ron`, Hull damage visible in cockpit HUD at a glance, save schema migrates old → new without data loss.

---

## Epic 6: Caravan Run Framework

**User outcome:** Player flies a Caravan run from start to target destination (5–8 min), with waypoint-pointer navigation, selectable difficulty (easy/medium/hard), trigger-volume combat pockets, tractor-beam intact-asteroid pickup, boost, pay-to-shoot economy with intact > destroyed yield math, salvage banking on success and death.

**FRs covered:** FR6, FR7, FR11, FR13, FR17, FR29, FR30, FR31, FR32, FR33, FR34, FR35

**Scope:** RunPlugin run-director (RunStarted/RunEnded lifecycle), single Caravan skeleton template with 3 difficulty parameter variants, waypoint-pointer world-space HUD, render-distance pocket-trigger system (Avian sensors), `BoostActivated` event + recharge resource, SalvagePlugin tractor-beam constraint/impulse on intact asteroids, economy math (shot-cost debit on WeaponFired, yield calc on AsteroidDestroyed vs AsteroidCaptured), Arena → Caravan state transition, salvage banking on both outcomes.

**M-alignment:** M5 ⚠️ **Danger Stretch** — stories in this epic MUST be sliced into weekly sub-milestones where each delivers one visible feature (waypoint pointer → first pocket trigger → difficulty curve → tractor beam → economy math → difficulty variants). No "I'll build Caravan for 6 weeks" commitments.

**Completion gate:** Player completes a 5-min easy Caravan run from start to destination, tractor-captures at least one intact asteroid, pocket combat triggers at least once.

---

## Epic 7: Roguelite Loop (EA-Viable)

**User outcome:** Meta-currency earned from runs, spendable in unlock shop for 5–10 permanent upgrades. "One more run" retention loop closed. Commercially viable as Itch.io release or Steam Early Access. Also: Intel x86_64 macOS binary added alongside arm64 (universal, still unsigned — FR48 further deferred to E10 per Till 2026-04-22).

**FRs covered:** FR18, FR19, FR20, FR21 (FR48 deferred E4→E7→E10; MVP ships unsigned macOS through M6)

**NFRs covered:** NFR-R4 (meta never lost between runs during normal play)

**Scope:** `PersistentMeta` resource extending save schema (meta-currency balance, unlocked upgrade IDs), run→meta conversion rate (configurable in `tuning.ron`), unlock shop UI accessible from main menu and post-run screen, 5–10 initial unlock definitions (e.g., ammo capacity +20%, sensor range +15%, boost recharge faster, etc.), `UnlockPurchased` event + save trigger, unlock effects wired to ship tunables.

**M-alignment:** M6 🏁 (EA-viable)

**Completion gate:** 10-run playtest with save persisting meta-currency across crashes, 3+ unlocks purchased and effects visible in gameplay.

---

## Epic 8: Perception — Sensors & Spatial Audio

**User outcome:** Player perceives unseen threats via cockpit radar and spatial stereo audio cues. Yield-delta indicator shows intact-vs-destroyed math on visible salvageable targets. First-launch headphone recommendation.

**FRs covered:** FR22, FR23, FR25, FR26

**NFRs covered:** NFR-A1 (colorblind redundant encoding), NFR-A2 (no color-only information)

**Scope:** PerceptionPlugin sensor range + threat detection (`EnemyDetected` / `HazardDetected` events), world-space radar mesh on cockpit model (scientific-instrument styling per Design Philosophy), AudioPlugin spatial channel setup (`bevy_kira_audio`), audio-cue routing from threat events, world-space yield-delta indicator on visible Salvageable entities, first-launch splash with headphone recommendation (`bevy_ui` modal shown before main menu), **damage-direction-indicator** on cockpit HUD (red arrow at screen edge pointing toward the origin of incoming fire when shields/hull take a hit — added during E5 decomposition 2026-04-22).

**M-alignment:** M7

**Completion gate:** Player identifies unseen enemy from audio direction before it enters visual range, radar shows correct markers, yield-delta visible on at least 3 salvage types.

---

## Epic 9: Post-Run Photo Mode

**User outcome:** From post-run / death screen, player enters Photo Mode with free-cam orbital/dolly movement, adjustable depth-of-field, time-frozen scene. Exports PNG screenshots in 16:9 landscape, 9:16 portrait, or 1:1 square aspect ratios. Marketing-ready aesthetic artifact pipeline.

**FRs covered:** FR40, FR41, FR42

**Scope:** `FreeOrbitCamera` component (shared with debug F3 camera — one impl, two gates), PhotoMode state entry from post-run screen only (no in-run access), DoF post-processing node, time-freeze on state entry (already paused), PNG export system (aspect-ratio preset enum, resolution scaling, file save to user-pictures dir or game subdirectory), optional toggleable watermark.

**M-alignment:** M8

**Completion gate:** Player exports PNGs in all 3 aspect ratios; toon + outline shader renders correctly at 360° camera angles (external-viewing shader validation).

---

## Epic 10: Polish Pass & MVP Completion

**User outcome:** Balance tuning, audio-pass-2, UI polish, 2–5 additional unlocks beyond M6's baseline, crash fixes, all MVP NFRs satisfied. Full polished MVP ready for Steam release.

**FRs covered:** FR48 (deferred E4→E7→E10 per Till 2026-04-22). Otherwise this epic hardens previously-delivered capabilities.

**NFRs covered:** NFR-P1 (60 FPS sustained), NFR-P2 (load ≤ 10 s), NFR-P3 (title→gameplay ≤ 5 s), NFR-P4 (no hitches > 100 ms), NFR-P5 (< 4 GB memory), NFR-R1 (zero-crash across all 4 user journeys), NFR-U1 (5-min Aha validated), NFR-A3 (HUD legibility 60–80 cm), NFR-L3 (no hard-coded strings audit), NFR-L1 (English ship-ready)

**Scope:** Profiling pass with tracy + flamegraph to hit 60 FPS on GTX 1060 / RX 580 / M1, asset-load-at-state-entry audit (no scattered AssetServer::load), audio SFX pass-2 (mixing, ambient drone layer polish per Design Philosophy "cosmic mystery"), UI polish (scientific-instrument styling refinement), 2–5 additional unlock definitions, crash-fix backlog from M6–M8 playtesting, string-table audit (no hard-coded player-facing strings), playtest validation of all 4 user journeys on all 3 platforms, **shield-absorb VFX** (brief toon-style flash on shield hit, tuned for readability without being intrusive — added during E5 decomposition 2026-04-22), **macOS code-signing + notarization** (FR48, deferred E4→E7→E10; requires Apple Developer Account €99/year at this milestone — enables signed MVP release for Steam if pursued, or Itch.io polish-release).

**M-alignment:** M9 🏁 (MVP polished)

**Completion gate:** All MVP success criteria from PRD true; zero-crash 10-run playtest on all 3 platforms at 60 FPS; Steam-release or Itch.io-polished-release ready.

---

<!-- RESUMPTION MARKER: Step 3 complete — all 10 Epics decomposed into stories. Next: Step 4 final validation. -->
