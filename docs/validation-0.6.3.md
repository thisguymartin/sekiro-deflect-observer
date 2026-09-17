# 0.6.3 visual refinement

This update follows the user's request to continue directly with the design.
The locally saved thumbnail for the supplied video was inspected again at
`dist/review-0.6.2/youtube-reference.jpg`. This is a thumbnail-based interpretation,
not a claim that the full reference video was reviewed in this pass.

The marker now has a larger ivory diamond, a longer thin needle and a small
magenta pointer above it. The dark ribbon extends past the brighter end chevrons
with tapered wings. Outlined English text replaces the rounded label box, with
clearance above the pointer. Green PARRY, orange DODGE and blue JUMP remain.
The pointer is gray for unverified or out-of-reach phases.

The functional lane remains 360 x 12 reference pixels, with decorative wings
bringing the total width to 424. The diamond center and colored boundaries use
the same animation-time mapping. The actual parry window, 150 ms parry estimate,
300 ms dodge/jump estimate, target reader, attack coverage, anchor and controls
are unchanged. Existing gameplay limitations in [the preview](cue-preview.md)
and [the reader validation](validation-0.6.md) still apply.

Before editing, the current drawing source and offline layout were saved as
`dist/review-0.6.3/before-cue-draw.rs` and `before-layout.png`. The shared ImGui
drawing code generates the new offline visual check; this is not gameplay or
contact-timing evidence.

```powershell
cargo run --locked --offline --example cue-layout --target x86_64-pc-windows-msvc -- dist/review-0.6.3/layout
python scripts/render-cue-layout.py dist/review-0.6.3/layout
```

Output: `dist/review-0.6.3/layout/cue-layout.png`.

The final offline render was visually inspected for READY, PARRY, DODGE and
JUMP, including label/pointer clearance, larger diamond shape and response colors.
This pass does not establish readability over every gameplay background.

## Release verification

All 35 existing Rust tests passed, along with formatting, all-target Clippy with
warnings denied, the optimized Windows build and the DLL non-Sekiro-host
rejection check. No new tests were added for the cosmetic change.

The ZIP was extracted into a new Mods folder. All six manifest entries matched,
the packaged DLL matched the tested release DLL, the launch profile referenced
that local DLL, and the ZIP matched its checksum sidecar. Older packages and
preexisting work were preserved.

- Launch: `Mods/SekiroDeflectObserver-0.6.3-preview/observer.me3`
- ZIP: `dist/SekiroDeflectObserver-0.6.3-preview-windows-x64.zip`
- ZIP SHA-256: `79648178d33ef5a83242f02325f1554ff02af63a92b2d6f8c60d4078aebd2799`
- DLL SHA-256: `711ae3c88571fa619dd373b90340f4e81b9b128c58f89e4d2848596cc711d2c9`

Process 20772 had loaded the 0.6.2-preview DLL during this update. Close Sekiro
fully, keep Steam running, launch the new profile and verify 0.6.3-preview in F9.
Live 0.6.3 rendering and gameplay timing have not been verified in this pass.
