# Incoming attack responses - 0.9.0-preview

The default HUD answers “what attack is coming and how can I respond?” It does
not wait for the enemy to face Wolf or enter a distance threshold. A current
attack capture can show a response immediately; it does not require three
captures to estimate animation speed. Since 0.10.0, the requested default cue
is a slim rail below the enemy's upper posture bar, with no large panel backing.

| Label | Meaning |
| --- | --- |
| PARRY | All resolved variants allow deflection |
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

The [generated evidence](incoming-coverage.json) records all exact model,
animation, phase, response and attack-parameter IDs, plus source hashes. Across
2,161 phases on 54 models it classifies **1,673 parry, 40 dodge, 56 jump,
63 Mikiri and 16 no-parry**; 313 remain unknown. These are data classifications,
not unique moves, tested successes or a claim of complete enemy/form support.
The old timing tables and coverage ledger remain separate and unchanged.

The old classifier excluded entire animations for generic effects or aiming
events. Incoming classification instead resolves the overlapping melee events
through BehaviorParam and AtkParam, requiring all candidate behavior variants
to agree on a response. In particular:

- General c1020 animation 3003 resolves to attacks 10200130/10201130. Generic
  effects no longer hide its common deflectable response. Animation 3004 still
  stays UNKNOWN because sword/spear variants disagree on deflectability.
- Bandit c1550 animation 3003 resolves to thrust 15500050 plus 15500940. The
  second row is explicitly named `見切られダミー` (Mikiri detection dummy), has
  thrust attributes and counter posture damage, and zero damage/effect payload.
  It no longer turns that thrust into an unrelated conflicting attack.
- Its BulletBehavior judge 980 resolves to bullet 15500980, attack 8 and warning
  effect 211000. The inspected route is a zero-damage perilous-warning carrier,
  with no child projectile. Arbitrary zero-damage or mixed projectile events
  are still rejected; only the checked warning route is excluded.
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
