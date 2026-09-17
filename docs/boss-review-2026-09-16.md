# General Naomori Kawarada recording review

Reviewed the supplied 39.25-second recording, `Sekiro 2026-09-16 11-34-12.mp4`,
including overview frames every two seconds and quarter-second frames from
16–24 seconds. The recording is 2560x720. The overlay is visible during combat.
Session 21976 identifies observer 0.6.4 and the supported executable hash.

The session's 1210–1300-second render-log interval contains model c1020,
matching the General shown in the recording. It records 848 neutral locked
submissions and 309 timeline submissions, with no parry/dodge/jump-now
submissions in that interval. A green interval can be visible as a future
estimate without the current pointer being inside it. Video/log timestamps
were not synchronized to individual presented frames, so these counts are
session evidence, not frame-exact contact measurements.

Findings:

- Animations 3003, 3007, 3063 and 3088 resolve to deflectable attack parameters
  but are excluded by the generator's broad special/movement-event checks.
  Type 700 look-target flags classify these as special; some also contain
  types 66/67. This produces unverified timelines for ordinary-looking swings.
- Animation 3004 joins both sword and spear behavior variations. Its resolved
  response is unverified. Correcting that requires the active behavior variation,
  not assigning one response to every c1020 actor.
- Animation -1 briefly appears between valid samples of the same attack. The
  current-batch reader returns -1 for an empty engine submission batch, among
  other cases. The logs lack ring bounds, so empty batches are a plausible cause,
  not established for every dropout. The renderer then loses its timeline.
- Several mapped animations end before reaching their estimated press interval
  and transition to other animations. Carrying a countdown across such a change
  would risk cueing an attack that was interrupted or replaced.
- The cue uses animation hitbox activation with a fixed lead. It does not
  predict blade/player contact, so changing distance, movement and lunges are
  not captured by its timing estimate. The distance/facing filter is coarse.

This is incomplete boss guidance, not evidence that a sorted list of attacks
is backwards. No timing or classification tables were changed based solely on
the clip. Useful next implementation work is runtime behavior-variation reads,
per-event response classification, instrumented current-batch continuity, and
contact validation against the recording. Do not fix flicker by reviving old
attack history or label every special animation as a sweep/grab.

Review artifacts and reproducible local mapping/log inspection scripts are in
`dist/review-0.6.4/boss-clip`. The original recording and gameplay were unchanged.
