"""Preserve working sources, Git history and built packages without changing the index."""
from datetime import datetime, timezone
from hashlib import sha256
from pathlib import Path
import json
import shutil
import subprocess
import zipfile

ROOT = Path(__file__).resolve().parent.parent
GIT = ['git', '-c', f'safe.directory={ROOT.as_posix()}']


def git(*args):
    return subprocess.check_output(GIT + list(args), cwd=ROOT)


def main():
    stamp = datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%SZ')
    output = ROOT / 'dist/backups' / f'0.6.3-preview-{stamp}'
    output.mkdir(parents=True, exist_ok=False)
    names = sorted(set(git('ls-files', '-z', '--cached', '--others', '--exclude-standard').decode().split('\0')) - {''})
    files = {}
    for name in names:
        path = (ROOT / name).resolve()
        if not path.is_relative_to(ROOT) or path.is_symlink():
            raise ValueError(f'Unexpected source path: {name}')
        if path.is_file():
            files[name] = path.read_bytes()
    provenance = {
        'created_utc': stamp,
        'head': git('rev-parse', 'HEAD').decode().strip(),
        'branch': git('branch', '--show-current').decode().strip(),
        'status': git('status', '--short').decode(),
        'source_sha256': {name: sha256(data).hexdigest() for name, data in files.items()},
        'scope': 'Current tracked and nonignored working files, including staged and uncommitted edits; not a copy of ignored research/game archives.',
    }
    with zipfile.ZipFile(output / 'source-working-tree.zip', 'x', compression=zipfile.ZIP_DEFLATED) as archive:
        for name, data in files.items():
            archive.writestr(name, data)
    with zipfile.ZipFile(output / 'source-working-tree.zip') as archive:
        assert archive.testzip() is None
        for name, data in files.items():
            assert archive.read(name) == data
    (output / 'snapshot-info.json').write_text(json.dumps(provenance, indent=2) + '\n', encoding='utf-8')
    (output / 'working-changes.patch').write_bytes(git('diff', '--binary', 'HEAD'))
    git('bundle', 'create', str(output / 'repository-history.bundle'), '--all')
    for suffix in ['windows-x64.zip', 'windows-x64.zip.sha256', 'drop-in-windows-x64.zip', 'drop-in-windows-x64.zip.sha256']:
        source = ROOT / 'dist' / f'SekiroDeflectObserver-0.6.3-preview-{suffix}'
        shutil.copy2(source, output / source.name)
    (output / 'RESTORE.txt').write_text(
        'Restore working files: extract source-working-tree.zip into a NEW folder.\n'
        'It contains the current working source, including edits not yet committed.\n'
        'Git history: git clone repository-history.bundle restored-history\n'
        'Then overlay source-working-tree.zip to recover the saved working version.\n'
        'Built ZIPs are included for running/sharing; source and Git bundle are for development.\n'
        'Keep another copy on a second drive or private cloud storage.\n', encoding='utf-8')
    manifest = ''.join(f'{sha256(path.read_bytes()).hexdigest()}  {path.name}\n' for path in sorted(output.iterdir()) if path.is_file())
    (output / 'SHA256SUMS.txt').write_text(manifest, encoding='utf-8')
    print(json.dumps({'backup': str(output), 'source_files_verified': len(files)}, indent=2))


if __name__ == '__main__':
    main()
