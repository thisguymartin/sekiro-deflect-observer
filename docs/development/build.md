# Build on Windows

Build in a new checkout if you need to preserve an existing package.
`scripts/build.ps1` replaces a ZIP with the same version in `dist`.

## Install the development tools

Install:

1. Visual Studio Build Tools with **Desktop development with C++**, the MSVC x64
   compiler, and the Windows SDK.
2. Rust through [rustup](https://rust-lang.org/tools/install/) with the MSVC
   toolchain.
3. Git if you clone the repository.

Open **Developer PowerShell for Visual Studio** in the repository root. Run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\build.ps1
```

The script checks formatting, runs Clippy and the Rust tests, builds the release
DLL, checks non-game host rejection, and creates:

```text
target/x86_64-pc-windows-msvc/release/sekiro_deflect_observer.dll
dist/SekiroDeflectObserver-0.12.4-preview-windows-x64.zip
dist/SekiroDeflectObserver-0.12.4-preview-windows-x64.zip.sha256
```

The generated tables are checked in. A normal build does not need local game
archives or Python.

## Build with GitHub Actions

The `Windows proof of concept` workflow runs `scripts/build.ps1`. Its artifact
contains the me3 package and checksum. Extract the outer workflow artifact to
find the mod ZIP.

The workflow does not publish a permanent GitHub or Nexus release.

## Resolve common build failures

| Failure | Action |
| --- | --- |
| `cargo` is not recognized | Install Rust and open a new terminal. |
| `link.exe` or the Windows SDK is missing | Install the C++ workload and use Developer PowerShell. |
| The Windows target is missing | Run `rustup target add x86_64-pc-windows-msvc`. |
| A dependency download fails | Restore network access, then retry the locked build. |
| Formatting, Clippy, or tests fail | Fix the reported failure before packaging. |

After the build passes, follow the [release test procedure](release.md).
