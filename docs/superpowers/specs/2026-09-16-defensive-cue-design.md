# Defensive timing cue and posture HUD — proposed design

Status: **approved by the user ("yes execute") on 2026-09-16; implementation in progress**. Read with
[the investigation](../../timing-investigation-2026-09-16.md) and
[work status](../../WORK-STATUS.md). This design preserves the native Rust DLL
and the newer 0.7.0 animation hook in the actual checkout.

## Scope and approach

The observer tells a locked-on player when to prepare and when an estimated or
calibrated defensive press interval is active. It never generates input, changes
the game's acceptance windows or animation speed, writes gameplay state, or
suggests an offensive punish from recovery. Existing render/animation detours
remain observation infrastructure; this work does not add gameplay detours.

Three approaches were considered:

1. **Recommended: portable decision state plus the existing renderer.** Preserve
   readers, source tables and conservative classification. Add explicit evidence,
   occurrence, phase, clock and pulse state. Carry source timestamps to rendering.
   This permits deterministic tests without pretending contact is measured.
2. Only move the existing bar and subtract a fixed delay. Smaller initially, but
   leaves repeated attacks, pauses, source freshness and per-hit pulses unresolved.
3. Build collision/contact and outcome hooks first. Potentially better evidence,
   but researched offsets and validated interception points are missing. This is
   research work, not a dependency for fixing demonstrable observer defects.

The implementation stays small: a portable timing module and a bounded config
module, with focused changes to existing reader, worker, draw and example files.
No plugin framework, account, telemetry, network client or settings UI.

## T1–T3: observations, timing and pulses

### Explicit data

Build on `Animation`, `Target`, `Response`, `LiveCue`, `Timeline` and the current
batch selector. Separate source capture time, read start/end, publication and
render-decision time. Use injected monotonic `Duration` values in portable tests;
convert the hook's clock to the same process epoch at its boundary.

An observation records validated owner identity (player/session, target handle,
model and animation module), source kind, source capture ID/time, animation ID,
previous/current animation clocks and the raw sequence field. A capture ID means
an observation, **not an attack occurrence**. Identical republished captures never
refresh clock progress, source age or occurrence identity.

An attack occurrence has a local generation attached to validated ownership and
animation continuity. Each hit has an ordinal and its exact generated activation
and deactivation boundaries. Response evidence and timing evidence are separate.
Evidence distinguishes unverified, extracted activation estimate, measured contact
interval, calibrated press interval and gameplay-validated trials. Calibration
never upgrades response eligibility on its own.

The raw sequence field is diagnostic until its semantics are established. Current
logs show it stable for 41,509 adjacent same-track pairs; that does not establish
what a change means. Target/module/session changes, animation transitions, empty
or ambiguous batches, errors and stale samples end continuity. A rewind clears
the current hit and pulse, then requires new forward observations; it is never
treated immediately as permission to replay a hit. A same-ID restart is admitted
only from observed rewind/restart continuity, not the animation name alone.
Unobserved repetitions during a gap remain unknown.

### Units and sign convention

Keep five distinct concepts:

| Concept | Proposed representation and meaning |
| --- | --- |
| Preparation | READY interval before press start; default lead 650 ms of animation progression to estimated contact |
| Contact | Animation-time interval plus evidence; baseline activation is a proxy, not measured contact |
| Press interval | Half-open `[start, end)` with response and clock-domain provenance |
| Preferred time/band | Optional; absent for activation-only data |
| Pulse envelope | Visual intensity over monotonic time, clipped to the press interval |

Let `a` be animation seconds at original capture `s`, `n` the render-decision
time, and `r` the locally observed animation-seconds/real-second rate. For valid,
stable continuity only, the estimated animation clock at decision is
`a_now = a + r * (n - s)`.

Display latency `Ld` and input latency `Li` are nonnegative **real milliseconds**;
positive values move guidance earlier. A contact offset is **animation
milliseconds**, with positive values meaning later than activation. For an
animation-domain estimated press interval `[A, B)`, the submission interval is
`[A - r*(Ld+Li)/1000, B - r*(Ld+Li)/1000)`. Sampling age is already included in
`a_now` and is not added again as a calibration offset. Increased latency shifts
both ends equally; it never widens the interval.

