---
stepsCompleted: ['step-01-init', 'step-02-discovery', 'step-02b-vision', 'step-02c-executive-summary', 'step-03-success', 'step-04-journeys', 'step-05-domain (skipped - low complexity)', 'step-06-innovation', 'step-07-project-type', 'step-08-scoping', 'step-09-functional', 'step-10-nonfunctional', 'step-11-polish', 'step-12-complete']
completedAt: '2026-04-21'
inputDocuments:
  - _bmad-output/brainstorming/brainstorming-session-2026-04-21-1114.md
documentCounts:
  briefs: 0
  research: 0
  brainstorming: 1
  projectDocs: 0
workflowType: 'prd'
project_name: 'asteriods3D'
user_name: 'Till'
date: '2026-04-21'
classification:
  projectType: 'desktop_app (game variant)'
  domain: 'gaming'
  domainComplexity: 'low'
  technicalComplexity: 'medium-high'
  projectContext: 'greenfield'
  notes: |
    Game-redirect flag in BMad CSV acknowledged; proceeding with PRD flow
    because narrative-light decision eliminates GDD lore burden, and the
    brainstorming doc already covers GDD-equivalent content (spine,
    mechanics, identity, scope). PRD focuses on feature/epic/milestone
    translation for dev workflow.
fixedInputs:
  tech:
    - 'Bevy (version-pinned)'
    - 'Avian (Bevy-native XPBD physics)'
  designDecisions:
    - 'Cockpit-only (E#6 resolved)'
    - 'Caravan as core loop (E#4 resolved)'
    - 'Narrative-light pilot (S#5 rejected)'
    - 'Vector Aesthetic + toon shader (M#10)'
  bindingConstraint: 'Motivation preservation across 10-14 months at 4-8h/week'
  stopAndShipWaypoints:
    - 'M3: small shippable Itch.io prototype'
    - 'M6: Early Access viable'
    - 'M9: Full MVP polished'
---

# Product Requirements Document - asteriods3D

**Author:** Till
**Date:** 2026-04-21

## Executive Summary

asteriods3D is a cockpit-only 3D asteroids-style shooter with roguelite progression, built in Rust + Bevy as a solo hobby project with dual learning and portfolio goals. The core experience is immersive space flight through a single first-person cockpit window — no external camera, no narrative cutscenes, no lore overlay. Players pilot through procedurally-decorated handcrafted "Caravan" runs (A→B traversal with opt-in arena combat pockets), managing a salvage economy where shooting costs currency and intact asteroids yield higher rewards. Progression is driven by meta-currency unlocks and (post-MVP) a modular weapon-crafting system. The 10-month milestone roadmap has stop-and-ship waypoints at M3 (shippable Itch.io prototype), M6 (Early Access viable), and M9 (full MVP).

Target audience: players who value atmospheric sim-lite flight over pure arcade reflex, who appreciate roguelite retention loops (Hades / Returnal / FTL sensibilities), and who are drawn to distinctive visual signatures. Secondary audience: Rust/Bevy developers following a public learning-artifact game project.

Problem framing: existing "Asteroids" inheritors are overwhelmingly arcade-pure (Geometry Wars, Super Stardust) or fully 3D external-cam space shooters (Everspace, Chorus). There is a structural gap between "reflex arcade" and "AAA space sim" — an immersive-cockpit Asteroids that treats asteroids as moving resources rather than obstacles to erase.

### What Makes This Special

Four differentiators compound into a coherent identity:

1. **Cockpit-only as identity anchor.** Unlike genre peers that toggle between cockpit and third-person, asteriods3D commits. This elevates otherwise-optional features (cockpit-pet companion, blind-flight hardcore mode, sensor-driven perception) into core mechanics, and preserves a natural VR pathway by default.

2. **Pacifism as viable economy, not gimmick.** Pay-to-shoot currency cost, intact-salvage yield premium, and audio-first enemy perception form a single mechanical system where non-lethal play is a real strategy — not an achievement niche. No written philosophy; just profitability math.

3. **Vector aesthetic in 3D.** Tron/Rez-adjacent toon-shaded vector style serves three roles simultaneously: distinctive Steam-thumbnail signature, scope-reducer (forgives low-poly meshes, shrinks the asset pipeline), and dedicated post-run photo-mode marketing vehicle.

4. **Forgiveness without softening.** Bullet-time, short-rewind, death-precognition, and partial-death mechanics enable error recovery while preserving skill expression. Skill still matters; frustration deaths don't.

Core insight: the Asteroids lineage is conventionally treated as an arcade reflex genre. asteriods3D treats it as an **immersive physics and economy puzzle inside a cockpit** — asteroids are resources in motion, the player is a pilot-in-space with limited FOV and extended senses, and the game rewards reading the space as much as shooting it.

Value proposition: *"3D Asteroids as it must feel from the cockpit — with vector aesthetics, audible space, and an economy where not shooting is a valid strategy."*

## Project Classification

- **Project Type:** Desktop application (game variant) — standalone Rust/Bevy binary, cross-platform (Windows / Linux / macOS), distributed via Itch.io for MVP and Steam post-MVP.
- **Domain:** Gaming — low regulatory and compliance complexity, standard software hygiene only.
- **Technical Complexity:** Medium-high — 3D rendering, custom WGSL toon shader, Avian XPBD physics, ECS composition, spatial audio, save-state persistence; stacked on the Rust + Bevy learning curve for a solo beginner developer.
- **Project Context:** Greenfield — no existing codebase, no legacy constraints; tech stack pinned (Bevy + Avian at specific versions).
- **Development Mode:** Hobby / solo at 4–8 h/week average; motivation preservation across a 10-month horizon is the binding constraint, not time-to-ship.

## Success Criteria

### User Success

A player's experience succeeds when these are true on first playthrough:

1. **Cockpit feel lands within 5 minutes.** Motion-sickness risk is managed; the HUD is legible; the player understands they're a pilot, not a camera operator. First-run dropout for cockpit-discomfort reasons is minimal.
2. **First "aha" moment: the economy is a real decision.** Within the first run, the player consciously holds fire on an asteroid to preserve it for higher salvage yield — and feels this as clever play, not forced pacifism.
3. **Audio extends the cockpit.** Within the first enemy encounter, the player identifies an unseen threat from the sensor/audio cue before it enters visual range and uses that information tactically.
4. **Retention signal at first death.** On first permadeath, the player sees meta-currency banked, understands they've made lasting progress, and starts a second run. "One more" behavior is the canonical success indicator.
5. **Photo-mode activation after a satisfying run.** A player voluntarily engages post-run photo mode and exports at least one screenshot or short clip. Signals the vector aesthetic is working as both play experience and identity artifact.
6. **The cockpit feels owned.** By run 5-10, the player has a preferred cockpit-pet (if unlocked), has mapped controls to their muscle memory, and notices if instrument feedback is off. Immersion has stabilized.

