# Observer DX11 isolation patch

Base: crates.io `hudhook` 0.9.2, upstream https://github.com/veeenu/hudhook.
The upstream LICENSE and MinHook license are retained. This local patch changes
only `src/renderer/backend/dx11.rs`.

The original backend draws directly on the host immediate context. Its manual
StateBackup omits output targets, ignores some NULL bindings, and misses shader
resources that output binding implicitly unbinds. It also skips restoration if
buffer upload fails. Our offscreen D3D11 WARP regression reproduces the output
target leak with the unmodified 0.9.2 backend.

Record drawing on a private deferred context and execute the finished list with
`RestoreContextState = TRUE`. Discard partial lists on draw failure, clear private
bindings before finishing to release swap-chain references, and do no work for
empty draw data. Font/texture uploads remain on the immediate context but touch
only renderer-owned resources, without changing bindings. No color-space,
gamma, HDR, swap-chain format, fullscreen, or game setting changes are made.
If a deferred context cannot be created, initialization fails without falling
back to drawing directly on the game's context.

Tests: `cargo test --locked --offline --target x86_64-pc-windows-msvc --test dx11-render-isolation`.
The test includes the production backend source and checks real WARP rendering,
repeated/empty frames, host target, aliased resources, NULL bindings, viewport,
and unchanged pixels outside the local cue for both UNORM and sRGB targets.

API contract: https://learn.microsoft.com/en-us/windows/win32/api/d3d11/nf-d3d11-id3d11devicecontext-executecommandlist
