"""Package the tested 0.6.3 observer with a pinned, separately licensed ASI loader.

Requires the official loader and license files saved in dist/drop-in-research.
Never installs files in the game directory or replaces earlier archives.
"""
from hashlib import sha256
from pathlib import Path
import json
import zipfile

ROOT = Path(__file__).resolve().parent.parent
VERSION = '0.6.3-preview'
RESEARCH = ROOT / 'dist/drop-in-research'
ORIGINAL = ROOT / f'dist/SekiroDeflectObserver-{VERSION}-windows-x64.zip'
OUTPUT = ROOT / f'dist/SekiroDeflectObserver-{VERSION}-drop-in-windows-x64.zip'


def checked(path, expected):
    data = path.read_bytes()
    if sha256(data).hexdigest() != expected:
        raise ValueError(f'Unexpected bytes in {path.name}')
    return data


def main():
    if OUTPUT.exists() or Path(str(OUTPUT) + '.sha256').exists():
        raise FileExistsError('Drop-in package already exists; preserve the old artifact.')
    checked(ORIGINAL, '79648178d33ef5a83242f02325f1554ff02af63a92b2d6f8c60d4078aebd2799')
    loader = checked(RESEARCH / 'loader/dinput8.dll', 'fa266e3513d02c08a1b808f28c10538a489eaffaa4b0707f7cc1066e71b5afd7')
    with zipfile.ZipFile(ORIGINAL) as original:
        observer = original.read('sekiro_deflect_observer.dll')
        assert sha256(observer).hexdigest() == '711ae3c88571fa619dd373b90340f4e81b9b128c58f89e4d2848596cc711d2c9'
        files = {
            'dinput8.dll': loader,
            'sekiro_deflect_observer.asi': observer,
            'START-HERE.txt': (ROOT / 'packaging/drop-in/START-HERE.txt').read_bytes(),
            'SekiroDeflectObserver/LICENSE.txt': original.read('LICENSE'),
            'SekiroDeflectObserver/OBSERVER-THIRD-PARTY-NOTICES.txt': original.read('THIRD-PARTY-NOTICES.txt'),
        }
    loader_notices = 'Ultimate ASI Loader 9.7.4 and bundled library notices\nhttps://github.com/ThirteenAG/Ultimate-ASI-Loader\n'
    for name, filename in [
        ('Ultimate ASI Loader / ThirteenAG', 'license'),
        ('injector / LINK/2012', 'injector-LICENSE.txt'),
        ('FunctionHookMinHook utility / praydog', 'injector-utility-LICENSE.txt.txt'),
        ('MinHook and Hacker Disassembler Engine', 'minhook-LICENSE.txt'),
        ('miniz', 'miniz-LICENSE.txt'),
    ]:
        loader_notices += f'\n--- {name} ---\n' + (RESEARCH / filename).read_text(encoding='utf-8') + '\n'
    files['SekiroDeflectObserver/LOADER-THIRD-PARTY-NOTICES.txt'] = loader_notices.encode()
    provenance = {
        'observer_version': VERSION,
        'observer_bytes': 'Identical to the tested me3 package; DLL extension changed to .asi only.',
        'loader': 'Ultimate ASI Loader 9.7.4 x64, ThirteenAG',
        'loader_url': 'https://github.com/ThirteenAG/Ultimate-ASI-Loader/releases/download/x64-latest/dinput8-x64.zip',
        'loader_archive_sha256': 'b145f768e4f1f4b5bf6e48ab7ff177c0694934d14ea8358c03b3f7628bcd54f4',
        'loader_dll_sha256': sha256(loader).hexdigest(),
        'source_reference_reviewed': '8ec2d64ff02a9f9efe6616a482cf057f6c6e7447',
        'source_reference_note': 'Consulted upstream source; the distributed loader binary was not rebuilt here.',
        'checks': ['Official archive digest matched GitHub metadata', 'x64 game imports DINPUT8.dll', 'DirectInput forwarding and adjacent ASI loading passed in an isolated native host', 'Observer rejected the non-game host before hooking'],
        'pending': ['Live Sekiro launch through this drop-in route', 'Clean install on a friend PC', 'Exact timing and full move coverage'],
    }
    files['SekiroDeflectObserver/BUILD-INFO.json'] = (json.dumps(provenance, indent=2) + '\n').encode()
    manifest = ''.join(f'{sha256(data).hexdigest()}  {name}\n' for name, data in sorted(files.items()))
    files['SekiroDeflectObserver/SHA256SUMS.txt'] = manifest.encode()
    with zipfile.ZipFile(OUTPUT, 'x', compression=zipfile.ZIP_DEFLATED) as archive:
        for name, data in sorted(files.items()):
            archive.writestr(name, data)
    with zipfile.ZipFile(OUTPUT) as archive:
        assert archive.testzip() is None
        assert set(archive.namelist()) == set(files)
        for name, data in files.items():
            assert archive.read(name) == data, name
    digest = sha256(OUTPUT.read_bytes()).hexdigest()
    Path(str(OUTPUT) + '.sha256').write_text(f'{digest}  {OUTPUT.name}\n', encoding='utf-8')
    print(json.dumps({'package': str(OUTPUT), 'sha256': digest, 'files_verified': len(files)}, indent=2))


if __name__ == '__main__':
    main()
