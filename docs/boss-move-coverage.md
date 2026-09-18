# Boss and move coverage - audited 2026-09-16

This is the checked-in runtime table inventory, not a claim of complete boss
support. Run `python scripts/update-move-coverage.py --check` to verify that this
summary and the phase ledger still match the exact generated tables.

The full [phase ledger](boss-move-phases.csv) preserves all 2,161 exact phase
records: model IDs, animation IDs, zero-based phase ordinals, activation/deactivation boundary
strings from Rust, current response classifications, source JSON hashes and
available attack-parameter references. Boundaries are animation seconds; the
Rust literals are f32 approximations. Original extraction precision is retained
in [response evidence](response-coverage.json) for mapped dodge/jump phases.
Animation variants and combo phases are not unique human moves. No human move
names, calibration results or success percentages were invented.

## Status definitions and counts

| Status | Meaning | Audited result |
| --- | --- | --- |
| Extracted | Phase present in the generated table; activation is not contact | 2,161 phases across 54 models |
| Classified estimate | Conservative response classification plus activation-based timing | 450 parry, 39 dodge, 58 jump = 547 phases |
| Extracted, unverified | No actionable response classification | 1,614 phases |
| Calibrated | Compatible per-move press/contact evidence and profile | 0 records |
| Gameplay validated | Complete trial evidence identifies this exact phase and successful/failed responses | 0 complete per-move records |
| Unsupported | No implemented response or required capability source | Mikiri; special counter guidance including lightning/terror/fire/projectile handling |
| Unknown | Encounter/form or runtime behavior routing not established | Do not infer support from an archive/model name |

There are 79 extracted archive records, 39 models with parry
estimates and 1,830 special-animation indicators. Special indicators
overlap other inventory and are not extra supported moves. An extracted phase
with activation at zero can have no usable advance interval; a row is not a
guarantee of an actionable cue.

Sources: [generated runtime tables](../src/attack_timings.rs),
[model/source hashes](enemy-coverage.json), [response/source evidence](response-coverage.json),
[generator](../scripts/generate-attack-timings.py),
[classifier](../scripts/attack_responses.py) and
[coverage generator](../scripts/update-move-coverage.py).
Generated data SHA-256 at audit:
`88fe0e54cd20834cb17582bba3d4637da650ed3f2ff4394af9d5da9b16ba1d71`.
The offline parameter join does not prove runtime behavior-variation selection.
The ledger does not manufacture missing per-parry parameter provenance: those
rows retain the exact timeline source hash and generator/classifier references;
expanded per-parry parameter provenance remains work.

An explicit type-4 `BulletBehavior_Midair` event makes every response for that
mixed animation unverified. Type-2 `BulletBehavior` and generic effect/aim markers
occur throughout existing parameter-backed Ogre, Ape and other action rows, so
their presence alone does not erase an exact dodge/jump parameter join. They
remain excluded from green/parry eligibility by the broader phase classifier.

## Encounter/form evidence

| Encounter or requested form | Exact model evidence | Current status / missing evidence |
| --- | --- | --- |
| Ordinary soldier baseline | c1010 in historical capture and current log | Extracted/classified estimate; no complete press/contact/outcome trial; validate first |
| General Naomori Kawarada | c1020 in an earlier local recording | Shared model/behavior variations unresolved; clip does not validate all c1020 actors |
| Chained Ogre | c5020; 26 phases, 6 parry, 5 dodge | Historical auxiliary-40000 selection failure; corrected current-batch selector has synthetic tests, new live timing trial required |
| Guardian Ape encounter | c5100; 93 phases, 4 parry, 1 dodge, 4 jump | Selected source-based mappings; no complete contact/press calibration |
| Guardian Ape sword/headless form | c5100 includes 100003xxx mappings; exact runtime form applicability unvalidated | Requested form remains unresolved; animation bank alone is not form validation |
| Headless Ape encounter / companion | Runtime model/form mapping not established in reviewed evidence | Requested related forms unresolved; do not transfer Guardian Ape trial claims |
| Other boss forms and variant archives | Exact inventory below, including zero-phase variants | A related model ID or shared archive is not inheritance of support |

