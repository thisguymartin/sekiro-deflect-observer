# 0.9.0-preview validation - 2026-09-17

The user explicitly changed the objective to identifying an incoming attack
and its defensive response regardless of whether it will reach Wolf. The new
default mode fulfills that scope; contact prediction and exact press timing
are not prerequisites. The raised posture placement from 0.8.1 is retained.

## Implemented

- `src/incoming.rs` consumes fresh captures from the existing completed-batch
  hook. It shows the next/current phase immediately without requiring reach,
  facing or a three-capture rate estimate. Wind-up and active attack have
  distinct states. No press interval, contact estimate or pulse is emitted.
- The new generator joins local TAE, BehaviorParam, AtkParam, Bullet and
  ThrowParam data. It resolves only the inspected warning-bullet route and
  explicit Mikiri marker hitboxes. Ambiguous variants remain unknown.
- Incoming classifications: 2,161 phases / 54 models / 1,673 parry / 40 dodge /
  56 jump / 63 Mikiri / 16 no-parry / 313 unknown. The prior timing tables and
  old coverage reports remain unchanged.
- Incoming mode and Mikiri hints default on. The Mikiri toggle is a user
  preference, not a skill-unlock read. Disabling it shows PARRY for these
  classified deflectable thrusts. Legacy timing mode remains selectable.
- [Incoming evidence](incoming-attacks.md) documents the fresh extraction,
  exact joins, source hashes and compiled-code inspection. A preserved loaded
  function was disassembled; no full decompilation or new game hook is claimed.

## Fresh automated checks

Pinned Cargo from `C:/Users/mpati/.cargo/bin`; MSVC target; locked offline Rust
dependencies. Raw outputs are in `dist/review-0.9.0/`.

| Check | Result |
| --- | --- |
| Formatting | Passed |
| Clippy, all targets with `-D warnings` | Passed |
| Rust all-target tests | 105 passed before the additional combo case |
| Final incoming regression suite | 9 passed, including the added combo case; total current coverage 106 Rust tests |
| Existing Python timing/data suite | 11 passed |
| New incoming classifier/data suite | 9 passed |
| Both generated-data consistency checks | Passed |
| MSVC release build | Passed |
| Native DLL load / non-game host rejection | Passed, workspace-local log directory |
| ASI forwarding / adjacent load / host rejection | Passed, isolated `dist/review-0.9.0/asi-smoke` |
| Shared ImGui eight-state 1080p gallery | Full vertex containment passed; image visually inspected |
| ZIP manifest and staged files | All seven content hashes and bytes matched; DLL/config matched build/source |

Runtime regressions cover immediate far/away-facing warnings, Mikiri fallback,
grabs/sweeps, variant uncertainty, active-versus-wind-up boundaries, recovery,
no invented press/contact/pulse, source failure/expiry/loss/clear/switch, and a
parry-to-Mikiri combo without skipping a disabled earlier hit. Classifier tests
ensure that damaging or unnamed dummies, scaling damage, unresolved projectiles,
child projectiles, nonstandard dispatch and uncertain imports are not promoted.

## Exact artifacts

DLL SHA256:
`81a4a94dbce3b0c6d5c85d0dc6d7c260e8144468e83b55cd92db20cbb36ae768`.

ZIP SHA256:
`ae6a4c606f6e7a68d03be6db716e04bca4de58f5db6046bb58779cd338af26c6`.

Incoming table SHA256:
`53c7ea280d84818366612ee06aa600f35c93598618ea32a727f6bf92dee4a143`.

Package: `dist/SekiroDeflectObserver-0.9.0-preview-windows-x64.zip`.
Staged profile: `Mods/SekiroDeflectObserver-0.9.0-preview/observer.me3`.
All previous version folders/packages were preserved. Work is uncommitted.

## Live check remaining

No Sekiro process was running during the investigation and no new gameplay was
performed. Close the game fully before loading this DLL; confirm 0.9.0 in F9
and its exact hash in startup logs. Check an ordinary sword attack while far
away, a grab, a sweep and a Mikiri-eligible thrust; confirm lock loss hides the
cue and attack cancellation does not retain the old response. Check the cue's
gap above Wolf's posture bar. Disable Mikiri hints for a save without the skill.

These checks validate implementation and static classification, not actual
deflect success, input latency or universal enemy/mod compatibility.
