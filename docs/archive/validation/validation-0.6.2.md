# 0.6.2 visual update

The user requested a marker closer to the supplied YouTube reference and asked
whether a longer deflect window was possible. This update changes appearance;
it does not modify combat rules or increase the timing estimates.

The original screenshot path was unavailable in this pass, and the video page
could not be loaded. The supplied video's [thumbnail](https://i.ytimg.com/vi/bejKX5vw2-Y/maxresdefault.jpg)
was downloaded and inspected instead: it shows a bright filled diamond, vertical
needle, translucent track, chevrons and a green segment. It is stored locally at
`dist/review-0.6.2/youtube-reference.jpg`; it is not bundled with the mod.

The new marker uses an ivory fill, white center, soft halo and thin vertical
needle. The track is 360 x 12 reference pixels; the colored segment has tapered
ends confined to the original estimate. English action labels remain, with
clearance above the needle. The diamond's center and the cue duration retain the
same timing meaning. Existing green/orange/blue response colors are preserved.

The actual shared ImGui drawing code produced the offline layout at
`dist/review-0.6.2/layout/cue-layout.png`, and it was visually inspected for marker
shape, label clearance and all three response colors. The old layout and drawing
source were copied to the same review folder before editing. This is a synthetic
visual check, not a new gameplay timing trial.

Reproduce without replacing the earlier layout evidence:

```powershell
cargo run --locked --offline --example cue-layout --target x86_64-pc-windows-msvc -- dist/review-0.6.2/layout
python scripts/render-cue-layout.py dist/review-0.6.2/layout
```

The runtime attack reader, generated tables, 150 ms parry estimate and 300 ms
dodge/jump estimate are unchanged from 0.6.1. All previous timing, coverage,
blend and head-anchor limitations remain. The running process at the start of
this update had loaded 0.6.1; it must fully exit before loading the new package.

A longer real deflect window is technically possible as a separate gameplay
modification. The [author's Easier Parries description](https://www.nexusmods.com/sekiro/mods/2131?tab=description)
documents increasing it by a configurable duration. That package was researched
only, not downloaded or installed. No compatibility or safety claim is made
for combining it with this observer. A longer visual warning alone would give
advance notice without changing which inputs Sekiro accepts.

## Release verification

35 existing Rust tests passed. Formatting, all-target Clippy with warnings denied,
the optimized Windows build and the DLL non-Sekiro-host rejection check passed.
No new tests were added for the cosmetic drawing change. The offline render was
visually inspected; live 0.6.2 rendering is not claimed as verified.

The ZIP was extracted into a new Mods directory. All six checksum-manifest files
matched, the DLL matched the tested release output, the profile referenced the
local DLL and the ZIP matched its checksum sidecar. Older packages were preserved.

- Launch: `Mods/SekiroDeflectObserver-0.6.2-preview/observer.me3`
- ZIP: `dist/SekiroDeflectObserver-0.6.2-preview-windows-x64.zip`
- ZIP SHA-256: `d5d71f152b92f93aa76aa4805547e6ef9734927adb406a3d987cba587e4078ea`
- DLL SHA-256: `bc01e1deaf7d8ac8ddd51abdf0d573ccc6d2e333f069f4f112bbbc9c589db4f3`

Close Sekiro fully, keep Steam running, launch the new profile, and verify
0.6.2-preview in F9. The running 0.6.1 process cannot adopt these visuals until
it restarts with the new DLL.
