# Attack classification evidence

The default HUD answers “what attack is coming and how can I respond?” It does
not wait for the enemy to face Wolf or enter a distance threshold. A current
attack capture can show a response immediately; it does not require three
captures to estimate animation speed. Since 0.10.0, the requested default cue
is a slim rail below the enemy's upper posture bar, with no large panel backing.

| Label | Meaning |
| --- | --- |
| PARRY | The identified variant permits deflection, or all fallback candidates agree |
| DODGE | Incoming grab identified by attack parameters |
| JUMP | Classified low sweep |
| MIKIRI | Thrust with an explicit Mikiri marker and player-to-enemy counter route |
| NO PARRY | Both guard-attribute configurations disable deflection; no specific alternative established |
| UNKNOWN | An attack phase is known, but its response cannot be resolved consistently |
| LOCKED | Fresh target, without a current attack phase or usable animation data |

The colored right-hand segment identifies the response during wind-up. Its
diamond approaches the center gate at activation. Parryable active phases use
a white segment and red strike emblem; other responses retain their colors.
Reduced-flash mode suppresses this white/red transition. These visuals do not
predict impact with Wolf or promise success. No contact/press interval
or timing pulse is created by incoming mode. Freshness, lock, ownership, focus,
hide and cancellation checks remain. Old attack history is never revived.

`incoming_cues = true` is the default. `mikiri = true` enables the Mikiri hint
and assumes the skill has been unlocked; there is no runtime skill read. Set
`mikiri = false` if needed: those verified-deflectable thrusts display PARRY.
Existing 0.8.x config files get these settings through defaults. Optional
`incoming_cues = false` retains the preceding estimated timing mode and its
reach/calibration behavior. The new mode ignores timing leads and latency.

## Evidence from the installed game

The executable in `C:/Program Files (x86)/Steam/steamapps/common/Sekiro` hashes to
`637aca527538c0ec6e1f136c8ed66046e95dfbdbb1f51926e134d9916398b856`.
Fresh extraction read all 79 base enemy archives and the local parameter binder.
No game file, save or gameplay memory was changed.

The [generated evidence](data/incoming-coverage.json) records all exact model,
animation, phase, response and attack-parameter IDs, plus source hashes. Across
2,112 fallback phases on 53 models it classifies **1,632 parry, 41 dodge, 59 jump,
66 Mikiri and 21 no-parry**; 293 remain unknown. The variant table has 3,730
entries for 78 NPC behavior variations. These overlap the fallback records and
are not additional unique moves. Neither table establishes successful gameplay
trials or complete enemy/form coverage.
The old timing tables and coverage ledger remain separate and unchanged.

The old classifier excluded entire animations for generic effects or aiming
events. Incoming classification instead resolves the overlapping melee events
through BehaviorParam and AtkParam. When available, the same-model NPC parameter
identity selects its behaviorVariationId. Otherwise all candidate variants must
agree. Missing parameter rows and unproven event formats are not guessed.

In 0.11.0, the read-only chain is ChrIns +0x30 -> ChrRes +0x628 -> NpcParam ID,
from the locally hashed Ela table. The executable, ID membership, model, owner
and value are checked. This is a static reference plus synthetic reader tests;
a fresh live identity check remains outstanding. This does not establish a
runtime discriminator for legacy named-form calibration profiles.

Specific corrections:

- c1010 variation 10102: spear thrust 200003004 is Mikiri; the extended dummy
  in 200003008 no longer merges three separate thrust activations. Sword
  variants cannot borrow that response.
- c1190 variation 11901: Snake Eyes anchor attacks 100003013/100003050 are
  grabs. Ordinary variation 11900 retains its separate bayonet response.
  Explicit throw-damage dispatch 304 no longer hides a preceding grab phase.
- c1400 3005/3073/3079: the warning carrier uses verified dummy attack 0, not 8.
  These now resolve to Mikiri; 3083/3087 resolve to low sweeps.
- c7100/c7110 3026: payload-free visual bullets no longer hide the three sword
  activations. c7110 3044 uses its actual parameter identity despite a stale
  c7100 reference-name prefix, restoring the sweep classification.
- Explicit non-opponent hitboxes do not define incoming attack timing, including
  c5080 5010 horse/object contact. Attack data with no established response
  remains UNKNOWN; it is not converted into a parry hint.

Earlier cases retained:

