# Developer guide

Sekiro Deflect Observer is a Rust Windows x64 DLL. It reads game observations,
classifies the locked enemy's attack, and submits a DX11 HUD through hudhook.

Start with these pages:

1. Read [how one attack becomes a cue](../how-it-works.md).
2. Review [architecture and ownership](architecture.md) before changing a module.
3. Follow the [Windows build guide](build.md).
4. Run the checks in [testing](testing.md).
5. Use the [release procedure](release.md) before sharing a package.

Research inputs and generated data live under [research](../research/README.md).
Historical results live under [the archive](../archive/README.md). Neither section
defines current product behavior.

The project pins its Rust toolchain and dependencies. Build the Windows target
through `scripts/build.ps1` so the package receives the same checks as CI.
