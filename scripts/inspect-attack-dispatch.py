"""Disassemble the preserved loaded animation consumer; compare installed PE bytes.

This reads files only. It is not a full decompiler or a new game-process hook.
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct
import sys

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / 'dist/game-tools'))
from capstone import Cs, CS_ARCH_X86, CS_MODE_64

EXPECTED_GAME = '637aca527538c0ec6e1f136c8ed66046e95dfbdbb1f51926e134d9916398b856'
EXPECTED_BOUNDARY = 'c8f27b1884982c7d4f6486251dd4f06b41a64e2e4a22bce946278f45d8cf909f'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('executable', type=Path)
    parser.add_argument('--capture', type=Path, default=ROOT / 'dist/review-0.6/animation-module-code.bin')
    parser.add_argument('--output', type=Path, default=ROOT / 'dist/response-research-2026-09-17')
    args = parser.parse_args()
    image = args.executable.read_bytes()
    if hashlib.sha256(image).hexdigest() != EXPECTED_GAME:
        raise ValueError('Unsupported executable')
    captured = args.capture.read_bytes()
    if len(captured) != 4096 or hashlib.sha256(captured[0x2f0:0x39b]).hexdigest() != EXPECTED_BOUNDARY:
        raise ValueError('Capture does not contain the researched boundary function')
    pe = struct.unpack_from('<I', image, 60)[0]
    count = struct.unpack_from('<H', image, pe + 6)[0]
    optional = struct.unpack_from('<H', image, pe + 20)[0]
    sections = []
    for index in range(count):
        at = pe + 24 + optional + index * 40
        size, rva, raw_size, raw = struct.unpack_from('<4I', image, at + 8)
        sections.append(dict(name=image[at:at+8].rstrip(b'\0').decode(), rva=rva,
                             size=size, raw=raw, raw_size=raw_size))
    code_rva = 0xb5c730
    section = next(section for section in sections if section['rva'] <= code_rva < section['rva'] + section['size'])
    offset = section['raw'] + code_rva - section['rva']
    instructions, calls = [], []
    for instruction in Cs(CS_ARCH_X86, CS_MODE_64).disasm(captured[0xb30:], 0x140000000 + code_rva):
        instructions.append(f'{instruction.address:x}: {instruction.mnemonic} {instruction.op_str}')
        if instruction.mnemonic == 'call':
            calls.append(instruction.op_str)
        if instruction.mnemonic == 'ret':
            break
    args.output.mkdir(parents=True, exist_ok=True)
    (args.output / 'animation-consumer.asm').write_text('\n'.join(instructions) + '\n')
    report = dict(executable_sha256=EXPECTED_GAME, captured_sha256=hashlib.sha256(captured).hexdigest(),
                  consumer_rva=hex(code_rva), calls=calls, instruction_count=len(instructions),
                  disk_prefix=image[offset:offset+16].hex(), loaded_prefix=captured[0xb30:0xb40].hex(),
                  matching_bytes=image[offset:offset+16] == captured[0xb30:0xb40],
                  capture_provenance='preserved prior live read, RVA 0xb5bc00; no new live capture',
                  sections=sections)
    (args.output / 'dispatch-inspection.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({key: report[key] for key in ('consumer_rva', 'calls', 'instruction_count', 'matching_bytes')}))


if __name__ == '__main__':
    main()
