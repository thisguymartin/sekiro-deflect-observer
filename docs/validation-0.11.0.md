# 0.11.0-preview: incoming response coverage and timing

The user's priority is bosses, plus grabs and Mikiri on smaller enemies. The
preserved 0.10.0 log from PID 7844 contains 967 UNKNOWN frames across 12
model/animation/phase combinations. Its full render stream reached 16 MiB at
846 seconds. Frame counts are not attack occurrences or success rates.

## Changes and evidence

- Optional NPC identity: ChrIns +0x30 -> ChrRes +0x628, joined to the extracted
  NpcParam.behaviorVariationId. Source is the locally hashed Ela SDT table.
  Check executable, ID membership, same model, and unchanged owner/ID. A missing
  identity uses the all-variants fallback. Identity changes also invalidate
  cached native animation captures and timing continuity.
- Spear soldier c1010/10102: restore Mikiri for 200003004 and each of the three
  activations in 200003008. Harmless counter dummies no longer merge the hits.
- Snake Eyes c1190/11901: separate grab responses from the ordinary gunner's
  bayonet response. Confirmed throw-damage-only dispatches do not mask an
  incoming grab earlier in the same animation.
- c1400 ninja: permit the verified dummy-0 perilous warning carrier, restoring
  three Mikiri and two sweep cases. Unknown/chained projectile payloads remain
  unresolved; the event's enable flag alone is never used to ignore a bullet.
- c7100/c7110 Genichiro: resolve payload-free visual bullets in 3026, restoring
  three sword phases per model. Correct the stale-name mismatch for c7110/3044.
- Non-opponent hitboxes do not generate incoming timing. This removes the
  c5080/5010 horse/object contact responsible for 335 of the recorded UNKNOWN
  frames. Zero-damage melee rows without explicit marker evidence remain
  unverified; they are not silently discarded.
- alerts.csv records received decision changes plus one-second heartbeats,
  with NPC identity, behavior variation, activation boundaries and progress.
  It has its own 16 MiB limit and continues after the full-frame log fills.
  Queue drops remain possible and are reported. This records draw submission,
  not presentation, contact, button presses or successful deflects.

The fallback table has 2,112 phases on 53 models: 1,632 parry, 41 dodge, 59 jump,
66 Mikiri, 21 avoid, 293 unknown. There are 3,730 overlapping variant-specific
entries for 78 behavior variations. Compared with 0.10.0, 198 variant/phase
cases overlapping previously unknown phases now resolve: 103 parry, 24 Mikiri,
18 jump, 47 avoid and 6 dodge across 82 model/animation pairs. These are not
unique moves or gameplay successes. Evidence: docs/incoming-coverage.json and
dist/review-0.11.0/coverage-delta.json.

Simulating the sparse logging keys on the old capture retains 1,159 of 50,234
rows, approximately 419 KB using the old columns, versus 16.8 MB. This is a
volume comparison; actual session capacity depends on transitions and drops.

## Validation

- 114 Rust checks passed: 74 unit, 1 native DX11 isolation, 13 incoming,
  6 layout, 2 lifecycle, 18 legacy timing. The final hook-only change reran all
  unit tests after the complete suite; Clippy compiled all targets afterward.
- 16 incoming Python regressions passed, including real extracted spear,
  ninja, Snake Eyes, Genichiro and object-only cases, plus harmful-payload and
  identity mutations. Exact generated evidence and variant sort order checked.
- 11 legacy extraction/classification checks and the unchanged timing ledger
  check passed. App Clippy with -D warnings and root formatting passed. Two
  existing unused-method warnings remain inside vendored hudhook.
- Offline MSVC release build passed. Standard non-game DLL host rejection and
  isolated DirectInput/ASI loader checks passed.
- DLL SHA256: 75d5a8646f604213962ffc52e5fabed61cbd903a2a6410dcdd5255551cf37a24.

## Outstanding gameplay checks

Sekiro was no longer running when a fresh read-only NPC check was attempted.
The optional request to reopen it did not receive an answer during this build.
NPC identity is backed by the external structure reference, extracted data and
synthetic reader checks, not a new live capture. Restart into the new candidate
and check a spear soldier's thrusts, Snake Eyes' grab and Genichiro's combo.

Nonstandard Guardian Ape attack types, unresolved common dispatches and real
or unresolved projectiles still produce UNKNOWN. Complete enemy/form support,
exact contact timing, and press-success windows are not established. No new
gameplay dispatch hook, automated input, game-file or graphics-setting edits,
save changes, restart, commit or push were performed.


## Packaged candidate

- Launcher: Mods/SekiroDeflectObserver-0.11.0-preview/launch-observer.cmd.
- ZIP SHA256: 9fa936941b5d3cbc9b667932b21ac0cf142a79b10932bb6cc17a8bd5ac4f154b.
- All eight content hashes verified against both the archive and extracted
  folder; release/staged DLL equality checked. Earlier releases preserved.
