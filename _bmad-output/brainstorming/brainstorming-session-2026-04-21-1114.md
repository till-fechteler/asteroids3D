---
stepsCompleted: [1, 2, 3]
phase_1_complete: true
phase_2_complete: true
phase_3_complete: true
phase_4_complete: true
inputDocuments: []
session_topic: '3D Asteroids-style game in Rust+Bevy — full-breadth concept exploration (features, differentiation, architecture, scope, risks)'
session_goals: 'Full 6-dimensional exploration: (1) feature & mechanics ideas, (2) differentiation vs genre classics, (3) technical architecture & engine decision (Bevy vs alternatives), (4) product & scope strategy, (5) edge cases & risks, (6) holistic exploration'
selected_approach: 'progressive-flow'
techniques_used: ['SCAMPER Method', 'Mind Mapping', 'Six Thinking Hats (in progress)', 'Resource Constraints']
phase_3_candidates_resolved: ['E#6 Camera Mode Tension', 'E#4 Arena Mode Tension', 'Tech-Pin Engine: Bevy (prior session)', 'Tech-Pin Physics: Avian (prior session)', 'C#6 Weapon Crafting (staged)', 'R#6 Audio-First (two-stage)', 'M#10 Vector Aesthetic (tech-spike)', 'S#5 Salvage Drone (REJECTED, replaced by narrative-light)', 'C#3 Caravan (MVP-reduced)', 'C#1+M#9 Meta+Partial-Death (C#1 MVP, M#9 post)', 'A#4 Subsystem Damage (2-subsystem MVP)']
phase_3_candidates_open: []
ideas_generated: []
context_file: ''
---

# Brainstorming Session Results

**Facilitator:** Till
**Date:** 2026-04-21

## Session Overview

**Topic:** 3D Asteroids-style game in Rust + Bevy — concept and scope brainstorming

**Goals:** Full-breadth exploration across all six dimensions:
1. Feature & mechanics ideas (gameplay elements, power-ups, weapon types, enemy variants, level variety, audio/atmosphere)
2. Differentiation from genre classics (Asteroids, Elite, Star Fox, Everspace)
3. Technical architecture & engine decision (Bevy vs. Fyrox / Macroquad / raw wgpu; ECS patterns; physics via Rapier; asset pipeline)
4. Product & scope strategy (MVP scope, feature sequencing, possible monetization, platforms, release model)
5. Edge cases & risks / pre-mortem (Bevy/3D/Rust pitfalls)
6. Holistic exploration across all dimensions

### Participant Context

- **Experience:** 20+ years programming; beginner in Rust/Bevy/3D-gamedev
- **Ambition:** Primarily learning/hobby project, commercial release possible if quality justifies it
- **Core concept (fixed inputs):**
  - Two game modes: classic arena destroy-all-asteroids, and A→B navigation through an asteroid field
  - Weapon progression via unlocks based on progress/score
  - Enemy spaceships as an additional difficulty layer in both modes
  - Two camera modes: cockpit view, and third-person (rear/top, adjustable)
  - Preferred stack: Rust + Bevy (open to alternatives if well-justified)

### Session Setup

**Approach:** Progressive Technique Flow (systematic progression from divergent to convergent thinking)

## Technique Selection

**Approach:** Progressive Technique Flow
**Journey Design:** Systematic development from expansive exploration to concrete roadmap

**Progressive Techniques:**

- **Phase 1 — Exploration:** SCAMPER Method (7 systematic creativity lenses) — targets broad coverage across 6 session dimensions with anti-bias domain pivots every 10 ideas
- **Phase 2 — Pattern Recognition:** Mind Mapping — cluster raw ideas into thematic branches and identify cross-connections
- **Phase 3 — Development:** Six Thinking Hats — evaluate top candidates through 6 lenses (facts / emotions / benefits / risks / creativity / process)
- **Phase 4 — Action Planning:** Resource Constraints — force MVP prioritization given hobby-scale time budget

**Journey Rationale:** The concept is already well-defined at the product-vision level, so we need a structured divergence (SCAMPER) rather than a wild one — it produces orthogonal variations at pace. Mind Mapping then makes the 6 session dimensions visible as branches. Six Hats hardens surviving ideas against both pragmatic and emotional critique. Resource Constraints closes the loop by matching ambition to Till's hobby bandwidth and Rust/Bevy learning curve.

## Technique Execution Results

### Phase 1 — SCAMPER Method

#### S — Substitute