Until measured response profiles exist, retain the current estimated widths:
150 animation ms before activation for parry, 300 for mapped dodge/jump. At 2x
playback these remain 75/150 real ms, rather than silently assuming the game's
acceptance window has a particular wall-clock behavior. A calibrated profile
must name its clock domain. Real-ms profile endpoints are converted using the
validated local rate; animation-ms endpoints stay in animation time.

For measured contact interval `[Cmin,Cmax]` and a measured real-time response
acceptance interval `[Emin,Emax]` before contact, use their conservative
intersection for press guidance: `[Cmax-r*Emax, Cmin-r*Emin)` after converting ms
to seconds. Empty intersections produce no action. A preferred instant/band must
be supported by that profile's trials and lie inside the resulting interval.
The baseline has no such profiles, so it has **no claimed optimal instant**.

### Progression and invalidation

Require three distinct fresh captures to establish two positive rate estimates.
Initially support rates 0.25–4.0, capture spacings 1–50 ms, and relative rate
difference at most 20%. These are conservative observer limits, not game rules.
Speed changes suspend extrapolated action until the new rate stabilizes. New
captures with unchanged time suppress action immediately; repeated delivery of
the same capture does not imply a pause or progress.

Extrapolate no farther than 25 real ms from capture, always below the existing
50 ms hard source/required-observation freshness ceiling. Beyond the extrapolation
bound suppress action rather than holding the estimate at its maximum. Negative
ages, nonfinite values, rewinds, switches and ambiguous blends invalidate it.
The polling path conservatively uses read start and retains the 10 ms read
budget. Hook capture age must survive `event_hook::apply`; polling must not make
an older capture appear new. Do not extrapolate into a different occurrence.

States are hidden, neutral, preparation, actionable and expired. Only actionable
can use an action label/color. READY uses a hollow marker; actionable uses a
filled marker and the response label; expired uses a neutral flat mark and END
briefly, or yields immediately to the next preparation interval. The elapsed hit
cannot delay the next hit. Unknown moves show only a small neutral LOCKED/UNKNOWN
state, with diagnostic explanations in F9.

Each `(owner generation, occurrence generation, phase ordinal)` can emit at most
one preferred-time pulse. Emit only while a fresh rendered decision is within
its valid interval and has crossed its supported preferred threshold. First
observation after the threshold, a frame stall skipping the interval, or stale
reacquisition must not emit a catch-up pulse. The default uncalibrated data can
show estimated actionable labels/shape changes, but cannot invent a preferred
flash. Synthetic profiles exercise pulses offline; real preferred flashes await
evidence-backed calibration. Pulse fade is clipped at expiration and never
retains press semantics. Next-hit preparation overrides earlier decoration.

## R1: defensive responses and capability

Preserve all existing conservative parry exclusions and exact model/phase joins.
Audit exclusions for every response, including mixed/projectile and uncertain
imports; a response override may only restrict eligibility. Approximate body
distance, height and facing remain explicitly coarse reach filters. Do not add
attack-specific reach or contact offsets without move evidence.

Mikiri remains explicitly unsupported unless an exact eligible thrust mapping
and validated player-capability read both exist. There is no such read or
implemented response in this checkout. User configuration cannot assert that
capability. A future `Mikiri` enum case must fail closed without both observations.
Lightning, terror, fire, projectiles and other special counters stay separate
unknown/unsupported categories. No automatic retaliation or offensive guidance.

## H1: compact fixed HUD

Default to screen-space placement. The initial fallback is configured geometry,
**not automatic posture-bar detection**. No validated native HUD anchor was
found in the reader. Use a centered, aspect-preserving playable viewport,
retaining 0.6.4's letterbox/pillarbox math. Without a validated lens, the fixed
mode may use an explicit configured 16:9 viewport fit; overhead still requires
a valid camera, depth and offscreen checks. Neither mode may relax timing reads.

