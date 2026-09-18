#!/usr/bin/env python3
"""Check local file targets in repository documentation."""

from __future__ import annotations

import re
import sys
from pathlib import Path
from urllib.parse import unquote


ROOT = Path(__file__).resolve().parent.parent
LINK = re.compile(r"!?\[[^\]]*\]\(([^)]+)\)")
HTML_LINK = re.compile(r'(?:href|src)="([^"]+)"')
IGNORED_PREFIXES = ("#", "http://", "https://", "mailto:", "data:")


def local_target(raw_target: str) -> str | None:
    target = raw_target.strip()
    if not target or target.startswith(IGNORED_PREFIXES):
        return None
    if target.startswith("<") and ">" in target:
        target = target[1 : target.index(">")]
    else:
        target = target.split(maxsplit=1)[0]
    target = unquote(target.split("#", 1)[0])
    return target or None


def document_files() -> list[Path]:
    files = [ROOT / "README.md", ROOT / "CHANGELOG.md"]
    files.extend((ROOT / "docs").rglob("*.md"))
    files.extend((ROOT / "docs").rglob("*.html"))
    files.extend((ROOT / "prototypes").rglob("*.md"))
    files.extend((ROOT / "tests").rglob("*.md"))
    return sorted(path for path in files if path.exists())


def main() -> int:
    failures: list[str] = []
    checked = 0

    documents = document_files()
    for source in documents:
        lines = source.read_text(encoding="utf-8").splitlines()
        pattern = HTML_LINK if source.suffix == ".html" else LINK
        for line_number, line in enumerate(lines, 1):
            for match in pattern.finditer(line):
                target = local_target(match.group(1))
                if target is None:
                    continue
                checked += 1
                resolved = (source.parent / target).resolve()
                if not resolved.exists():
                    relative_source = source.relative_to(ROOT)
                    failures.append(f"{relative_source}:{line_number}: missing {target}")

    if failures:
        print("\n".join(failures), file=sys.stderr)
        return 1

    print(f"Checked {checked} local links in {len(documents)} documentation files.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