### Project Success

This is a solo hobby project with optional commercial realization. Success is measured primarily against the milestone map (Phase 4) and secondarily against external reception if and when shipped.

**Milestone Success (primary):**
- **M3 ship-readiness (≈month 3):** "3D Vector-Asteroids with cockpit + enemies" shippable as an Itch.io prototype. Represents a complete small game — stop-and-ship fallback holds.
- **M6 ship-readiness (≈month 7):** Roguelite loop closed. Viable as Itch.io release or Steam Early Access. Represents commercial minimum.
- **M9 ship-readiness (≈month 10):** Polished MVP.
- **Motivation preservation:** Till remains actively motivated through the 10-month horizon without extended abandonment. No week-long stretch should end in "why am I doing this" without a visible win.

**Learning / Portfolio Success (primary secondary):**
- Till is fluent in ECS-idiomatic Bevy by M3 (no more pattern-matching to OOP), comfortable with Avian physics by M4, and can explain the toon shader from scratch by M1.
- The public repository (if made public) and any companion blog posts produce plausible Bevy-community-visible artifacts that could function as portfolio evidence.

**Commercial Reception (optional, only if released):**
- Itch.io prototype (M3): any positive community engagement — even a small Rust/Bevy subcommunity following — counts as success at this stage. No revenue target.
- Steam / polished release (M9 onward): Steam "Mostly Positive" or better, or Itch.io ≥ 3.5/5 average rating, should the project proceed to paid release. Revenue is explicitly not a success metric; break-even on tool costs (asset packs, Steam fee if relevant) is sufficient.

### Technical Success

- **Performance:** 60 FPS sustained on a 2020-era mid-range GPU (GTX 1060 / RX 580 class) at 1080p with vector aesthetic active.
- **Stability:** Zero-crash Arena (M3) and Caravan runs (M5 onward) in normal play. Save/load is deterministic — no progress loss on crash or quit.
- **Cross-platform parity:** Windows, Linux, and macOS are all first-class targets from M0. No platform is de-prioritized; each milestone gate requires successful build and playtest on all three.
- **Bevy / Avian version discipline:** Versions pinned at project start. Upgrades happen deliberately at M4, M6, and M9 transitions only, with a migration-hour budget tracked against the milestone.
- **ECS hygiene:** All gameplay entities use component composition (no god-structs, no inheritance-shaped ECS). Becomes the portfolio-worthy code quality signal.
- **Audio pipeline:** Stereo spatial positioning works correctly on headphones by M7. HRTF / deeper spatialization is post-MVP.

### Measurable Outcomes

| Outcome | Target | Measured When |
|---------|--------|---------------|
| M3 build shippable to Itch.io | Zero-crash 3-run playtest passes | M3 gate |
| M6 build roguelite loop complete | 5 meta-unlocks playable, save persists across 10 runs | M6 gate |
| M9 MVP polished | All MVP success criteria above are true | M9 gate |
| 60 FPS target | Sustained on GTX 1060 at 1080p | Each milestone gate |
| Learning velocity | Till self-rates Bevy fluency "comfortable" | M3, M6, M9 retros |
| Motivation streak | No ≥2-week stretch without commit or play-test | Ongoing, reviewed monthly |
| Bevy upgrade survival | Project survives at least one Bevy minor version bump | M6 or earlier |

## Product Scope

### MVP — Minimum Viable Product

The MVP is the full M0–M9 milestone map (~300 h, ~42 weeks at 6 h/week average). Stop-and-ship waypoints at M3 and M6 are designed into scope so that partial completion still yields a shippable artifact.

