# Share the 0.6.3 friend beta

Send `SekiroDeflectObserver-0.6.3-preview-drop-in-windows-x64.zip`. Its two runtime
files go beside `sekiro.exe`; the friend launches through Steam normally. No
separate me3 installation is needed for this variant. `START-HERE.txt` in the ZIP
contains the three-step instructions, controls, removal and troubleshooting.

If `dinput8.dll` already exists, do not overwrite it. Compatibility with an
existing loader needs to be resolved first. Do not load the observer twice by
combining this package with the observer's me3 launch profile.

This is an experimental friend beta. The existing observer binary is unchanged;
the drop-in startup route still needs a live Sekiro and clean-PC trial. Timing,
response classification and boss/move coverage retain their existing limits.

## Package and evidence

- Drop-in ZIP SHA-256: `3fd294cc2b7f5f6ecffd906c3041cfd2871c813842a5c9baade2d4e80891bfc6`
- Observer ASI SHA-256: `711ae3c88571fa619dd373b90340f4e81b9b128c58f89e4d2848596cc711d2c9`
- Loader DLL SHA-256: `fa266e3513d02c08a1b808f28c10538a489eaffaa4b0707f7cc1066e71b5afd7`
- Bundled loader: [Ultimate ASI Loader](https://github.com/ThirteenAG/Ultimate-ASI-Loader) 9.7.4 x64 by ThirteenAG.

The supported Sekiro executable imports DINPUT8.dll. An isolated native smoke
host loaded the adjacent ASI and exercised DirectInput8Create forwarding; the
observer then rejected the non-game host before installing hooks. The smoke
test creates/releases the DirectInput interface without polling or controlling
input devices. Its LOCALAPPDATA is redirected into its test directory.

The first smoke attempt could not establish startup because its log path was
outside the test sandbox; rerunning with a local test log directory passed.
The successful repeat through `scripts/test-asi-loader.ps1` used PID 28252.
The ZIP was reopened and all eight files compared with their packaged bytes.
No game files, saves, recordings, development tools or me3 binaries are included.
The loader is distributed with its and its linked-library license notices.

The [build-and-run guide](windows.md) now covers both installation methods,
building the observer from source, and running a freshly compiled DLL with the
drop-in loader. The normal build command creates the me3 package.

To repackage the tested drop-in release, retain `dist/drop-in-research`, the
original 0.6.3 ZIP, and its extracted `Mods/SekiroDeflectObserver-0.6.3-preview`
folder for the smoke check. Then run `scripts/test-asi-loader.ps1` followed by
`scripts/package-drop-in.py` in a separate restore folder. See the guide's
[complete restore steps](windows.md#reproduce-the-existing-drop-in-zip).
The packaging script refuses to replace an existing archive and requires the
pinned binary hashes. The loader URL uses a moving upstream tag: a future
download may differ and must not silently replace the pinned binary.

## Preserve the working version

Keep the original 0.6.3 me3 ZIP, drop-in ZIP and their checksum files. A source
snapshot must include the current working files, including uncommitted changes;
Git history alone does not save those edits. Store a copy of the backup on a
second drive or your own cloud storage. This workspace copy is not off-device
backup. Keep screenshots with their version/evidence labels.

The source snapshot includes generated timing tables, so rebuilding the observer
does not require distributing or re-extracting the game's archives. Raw game
archives and local captures are excluded from the source ZIP. Keep the local
research directory separately if you want to regenerate the timing data later.

## Nexus beta draft

Suggested title: **Sekiro Deflect Observer - Experimental Beta**.
Version: **0.6.3-preview**. Label the download **Drop-in Windows x64 - Beta**.
Use the text in [the listing draft](nexus-beta-draft.md), add accurate versioned
images, and list the bundled ASI loader in credits. The drop-in package needs no
external me3 requirement. A me3 variant, if uploaded separately, must state its
me3 requirement.

Nexus requires accurate descriptions, credits and relevant tags, including
disclosure of generated code/UI and AI-written page media. For this Codex-written
implementation, use **AI-Generated Content**; using the supplied AI-written
description also calls for **AI Media**. Check the current options at submission.
See [Nexus submission guidelines](https://help.nexusmods.com/article/28-file-submission-guidelines).

Upload the actual runtime ZIP, not the private source backup or a ZIP containing
another ZIP. Nexus recommends standard ZIP/7z and rejects nested archives that
cannot be scanned. See [Nexus archive guidance](https://help.nexusmods.com/article/117-why-has-my-mod-been-quarantined).

No Nexus page, public release or file upload has been published by this preparation.
