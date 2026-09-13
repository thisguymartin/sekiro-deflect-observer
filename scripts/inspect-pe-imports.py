"""Print a Windows PE's imported DLLs without loading or executing it."""
import json
from pathlib import Path
import struct
import sys


def inspect(path):
    data = Path(path).read_bytes()
    pe = struct.unpack_from('<I', data, 0x3c)[0]
    if data[:2] != b'MZ' or data[pe:pe + 4] != b'PE\0\0':
        raise ValueError('Not a PE file')
    machine, count = struct.unpack_from('<HH', data, pe + 4)
    optional_size = struct.unpack_from('<H', data, pe + 20)[0]
    optional = pe + 24
    magic = struct.unpack_from('<H', data, optional)[0]
    if magic not in (0x10b, 0x20b):
        raise ValueError('Unknown optional header')
    directories = optional + (112 if magic == 0x20b else 96)
    sections = []
    for i in range(count):
        base = optional + optional_size + 40 * i
        virtual_size, rva, raw_size, offset = struct.unpack_from('<IIII', data, base + 8)
        sections.append((rva, max(virtual_size, raw_size), offset))

    def file_offset(rva):
        for start, size, offset in sections:
            if start <= rva < start + size:
                return offset + rva - start
        raise ValueError(f'RVA outside sections: {rva:x}')

    import_rva, _ = struct.unpack_from('<II', data, directories + 8)
    imports = []
    if import_rva:
        base = file_offset(import_rva)
        while True:
            entry = struct.unpack_from('<IIIII', data, base)
            if not any(entry):
                break
            name = file_offset(entry[3])
            imports.append(data[name:data.index(0, name)].decode('ascii'))
            base += 20
    return {'machine': hex(machine), 'imports': imports}


if __name__ == '__main__':
    print(json.dumps(inspect(sys.argv[1]), indent=2))