The five Ogre dodge phases are 100003005 `[0.566666663,0.633333325)`,
100003007 `[2.166666746,2.266666651)`, 100003008 `[1.000000000,1.100000024)`,
100003013 `[1.133333325,1.366666675)` and
100003014 `[1.333333373,1.466666698)`. These are extracted animation boundaries,
not observed player-contact intervals. Ape response phases and every other
boundary are preserved in the ledger; no similar ID receives their profiles.

Runtime calibration profiles are active only when `form = "model-animation-phase"`
states that the evidence covers the entire exact model/animation/phase key.
Named encounter or boss forms remain inactive because the runtime has no validated
form observation source. A calibration profile never changes response eligibility.

## Exact model inventory

Every extracted archive is listed, including archives without resolved phases.
Unverified is the phase total minus the three classified-response counts; it is
not the generator's `uncertain_phases` field, which also includes dodge/jump.

| Model | Phases | Parry estimate | Dodge estimate | Jump estimate | Unverified | Calibration / gameplay validation |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| c1000 | 0 | 0 | 0 | 0 | 0 | none recorded |
| c1001 | 0 | 0 | 0 | 0 | 0 | none recorded |
| c1010 | 79 | 15 | 0 | 2 | 62 | none recorded |
| c1013 | 0 | 0 | 0 | 0 | 0 | none recorded |
| c1020 | 73 | 29 | 1 | 3 | 40 | none recorded |
| c1030 | 6 | 6 | 0 | 0 | 0 | none recorded |
| c1040 | 19 | 10 | 0 | 3 | 6 | none recorded |
| c1050 | 28 | 0 | 0 | 1 | 27 | none recorded |
| c1060 | 14 | 9 | 0 | 2 | 3 | none recorded |
| c1070 | 26 | 11 | 1 | 2 | 12 | none recorded |
| c1080 | 3 | 3 | 0 | 0 | 0 | none recorded |
| c1090 | 0 | 0 | 0 | 0 | 0 | none recorded |
| c1100 | 1 | 0 | 0 | 0 | 1 | none recorded |
| c1110 | 2 | 1 | 0 | 0 | 1 | none recorded |
| c1120 | 13 | 12 | 0 | 0 | 1 | none recorded |
| c1130 | 33 | 7 | 0 | 0 | 26 | none recorded |
| c1140 | 22 | 6 | 2 | 0 | 14 | none recorded |
| c1150 | 6 | 4 | 0 | 0 | 2 | none recorded |
| c1180 | 88 | 13 | 0 | 1 | 74 | none recorded |
| c1190 | 26 | 10 | 1 | 0 | 15 | none recorded |
| c1200 | 19 | 0 | 2 | 1 | 16 | none recorded |
| c1210 | 1 | 0 | 0 | 0 | 1 | none recorded |
| c1211 | 5 | 2 | 0 | 0 | 3 | none recorded |
| c1212 | 0 | 0 | 0 | 0 | 0 | none recorded |
| c1220 | 30 | 12 | 1 | 0 | 17 | none recorded |
| c1240 | 14 | 2 | 0 | 0 | 12 | none recorded |
| c1250 | 61 | 18 | 2 | 2 | 39 | none recorded |
| c1260 | 0 | 0 | 0 | 0 | 0 | none recorded |
| c1300 | 8 | 3 | 3 | 0 | 2 | none recorded |
| c1310 | 79 | 0 | 0 | 0 | 79 | none recorded |
| c1320 | 4 | 0 | 0 | 0 | 4 | none recorded |
| c1350 | 33 | 2 | 2 | 0 | 29 | none recorded |
| c1360 | 21 | 8 | 1 | 0 | 12 | none recorded |
| c1370 | 34 | 0 | 0 | 0 | 34 | none recorded |
| c1400 | 41 | 25 | 0 | 2 | 14 | none recorded |
| c1450 | 80 | 3 | 0 | 0 | 77 | none recorded |
| c1451 | 0 | 0 | 0 | 0 | 0 | none recorded |
| c1460 | 0 | 0 | 0 | 0 | 0 | none recorded |
| c1470 | 67 | 22 | 0 | 3 | 42 | none recorded |
| c1500 | 34 | 21 | 3 | 0 | 10 | none recorded |
| c1550 | 64 | 34 | 0 | 0 | 30 | none recorded |
| c1700 | 81 | 15 | 0 | 0 | 66 | none recorded |
| c5000 | 114 | 12 | 0 | 7 | 95 | none recorded |
| c5001 | 0 | 0 | 0 | 0 | 0 | none recorded |
| c5010 | 53 | 0 | 0 | 0 | 53 | none recorded |
| c5020 | 26 | 6 | 5 | 0 | 15 | none recorded |
| c5040 | 20 | 0 | 0 | 0 | 20 | none recorded |
| c5050 | 13 | 0 | 0 | 0 | 13 | none recorded |
| c5060 | 59 | 19 | 3 | 3 | 34 | none recorded |
| c5061 | 0 | 0 | 0 | 0 | 0 | none recorded |
| c5063 | 0 | 0 | 0 | 0 | 0 | none recorded |
| c5070 | 3 | 0 | 0 | 1 | 2 | none recorded |
| c5080 | 81 | 0 | 2 | 0 | 79 | none recorded |
| c5090 | 39 | 8 | 1 | 2 | 28 | none recorded |
| c5100 | 93 | 4 | 1 | 4 | 84 | none recorded |
| c5200 | 46 | 0 | 0 | 0 | 46 | none recorded |
| c5300 | 7 | 5 | 0 | 0 | 2 | none recorded |
| c5310 | 12 | 0 | 0 | 0 | 12 | none recorded |
| c5320 | 4 | 0 | 0 | 0 | 4 | none recorded |
| c5400 | 130 | 11 | 1 | 6 | 112 | none recorded |
| c5401 | 0 | 0 | 0 | 0 | 0 | none recorded |
| c5403 | 0 | 0 | 0 | 0 | 0 | none recorded |
| c5410 | 0 | 0 | 0 | 0 | 0 | none recorded |
| c7000 | 33 | 18 | 1 | 5 | 9 | none recorded |
| c7010 | 0 | 0 | 0 | 0 | 0 | none recorded |
| c7011 | 0 | 0 | 0 | 0 | 0 | none recorded |
| c7020 | 96 | 6 | 0 | 0 | 90 | none recorded |
| c7021 | 0 | 0 | 0 | 0 | 0 | none recorded |
| c7100 | 87 | 24 | 3 | 4 | 56 | none recorded |
| c7101 | 0 | 0 | 0 | 0 | 0 | none recorded |
| c7103 | 0 | 0 | 0 | 0 | 0 | none recorded |
| c7110 | 85 | 20 | 2 | 2 | 61 | none recorded |
| c7111 | 0 | 0 | 0 | 0 | 0 | none recorded |
| c7200 | 0 | 0 | 0 | 0 | 0 | none recorded |
| c7201 | 0 | 0 | 0 | 0 | 0 | none recorded |
| c7210 | 0 | 0 | 0 | 0 | 0 | none recorded |
| c7300 | 0 | 0 | 0 | 0 | 0 | none recorded |
| c7400 | 45 | 14 | 1 | 2 | 28 | none recorded |
| c7401 | 0 | 0 | 0 | 0 | 0 | none recorded |

## Evidence requirements before promoting a move

Retain exact executable/DLL/data hashes and observer version; encounter and form;
model, animation and phase ID; contact and actual press observations; trial count
including successes, failures and ambiguous outcomes; distance/angle; frame rate,
display mode, game modifiers and links to original logs/video. State uncertainty
and measurement resolution. A draw call, TAE crossing or effect 105010 alone is
not a successful deflect. There are no complete records to promote in this audit.

The latest reviewed 0.7.0 log has 83 parry **submissions**, not 83 deflects.
Historical [0.6 evidence](validation-0.6.md) retains its original scope. No new
gameplay acceptance criterion is complete.

Next validation order: soldier baseline, Ogre track regression, selected Ape
responses, then representative supported boss combos. Record each run with the
[trial template](../tests/compatibility/cue-trial-template.md).
