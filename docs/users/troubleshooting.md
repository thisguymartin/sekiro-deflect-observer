# Troubleshoot the observer

Start with the problem that matches what you see.

| Problem | Action |
| --- | --- |
| No overlay with me3 | Extract the package fully. Keep the DLL beside `observer.me3`. Inspect the launch output and local logs. |
| The rail is misplaced | Use F6 and F7. Press F10 to reset offsets. Check `anchor`, `width`, and offsets in `cue.toml`. |
| `LOCKED` remains during attacks | Open F9. The move may have no supported classification, or the reader may lack fresh animation data. |
| F9 reports an unsupported build | Record the executable hash and observer version. The observer rejects unknown executables. |
| The moon is amber | Open F9 for the practice status. Disable and re-enable practice after an external speed change. |
| me3 is not recognized | Install me3, reopen the terminal, or open `observer.me3` directly. |
| The game crashes after installation | Close the game and remove the observer profile. Preserve the logs before retrying. |

Settings and logs live in `%LOCALAPPDATA%\SekiroDeflectObserver`.

When reporting a problem, include:

- The observer version from F9.
- The exact error text.
- The enemy and move, when relevant.
- The Windows version, GPU driver, and loader version.
- A short clip for a visual or timing problem.

Do not upload game executables, save files, or memory dumps with a routine bug
report. A CSV draw row proves that the observer submitted draw commands. It does
not prove that a frame appeared on screen.

Developers can find build failures in the
[Windows build guide](../development/build.md).
