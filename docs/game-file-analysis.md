# Local attack-timeline findings: 2026-09-12

The installed game contains usable animation event data. The player does not
need to chart every enemy's timing by hand. A read-only inspector now indexes
the archives and extracts those timelines into the project workspace.

## Installation and scope

Game directory: `C:\Program Files (x86)\Steam\steamapps\common\Sekiro`.

Executable SHA-256:
`637aca527538c0ec6e1f136c8ed66046e95dfbdbb1f51926e134d9916398b856`.
This matches the existing research build's gate. The directory contains packed
`Data1` through `Data5` archives, with no unpacked `chr` directory.

Analysis indexed 8,765 entries across the five archives. The downloaded UXM
dictionary supplies path matches for 8,752 entries, including 142 character
animation archives. An animation archive is not equivalent to an enemy: some
contain player animations, throw animations, or shared/empty animation data.

Selected archives were decrypted in owned memory, decompressed using Windows
AES and the Oodle codec shipped with the game, and parsed as BND4/64-bit Sekiro
TAE containers. JSON outputs were written under `dist/game-analysis` only. The
game executable, archives, launch configuration, and saves were not modified.
No third-party practice table was executed.

## Actual extracted data

| Archive | Animation entries | Valid attack hitbox events (TAE type 1) |
|---|---:|---:|
| `c0000.anibnd.dcx` (player) | 2,209 across 65 TAEs | 1,514 |
| `c1010.anibnd.dcx` | 500 | 108 |
| `c1020.anibnd.dcx` | 364 | 87 |
| `c7100.anibnd.dcx` | 189 | 85 |
| `c5400.anibnd.dcx` | 347 | 137 |

The [character reference](https://github.com/SekiroResurrection/modding-wiki/wiki/Characters)
identifies c1010 as Ashina Soldier, c7100 as Genichiro Ashina, and c5400 as Sword
Saint Isshin. The archive counts above are measured from this installation.
They count timeline events, not unique enemy moves or confirmed contacts.

In the player's TAE 4050, animation 203000 contains an AddSpEffect event
(type 67) applying effect 105010 from 0.0 to 0.20000000298 seconds. This
corroborates the published 0.2-second ordinary deflect animation duration using
local data. Other animations contain different durations; this is not a
universal acceptance-window measurement or a reason to hardcode one timer.

The parser preserves unusual event times, including negative pre-roll, while
marking them ineligible for timing calculations. It also preserves animation
import references rather than presenting an empty alias as a complete attack.
Resolving aliases, behavior IDs, weapon hitbox geometry, and deflectability
remains necessary before constructing a usable runtime timing map.

## Runtime reader leads and limits

The pinned [ElaDiDu practice-table reference](https://github.com/ElaDiDu/Sekiro-Practice-CT/blob/328065da6dc3c5c9138cd31030791804e8e14e4e/Ela_Sekiro_Table.CT)
documents an animation ID at the module collection (`+0x1ff8`), animation
module (`+0x10`), then `+0x20`. Live inspection established that this is a
rotating history, so reading the fixed first entry gives an old frame.
Do not use its adjacent `+0x24` float as a current animation clock. The
corrected research reader selects the latest completed ring entry using the
next-write index at `+0xe8`; see [live reader findings](enemy-reader-research.md).
The table also supplies reference signatures for WorldChrMan, FieldArea,
and LockTgtMan.

All three reference signatures had zero matches in the executable's on-disk
code section. The inspector records this result explicitly. The executable
also has a `.bind` section; that alone does not establish why the patterns
did not match. Subsequently, the read-only live inspector found reference
instructions in the loaded image resolving all three expected global RVAs.
It also inspected the native target lookup and animation history writer
without calling them. These live observations apply to the exact hash above;
they do not establish support for other builds or all gameplay states.

An attack hitbox event describes an active attack phase. It does not, by
itself, establish when the moving blade intersects the player. The overhead
cue still needs integration of the research reads into the DLL, reach/contact
calculation, visually verified camera projection, and representative in-game verification. File analysis
answers whether attack data is available; it does not complete the cue.

## Reproduce the analysis

The [download script](../scripts/fetch-game-research.ps1) fetches public source
text into the ignored `dist` directory. It does not execute those references.
The archive inspector records SHA-256 hashes of the fetched references in its
inventory so the exact local source inputs are inspectable.

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\fetch-game-research.ps1
python .\scripts\inspect-game-archives.py 'C:\Program Files (x86)\Steam\steamapps\common\Sekiro' --timeline /chr/c0000.anibnd.dcx --timeline /chr/c1010.anibnd.dcx --timeline /chr/c7100.anibnd.dcx --timeline /chr/c5400.anibnd.dcx
```

The inspector requires Windows and Python's standard library. It hash-gates
the executable and reads game files without opening the game process. Outputs
include `archive-inventory.json` and one `*.timelines.json` per requested archive.

Verification: successful reads of the five archives above, consistency of the
local ordinary-deflect event with the published animation duration, and a NIST
AES-128 known-answer check for the Windows decryption adapter. Follow-up
read-only captures observed locked soldiers/general during normal play. The
latest capture matched all 15 observed soldier animation IDs exactly to their
TAE entries. No in-game timing cue has been implemented or verified.

Format provenance: [UXM](https://github.com/Nordgaren/UXM-Selective-Unpack),
[SoulsFormats](https://github.com/JKAnderson/SoulsFormats), and
[SoulsFormatsNEXT](https://github.com/soulsmods/SoulsFormatsNEXT). The new Python
inspector implements the required read-only parsing; downloaded game data and
reference source files are not included in the observer package.