Proposed fallback lane center is `(0.5, 0.87)` in the playable viewport. Reserve
the band beginning at normalized y=0.90 for the posture HUD, with a 12-reference-
pixel gap between it and the cue's **full** lower bound. This is a placement
assumption to verify against the actual bar, including glow and UI scaling.
Reference units scale from a 1080-pixel-high playable viewport. The lane is
240 by 8 reference pixels, with compact label/marker/outline enclosed in computed
bounds. Remove the old 424-pixel ribbon and long needle from the default layout.
Clamp the full rectangle to safe margins and the configured posture exclusion
band; if it cannot fit, suppress with an F9 reason. Do not clip an actionable
label into an unreadable fragment.

Offsets, scale and the posture-band position accommodate UI scaling and custom
layouts. A configured viewport rectangle can describe a centered or inset game
view; unknown stretched/cropped layouts require configuration or suppression.
Fixed placement never uses Wolf's projected position and therefore cannot drift
with jumps, crouches, camera motion or boss proximity. Optional overhead uses the
same compact drawing and timing decisions, with invalid/offscreen suppression.

Verify synthetic layouts at 1280x720, 1920x1080, 2560x1440, 3840x2160,
3440x1440 and 5120x1440; add letterboxing, portrait/small windows, UI scale
extremes and inset viewports. Live fullscreen/windowed/borderless placement and
the actual posture gap remain separate acceptance checks. Existing screenshots
do not establish the new placement, and several show no visible player posture.

## H2: lock and lifecycle

Keep hash gating, owner rechecks, current-batch-only selection, competing-track
rejection, no stale ring fallback and quick-combo priority. Publish invalidation
before logs/config work. Observed lock loss, target switch, invalid/dead player
or target, read failure or stale data clears the decision and pulse immediately.
Rendering checks the current invalidation generation before submission so a
copied snapshot cannot survive an already-observed loss. Reacquisition starts
with a fresh owner observation and new continuity; no old green zone.

F8 controls visibility; F9 may show diagnostics without a target but cannot
enable gameplay cues. Hiding clears pulse emission eligibility so showing the
HUD cannot replay a missed pulse. Retain focus checks, fresh-key detection and
pass-through messages. No normal combat input is captured or synthesized.

There is no validated direct menu/pause read in this checkout. Stop/rewind and
missing-player observations can suppress unsafe cues but are not proof of
immediate menu detection. Research a hash-gated read-only playability source
before claiming that acceptance criterion; never add a guessed offset. Log this
as an explicit remaining limitation if it cannot be established. Measure both
observation-to-hidden-submission delay and visible delay separately.

## C1: local persistent configuration

Proposed file: `%LOCALAPPDATA%/SekiroDeflectObserver/cue.toml`, with a usable
default copy in `packaging/cue.toml`. TOML is a small conventional format; cached
`toml` sources are present. Pin the selected dependency only after checking the
actual compatible offline graph and official upstream API documentation (no
Context7 tool is exposed in this session). No custom general-purpose parser.

All file reads, parsing, validation and writes run in the worker. The renderer
only sees a validated immutable snapshot and its generation. Check for changes
once per second; valid reloads apply atomically and clear timing/pulse continuity
where timing or enabled responses change. Malformed reload retains the entire
last valid config and reports a concise F9/log error. Missing/invalid initial
file uses safe defaults; do not overwrite an invalid user file automatically.
Reject unknown keys, invalid enums, NaN/infinity, invalid hex colors, duplicate
or mismatched profile identities and impossible interval ordering.

Proposed defaults and inclusive bounds (not yet implemented):