**MVP includes:**
- Cockpit-only camera, single first-person view
- WASD + mouse flight controls with decoupled aim (M#7), toggleable inertial dampener (A#2)
- Arena tutorial zone (E#4 resolution): one hand-designed combat arena serving as diegetic tutorial
- Caravan mode (C#3, reduced MVP): 1 run-skeleton template, 3 difficulty variants (easy / medium / hard), 5-8 minute run length, waypoint-pointer navigation, rendering-distance pocket triggers
- Ship state (A#4, reduced MVP): 2 subsystems only — Hull (total HP) + Shields (regenerating)
- Combat: 3 prefab weapon archetypes (no crafting UI in MVP per C#6 staged rollout)
- Enemies: 1 enemy ship type with basic AI, introduced in Arena, scaling in Caravan (A#1 Heat modifiers deferred to post-MVP)
- Roguelite meta (C#1): meta-currency, 5–10 permanent unlocks, unlock shop UI. Classic permadeath (M#9 Partial Death deferred post-MVP).
- Vector aesthetic (M#10): custom WGSL toon shader + outline, restrained palette with semantic accent colors (E#9 loosened — enemy/salvage/hazard colors allowed)
- Audio (R#6 reduced MVP): sensor UI (cockpit radar) + basic stereo positioning. Sensor is primary info source; audio reinforces.
- Post-run photo mode (P#1): free-cam, DoF, screenshot / short clip export. Marketing-ready.
- Title screen, restart flow, basic settings (volume, sensitivity), credits screen
- Save/load (JSON + Serde): persistent meta-currency and unlocks
- Tech: Bevy (pinned), Avian (pinned), bevy_mod_outline (pinned), bevy_kira_audio (pinned)

**MVP explicitly excludes (deferred to Growth):**
- Modular weapon crafting UI (C#6 full)
- Sensors / Engine / Weapons subsystems (A#4 full 5-system model)
- Partial Death (M#9)
- Heat modifiers (A#1)
- Short-burst / Expedition length presets (M#8)
- Multiple run-skeleton templates (E#8 curated 20–30 variants)
- Cockpit-pet (M#2) unlock system
- Kill-cam (M#6)
- Bullet-time / Rewind / Precognition (M#3, R#4, R#5)
- Blind Flight hardcore mode (M#5)
- Tower Defense bonus mode (R#8)
- HRTF / deep spatial audio

### Growth Features (Post-MVP)

Ordered by strategic value, not chronology. Specific sequencing decided per-unit once MVP ships.

1. **Modular weapon crafting (C#6) — full rollout in two stages.** Post-MVP-1: crafting UI + 3 modules per slot (9 combos). Post-MVP-2: modifier slot, scale toward 27+ combinations. E#2 "crafting-only" remains retracted.
2. **Subsystem expansion (A#4).** Engine (speed degradation + repair kit), then Weapons (weapon failure on hit), then Sensors (explicitly coupled with R#6 audio-first deepening so they tune together).
3. **Partial Death (M#9).** Layer onto C#1 meta once the permadeath balance is felt. Escape-pod-with-N%-random-modules framing.
4. **Heat modifiers (A#1).** Hades-style difficulty toggles for higher meta-currency yield.
5. **Run variety (E#8).** Scale from 1 → 5–10 → 20–30 hand-designed run skeletons. Procedural decoration on top.
6. **Session length presets (M#8).** Short-burst 3–5 min + Expedition 30–45 min as Caravan length parameters.
7. **Audio-first deepening (R#6 stage 2).** HRTF, positional-audio polish, optional sensor-UI-reduction as Blind Flight hardcore mode (M#5).
8. **Atmospheric polish.** Mega-wreck structures (M#1), boss asteroids (M#4), cockpit-pet (M#2), kill-cam (M#6).
9. **Forgiveness mechanics (M#3, R#4, R#5).** Bullet-time, rewind, precognition. Fine-tuned against C#1 balance already established.
10. **Tower Defense bonus mode (R#8).** Re-use crafting-module system in a stationary-turret context. Low-effort secondary mode.
11. **Asteroid motion — Kepler orbits / scripted splines (Design Principle 3 restoration).** MVP ships static asteroids by design-principle deferral (2026-04-22). Post-MVP introduces deterministic trajectory parameters so asteroid-reading becomes a real player skill. Elevated strategic priority: restores a PRD design-principle commitment rather than adding a flourish. Candidate for first or second post-MVP slot depending on M6 playtest signals.

### Vision (Future)

Explicitly "if the project lives a long life" — not a plan, a direction.

1. **VR port of cockpit mode (P#8).** Cockpit-only identity makes this the natural future platform. Scope pathway preserved by MVP decisions.
2. **Modding API over weapon crafting (P#9).** Crafting modules exposed to players via scripting (e.g., Rhai). Community-driven content pipeline.
3. **Seed-based level sharing (P#3).** Players identify runs by seed, share interesting seeds socially. Zero-server UGC.
4. **Orbital Mechanics Lab sandbox (P#4).** Edu spin-off exposing the KSP-lite physics layer. Astronomy / education-market play.
5. **Diegetic micro-drone third-person (Variant D from E#6).** Narrative-grounded external sensor — not a camera option, a gameplay feature. Only if a narrative anchor ever emerges.

## User Journeys

For a single-player offline game, the meaningful journey set is narrower than for most software products. Admin, support, operations, and API journeys are explicitly not applicable (no backend, no server, no external surface, no support team). Three player-facing journeys plus one aesthetic / marketing journey cover the relevant interactions.

### Journey 1 — The Newcomer (First 30 Minutes, Success Path)

**Persona:** Max, 28, software engineer. Plays roguelites in the evenings (Hades, FTL, Returnal). Saw a vector-style cockpit screenshot on Mastodon, bought the game on Itch.io for €8. No prior knowledge.

**Opening scene.** Title screen. Minimal — a ship silhouette, a single accent-color highlight, three menu options. No wall-of-text tutorial. He clicks "Begin."

**Rising action.** Arena tutorial zone. He spawns in the cockpit. His first instinct: "this isn't a camera, I'm in here." He can see his wingtips through the window. Controls feel like flying, not driving. First asteroid is slow and large — clearly shootable. He fires. It fragments. A small green marker pulses on the radar. "Huh." He drifts to the marker, collects the salvage. The score UI stays subtle.

**"Aha" moment.** Second encounter. A slow, bright asteroid. As he aims, the HUD shows a subtle indicator: *Intact +15 salvage / Destroyed +3*. He pauses, boosts around it, grabs it whole with his tractor beam. The reward bar jumps. He grins. The economy is a real choice, not a moralistic nudge.

**Climax.** First enemy ship. He doesn't see it. A soft low-frequency tone fades in, spatially positioned behind-right. He glances at the radar — red dot. He banks, fires, it dies. He consciously registers: *the sensors and audio extended my awareness beyond the cockpit window.*

**Resolution.** Arena ends. Transitions to his first Caravan run. He picks medium difficulty. Six minutes of flight with pocket encounters. He dies around minute four to a mistake he understands. Meta-currency banks. He unlocks his first permanent upgrade (ammo capacity +20%). Without thinking, he clicks "Next Run."

**Emotional arc:** curious → engaged → surprised → rewarded → hooked.

**Journey reveals requirements for:**
- Minimal title screen + menu flow
- Arena → Caravan transition without friction
- Cockpit rendering with wingtip-visible framing (spatial anchoring)
- HUD clarity: salvage-economy indicator, shields, hull, ammo
- Radar / sensor UI with threat markers
- Spatial stereo audio on headphones
- Save/load meta-currency persistence across run boundaries
- Unlock shop with at least one immediately-acquirable upgrade
- Tractor-beam intact-asteroid pickup mechanic
- Subtle, legible information design (no tutorial text)

### Journey 2 — The Engaged Player (Run 20, Advanced / Pacifist Specialization)

**Persona:** Max again, three weeks and roughly 20 runs later. Has unlocked all 10 MVP meta-unlocks and tried each weapon archetype. He's now experimenting with playstyles the game implicitly encourages.

**Opening scene.** He starts a run on Hard difficulty. Equips minimal weaponry. Self-imposed goal: "complete this Caravan without destroying a single asteroid."

**Rising action.** He evades enemy ships rather than engaging. Boosts through denser asteroid fields. Tractor-beams intact rocks at every opportunity. His salvage-per-minute metric climbs unusually high — the economy *rewards* this playstyle structurally, not rhetorically.

**Climax.** An enemy ship corners him. In the MVP he has no disable-don't-destroy weapon option — that's post-MVP (A#4 subsystem expansion). He has to break his self-imposed pacifism to survive. He does it. Mutters "fine." Survives.

**Resolution.** Finishes the run with his highest salvage haul yet. Unlocks a cosmetic. More importantly: realizes the game has a playstyle *he discovered*, not one the tutorial handed him. Doesn't close the game — starts another.

**Emotional arc:** curious → committed → creative-constrained → mildly compromised → validated → hungry.

**Journey reveals requirements for:**
- Difficulty levels that alter enemy density / aggression, not just HP sponges
- Tractor-beam reliable on intact asteroids of varying masses
- Salvage-per-minute feedback loop legible to the player
- Progression depth beyond first 10 unlocks (flags Growth-stage roadmap priority)
- Post-MVP hook: disable-don't-destroy mechanics (subsystem-damage weapons in A#4 expansion)
- Balance question (design decision, not a feature): should pacifist runs yield comparable or superior meta-currency?

### Journey 3 — The Frustrated Player (Failed-Run Recovery, Edge Case)

**Persona:** Sarah, 35, enjoys FTL and Slay the Spire. Downloaded the Itch.io build. Patient but not infinitely patient. On her 4th run, she dies to an enemy she never saw on screen.

**Opening scene.** Mid-run, roughly four minutes in. She's feeling confident. Suddenly shields drop, then hull. Dead.

**Rising action.** She's angry. "I didn't even see that enemy." She considers quitting. Hovers over the close button. Hesitates.

**Climax.** She notices the meta-currency banked from the failed run. Checks the unlock shop. One option catches her eye: *Sensor Range +15%*. She spends her currency. Starts a new run.

**Resolution.** Mid-run, an enemy appears on her expanded sensor a beat earlier than last time. She disengages, survives, finishes. Her frustration has been channeled into progression, not quitting. She's now strategically prioritizing which unlocks to buy next.

**Emotional arc:** confident → ambushed → angry → tempted-to-quit → curious-about-unlock → restored → strategic.

**Journey reveals requirements for:**
- Clear post-death feedback — what killed me, where was it, why didn't I see it
- Meta-currency banked indicator visible within 2 seconds of death
- Unlock shop legibility — each unlock must read as a clear answer to a felt problem
- Audio-cue volume and clarity on mid-tier hardware (laptop speakers risk)
- Headphone recommendation on splash screen (from R#6 design)
- Save/load reliability post-death — losing currency to a crash is the one retention-killing scenario

### Journey 4 — The Photographer (Post-Run Aesthetic Artifact, Marketing Vehicle)

**Persona:** Elena, 24, art-school alumna. Curates an indie-games aesthetics feed on social. Bought the game specifically because of the vector screenshot on the Steam page.

**Opening scene.** Finishes a run, dies mid-Caravan. The death screen offers a subtle "Photo Mode" prompt alongside the standard "Retry / Menu."

**Rising action.** Enters photo mode. Free-cam orbits her dead ship, scattered intact salvage, and a half-collapsed mega-wreck in the distance. Toon shader + outline makes every frame poster-ready. She rotates camera angles, adjusts depth-of-field, catches a clean silhouette of her ship against the mega-wreck.

**Climax.** Exports a PNG at 1080×1920 (the 9:16 preset). Opens social. Posts it with the game tag.

**Resolution.** Next day, two followers DM her asking "what game is that?" The aesthetic is quietly doing marketing work — zero cost to Till, zero effort to Elena beyond an action she enjoys anyway.

**Emotional arc:** satisfied → creative → proud → promotional (as genuine creative self-expression, not ad-hoc).

**Journey reveals requirements for:**
- Photo mode UI accessible from end-of-run screen only (not mid-run)
- Free-cam with orbit + dolly + DoF
- Time freeze (inherited for free — post-run means the simulation is already paused)
- PNG export at multiple aspect-ratio presets (16:9 landscape, 9:16 portrait, 1:1 square)
- Vector aesthetic quality preserved at 360° camera angles (not only cockpit POV — shader must survive external viewing)
- Subtle watermark or game title in exported images (marketing back-link; can be toggleable)
- Post-MVP: short-clip export (MP4 or GIF)

### Journey Requirements Summary

Cross-journey capability inventory, grouped by MVP readiness.

**Required in MVP (all four journeys):**
- Cockpit rendering with spatial anchoring (wingtip visibility)
- HUD with salvage-economy, shields, hull, ammo
- Radar / sensor UI with threat markers and range tuning
- Spatial stereo audio positioning (headphone-optimized)
- Arena tutorial → Caravan transition flow
- Save/load with meta-currency persistence
- Unlock shop (5-10 unlocks minimum)
- Death screen with feedback (what killed me, currency banked)
- Title screen + menu flow + restart loop
- Photo mode with free-cam, DoF, multi-aspect PNG export
- Tractor-beam intact-asteroid pickup
- Difficulty levels that alter density / aggression

**Flagged as design decisions (not pure feature items):**
- Pacifist-run meta-currency balance (Journey 2) — design decision needed before M6
- Information-design discipline: no tutorial text, everything learned via play + diegetic cues (Journey 1)

**Explicitly non-journeys for this product:**
- No admin / operations user journey (no backend, no config surface)
- No support / troubleshooting journey (no support channel beyond Itch.io / Steam community thread)
- No API / integration user journey (no API surface in MVP; modding API only appears post-MVP as Vision-stage feature)

## Innovation & Novel Patterns

### Detected Innovation Areas

Two genuine innovation claims, beyond what "What Makes This Special" already stated:

1. **Pacifism as a mechanical economy (not an achievement niche).** The combined system of pay-to-shoot currency cost, intact-salvage yield premium, and audio-first perception is not a stylistic preference — it's a profit-math optimization problem. Space shooters historically place "non-lethal playstyle" in achievement lists, not at the heart of the economy. This project proposes it as the economy's default gradient.

2. **Asteroids reframed as resources in motion.** The genre convention is that asteroids are obstacles to erase. This project's core insight inverts that — they are moving resources whose destruction is strictly less profitable than capture. The player's question shifts from "which asteroid first?" to "which asteroid *and how*?"

### Market Context & Competitive Landscape

- **Direct comparables (none are full overlap):** Elite Dangerous (cockpit, no roguelite, no pacifism-economy), Everspace 2 (external-cam, no pacifism-economy), House of the Dying Sun (cockpit + tactical, but no roguelite, no salvage-economy), FTL (permadeath + roguelite but 2D, ship-command not pilot).
- **Indirect comparables (aesthetic):** Rez Infinite (audio-visual synesthesia, on-rails), Thumper (rhythm-violence, on-rails), Tron-adjacent vector games (various).
- **Gap:** no title occupies the intersection of (cockpit-only) × (roguelite progression) × (economic-pacifism-as-default) × (vector aesthetic in 3D). The gap is small because none of those dimensions individually is underserved — but the specific intersection is structurally empty.

### Validation Approach

- **M3 gate (internal):** Playtest by Till. The pacifism-economy loop must be *discoverable without explanation* — a new player must at some point spontaneously hold fire on an asteroid because the salvage math said so. If not, the signaling design needs tuning (not the feature removed).
- **M6 gate (community, if released):** Itch.io / Reddit commentary specifically naming the salvage economy or audio-first perception as reasons for engagement. Absence of such commentary = signal that the differentiator is not communicating; revisit messaging or mechanic intensity.
- **M9 gate (review):** Steam reviews mentioning economic choice or cockpit immersion more often than shooting or graphics suggests the design intent has landed.

### Risk Mitigation

- **If pacifism-economy doesn't land:** The economy gradient works at any pacifist-intensity level. If players ignore the intact-salvage premium and shoot everything, the game remains a competent cockpit arcade roguelite — degrades gracefully to a less-differentiated but still-playable product. No load-bearing feature depends on pacifism being embraced.
- **If audio-first perception doesn't land:** Per the brainstorming's Phase-3 resolution, R#6 is deliberately staged — MVP has sensor UI *plus* audio; audio-only is post-MVP hardcore mode. Fallback already baked in.
- **If cockpit-only limits audience:** Per E#6 Phase-3 resolution, this was an accepted trade-off (motion-sickness audience ~15-30% lost by design). Recovery path would be diegetic-micro-drone third-person (Variant D), but that's a Vision-stage feature, not a mitigation available during MVP.
- **If vector aesthetic tech-spike underwhelms (M1):** Per M#10 Phase-3 resolution, fall back to flat-shaded low-poly + simple rim-light. Less spectacular, scope lower, decision to be made at M1 gate.

## Desktop Game — Project-Type Specific Requirements

### Project-Type Overview

asteriods3D is a desktop game application built in Rust + Bevy, distributed as a standalone binary for Windows, Linux, and macOS. The primary distribution target for MVP is Itch.io; Steam is the secondary target post-M6. The application is fully offline — no server, no account system, no telemetry (E#5 resolved). Save data lives on-device per OS convention. Input is primarily keyboard + mouse, with controller support as a stretch MVP goal.

### Platform Support

- **Windows 10+** (first-class) — largest potential audience. WGSL → DirectX 12 via wgpu.
- **Linux** (first-class) — significant Bevy community overlap with Linux users. WGSL → Vulkan via wgpu.
- **macOS** (first-class) — Apple Silicon and Intel x86_64 both targeted. WGSL → Metal via wgpu. Each milestone gate requires successful build and playtest on macOS alongside Windows and Linux; no platform is de-prioritized.
- **No mobile, no browser/WASM** in MVP. WASM build is a Vision-stage possibility via Bevy's WebGL2 backend but deprioritized.

**Minimum hardware (targets):** GTX 1060 / RX 580 class at 1080p for 60 FPS on Windows and Linux. On macOS: Apple Silicon (M1 or newer) or 2016+ Intel Mac with discrete GPU, 60 FPS at 1080p equivalent. CPU: any 2016+ quad-core. RAM: 4 GB for the process.

### System Integration

- **Input:** Keyboard + mouse via Bevy's built-in input. Gamepad support (Xbox / DualSense / generic XInput) evaluated at M2; shipped if integration cost is < 4 h. Controller support is MVP-stretch, not MVP-required.
- **Graphics API:** wgpu abstraction (managed by Bevy). No direct Vulkan / Metal / DX12 calls. Custom WGSL shader for toon + outline. Shader validated on all three Metal / Vulkan / DX12 backends at M1 tech-spike.
- **Audio:** `bevy_kira_audio` as the audio backend. Stereo output in MVP. Headphones recommended; detection not attempted (show recommendation on splash screen per R#6 resolution).
- **Save-file location:** per-OS convention via the `directories` crate (Windows: `%APPDATA%/asteriods3D/`; Linux: `$XDG_DATA_HOME/asteriods3d/` or `~/.local/share/asteriods3d/`; macOS: `~/Library/Application Support/asteriods3D/`). JSON save format with Serde.
- **macOS signing / notarization:** code-signing and notarization required for macOS distribution outside the App Store. Budget 2–4 h at M3 milestone to establish signing flow with an Apple Developer account. Without notarization, Gatekeeper will block the app on first launch — non-negotiable for first-class macOS support.
- **No shell integration.** No file associations, no URL handlers, no system tray, no background processes.

### Update Strategy

- **MVP (Itch.io):** Manual update via the itch.io app or direct download. No auto-update mechanism built in-engine.
- **M6 onward (if Steam-released):** Steam handles auto-update natively. No additional in-engine update code required.
- **Self-hosted / DRM-free channels:** No in-engine auto-update planned. Optional HTTP version-check against a static manifest URL could be added post-MVP if user demand surfaces — not scoped for MVP.

### Offline Capabilities

The application is **fully offline by design** (E#5 resolved).

- No server dependencies.
- No account system.
- No telemetry or analytics (privacy-respectful default; no ingestion pipeline to maintain either).
- No online leaderboard in MVP. Local high-score and seed-sharing only; post-MVP Steam leaderboards are a possible M6+ feature if Steam release proceeds.
- No multiplayer. Not planned in any milestone.
- No cloud save in MVP. Steam Cloud Save integration in M6 if Steam release proceeds (low effort once Steamworks is integrated).

This has a concrete implication: no post-shutdown rot. The project survives indefinitely on players' machines after any notional end-of-life.

### Distribution & Packaging

- **MVP (M3 Itch.io prototype):** Single ZIP per platform with the compiled binary plus an `assets/` folder. No installer. macOS build ships as a signed + notarized `.app` bundle inside the ZIP.
- **M6 (Early Access on Steam):** Steamworks SDK integration, Steam manifest, Steam branches (beta + stable), Steam Cloud opt-in. Steam handles macOS code-signing via Steamworks.
- **Asset packaging:** assets bundled via Bevy's standard asset loader (run-time file access from `assets/` next to the binary). No asset encryption in MVP.
- **Binary size target:** < 100 MB including assets for MVP. Compressed audio (Opus or MP3), compressed textures where used, glTF meshes with Draco compression if ever needed.

### Implementation Considerations

- **Bevy version pinning:** Hard-pinned to a specific Bevy version at M0 start. Upgrade budget is planned at M4, M6, and M9 only; 4–6 h per minor-version migration budgeted inside the milestone.
- **Avian version pinning:** Must be compatible with the pinned Bevy version. Upgrades co-scheduled with Bevy upgrades.
- **Third-party crate risk:** `bevy_mod_outline`, `bevy_kira_audio`, and any shader library are pinned dependencies with upgrade-churn risk. Before adoption, verify Bevy-version compatibility and maintenance status. Fork-readiness is acceptable for small plugins; any plugin whose maintenance lapses should be viable to fork and maintain inline.
- **Performance profiling:** from M2 onward, regular profiling checkpoints using `cargo flamegraph` and/or `tracy` to catch regressions early. The 60 FPS target is a non-negotiable MVP success criterion and applies to all three platforms.
- **Shader development:** toon + outline shader built and tech-spiked at M1 with explicit validation on Metal (macOS), Vulkan (Linux), and DX12 (Windows) — WGSL-to-backend translation quirks are the most common cross-platform shader failure mode. Fallback to flat-shaded low-poly + rim-light if the M1 tech-spike underwhelms (per M#10 Phase-3 resolution).
- **Build / CI:** three-OS build-verification matrix in CI (GitHub Actions or similar), running on Windows, Linux, and macOS runners. Release builds use the `--release` profile with LTO.
- **Assets pipeline:** Blender → glTF 2.0 export is the single path (per Till's tech decisions). No FBX, no Unity intermediates. Textures kept low-res to match vector aesthetic and conserve binary size.

## Scoping Rationale & Risk Mitigation

### MVP Strategy & Philosophy

This is an **Experience MVP** with **staged stop-and-ship waypoints**, not a lean-startup validate-hypothesis MVP (there is no market to validate; commercial success is optional), and not a feature-complete MVP either (that would bloat a 10-month hobby budget into a 20-month one). The philosophy: *each milestone gate produces a demonstrably playable artifact, and three of those gates (M3, M6, M9) produce shippable artifacts.*

This frames scope decisions as:

- **Must-have in MVP** if it's required to make the core experience land for a new player in their first 30 minutes (Journey 1).
- **Defer to Growth** if it deepens the experience for run-20 players (Journey 2) but isn't required for first-30-minutes magic.
- **Cut from MVP** if implementation cost > reward within the motivation-preservation budget (4–8 h/week over 10 months ≈ 300 h total).

The feature lists in *Product Scope* above already reflect these decisions. The rationale:

- **Cockpit + Arena + Caravan + meta-progression = MVP.** Without any of these four, the core experience doesn't land. All are in Journey 1's critical path.
- **Crafting (C#6) + Partial Death (M#9) + Heat (A#1) + Sensor subsystem (A#4 expansion) = Growth.** Each deepens experience for engaged players (Journey 2) but isn't required for first-30-minutes magic.
- **VR + Modding + Seed-sharing + Orbital Sandbox = Vision.** These are platform and community extensions, not core-experience features.

### Resource Requirements

- **Developer time:** Till, solo, 4–8 h/week average (6 h planning baseline). Over a 10-month horizon: ~260 h implementation + ~60 h asset work + ~40 h learning/research ≈ 300–360 h total budget.
- **Skills developed in parallel to shipping (not prerequisites):** Rust idiomatic usage, Bevy ECS, Avian physics, WGSL shaders, Blender low-poly modeling. Each has its own "comfortable-by" milestone gate per *Project Success*.
- **Tooling:** VS Code or Cursor, Rust + cargo toolchain, Blender (free), GitHub (free tier).
- **Paid accounts required:** **Apple Developer account (€99/year)** for macOS code-signing and notarization from M3 onward — non-negotiable for first-class macOS support. Steamworks registration ($100 one-time) at M6 if Steam release proceeds.
- **External assets as fallback:** if Blender learning stalls, asset-store / OpenGameArt / CC0 sourcing is explicitly acceptable. No purity about self-made assets. Budget overruns here do not block milestone gates.
- **No team scaling.** If Till runs out of time, scope shrinks or milestone timelines extend — no plan for hiring, collaborators, or outsourcing beyond asset-sourcing.

### Risk Mitigation Strategy

#### Technical Risks

1. **Bevy API churn.** 10+ month horizon likely sees 2–4 Bevy releases. *Mitigation:* hard-pinned Bevy/Avian versions; upgrades deliberately scheduled at M4, M6, M9 gates with 4–6 h migration budget each. No ad-hoc upgrades mid-milestone.
2. **WGSL shader complexity for a beginner on three graphics backends (Metal, Vulkan, DX12).** *Mitigation:* M1 is a dedicated tech-spike specifically to de-risk this. Fallback plan (flat-shaded low-poly + rim-light) documented. Three-OS CI validates backend parity.
3. **Third-party crate maintenance risk** (`bevy_mod_outline`, `bevy_kira_audio`, others). *Mitigation:* each crate evaluated for fork-readiness before adoption; small enough to maintain inline if upstream stagnates.
4. **macOS cross-platform parity** (WGSL→Metal translation, code-signing, notarization, Apple Silicon + Intel). *Mitigation:* three-OS CI matrix from M0; Apple Developer account established by M3; shader validation on Metal at M1 spike.
5. **Performance regression invisibility.** *Mitigation:* flamegraph / tracy profiling checkpoints from M2 onward. 60 FPS is a non-negotiable milestone gate on all three platforms.

#### Motivation / Resource Risks (primary binding risks for a hobby project)

1. **M5 "danger stretch" — 6-week Caravan framework build with no visible feature deltas for weeks.** *Mitigation:* slice M5 into weekly sub-milestones where each adds one visible feature (waypoint pointer → first pocket trigger → difficulty curve → …). Never commit to "I'll build Caravan for 6 weeks."
2. **4 h weeks more common than 8 h weeks over 10 months.** *Mitigation:* plan conservatively on 4 h baseline; 8 h weeks are bonus, not the plan.
3. **Asset scope underestimated** (60 h budget is optimistic for a Blender beginner). *Mitigation:* external asset sourcing explicitly acceptable; no purity on self-made assets. Can't block milestone gates.
4. **Motivation death by long invisible stretches.** *Mitigation:* the milestone map is deliberately ordered so each M-gate yields a perceptibly improved playable state. No milestone produces only under-the-hood progress.
5. **Hobby schedule collapse** (life events, new job, etc.). *Mitigation:* stop-and-ship waypoints at M3 and M6 preserve shippable artifacts even if the project ends partway. No milestone depends on a subsequent one completing.

#### Audience Risks (low for hobby, but noted)

1. **Differentiator doesn't land** (pacifism-economy ignored by players). *Mitigation:* documented in *Innovation & Novel Patterns* — game degrades gracefully to competent cockpit arcade roguelite.
2. **Cockpit-only alienates motion-sickness audience (~15–30%).** *Mitigation:* documented acceptance of this trade-off per E#6 Phase-3 resolution. Design commitment, not a defect.
3. **Vector aesthetic looks generic** (Limbo / Obra-Dinn-clone risk). *Mitigation:* semantic accent colors (E#9 loosened) differentiate from monochrome-pure indie aesthetic. M1 tech-spike validates distinctiveness before committing.

## Design Philosophy

A tonal and design-principle anchor. These are not testable requirements (those live in FRs and NFRs) but load-bearing decisions that shape downstream UX, architecture, asset, and feature work. Captured explicitly so future design sessions do not drift away from them.

### Tonal Direction — "Cosmic Mystery over War"

The post-S#5-rejection tonal anchor (per Phase-3 resolution, Option 1 + Option 4 flavor). The game carries an explorer / survey-vessel / archaeological flavor *visually and sonically*, without any written narrative content.

- **Ship design leans survey vessel, not combat jet.** Sensor arms, scanner optics, visible scientific instrumentation — rather than missile racks, aggressive angles, or military lineage.
- **Cockpit HUD leans scientific instrument panel**, not military targeting overlay. Information-rich but calm; readouts preferred over warning klaxons.
- **Audio and color palette lean "cosmic mystery"** rather than "war." Ambient drones, harmonic tones, deep-space hush. Red and amber used sparingly (hazard signaling only) rather than as a dominant combat-palette.
- **Destroyed asteroids occasionally reveal visual artifacts** — glinting relics, unusual crystalline interiors, geometric anomalies. No text, no lore overlay, no codex; only visual suggestion. Players who notice feel rewarded; players who do not, miss nothing functional.
- **Explicitly not done:** no written lore, no logbooks, no faction names, no pilot identity, no backstory text, no dialogue, no cutscenes. Zero writing debt.

This direction is load-bearing for asset work (Blender modeling), shader work (toon palette choices), audio design (drone / harmonic bias over military / percussive), and UX design (scientific-instrument HUD styling). Downstream sessions must respect it unless Till explicitly retracts.

### Design Principles

Five principles that constrain feature-level and implementation-level decisions. Each has a testable surface somewhere in the FRs or NFRs, but exists at the principle layer first — principle wins ties, and a future feature proposal that violates a principle must be explicitly re-raised to Till.

1. **Information-design discipline: no tutorial text.** All learning occurs through play and diegetic HUD cues (surfaces in FR28). Any proposed feature that requires tutorial text, modal hints, or tooltip overlays violates this principle — re-raise.

2. **No visible numeric score.** Player progress feedback flows exclusively through salvage currency (in-run), meta-currency (per-run reward), and unlocks (between-run progression). No score counters on the HUD, no high-score leaderboards inside the game. *The economy is the score.*

3. **Asteroid motion is predictable, not random** *(deferred to Growth per 2026-04-22 implementation-readiness review — see below).* Asteroids follow configurable trajectories — Kepler-like orbits or scripted splines (per A#3) — not randomized paths. Player skill is trajectory-reading and planning, not reflex-chasing. Asteroid-spawn and asteroid-motion configuration in FR-layer implementations must expose deterministic trajectory parameters.

   **MVP deferral:** MVP ships static (non-moving) asteroids. The principle is retained as a Growth-stage commitment — Post-MVP will introduce Kepler-like / spline-based asteroid motion. Rationale: static asteroids satisfy the Arena→Caravan combat loop and Journey-1 "aha" economic moment without taking on trajectory-prediction implementation cost inside the 300 h MVP budget. Decision recorded 2026-04-22 as option (b) in the implementation-readiness P1 gate; consistent with the staged-rollout pattern used elsewhere in scope.

4. **Death is feedback, not punishment.** The post-run summary (FR38) presents cause of death, salvage banked, and retry — framed as a learning beat, not a defeat screen. Visual styling stays in the vector aesthetic: no "GAME OVER" overlay, no red screen fill, no defeat music. The Returnal / Hades framing of death-as-narrative-cycle applies even though this game has no narrative.

5. **Graceful degradation at every novel point.** Every load-bearing novel mechanic has a documented fallback path if the mechanic underperforms: pacifism-economy, audio-first perception, vector aesthetic, cockpit-only commitment. Fallbacks are documented in *Innovation & Novel Patterns* and *Scoping Rationale*. A novel mechanic without a graceful-degradation path is not acceptable — re-raise.

## Functional Requirements

These requirements define the MVP capability contract. Every capability listed here must be implementable and testable. Capabilities not listed here will not exist in the MVP. Growth-stage and Vision-stage features are documented in *Product Scope* and are explicitly out of scope for this contract.

Actor conventions: "Player" denotes the human user. "Game" denotes system behavior not driven by explicit player action. All FRs are implementation-agnostic — they define WHAT must exist, not HOW.

### Flight & Controls

- **FR1:** Player can pilot a ship through 3D space via keyboard + mouse input.
- **FR2:** Player can translate the ship in six directions (thrust forward, reverse, lateral strafe left/right, vertical up/down).
- **FR3:** Player can rotate the ship around all three axes (pitch, yaw, roll).
- **FR4:** Player can aim weapons independently of ship heading (decoupled aim).
- **FR5:** Player can toggle an inertial dampener that modulates Newtonian drift against arcade tightness.
- **FR6:** Player can initiate a boost that temporarily increases thrust at the cost of a rechargeable resource.
- **FR7:** Player can tractor-beam intact asteroids and debris toward the ship for salvage pickup.
- **FR8:** Player views gameplay exclusively through a first-person cockpit view; no external camera toggle is available during active gameplay.

### Combat System

- **FR9:** Player can fire weapons that emit projectiles with ballistic trajectories.
- **FR10:** Player ship equips up to 3 weapons drawn from a pool of 3 prefab archetypes.
- **FR11:** Each fired projectile deducts a configurable amount from the player's salvage currency (pay-to-shoot economy).
- **FR12:** Projectiles can damage asteroids, enemy ships, and debris.
- **FR13:** Destroyed asteroids yield salvage at a configurably lower rate than intact asteroids captured via tractor beam.
- **FR14:** Enemy ships detect, pursue, and attack the player within a configurable engagement range.
- **FR15:** Player ship has Hull and Shields subsystems: Shields regenerate after a cooldown following damage; Hull does not regenerate during a run.
- **FR16:** When player Hull reaches zero, the run ends (permadeath).

### Economy & Salvage

- **FR17:** Player accumulates salvage currency during a run from intact pickups, destroyed asteroids, and other configurable sources.
- **FR18:** On run completion (successful or failed), banked salvage converts to persistent meta-currency at a configurable rate.
- **FR19:** Meta-currency persists across runs via save data.
- **FR20:** Player can spend meta-currency between runs at an unlock shop to acquire permanent upgrades.
- **FR21:** The unlock shop offers between 5 and 10 distinct permanent upgrades affecting ship capabilities.

### Perception & Sensors

- **FR22:** The cockpit HUD displays a radar showing threat markers, range, and relative direction of detected entities.
- **FR23:** The Game emits spatial stereo audio cues for enemies, hazards, and salvage-of-interest; cues indicate approximate direction.
- **FR24:** The cockpit HUD displays current shields, hull, ammunition status, and salvage-currency balance.
- **FR25:** The cockpit HUD indicates the economic yield delta between intact-capture and destroy for salvageable targets in view.
- **FR26:** On first launch, Game displays a splash screen recommending headphones for optimal spatial audio perception.

### Run Structure & Progression

- **FR27:** On the first session, Player is placed in a hand-designed Arena tutorial zone.
- **FR28:** Game presents no written tutorial text; all learning occurs through play and diegetic HUD cues.
- **FR29:** After completing the Arena tutorial, Player transitions to Caravan mode for subsequent runs.
- **FR30:** A Caravan run lasts 5 to 8 minutes from start to target destination.
- **FR31:** Caravan runs use a single MVP run-skeleton template with three selectable difficulty variants (easy, medium, hard).
- **FR32:** Caravan runs contain in-route combat pockets that trigger when the player enters configurable rendering-distance thresholds.
- **FR33:** Player can navigate to the run's target destination via a waypoint-pointer indicator rendered in the cockpit.
- **FR34:** On successful Caravan completion, Player banks accumulated salvage currency.
- **FR35:** On Caravan death, Player banks accumulated salvage currency; previously purchased meta-unlocks persist.

### UI & Feedback

- **FR36:** Player can access a title screen with options to start a new run, access settings, view credits, or quit.
- **FR37:** Player can adjust volume (master and SFX) and mouse sensitivity in a settings menu.
- **FR38:** On death, Player sees a post-run summary showing cause of death, salvage banked this run, and options to retry immediately, access Photo Mode, or return to menu. Visual framing follows *Design Principle 4* — no "GAME OVER" overlay, no red screen, no defeat music.
- **FR39:** Player can restart a run immediately after death without returning to the title screen.
- **FR40:** Player can enter a Photo Mode accessible only from the post-run or death screen; it is not accessible during active gameplay.
- **FR41:** Photo Mode provides free-cam orbital/dolly movement, adjustable depth-of-field, and time-frozen simulation.
- **FR42:** Player can export Photo Mode screenshots as PNG images in 16:9 landscape, 9:16 portrait, and 1:1 square aspect ratios.
- **FR43:** Game pauses the simulation when Player opens the in-run pause menu or when the application window loses focus.

### Persistence & Platform

- **FR44:** Game persists meta-currency, unlocked upgrades, and settings to an OS-convention save location.
- **FR45:** On first launch, Game creates a default save file at the OS-convention save location.
- **FR46:** Save data survives unexpected termination (crash, force-quit, power loss) without corruption.
- **FR47:** Game runs as a native binary on Windows 10+, Linux (major distros: Ubuntu LTS, Fedora, Arch equivalents), and macOS (Apple Silicon and Intel x86_64).
- **FR48:** The macOS binary is code-signed and notarized for distribution outside the Apple App Store.

### Visual Presentation

- **FR49:** Game renders all 3D geometry using a toon-shading material with silhouette outlines.
- **FR50:** Game applies semantic accent colors to entity categories (enemies, salvage, hazards, player-owned) against a restrained base palette; accent colors are distinguishable under the vector aesthetic.

## Non-Functional Requirements

### Performance

- **NFR-P1:** The Game renders at a sustained 60 FPS minimum at 1080p on a reference hardware baseline (NVIDIA GTX 1060 / AMD RX 580 / Apple M1) with the vector aesthetic shader active. Frame rate drops below 60 FPS during normal gameplay constitute a regression.
- **NFR-P2:** The Game loads from double-click to title screen within 10 seconds on reference hardware with SSD storage.
- **NFR-P3:** Transition from title screen to active Caravan gameplay completes within 5 seconds on reference hardware.
- **NFR-P4:** No visible frame hitches exceeding 100 ms occur during steady-state gameplay. Hitches exceeding 50 ms during cockpit↔Caravan transitions or save/load operations are acceptable but logged during development for profiling.
- **NFR-P5:** Process memory usage during steady-state gameplay stays below 4 GB.

### Reliability

- **NFR-R1:** The Game does not crash during normal play across all four documented user journeys. Crash-free playtest is a milestone gate criterion at M3, M6, and M9.
- **NFR-R2:** Save data is not corrupted by ungraceful termination (process kill, power loss, OS crash, alt-F4). Save writes are atomic — either the new save is complete or the previous save remains valid.
- **NFR-R3:** The Game recovers gracefully from a missing save file: first launch creates a default save; subsequent missing files present a "restart with default save?" prompt rather than silent data loss or crash.
- **NFR-R4:** Meta-currency and unlocked upgrades are never lost between runs during normal play.

### Accessibility

- **NFR-A1:** Semantic accent colors assigned to entity categories (per FR50) remain visually distinguishable under common color-blindness conditions (protanopia, deuteranopia, tritanopia). Color is not the sole signal — shape, position, and audio cues provide redundant encoding.
- **NFR-A2:** No information critical to gameplay is conveyed by color alone.
- **NFR-A3:** HUD text is legible at a viewing distance of 60–80 cm from a 1080p display at the UI's default scale.

**Accessibility scope boundaries (documented for transparency, not implemented in MVP):**

- Motion-sickness audience (~15–30%) is an accepted design trade-off per E#6 Phase-3 resolution. No MVP mitigation.
- Full key / button rebinding: deferred to post-MVP.
- Screen-reader support: not applicable (3D spatial game; no text-heavy UI).
- Closed captions for audio cues: post-MVP consideration — audio-spatial cues are gameplay-relevant, and text descriptions cannot preserve the directional mechanic.

### Usability

- **NFR-U1:** A new player reaches the first "aha" moment (holds fire on an intact asteroid for higher salvage yield) within their first 5-minute Arena session. Validated in playtest at the M3 gate.
- **NFR-U2:** All HUD elements required to make tactical decisions (shields, hull, salvage, radar, economic yield delta) are simultaneously visible in the cockpit view.
- **NFR-U3:** Player can identify the current state of all ship subsystems (Hull, Shields) at a glance without looking away from the primary cockpit view.

### Localization

- **NFR-L1:** MVP ships in English only.
- **NFR-L2:** German localization is a post-MVP deferred target (the developer's native language; text-light design means low translation cost once enabled).
- **NFR-L3:** All player-facing strings are loaded from an external string table (JSON or RON) rather than hard-coded in source — a lightweight structural commitment that preserves localization readiness at near-zero MVP cost and avoids a refactor later.

### Not Applicable

The following NFR categories do not apply to this product and are explicitly skipped:

- **Security:** No personal data collected, no payments processed, no compliance obligations. Save-file integrity is addressed under Reliability (NFR-R2).
- **Scalability:** Offline single-player game. No servers, no concurrent-user considerations.
- **Integration:** No external APIs or data imports. Gamepad support (if added) is platform input, not system integration.
