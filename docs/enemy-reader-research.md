# Read-only live enemy research: 2026-09-12

The user played normally while the Python research tool observed Sekiro process
24508. Enemy identification, a fresh animation history entry, and normal camera
pose can now be read from this exact executable. The observer DLL and existing
package were not changed during the capture. Subsequently the
[0.4.0 preview](cue-preview.md) integrated these reads and added an experimental
overhead light; exact parry contact timing remains unvalidated.

Executable SHA-256:
`637aca527538c0ec6e1f136c8ed66046e95dfbdbb1f51926e134d9916398b856`.

The tool requests only `PROCESS_VM_READ | PROCESS_QUERY_INFORMATION`. It does
not inject code, call game functions, write game memory, or generate inputs.
Reads use `ReadProcessMemory` into owned buffers with pointer and count limits.
Snapshots recheck ownership but cannot provide an atomic engine snapshot.

## Loaded-image evidence

The source references are pinned in the download script, including
[SekiroTool offsets](https://github.com/borgCode/SekiroTool/blob/189649781b00ba1f2fddb5d8bbfff7684ff5b647/SekiroTool/Memory/Offsets.cs)
and the
[ElaDiDu practice table](https://github.com/ElaDiDu/Sekiro-Practice-CT/blob/328065da6dc3c5c9138cd31030791804e8e14e4e/Ela_Sekiro_Table.CT).
These were read as source text, not executed. Offsets below were additionally
checked through disassembly of the hash-gated running game's loaded image.

| Global | Expected RVA | Loaded reference instruction RVA |
|---|---:|---:|
| WorldChrMan | `0x3d7a1e0` | `0xa66d99` |
| FieldArea | `0x3d5c0a0` | `0x80e744` |
| LockTgtMan | `0x3d78058` | `0xb30e0d` |

WorldChrMan's pattern has four matches; only one resolves the expected global.
All candidates are preserved in the output. FieldArea and LockTgtMan each have
one matching reference. RendMan's pattern has 13 references to `0x3d94970`,
and is not accepted by the current unique-reference rule or used by this reader.
On-disk pattern misses must not be mistaken for misses in the loaded image.

## Locked character lookup

Loaded function `0x9c5ef0` returns no lock point when LockTgtMan `+0x2830` is
zero. Otherwise it traverses the list starting at `+0x10`, following point
`+0x80` links, selecting the point whose `+0x90` flags include `0x20`.
The research equivalent rejects cycles and lists exceeding 128 points.

Loaded call sites pass point `+0x88` to function `0x9c31f0`, which reads a
character handle at accessor `+0x78`, requires handle type 1, and resolves it
through WorldChrMan. This accessor is not the player or LockTgtMan itself.

Functions `0xa49f90`, `0xa07b40`, `0xc33200`, and `0xc331d0` establish the
lookup used by the research reader:

- Character handle group: `(handle >> 14) & 63`; index: `handle & 16383`.
- WorldChrMan `+0x10` points to group metadata; count at metadata `+0x18`, plus 3.
- Group bucket: WorldChrMan `+0x518 + group*8`.
- Bucket count at `+0`; entry array pointer at `+8`; entry stride `0x38`.
- Entry's first pointer identifies ChrIns. ChrIns `+8` must match the handle.

Bounds and pointer/handle ownership are checked before accepting the actor.
ChrIns `+0x1ff8` points to modules; physics module `+0x68` supplies world position
at `+0x80`. Data module `+0x18` supplies HP/max HP at `+0x130`/`+0x134`.
Character model name is read from the resource pointer at ChrIns `+0x30`.

## Animation history correction

The fixed offsets animation module `+0x20`/`+0x24` appeared to update about
once per ten frames. Loaded writer `0xb5c580` explains why: these belong to the
first entry of a rotating history, not a dedicated current-animation field.

There are ten entries with stride **0x14**, starting at module `+0x20`.
The next-write index is the signed integer at `+0xe8`. The last completed
entry is `(next_write + 9) % 10`. Each entry contains animation ID, previous
animation time, next animation time, duration, and sequence integer. The
research reader records the next animation time and checks that the write
index did not change during the read. This matches the writer's observed
stores and yielded fresh frame observations during subsequent play.

Multiple animation tracks/blends may be submitted in one engine frame.
Selecting the last entry is a research observation, not proof that it is
always the sole attack animation. Blend, cancellation, pause, and loading
behavior still need checks before using it as a press prompt.

**Earlier captures without `animation_ring_head` contain stale history frames
and must not be used for precise press timing.** The corrected captures include
previous/next time, duration, sequence, and ring head explicitly.

## Camera and candidate overhead anchor

FieldArea `+0x30` identifies ChrCam. Loaded virtual function `0xf01260` consumes
basis vectors at `+0x10`, `+0x20`, `+0x30`, position at `+0x40`, and lens values
at `+0x50` through `+0x5c`. Observed vectors are orthonormal. The reader checks
finite values, basis orthogonality, lens bounds, and owner stability. It rejects
the known debug camera mode from GameRend `+0xe0`.

The target accessor's first vector is a lock anchor around the torso. The
candidate overhead anchor uses its x/z and bounding-box maximum y at accessor
`+0x50`, plus 0.15 world units. This is not a verified head bone. Research
projection returns normalized screen coordinates only for visible points and
16:9 or wider camera aspect; narrower views require the native FOV conversion.
No cue is rendered by this tool. Camera mode selection and pixel alignment
have not been visually verified.

## Capture results

Ignored outputs are under `dist/game-analysis`; raw records may contain live
process pointers and locally extracted game data and are not packaged.

- The first nearby-actor capture had 827 rows with no failed traversals.
- The next 662-row capture identified the same locked general, model c1020,
  throughout, with no lock-reader errors. Its fixed-entry animation clock was
  subsequently found stale and is unsuitable for timing validation.
- Corrected capture `live-24508-1789258535394975800.actors.jsonl` has 662 rows,
  one failed traversal, 316 locked soldier samples, and 316 distinct locked
  animation/time frames. All 15 observed animation IDs match exactly one entry
  in the extracted c1010 TAE; no ID truncation or guessed fallback was used.
- Camera pose was read in 661 rows; overhead projection returned visible
  coordinates in 286 rows. These are numerical checks, not visual alignment
  or successful-parry checks.

`live-capture-summary.json` records hashes of the corrected capture and its
timeline input, exact animation matches, hitbox event ranges, and limitations.

## Reproduce

With a running supported game, substitute its actual PID:

```powershell
python .\scripts\inspect-live-game.py 24508 --duration 20
```

The capture is limited to 60 seconds per invocation. Python's standard library
is sufficient for the reader; the existing fetched practice-table text is
required for signature verification. Capstone was used separately for research
disassembly and is not required to run the capture script.

Remaining product work: verify overhead placement visually, resolve timeline imports and attack
behavior/geometry, predict player contact rather than equating it with hitbox
activation, and test that the cue guides a successful manual deflect.