- General c1020 animation 3003 resolves to attacks 10200130/10201130. Generic
  effects no longer hide its common deflectable response. Animation 3004 still
  stays UNKNOWN without a specific NPC identity because sword/spear variants disagree on deflectability.
- Bandit c1550 animation 3003 resolves to thrust 15500050 plus 15500940. The
  second row is explicitly named `見切られダミー` (Mikiri detection dummy), has
  thrust attributes and counter posture damage, and zero damage/effect payload.
  It no longer turns that thrust into an unrelated conflicting attack.
- Its BulletBehavior judge 980 resolves to bullet 15500980, attack 8 and warning
  effect 211000. The inspected route is a zero-damage perilous-warning carrier,
  with no child projectile. Dummy attacks 0 and 8 are allowed only after checking
  their payload. Visual-only carriers additionally require no effects, child
  bullets, generated objects or auto-search NPC route. Other projectiles remain
  unresolved; the bullet enable flag alone is not used to ignore them.
- Mikiri requires actual thrust attributes, a simultaneous explicitly named
  marker hitbox and a player-to-that-model Mikiri route in ThrowParam. The
  Japanese `見切り` row names and throw kinds 30000/30100/30110 establish those
  routes. A thrust name alone is insufficient. Without the marker/route,
  deflectable thrusts fall back to PARRY. Grab/sweep/unknown are never Mikiri.

These interpretations use the original parameter schema and the modding tools'
reverse-engineered field names: [Paramdex AtkParam](https://github.com/soulsmods/Paramdex/blob/master/SDT/Defs/AtkParam.xml),
[ThrowParam](https://github.com/soulsmods/Paramdex/blob/master/SDT/Defs/ThrowParam.xml),
[Smithbox attack attributes](https://github.com/vawser/Smithbox/blob/main/src/Smithbox.Data/Assets/PARAM/SDT/Param%20Enums/ATKPARAM_ATKATTR_TYPE.json)
and the locally hashed DSAnimStudio Sekiro TAE template. They still need
representative gameplay checks, especially across behavior variants and mods.

## Compiled-code investigation

The existing hook already listens at the completed animation-batch boundary,
RVA `0xb5bef0`. Inspection of the preserved loaded code at `0xb5c730` shows it
iterating the same ten-entry ring, looking up an animation through `0x98ed70`,
then calling `0xb58530` with the timeline pointer and previous/current animation
times. This supports the animation/event path used by the observer.

`scripts/inspect-attack-dispatch.py` reproduces that disassembly and compares
the installed executable bytes with the preserved loaded code. Those bytes
differ at the known function address; disassembling that disk region directly
does not recover the same function. No full Ghidra decompilation or new runtime
event-dispatch/result hook was performed. Actual hitbox dispatch interception
would require a fresh loaded-image investigation beyond the preserved fragment.
The requested incoming response display can use the existing batch hook and
the freshly resolved data without waiting for contact-result reverse engineering.

## Reproduce

```powershell
python scripts/inspect-game-archives.py 'C:/Program Files (x86)/Steam/steamapps/common/Sekiro' --all-enemies --output-directory dist/response-research-2026-09-17/extracted
python scripts/inspect-attack-params.py 'C:/Program Files (x86)/Steam/steamapps/common/Sekiro' --output-directory dist/response-research-2026-09-17/extracted --include-warning-data
python scripts/generate-incoming-attacks.py --check
python scripts/test-incoming-attacks.py
python scripts/inspect-attack-dispatch.py 'C:/Program Files (x86)/Steam/steamapps/common/Sekiro/sekiro.exe'
```

The download script supplies schema references; generated hashes identify the
actual versions used. The disassembler also needs Capstone in `dist/game-tools`
and the preserved loaded-code fragment. These development tools are not player
requirements. No decompiled game program is shipped in the mod.

## Longer-session diagnostics

`observer-PID.alerts.csv` complements the full-frame render CSV. It records
received decision/phase/identity transitions and a one-second heartbeat with
the same source hashes, plus NPC ID, variation, activation boundaries and
progress. It continues when the full-frame stream reaches 16 MiB; its own
16 MiB cap remains. The queue is bounded and may drop records, reported by F9.
This is sparse draw-submission evidence, not every frame, confirmed visibility,
contact or successful input.

The preserved 0.10.0 log contains 50,234 frames over 846 seconds. A simulation
of the sparse keys retained 1,159 rows, about 419 KB in the old column format.
This demonstrates reduced volume, not a guaranteed session duration.
