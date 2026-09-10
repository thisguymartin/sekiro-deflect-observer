Sekiro Deflect Observer 0.2.0-dev
Windows x64 overlay proof of concept

THIS BUILD DOES NOT DETECT DEFLECT WINDOWS.

It loads a DirectX 11 overlay inside Sekiro, reports that the observer is
loaded, and displays UNKNOWN because no verified game-state reader exists.
It does not turn green in response to LB/L1. Use this build to verify the
loader, rendering, hotkey, and startup diagnostics on your Windows PC.

INSTALL AND RUN

1. Install me3 from https://github.com/garyttierney/me3/releases
2. Extract this entire ZIP to a separate folder, such as
   C:\Mods\SekiroDeflectObserver. Keep the files together.
3. Start Steam and close Sekiro if it is already running.
4. Double-click observer.me3, or run launch-observer.cmd.
5. Look for the Sekiro Deflect Observer panel in the top-left corner.
6. Press F8 once to hide it. Press F8 again to show it.

Use windowed or borderless mode for the first test. Test fullscreen
separately. In-game behavior has to be validated on your setup.

No Cheat Engine, Rust, Visual Studio, Python, account, or backend is needed
to run this built package. Rust and the C++ build tools are development
dependencies used to create the DLL.

STOP AND REMOVE

F8 hides the panel. It does not unload the DLL. Close Sekiro to stop it.
For an ordinary session, start Sekiro normally through Steam instead of this
profile. To remove the observer, close the game and delete this extracted
folder. Do not delete a shared me3 installation used by other mods.

Startup logs are kept under %LOCALAPPDATA%\SekiroDeflectObserver.
You may delete that project-specific log folder after closing the game.
The observer makes no network requests. me3 and Steam are separate software
with their own behavior. Rendering hooks change the render process, but
this build contains no gameplay-memory writer or game-state reader.

REPORT A TEST

Record Windows build, Sekiro version, GPU and driver, display mode, me3
version, whether the panel appeared, whether F8 worked, and whether normal
controls still worked. Include the observer log and any me3 launch error.
Do not upload sekiro.exe, game assets, saves, or full memory dumps.

Source and development instructions:
https://github.com/thisguymartin/sekiro-deflect-observer

See LICENSE and THIRD-PARTY-NOTICES.txt for license information.