| Setting | Default | Allowed range / meaning |
| --- | --- | --- |
| anchor | posture | posture / overhead |
| offset_x / offset_y | 0 / 0 | -480..480 / -160..160 reference pixels; full bounds still enforced |
| scale | 1.0 | 0.5..1.5 |
| width | 240 | 160..360 reference pixels |
| opacity | 0.95 | 0.2..1.0 |
| label_size | 20 | 14..30 reference pixels |
| safe_margin | 24 | 8..96 reference pixels |
| posture_band_top | 0.90 | 0.75..0.98 of configured viewport |
| posture_gap | 12 | 8..64 reference pixels |
| parry / dodge / jump colors | #2EF245F2 / #FF661FFF / #2ECCFFFF | exactly #RRGGBBAA; alpha must keep labels legible |
| ready / expired colors | #E0E5EBFF / #899099CC | neutral colors; action meaning still comes from state |
| outline / glow intensity | 0.8 / 0.35 | 0..1 each |
| pulse intensity / duration | 0.6 / 80 | 0..1 / 16..120 real ms, clipped to valid interval |
| reduced_flash | false | true disables brightness/size pulse; label/shape remain |
| preparation_lead | 650 | 350..1500 animation ms; READY only before press start |
| display_latency / input_latency | 0 / 0 | 0..120 real ms each, sum at most 150; positive shifts earlier |
| enabled responses | parry, dodge, jump | subset of implemented evidence-backed responses; no eligibility override |
| visible / diagnostics | true / false | F8 / F9 initial states |
| diagnostic_logging | true | bounded local logs; essential startup/config errors remain readable |

Response profiles are separate from these appearance/system settings. Baseline
estimated widths stay 150/300/300 animation ms and may be narrowed, not widened,
without measured evidence. Per-move profiles include executable SHA-256, generated
data SHA-256, schema version, exact model, animation, phase ordinal/bounds,
response, clock domain, contact offset/interval, calibration evidence references
and trial metadata. No compatible record means no per-move override. Bounded
offsets alone cannot bypass a missing source, uncertain response or failed read.

F6/F7 queue 8-reference-pixel placement changes for the worker to persist. A
single focused F10 press queues a placement reset; deleting the file with the
game closed resets all settings. No render-callback disk I/O. Persist with a
temporary file in the same directory, flush and an atomic Windows replacement;
failure retains the old file and current valid snapshot. Handle reload/write
conflicts explicitly rather than silently overwriting concurrent external edits.

## V1–V2: staged verification and evidence

After approval, use `writing-plans` for an executable task plan and TDD for each
behavior change. First make the synthetic failures in the investigation fail
against current code. Use real decision outputs and pulse counts, not assertions
that mirror helper implementation. Add boundary/latency-sign, speed, stop,
rewind/repetition, multi-hit, cancellation, lock/owner switch, bad read, stale
capture and frame-stall cases. Keep the Ogre batch and quick-combo tests.

Test missing/invalid initial config, malformed reload preserving all prior
values, atomic persistence, concurrent changes, bounds and profile identity.
Cover every response and exclusion. Use the actual shared renderer in the
offline example for layout evidence; label every output synthetic.

Run fmt, locked offline tests, Python data tests, diff checks, Windows Clippy,
the documented build and DLL/loader checks. Preserve existing packages and
record exact toolchain/target and output. A plain default-target test currently
fails for missing GNU `dlltool.exe`; explicit Windows MSVC tests pass. This must
be visible in the record rather than hidden by a different command.

Live validation order: ordinary soldier, Ogre attack/auxiliary track, selected
Ape responses, then representative supported boss combos. Every trial records
executable/DLL/data hashes, version, encounter/form, exact phase, source age and
clock, render time, display mode/viewport, actual input/contact/outcome evidence,
distance/angle, frame rate/modifiers and successes, failures and ambiguity. Do
not attribute a log to the current disk DLL without loaded-module/hash evidence.
No draw submission or candidate effect 105010 proves a successful deflect.

## D1–D2: completion and handoff

Generate coverage from exact checked-in tables, with source references and
separate extracted/classified-estimate/calibrated/validated/unsupported/unknown
statuses. Keep boss forms and unresolved variants separate. Retain historical
validation documents; update present-tense README, cue, architecture, Windows,
manual checklist and changelog claims only when corresponding behavior changes.

Append a dated session entry to `docs/WORK-STATUS.md` after every job, using the
acceptance IDs. Record changed files, commands/results, working-tree/commit state,
synthetic versus live evidence and concrete remaining actions. Print its remaining
work section in job output. No gameplay criterion becomes complete from unit
tests or synthetic images alone.
