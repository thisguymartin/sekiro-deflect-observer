# Install and remove the observer

The current package requires Windows x64, the Steam version of Sekiro, and me3.
You do not need development tools to run it.

## Install

1. Close Sekiro completely.
2. Extract `SekiroDeflectObserver-0.12.4-preview-windows-x64.zip` into its own
   folder, such as `C:\Mods\SekiroDeflectObserver`.
3. Install me3 from its
   [official Windows release](https://github.com/garyttierney/me3/releases).
4. Keep Steam running.
5. Open `observer.me3` or run `launch-observer.cmd` from the extracted folder.
6. Load a save and lock onto a living enemy.
7. Press **F9** and confirm that it shows version `0.12.4-preview`.

GitHub's **Code > Download ZIP** downloads source code. It does not contain a
ready-to-run package.

To start the observer from a terminal, run this command inside the package
folder:

```powershell
me3 launch --game sekiro --profile .\observer.me3
```

Keep the DLL and `observer.me3` in the same folder. Remove an older drop-in
observer before using the me3 package. A running game keeps its loaded DLL, so
restart Sekiro after replacing a build.

## Verify the first launch

After the game loads:

1. Press **F9** and confirm the expected version.
2. Lock onto an enemy and look for the top-center rail.
3. Press **F6** and **F7** to check placement controls.
4. Press **F8** twice to hide and restore the HUD.
5. Press **F11** only if you want to enable enemy-speed practice.

Read [the HUD guide](../gameplay/hud.md) before interpreting an attack label.
If the overlay does not appear, use the
[troubleshooting guide](troubleshooting.md).

## Remove

1. Close Sekiro.
2. Start Sekiro normally through Steam without the observer profile.
3. Delete the extracted observer folder if you no longer need it.

Keep shared me3 files if another mod uses them. The me3 package does not require
you to replace game files or save files.