**[Substitute #1] Gravity Swarm** *(Gameplay / Physics)* — accepted
_Concept:_ Replace inert asteroid chunks with gravitationally active bodies (micro black holes, frozen comets, magnetic ores) that bend projectiles and ship trajectory.
_Novelty:_ Turns "dodge or destroy" into "read the gravity field" — makes space inertia itself a tactical layer.

**[Substitute #3] Engine switch — Fyrox as alternative to Bevy** *(Tech Stack)* — flagged for deeper evaluation in Phase 3
_Concept:_ Use Fyrox (Rust-native 3D engine with built-in editor, physics, animation) instead of Bevy.
_Novelty:_ Stability and editor ergonomics vs. Bevy's ECS-first but still-rapidly-evolving API. Meaningful tradeoff for a beginner.

**[Substitute #5] Sentient Salvage Drone** *(Story / Identity)* — accepted
_Concept:_ Player isn't a pilot but an AI salvage drone recovering fragments of a collapsed civilization. Asteroids become reactive relics.
_Novelty:_ Adds narrative layer without cutscenes — destruction itself can trigger lore.

**[Substitute #6] Orbital Loop** *(Mode 2 redesign)* — accepted
_Concept:_ Replace the A→B passage with orbiting a planet for N laps, asteroid density escalating each lap. Survival-on-a-curved-path instead of waypoint racing.
_Novelty:_ Merges classic-mode survival with mode-2 traversal; lets gravity become a gameplay element rather than just backdrop.

**Till's selection pattern:** Three of four picks (Gravity Swarm, Salvage Drone, Orbital Loop) push toward a **physics-aware, atmospheric variant** of Asteroids rather than pure arcade. Plus an open question on engine choice (#3, flagged "serious alternative").

#### C — Combine

**[Combine #1] Roguelite Run-Meta** *(Genre Fusion / Progression)* — accepted
_Concept:_ Each mission is a run — on death the ship is lost but a meta currency (salvaged relics, tech fragments) persists to unlock weapons, ships, perks. Mode 2 becomes the natural run shape.
_Novelty:_ Turns the "weapons unlock by progress" feature into a motivating system loop; high retention without content bloat — friendly to a solo dev.

**[Combine #2] Bevy + Avian (+ optional bevy_mod_outline)** *(Tech Stack)* — accepted, Avian flagged for Phase-3 evaluation vs Rapier
_Concept:_ Bevy for ECS/rendering + **Avian** (formerly `bevy_xpbd`, Bevy-native ECS-idiomatic physics) for collision / gravity fields. Optional cel-shaded outlines for visual identity.
_Novelty:_ Till surfaced **Avian** as alternative to the initially-suggested Rapier. Avian is Bevy-first and ECS-native — potentially more ergonomic for a Bevy learner, though Rapier has more features and is battle-tested. Second open tech decision for Phase 3.

**[Combine #3] Modi-Fusion "Caravan"** *(Gameplay structure)* — accepted
_Concept:_ Modes 1 and 2 aren't separate menu items — they're nested. Player travels A→B (mode 2), and can *opt into* asteroid-arena pockets along the way (mode 1) for risk/reward farming.
_Novelty:_ Eliminates mode-selection UI, produces emergent player choices, naturally matches roguelite run structure (#1).

**[Combine #6] Modular Weapon Crafting** *(Progression system)* — accepted
_Concept:_ Instead of linearly unlocking weapons, player assembles them from modules (projectile type × energy source × modifier). Salvaged asteroid fragments supply modules.
_Novelty:_ Less grind, more tinkering depth; synergizes with Salvage Drone identity (S#5) and Roguelite Meta (C#1).

**Emerging Spine — noticed:** S#5 (Salvage Drone) + C#1 (Roguelite Runs) + C#3 (Caravan) + C#6 (Weapon Crafting) form a coherent game system, not four loose features. This is starting to look less like "Asteroids clone" and more like "**Roguelite salvage/survival game with Asteroids-grade dogfight core.**" Flag for Phase 3 consolidation.

**Tech Pins open for Phase 3:**
- Engine: Bevy vs. Fyrox
- Physics (if Bevy): Avian vs. Rapier3D

#### A — Adapt (all 6 accepted by Till)

**[Adapt #1] Hades-style Heat / Difficulty Modifiers** *(Progression / Replay)* — accepted
_Concept:_ After a successful run, player toggles modifiers (denser asteroid fields, smarter enemies, gravity always active) for higher meta-currency yield.
_Novelty:_ Scales difficulty without producing new content. Maps directly to Till's "enemy ships as difficulty layer" feature.

**[Adapt #2] Everspace/Chorus Flight-Feel** *(Controls)* — accepted
_Concept:_ Compromise between Newton physics (inertia, drift, strafing in vector space) and arcade tightness — thrust vectoring plus toggleable inertial dampener.
_Novelty:_ Proven pattern solves "true 3D movement is confusing." Strong learning project for input→force→rigid-body mapping.

**[Adapt #3] KSP-lite Orbital Mechanics** *(Physics)* — accepted
_Concept:_ Asteroids follow predictable paths (Kepler orbits or scripted splines), not static placement. Player must predict trajectories 3s ahead.
_Novelty:_ Shifts skill from reflex to trajectory-reading. Cheap to implement in Bevy (Bezier/ellipse components), no heavy physics required.

**[Adapt #4] FTL-style Subsystem Damage** *(Ship state)* — accepted
_Concept:_ Instead of single HP pool, ship has discrete systems (hull/engine/weapons/sensors/shields) each with independent HP. Damage causes system failures, not instant death.
_Novelty:_ Perfect fit with Salvage Drone narrative (build/repair). Tactical depth, fewer cheap deaths. ECS-idiomatic (each subsystem as child entity).

**[Adapt #5] Wipeout/Rez Audio-Driven Design** *(Atmosphere / Audio)* — accepted
_Concept:_ Dynamic music layering per game state (Wipeout) + synesthetic audio-visual feedback on destruction (Rez). Destroyed asteroids "sing back" harmonically.
_Novelty:_ Low budget, high identity impact. `bevy_kira_audio` supports layering natively.

**[Adapt #6] Roguelike ECS Composition Pattern** *(Architecture)* — accepted
_Concept:_ Adopt Caves of Qud / Dwarf Fortress style entity composition — everything is a bundle of small components (`Damageable`, `Salvageable`, `GravitySource`, `ThrusterPropelled`). Ships, asteroids, projectiles share components freely.
_Novelty:_ Pure Bevy-ECS idiom. Teaches ECS "the right way" rather than OOP-in-ECS. Enables emergent behaviors (e.g., asteroid that also thrusts = mimic enemy) with zero additional code.

**Till's selection pattern:** **Full acceptance of all 6 Adapts** signals strong consistency with the emerging spine — he's curating, not reflexively approving. Adapt #1, #4, #6 directly strengthen the salvage/roguelite spine; #2, #3, #5 define feel/identity without fighting it. No contradictions in the set.

#### M — Modify / Magnify / Minify (all 10 accepted)

**[Modify #1] Mega-Wreck Structures** *(Scale)* — accepted. Colossal dead megastructures scattered in asteroid fields for orientation, scale, loot, and story beats.
**[Modify #2] Cockpit-Pet** *(Atmosphere)* — accepted. Salvaged lifeform/crystal/AI-core living in cockpit, reacts to events. Cheap immersion win, USP for cockpit camera mode.
**[Modify #3] Salvage Pulse — Bullet-Time** *(Skill mechanic)* — accepted. ~2s time-slow on energy cost. Excellent pairing with Adapt #3 (orbital prediction).
**[Modify #4] Boss Asteroids** *(Enemy variety)* — accepted. Named mega-asteroids with scripted mechanics (Sleeper, Crystal King, Monolith). Signature moments.
**[Modify #5] "Blind Flight" — HUD removed** *(Immersion mode)* — accepted. Optional hardcore mode with only cockpit instruments + audio cues.
**[Modify #6] Cinematic Kill-Camera** *(Feedback)* — accepted. Auto-triggered slow-mo cinematic cuts on significant events.
**[Modify #7] Decoupled Aim** *(Controls)* — accepted. Mouse aims independently of ship heading — twin-stick in 3D.
**[Modify #8] Short Burst + Expedition modes** *(Session design)* — accepted. Run-length selector (3–5 min vs. 30–45 min). Same system, reparametrized.
**[Modify #9] Partial Death — Core Survives** *(Death system)* — accepted. Soft-permadeath: ship+modules lost, drone core survives with % random modules salvaged.
**[Modify #10] Vector Aesthetic in 3D** *(Visual identity)* — accepted, **marked for early tech-prototype in Phase 4**. Tron-style cel-shading + neon outlines + emissive bloom in Bevy via `bevy_mod_outline` + custom toon material. Deep-dive covered separately.

**Till's selection pattern:** Full acceptance (10/10) with explicit deep-dive on #10 — visual identity is clearly important to him. The Mega-Wreck + Boss Asteroids + Vector Aesthetic + Cockpit-Pet cluster defines the "atmosphere/identity" axis of the spine alongside the "system" axis (Salvage/Roguelite/Caravan/Crafting).

#### P — Put to Other Uses

**[Put #1] Photo Mode / Replay Exporter** *(Community / Marketing)* — accepted. Frame-capable photo mode with free cam, DoF, time-freeze; MP4/GIF replay export. Turns vector aesthetic into zero-cost marketing engine.

**[Put #3] Seed-based Level Sharing** *(UGC, no-server)* — accepted. Runs identified by seed, players share interesting seeds. Cheap infinite content.

**[Put #4] "Orbital Mechanics Lab" Sandbox** *(Edu spin-off)* — accepted. Sandbox mode exposing KSP-lite orbital physics as teaching tool. Secondary use for astronomy/education market.

**[Put #6] Bevy Community Tech Demo / Portfolio** *(Career leverage)* — accepted. Explicit dual-use as public Bevy learning artifact: blog posts, open repo. Dovetails with Till's learning goal.

**[Put #8] VR Port of Cockpit Mode** *(Future platform)* — accepted. Cockpit + salvage narrative is natural VR fit. Flagged as post-MVP future option.

**[Put #9] Modding API over Weapon Crafting** *(Modding ecosystem)* — accepted. Weapon module system (C#6) exposed to players via scripting (e.g., Rhai). Community-driven content.

**Rejected:** #2 Zen Drift meditation mode, #5 Twitch chaos mode, #7 audio-first accessibility, #10 desktop screensaver companion.

**Till's rejection pattern — important signal:** Till rejects **non-game lifestyle spin-offs (#2, #10), streamer-bait (#5), and accessibility-as-separate-feature (#7)**. He keeps the extensions that either **strengthen the core game**, **extend Till's learning/career value**, or are **plausible future platform/market extensions**. The rejection of #7 is notable — likely not because he's indifferent to accessibility, but because it's framed as a separate mode rather than woven into core design. Re-raise in Phase 3 as "bake audio cues into primary design" rather than optional mode.

**Keep-Pattern so far:** The Spine is now genuinely coherent. Till is not feature-chasing; he's curating for focus.

#### E — Eliminate (all captured as seeds, deferred evaluation to Phase 3)

Till skipped immediate evaluation (said "next letter") — all 10 E-seeds carry forward to Phase 3 Six Hats where the controversial ones (especially E#4 and E#6) get debated.

**[Eliminate #1] No visible score** — Feedback via salvage & meta-progress only.
**[Eliminate #2] No prefab weapons; crafting is the only path** — Forces C#6 (crafting) into the centerpiece role.
**[Eliminate #3] No tutorial — learn-by-dying + diegetic hints** — Souls/Into-the-Breach school.
**[Eliminate #4] Drop classic Arena (Mode 1) entirely, keep only Caravan** ⚠️ **conflicts with Till's original spec**. Raise for debate in Phase 3.
**[Eliminate #5] No online, no server, fully offline** — Cuts a whole category of work; no post-shutdown rot.
**[Eliminate #6] Drop 3rd-person camera, cockpit-only** ⚠️ **conflicts with Till's original spec (two camera modes)**. Bold focus statement; debate in Phase 3.
**[Eliminate #7] No enemy ships in first N runs** — Staggered learning curve, narrative escalation.
**[Eliminate #8] No pure-random generation — curated 20–30 hand-designed run skeletons re-decorated** — "Procgen-lite" like Hades/Slay the Spire. Probably the right answer.
**[Eliminate #9] Monochrome + single accent color** — Limbo/Obra-Dinn-level visual signature. Sharpens Modify #10.
**[Eliminate #10] No Game Over screen — death as narrative cycle** — Returnal-style; reduces frustration without softening stakes.

**Two flagged tensions with original spec:** E#4 (drop Mode 1) and E#6 (drop 3rd-person cam) — these must be explicitly re-evaluated in Phase 3.

#### R — Reverse

**[Reverse #2] Preserve, don't destroy** *(Objective inversion)* — accepted. Tractor-beam valuable asteroids intact; shoot only enemies who try to destroy them first. Lifts salvage-drone narrative to gameplay level.
**[Reverse #4] Rewind as core feature** *(Time)* — accepted. Last ~3–5s can be rewound. Ship state (pos+vel history) ring-buffer. Forgiveness mechanic that stays skill-expressive.
**[Reverse #5] Precognition** *(Tension design)* — accepted. Before imminent death, game shows ~1–2s warning allowing reaction. Narrative-integrable ("drone core anticipates disasters").
**[Reverse #6] Sensor-first perception** *(Info asymmetry)* — accepted. Enemies primarily perceivable via spatial audio + sensor signatures, not direct sight. **This is the ANSWER to the earlier P#7 accessibility rejection** — audio-first becomes core design, not bolt-on mode.
**[Reverse #7] Pay-to-shoot economy** *(Economy inversion)* — accepted. Each shot costs salvage currency; no-shot-runs yield huge meta-bonus. Splits playstyle into pacifist vs. aggressive naturally.
**[Reverse #8] Tower Defense mode** *(Genre flip, code reuse)* — accepted. Crafting modules become stationary turrets defending a station from incoming asteroids. Reuses C#6 weapon crafting in a second gameplay context. Low-effort bonus mode.

**Rejected:** #1 (play-as-asteroid, off-theme), #3 (start OP / lose power, fights his progression vision), #9 (reverse difficulty curve), #10 (reverse chronology narrative).

**Till's selection pattern:** Two clear themes emerge:
- **Pacifist/stealth viability** (#2, #6, #7) — he's designing a game where not-shooting is a valid strategy, not a meme. Huge differentiator.
- **Forgiveness without difficulty softening** (#4, #5) — error-recovery mechanics that stay skill-expressive.
- **Code/system reuse** (#8) — tower defense is free bonus mode if crafting system exists anyway.

**Critical revelation:** R#6 + P#7 reconciliation — audio-first perception belongs to **core design**, not accessibility-as-optional-mode. This is the deepest insight of Phase 1.

---

### Phase 2 — Mind Mapping Summary

**Central concept:** "Roguelite Salvage-Survival with Asteroids-grade dogfight core."

**6 dimension-branches** mapped (see detailed tree in session transcript):
1. Feature & Mechanics — organized into Progression, Gameplay-Core, Ship-Feel, Forgiveness sub-branches
2. Differentiation — Narrative USP, Pacifist/Stealth viability, Visual Identity, Sensory Identity
3. Architecture & Tech — open Engine/Physics decisions, rendering stack, ECS composition pattern
4. Product & Scope — marketing, UGC, scope reducers, career leverage, future platforms
5. Edge Cases & Risks — 2 spec-tensions (E#4, E#6), tech risks, solo-dev scope risks
6. Holistic / Identity — 4 emergent guiding principles

**4 Guiding Principles (emergent from clustering):**
1. 🌟 **Pacifist viable, not a gimmick** (R#2+R#6+R#7+A#4 form a real system)
2. 🌟 **Atmospheric over arcade** (S-picks + M-picks + Audio harmonize)
3. 🌟 **Forgiveness without softening** (M#3+M#9+R#4+R#5 — errors teach, skill matters)
4. 🌟 **Dual-use: Game + Bevy Portfolio** (every tech decision serves both goals)

**High-Leverage Cross-Connections (ideas serving multiple branches):**
- C#6 Modular Weapon Crafting → Features + Differentiation + Modding + Tower Defense
- R#6 Audio-First Perception → Differentiation + Accessibility (replacing P#7) + Tech learning
- M#10 Vector Aesthetic → Differentiation + Scope-reducer + Marketing + Career
- S#5 Salvage Drone Identity → Narrative spine justifying Crafting, Partial Death, Pacifist runs
- C#3 Caravan → Core mode + eliminates mode-selection UI + natural roguelite run shape

**Till accepted full Phase 2 framing and requested all 7 candidates be evaluated in Phase 3.**

---

### Phase 1 Summary

**SCAMPER complete.** ~58 seed ideas surfaced across all 7 lenses with deliberate domain pivots every ~2 ideas (anti-bias protocol honored).

**Curated totals:**
- S: 4 accepted (Gravity Swarm, Fyrox option, Salvage Drone, Orbital Loop)
- C: 4 accepted + 1 new tech question (Avian vs Rapier) — Roguelite Meta, Bevy+Avian, Caravan, Weapon Crafting
- A: 6 accepted — Heat modifiers, Flight-feel, KSP-lite, Subsystem damage, Audio-driven, ECS composition
- M: 10 accepted — full slate including Vector Aesthetic deep-dive
- P: 6 accepted, 4 rejected
- E: 10 deferred to Phase 3 for explicit debate (2 flagged as conflicting with original spec)
- R: 6 accepted, 4 rejected

**Emergent game concept (the "Spine"):**

> **Working title-space: "Roguelite Salvage-Survival with Asteroids-grade Dogfight Core"**
>
> - **Identity:** AI salvage drone recovering a dead civilization (S#5)
> - **Journey:** Caravan mode — A→B with opt-in arena pockets (C#3)
> - **Run structure:** Roguelite with soft-permadeath "core survives" (C#1 + M#9)
> - **Progression:** Modular weapon crafting from salvaged modules (C#6), Hades-style heat modifiers (A#1)
> - **Feel:** Everspace-style flight with KSP-lite predictable orbits (A#2 + A#3), decoupled aim (M#7), optional bullet-time/rewind (M#3 + R#4)
> - **Visual identity:** Vector/Tron aesthetic, monochrome+accent, mega-wreck structures for scale (M#10 + E#9 + M#1)
> - **Combat/Stealth:** Pacifist runs viable; audio-first enemy perception (R#6 + R#7)
> - **Ship state:** FTL-style subsystem damage + cockpit-pet companion (A#4 + M#2)
> - **Bonus Mode:** Tower Defense reusing crafting system (R#8)
> - **Tech:** Bevy + Avian (or Rapier) + `bevy_mod_outline` + custom toon shader (C#2 + A#6)

**Tech Pins (resolved in prior session, carried into this brainstorming):**
1. ✅ **Engine: Bevy**
2. ✅ **Physics: Avian** (Bevy-native, ECS-idiomatic XPBD physics)

**Open Design Tensions for Phase 3:**
1. E#4: Drop classic Arena mode? (conflicts with original 2-mode spec) — ✅ **resolved, see Phase 3 below**
2. E#6: Cockpit-only, drop 3rd-person? (conflicts with original 2-cam spec) — ✅ **resolved, see Phase 3 below**

---

### Phase 3 — Six Thinking Hats

#### Candidate E#6 — Cockpit-only vs. 2-Camera Spec

**Question:** Should we drop the 3rd-person camera from the original spec and commit to cockpit-only?

##### ⚪ White Hat — Facts

- **Original spec:** cockpit + 3rd-person (rear/top, adjustable). E#6 proposes: cockpit-only.
- **Spine features coupled to camera:** S#5 Salvage Drone narrative, M#2 Cockpit-Pet (invisible without cockpit), M#5 Blind Flight (only sensible in cockpit), R#6 Audio-first perception (compensates narrow FOV), M#7 Decoupled Aim, P#8 VR Port (natural fit for cockpit), M#10 Vector Aesthetic (stronger poster-shot in 3rd-person).
- **Engineering reality (Bevy, solo, Rust beginner):**
  - Cockpit-only: 1 interior model, screen-space HUD, fixed camera transform. Low complexity.
  - 3rd-person: exterior ship model, spring-arm camera controller with collision avoidance (asteroids!), smoothing, occlusion. Bevy has no canonical 3rd-person controller — self-built. Non-trivial.
  - Both: dual HUD layouts, doubled QA, both interior + exterior ship assets.
- **Genre references:** Everspace 2/3 (both, 3rd-person default), Elite Dangerous (cockpit-only, identity anchor), Star Fox 64 (3rd-person exclusive), House of the Dying Sun (cockpit-only with strong sensor/UI identity).

##### 🔴 Red Hat — Emotions

**Till's gut signal:** _"Sicherheit durch Wechseln der Sicht"_ (safety of being able to switch view).

Initial emotional pull toward 2-camera-spec as a safety net. Anxiety around "what if I don't like cockpit when it's done" and loss of poster-shot marketability. Counter-pull toward cockpit-only coming from focus-mut, identity coherence, and salvage-drone embodiment.

##### 🟡 Yellow Hat — Benefits (both sides)

**Cockpit-only:** Identity density (all 4 guiding principles sharpen simultaneously), ~2–4 weeks saved on camera controller work, VR path open by default, genre differentiator, audio-first becomes *necessary* not optional, M#2 Cockpit-Pet gets permanent stage.

**2-Camera-spec:** Player compatibility (motion-sickness tolerant), stronger marketing screenshots, debug leverage during development, gameplay flexibility for boss encounters, Star-Fox-nostalgia hook, retreat path to "arcade feel" if game becomes too grüblerisch.

**Till's selection:** *"Identitätsdichte" (Cockpit) + "Debug-Leichtigkeit" (2-cam)* — picked the strongest point from each side. Signal: the answer is asymmetric, not binary.

##### ⚫ Black Hat — Risks (both sides)

**Cockpit-only:**
- 🚨 Cockpit model becomes critical path — no fallback if quality disappoints.
- 🚨 Spatial-awareness problem real — couples E#6 tightly to R#6 quality (make-or-break).
- 🚨 Motion-sickness audience (~15–30%) lost irreversibly.
- 🚨 M#7 Decoupled Aim visualization harder in narrow FOV.
- 🚨 Marketing screenshots weaker.
- 🚨 Self-doubt loop: no built-in fallback if cockpit-sentiment shifts mid-dev.
- 🚨 Precedent of overriding own spec — dangerous habit for hobby-scope stability.

**2-Camera-spec:**
- 🚨 Scope creep through undecidedness — dual HUD, dual QA, dual tutorial, dual control tuning. *Classic solo-dev killer over project lifetime.*
- 🚨 Identity dilution — becomes "yet another space shooter."
- 🚨 M#2, M#5, R#6 all degrade from core features to optional side-content.
- 🚨 Flight-feel (A#2) must be tuned twice — different FOV regimes.
- 🚨 Phantom-flexibility: "decide later" = never deciding.

**Key insight:** Cockpit-only risk is *quality-dependent*. 2-cam risk is *scope-dependent*. For solo hobby projects, scope risk is typically more lethal (burnout/boredom kills more projects than mediocrity).

##### 🟢 Green Hat — Creative Variants

Four variants of "3rd-person as dev/debug-option" explored:
- **A. Dev-Only, Release-Stripped:** `#[cfg(debug_assertions)]` toggle. Minimal scope (~1 day). Zero release benefit.
- **B. Photo/Replay-Mode = 3rd-Person System:** fuse with already-planned P#1. Photo Mode *is* the 3rd-person controller. Zero added scope. Solves marketing + dev-tool + replay + kill-cam (M#6) simultaneously.
- **C. Silent setting, full release parity:** rejected — covertly returns to 2-camera-spec via options menu.
- **D. Diegetic 3rd-person — Salvage Drone micro-drone:** narrative-grounded external sensor, gameplay feature not camera option. Strengthens spine but is not a dev tool. Candidate for post-MVP.

##### 🔵 Blue Hat — Decision & Synthesis

**✅ Decision: B + A combined.**
- **Gameplay:** 100% cockpit-only. No in-run camera switching.
- **Release:** Photo/Replay-Mode (reuses P#1) accessible only **after run-end** — serves marketing, kill-cam (M#6), community export, and posthumous ship-admiration.
- **Development:** Simple debug 3rd-person camera via `F3`/console toggle, stripped from release builds. ~1 day scope.
- **Variant D** parked as potential post-MVP narrative feature.

**Ripple effects on Spine (automatically locked in by this decision):**
- R#6 Audio-first perception → **make-or-break core mechanic** (priority boost).
- M#2 Cockpit-Pet → permanent character anchor, not optional gimmick.
- M#5 Blind Flight → natural hardcore tier of the core immersion.
- P#1 Photo Mode → dual-role, priority lifted (marketing + 3rd-person access).
- M#6 Kill-Cam → shares camera code with P#1 (scope synergy).
- P#8 VR Port → pathway remains open by default.
- M#7 Decoupled Aim → new design task: out-of-FOV target indicator in cockpit.
- Cockpit model + HUD → **critical quality path**, early prototype in Phase 4 strongly recommended.

**Accepted trade-offs (explicit, not handwaved):**
- Motion-sickness audience lost (price of the handwriting).
- Cockpit quality risk carries no fallback (by design — fallback would undermine the decision).

**Method note:** Till surfaced the compromise himself in the Yellow Hat ("Identitätsdichte + Debug-Leichtigkeit") — Green Hat just operationalized it. This is Six Hats working as intended: parallel perspectives produce synthesis without debate.

---

#### Candidate E#4 — Drop Classic Arena Mode vs. Original 2-Mode Spec

**Question:** Should classic Arena mode be dropped entirely in favor of Caravan-only (C#3)?

**Resolution (decided in prior session, carried into this brainstorming as a fixed decision):**

✅ **Arena survives, but only as the Tutorial Zone. Primary focus is Caravan + Arena-Pockets embedded inside Caravan runs.**

##### Rationale (consolidated from prior-session reasoning)

- **Arena as narrative tutorial zone:** first encounter with combat happens in a bounded arena ("training simulation" / "drone memory echo" / equivalent diegetic framing), so E#3 "no tutorial" goal is preserved while still giving players a soft landing.
- **Arena-pockets (within Caravan) absorb the genre-classic gameplay moments** — players get the "destroy-all-asteroids" rush contextually during runs, not as a separate menu entry.
- **Development runway preserved:** Arena remains an early, low-complexity build target — combat core can be prototyped in Arena before the Caravan framework is built around it. Motivation and early-playability risks from dropping Arena entirely are mitigated.
- **Focus is unambiguous:** Caravan is the game. Arena is a service component (tutorial + pocket template), not a parallel mode competing for polish time.

##### Ripple effects on Spine

- **C#3 Caravan** is now the singular core loop. All progression, meta, difficulty-scaling (A#1 Heat) attach to Caravan.
- **M#8 Short Burst vs. Expedition** (3–5 min vs. 30–45 min) becomes two Caravan length presets, not Arena-vs-Caravan.
- **E#8 Curated 20–30 Run Skeletons** applies to Caravan only. Arena tutorial stays hand-designed single instance.
- **E#3 No Tutorial** is satisfied: the tutorial *is* an Arena, but framed diegetically — not a skippable text overlay.
- **R#8 Tower Defense** bonus mode retains its own identity (it is not "Arena with turrets") — remains post-MVP scope.
- **Arena asset reuse:** combat mechanics, enemy AI, weapon crafting, audio perception all prototype in Arena first, then graduate to Caravan. Scope-efficient.

##### Accepted trade-offs

- Classic-Asteroids-Score-Attack as a standalone menu mode is gone. Daily/seeded leaderboards, if added, attach to Caravan runs (P#3).
- Arena-as-dominant-mode nostalgia is absorbed into Arena-pockets rather than served directly.

---

---

#### Quick-Pass: Phase-2 Candidate Evaluation

Compact 3-hat treatment (Yellow / Black / Blue → decision) for each of the 7 Phase-2 cross-connection candidates.

##### 1/7 — C#6 Modular Weapon Crafting

**🟡 Yellow:** Content depth without asset mass (3×3×3=27 weapons from 9 assets). Strong spine synergy (S#5 salvage, C#1 meta-currency, R#8 turrets, P#9 modding). Emergent community build-sharing. ECS composition learning showcase.

**⚫ Black:** Combinatoric balancing hell (100+ effective weapons, solo QA nearly impossible). Crafting-UI is its own design problem in 3D. Tutorial burden conflicts with E#3. MVP-trap: 3 months in system before combat is polished. E#2 ("no prefab weapons, crafting only") forces crafting into MVP scope.

**🔵 Decision — Staged rollout:**
- **MVP:** 3 prefab weapon archetypes, no crafting UI. Combat-core first, playable in 6–8 weeks.
- **Post-MVP-1:** Crafting UI + 3 modules per slot (9 combos).
- **Post-MVP-2:** Modifier slot, scale to 27+.
- **E#2 retracted:** prefab weapons are allowed in MVP; crafting unlocks as mid-game feature later.
- **R#8 Tower Defense + P#9 Modding** explicitly post-MVP.

**Ripple:** Arena (as tutorial zone) is the natural first playground for prefab weapons. Crafting system lifts from mid-game progression moment onward.

##### 2/7 — R#6 Audio-First Perception

**🟡 Yellow:** Already elevated to make-or-break by E#6 decision. Strong USP (no other 3D space shooter leads with audio perception). Accessibility baked into core (resolves P#7 rejection retroactively). Narrative fit (drone "hears" the void via sensors). Portfolio value for P#6 (HRTF/spatial-audio via `bevy_kira_audio`).

**⚫ Black:** Audio quality is *invisible work* — months with no screenshot progress, motivation risk. Hardware variance (laptop speakers ≠ headphones ≠ surround) — bad playback breaks core mechanic. Headphone-dependency = UX barrier. Balancing audibility distance is hard to A/B test solo. Three learning curves stacked (Rust + Bevy + spatial audio).

**🔵 Decision — Two-stage implementation, sensor-UI primary in MVP:**
- **MVP:** Enemies *are* visible on sensor UI (cockpit radar) + basic stereo audio positioning. Sensor is primary info source; audio reinforces it.
- **Post-MVP:** Deepen HRTF spatial audio, make sensor UI *optionally reducible* (hardcore toggle). M#5 Blind Flight becomes the extreme stage.
- **Never pure audio-exclusive:** even in hardcore mode, minimal sensor markers remain.
- **Headphone recommendation** on splash screen, not enforced.

**Ripple:** R#6 stays as differentiator but risk shifts from "make-or-break" to "make-or-underwhelming" — game is playable with mediocre audio, extraordinary with great audio. M#5 Blind Flight becomes the explicit hardcore tier.

##### 3/7 — M#10 Vector Aesthetic

**🟡 Yellow:** Largest visible differentiator on Steam page — thumbnail recognition. Scope reducer: vector style forgives low-poly meshes (silhouette + outline carry). Asset pipeline shrinks (no PBR, no normal maps, no hi-res textures). Strong cockpit synergy — clean lines + emissive HUD shine in E#6 setup. Portfolio showcase for custom WGSL toon shader in Bevy (P#6). Photo Mode (P#1) produces definitive trailer material.

**⚫ Black:** Shader learning curve steep — WGSL + Bevy material system + post-processing pipeline, 3 stacked layers, 2–4 weeks before first visible prototype for a Bevy beginner. `bevy_mod_outline` third-party dependency — Bevy API breaks risk. "Indie monochrome" risk — can look like generic Limbo/Obra-Dinn clones if not distinctive. Emotional range loss if strict monochrome (no danger=red, safe=blue semantic cueing). Vector ≠ automatically good — quality depends on line-distance, outline-thickness logic, shader smoothing.

**🔵 Decision — Early tech-spike + keep accent-color space open:**
- **Week 2–4 of project:** dedicated tech-spike — one asteroid + ship with toon shader + outline, tested in cockpit view. Decide aesthetic commitment *after* seeing prototype, not before.
- **Fallback plan:** if tech-spike underwhelms, fall back to flat-shaded low-poly + simple rim-light. Less spectacular but still stylized, scope lower.
- **Accent-color space kept open** — E#9 strict monochrome+1 is **loosened**. Multiple accent colors allowed with semantic meaning (e.g. enemy=red, salvage=cyan, hazard=yellow). Color becomes info channel, not purely aesthetic.
- **`bevy_mod_outline` pinned** to a specific Bevy version to control upgrade risk.

**Ripple:** E#9 (monochrome+1) retracted in favor of "restrained palette with semantic accents." Photo Mode (P#1) priority boosted further — vector screenshots are definitive marketing asset.

##### 4/7 — S#5 Salvage Drone Identity — ❌ **REJECTED**

**Till's decision:** *"S#5 streichen. Damit werde ich nicht warm."*

This is a significant spine-level shift — S#5 was the narrative anchor that justified multiple connected features. Rejection is honored and final; features that leaned on S#5 need re-grounding.

**Features that lose their narrative justification and need replacement:**
- **M#9 Partial Death — "core survives":** what survives now? (pilot in escape pod? Neural backup? Respawn token?) — needs re-framing.
- **M#2 Cockpit-Pet:** whose pet? (pilot companion? Alien stowaway? AI copilot?) — needs re-framing.
- **R#2 Preserve-don't-destroy / R#7 Pay-to-shoot:** pacifist framing loses the "salvager bringing back fragments" rationale — needs economic/narrative replacement.
- **R#6 Audio-First:** diegetic rationale weakens ("drone hears via sensors" → "pilot uses sensor overlay because …") — mechanic survives, narrative framing is open.

**Features that survive S#5 rejection unchanged:**
- C#6 Weapon Crafting (modules come from "salvage," attribution of salvager is flexible)
- A#4 FTL Subsystem Damage (mechanical, narrative-neutral)
- M#10 Vector Aesthetic (independent of narrative)
- E#6 Cockpit-only (independent)
- M#5 Blind Flight (independent)
- C#3 Caravan, C#1 Roguelite Meta (structural, identity-agnostic)

**Replacement identity — Till's decision:**

✅ **Option 1 (narrative-light "just a pilot") as foundation, with Option 4 (explorer/archaeologist) as visual/atmospheric flavor only — no written lore.**

- **Primary:** no explicit protagonist framing, no lore chapters, no data shards, no story text. Zero writing debt.
- **Flavor-only explorer vibe:** ship design leans survey/research (sensor arms, scanner optics, not pure combat jet). Cockpit has science-instrument vibes, not military HUD. Destroyed asteroids occasionally *reveal* interesting artifacts visually (no text). Audio/color palette leans "cosmic mystery" over "war."
- **What is explicitly NOT done:** no written lore, no logbooks, no home-quest, no pilot name, no backstory, no faction.
- **Future-open:** if Till later becomes attached to a specific identity, it can be layered onto the mechanically-agnostic base without rework.

**Re-grounding of previously S#5-dependent features:**
- **M#9 Partial Death:** "escape pod with N% random equipment survives" — no narrative justification needed.
- **M#2 Cockpit-Pet:** optional unlockable cosmetic companion. Build it if inspired, drop it if not. No narrative obligation.
- **R#2 Preserve / R#7 Pay-to-shoot:** purely economic framing — intact-salvage yields higher reward than destroyed, shooting costs ammunition currency. Not pacifist philosophy, just profitability math.
- **R#6 Audio-First:** dry UX rationale — "cockpit FOV is limited, sensor overlay extends perception." No narrative dressing.

##### 5/7 — C#3 Caravan (core mode skeleton)

**🟡 Yellow:** Confirmed core mode by E#4 resolution. E#8 curated run-skeletons is the right framing — hand-designed structure, procedural decoration. M#8 short-burst/expedition maps cleanly to length parameters, not separate modes. Classic Hades/Slay-the-Spire progression arc.

**⚫ Black:** Complexity monster — path logic + pocket triggers + skeleton instantiation + progression state + reward attribution = 4–6 weeks Bevy work before combat tuning. "Path in 3D space" is an unsolved design problem (invisible line? HUD waypoints? Stellar navigation?). Skeleton content is content work — 20–30 hand-designed = weeks of level design solo. Balancing a 3–5 min run AND a 30–45 min run on the same system means two tuning curves.

**🔵 Decision — Reduced MVP, skeleton variety post-MVP:**
- **MVP:** 1 skeleton template, 3 difficulty variants (easy/medium/hard), 5–8 minute run length. No short/long length presets. Path = invisible target coordinate with **waypoint pointer in cockpit**, rendering-distance triggers pocket events.
- **Post-MVP-1:** 5–10 skeleton templates for real variance.
- **Post-MVP-2:** M#8 short-burst + expedition length presets.
- **Post-MVP-3:** scale to E#8 volume (20–30 skeletons).
- **Arena tutorial (from E#4)** is the only non-Caravan content in MVP — combat-core entry, transitions into Caravan.

**Ripple:** Caravan grundgerüst MVP target is ~3–4 weeks instead of 4–6. Earlier playable state.

##### 6/7 — C#1 Roguelite Meta + M#9 Partial Death

**🟡 Yellow:** Motivation loop — death never yields zero progress (meta-currency or lucky drop). Proven Hades/Isaac pattern. Retention without content bloat (20–40 hrs of novelty from few hand-designed items). Natural fit with Caravan runs. M#9 softens frustration from unfair deaths without erasing stakes. Both work without S#5 narrative — pure mechanical framing ("escape pod with N% inventory").

**⚫ Black:** Meta-unlock balancing trap — too strong → early-easy, too weak → grindy, solo tuning hard without playtesters. Two progression systems can collide (in-run vs. between-run). M#9 risks making death meaningless — tension killer if loss isn't meaningful. M#9 edge-cases (which modules survive? none-valuable case?). Save-system persistence adds testing burden.

**🔵 Decision — C#1 in MVP, M#9 post-MVP:**
- **MVP — C#1 Meta-Currency + Unlock Shop:** classic permadeath with meta persistence. 5–10 permanent unlocks (starter configs, extra ship slot, mild stat boosts, new prefab weapon).
- **Post-MVP — M#9 layered in once C#1 balance is felt.** Adding M#9 early debugs two variables simultaneously.
- **Caravan length 5–8 min MVP** keeps per-death loss moderate, reducing urgency of M#9.
- **Meta-unlocks intentionally small in MVP** (5–10 items, each with felt effect). Scale to 20–50 only after core stabilizes.

**Ripple:** Save/persistence system enters MVP scope (JSON/Serde file writer). M#9 moves to post-MVP roadmap.

##### 7/7 — A#4 FTL Subsystem Damage

**🟡 Yellow:** Depth without more enemies — weak hit can disable sensors, changing combat radically. Narrative tension beyond HP ("engine burning, drifting into asteroids"). Textbook ECS composition example (A#6) for Bevy didactics. Clean coupling with C#6 crafting (modules sort by subsystem) and C#1 meta (upgrade per-subsystem robustness). Enables "disable-don't-destroy" pacifist combat (R#2).

**⚫ Black:** 5 subsystems = 5× UI + 5× feedback + 5× balance curves. Movement/shooting failures risk frustration ("engine dead, can't steer"). Sensor damage interacts dangerously with R#6 — sensor-kill = blind. Tutorial burden vs. E#3 "no tutorial." MVP danger — designing combat around 5 vectors from start explodes balance work.

**🔵 Decision — Reduced subsystem model in MVP, staged expansion:**
- **MVP:** **2 subsystems** instead of 5 — *Hull* (classic total HP) + *Shields* (regenerating, protects Hull). ECS subsystem architecture learned, minimal subsystem flavor, HP-like feel.
- **Post-MVP-1:** Engine system (speed degradation, repair-kit mechanic).
- **Post-MVP-2:** Weapons system (weapon failure on hit).
- **Post-MVP-3:** Sensors system — **explicitly coupled to R#6 post-MVP phase** so audio-first and sensor damage tune together.
- **ECS architecture generic from day 1** — each subsystem a component, adding new ones is cheap. Main cost is balance, not code.
- **Enemy subsystems mirror-symmetric:** 2 in MVP (Hull + Weapons), scaling with player's.

**Ripple:** A#4 is an *architectural* MVP decision (cheap post-MVP expansion) rather than a *gameplay* MVP decision. Balance effort stays tractable. Sensor subsystem expansion gated on R#6 deepening.

---

**Phase 3 complete.** All 7 Phase-2 candidates resolved via 3-hat compact evaluation. S#5 rejected with explicit ripple handling. Two tensions (E#4, E#6) resolved via full 6-hat sequences. Tech pins (Bevy + Avian) confirmed as fixed inputs.

---

### Phase 4 — Resource Constraints / Motivation-Driven Milestone Map

**Input parameters (confirmed by Till):**
- Time budget: 4–8 hours/week (plan conservatively at 6h/week average).
- MVP deadline: none — horizon irrelevant.
- Blender + 3D-modelling skills: rudimentary, learning in parallel.
- M3 (Arena + enemies) valued as a genuine stop-and-ship fallback, not just theoretical.

**Reframing:** With no deadline, the binding constraint is not time-to-ship but **motivation preservation across a 10–14 month calendar**. Hobby projects die from long stretches without visible progress, not from hours. Milestone ordering is therefore optimized so every milestone yields a perceptibly improved playable state.

**Asset-estimate adjustment:** baseline 40h asset work bumped to ~60h to account for parallel Blender learning on rudimentary base skills.

#### Milestone Map

| # | Name | Weeks | Hours | Delivers |
|---|------|-------|-------|----------|
| M0 | Hello Bevy | 1–2 | ~12h | Ship renders, WASD cockpit movement. Foundation. |
| M1 | Vector-Spike Decision | 3–4 | ~12h | Toon-shader + outline prototype. Go/fallback decision. |
| **M2** | **Arena-Tutorial playable** 🏁 | 5–8 | ~24h | First Playable. Prefab weapon, asteroid spawner, hit detection, HUD basics. |
| M3 | Enemies alive 🏁 | 9–12 | ~20h | 1 enemy ship + AI, 2 more prefab weapons. **Stop-and-ship fallback target.** |
| M4 | Ship has state | 13–16 | ~18h | Hull + Shields subsystems, save system skeleton. |
| M5 | Caravan runs ⚠️ | 17–24 | ~40h | 1 skeleton, waypoint pointer, pocket triggers, 3 difficulty params. **Danger stretch** — slice into weekly sub-milestones. |
| M6 | Loop closes 🏁 | 25–28 | ~18h | Meta-currency + 5 unlocks + shop UI. Retention loop complete. |
| M7 | Ears on | 29–32 | ~18h | Sensor UI (cockpit radar) + stereo positional audio + SFX pass. |
| M8 | Post-Run Photo Mode | 33–36 | ~16h | Free-cam, DoF, screenshot/replay export. Marketing-ready. |
| M9 | Polish pass | 37–42 | ~24h | Balance tuning, audio-pass-2, UI polish, 2–5 more unlocks, crash fixes. |

**Totals:** ~200h implementation + ~60h assets + ~40h learning/research = **~300h over ~42 weeks (~10 months)** at 6h/week average.

#### M3 as stop-and-ship fallback (Till-confirmed priority)

Till explicitly values M3 as a genuine fallback exit. M0–M3 are therefore deliberately scoped to produce a **self-contained shippable small game** on Itch.io if Till chooses (or is forced by life) to stop there.

**Additional scope included in M0–M3 to make M3-ship viable:**
- Minimal title screen + restart flow (M2)
- Basic settings (volume, sensitivity) (M3)
- Arena-score loop with restart (M3)
- Basic credits/about screen (M3)

These small additions should add ~4–6h across M2/M3, already budgeted within the stated hours.

#### Stop-and-ship waypoints

- **After M3 (~month 3):** "3D Vector-Asteroids with cockpit + enemies." Shippable as Itch.io prototype. Complete small game.
- **After M6 (~month 7):** "Roguelite 3D Arena-Asteroids with progression." Commercially viable as Itch.io or Steam Early Access.
- **After M9 (~month 10):** Full polished MVP.

#### Risk flags

1. **M5 danger stretch (6 weeks of Caravan framework).** Highest probability abandonment point. Must be sliced into weekly sub-milestones where each adds one visible feature (Week 1: waypoint pointer; Week 2: first pocket trigger; Week 3: difficulty curve; …). Do not commit to "I'll build Caravan for 6 weeks."
2. **Bevy API churn.** 10+ month timelines see 2–4 Bevy releases. Pin a version at project start (e.g., 0.15), only upgrade during M4/M6/M9 transitions. Migration churn otherwise eats weeks.
3. **4h-weeks more common than 8h-weeks.** Plan conservatively on 4h baseline — 8h weeks are bonus, not the plan.
4. **Asset scope underestimated potential.** 60h is still optimistic for someone learning Blender in parallel. Keep asset-store / OpenGameArt as fallback option — no purity about self-made assets.
5. **S#5 rejection locked; re-visiting costs weeks.** Narrative-light direction is now a load-bearing decision. If Till later wants a narrative anchor, it should layer on top of existing systems, not replace them.

---

### Session Complete

**Output artifact:** this document serves as the concept foundation for the project. The **Spine-Update after Phase 3** section + the **Milestone Map** in Phase 4 form the actionable handoff.

**Recommended next steps outside this brainstorming session:**
- Translate Spine + Milestones into a proper Product Brief (BMad skill: `bmad-product-brief`) or PRD (`bmad-create-prd`).
- Pin Bevy + Avian versions in `Cargo.toml` before starting M0.
- Treat M1 (Vector-Spike decision) as a hard checkpoint — document the outcome before committing to M2.







