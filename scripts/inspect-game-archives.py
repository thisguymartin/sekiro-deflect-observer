"""Read-only inventory of Sekiro archives using downloaded format references.

Outputs go to dist/game-analysis. No game files are written or patched.
Public layout/key provenance: scripts/fetch-game-research.ps1.
"""
import argparse
import base64
import collections
import ctypes
import hashlib
import json
import math
from pathlib import Path
import re
import struct
import zlib

ROOT = Path(__file__).resolve().parent.parent
OUTPUT = ROOT / "dist/game-analysis"
REFERENCES = OUTPUT / "references"
RESEARCH_HASH = "637aca527538c0ec6e1f136c8ed66046e95dfbdbb1f51926e134d9916398b856"


def inspect_executable(game):
    data = (game / "sekiro.exe").read_bytes()
    fingerprint = hashlib.sha256(data).hexdigest()
    if fingerprint != RESEARCH_HASH:
        raise ValueError("Executable differs from this project's research build")
    pe, = struct.unpack_from("<I", data, 60)
    count, = struct.unpack_from("<H", data, pe + 6)
    optional_size, = struct.unpack_from("<H", data, pe + 20)
    optional = pe + 24
    text = None
    for index in range(count):
        offset = optional + optional_size + index * 40
        if data[offset:offset + 8].rstrip(b"\0") == b".text":
            _, rva, size, raw = struct.unpack_from("<IIII", data, offset + 8)
            text = data[raw:raw + size]
            break
    if text is None:
        raise ValueError("Executable has no code section")
    table = (ROOT / "dist/game-analysis/Ela_Sekiro_Table.CT").read_text(encoding="utf-8-sig")
    results = []
    pattern = r'\{name = "([^"]+)", staticOffset = "([^"]+)", aob = "([^"]+)", aobOffset = (\d+)\}'
    for name, expected, signature, instruction_offset in re.findall(pattern, table):
        if name not in ("WorldChrMan", "FieldArea", "LockTgtMan"):
            continue
        mask = b"".join(b"." if token == "??" else re.escape(bytes([int(token, 16)])) for token in signature.split())
        matches = list(re.finditer(mask, text, re.S))
        result = dict(name=name, match_count=len(matches), expected_rva="0x" + expected)
        if len(matches) == 1:
            instruction = matches[0].start() + int(instruction_offset)
            displacement, = struct.unpack_from("<i", text, instruction + 3)
            target = rva + instruction + 7 + displacement
            result.update(instruction_rva=hex(rva + instruction), resolved_rva=hex(target),
                          agrees_with_reference=target == int(expected, 16))
        results.append(result)
    return dict(sha256=fingerprint, static_signature_checks=results)


def der_value(data, offset):
    tag, length = data[offset:offset + 2]
    offset += 2
    if length & 128:
        count = length & 127
        length = int.from_bytes(data[offset:offset + count], "big")
        offset += count
    if offset + length > len(data):
        raise ValueError("Truncated DER value")
    return tag, data[offset:offset + length], offset + length


def rsa_key(pem):
    der = base64.b64decode("".join(pem.splitlines()[1:-1]), validate=True)
    tag, sequence, end = der_value(der, 0)
    if tag != 48 or end != len(der):
        raise ValueError("Not a PKCS#1 RSA public key")
    tag, modulus, offset = der_value(sequence, 0)
    if tag != 2:
        raise ValueError("Missing RSA modulus")
    tag, exponent, end = der_value(sequence, offset)
    if tag != 2 or end != len(sequence):
        raise ValueError("Missing RSA exponent")
    return int.from_bytes(modulus, "big"), int.from_bytes(exponent, "big")


def decrypt_header(encrypted, pem):
    modulus, exponent = rsa_key(pem)
    block_size = (modulus.bit_length() + 7) // 8
    if len(encrypted) % block_size:
        raise ValueError("Truncated encrypted header block")
    blocks = []
    for offset in range(0, len(encrypted), block_size):
        value = int.from_bytes(encrypted[offset:offset + block_size], "big")
        if value >= modulus:
            raise ValueError("Invalid RSA block")
        blocks.append(pow(value, exponent, modulus).to_bytes(block_size - 1, "big"))
    return b"".join(blocks)


def path_hash(path):
    result = 0
    for char in path.replace("\\", "/").lower():
        result = (result * 37 + ord(char)) & 0xffffffff
    return result


def read_archive(header_path, pem, names):
    encrypted = header_path.read_bytes()
    data = encrypted if encrypted.startswith(b"BHD5") else decrypt_header(encrypted, pem)
    if data[:4] != b"BHD5" or data[4] != 255:
        raise ValueError("Expected a little-endian BHD5 header")
    version, file_size, bucket_count, bucket_offset = struct.unpack_from("<iiii", data, 8)
    if version != 1 or not 0 <= bucket_count <= 100000 or file_size > len(data):
        raise ValueError("Invalid BHD5 header")
    bdt_size = header_path.with_suffix(".bdt").stat().st_size
    entries = []
    for index in range(bucket_count):
        count, offset = struct.unpack_from("<ii", data, bucket_offset + index * 8)
        if count < 0 or count > 100000 or offset < 0 or offset + count * 40 > len(data):
            raise ValueError("Invalid BHD5 bucket")
        for entry_index in range(count):
            hashed, padded_size, file_offset, sha_offset, aes_offset, size = struct.unpack_from(
                "<Iiqqqq", data, offset + entry_index * 40
            )
            if padded_size < 0 or file_offset < 0 or file_offset + padded_size > bdt_size:
                raise ValueError("Archive entry exceeds BDT bounds")
            entry = dict(archive=header_path.stem, hash=hashed, paths=names.get(hashed, []),
                         offset=file_offset, padded_size=padded_size, size=size)
            if aes_offset:
                key = data[aes_offset:aes_offset + 16]
                range_count, = struct.unpack_from("<i", data, aes_offset + 16)
                if len(key) != 16 or not 0 <= range_count <= 100000:
                    raise ValueError("Invalid AES metadata")
                ranges = [struct.unpack_from("<qq", data, aes_offset + 20 + i * 16)
                          for i in range(range_count)]
                if any(a != -1 and b != -1 and not 0 <= a <= b <= padded_size for a, b in ranges):
                    raise ValueError("AES range exceeds entry")
                entry.update(aes_key=key.hex(), aes_ranges=ranges)
            entries.append(entry)
    return dict(archive=header_path.stem, header_sha256=hashlib.sha256(encrypted).hexdigest(),
                file_count=len(entries), bucket_count=bucket_count), entries


def aes_ecb_decrypt(data, key):
    """Use Windows' system AES implementation; no downloaded code is executed."""
    dll = ctypes.WinDLL("bcrypt")
    handle, ulong, pointer = ctypes.c_void_p, ctypes.c_ulong, ctypes.c_void_p
    declarations = {
        "BCryptOpenAlgorithmProvider": [ctypes.POINTER(handle), ctypes.c_wchar_p, ctypes.c_wchar_p, ulong],
        "BCryptSetProperty": [handle, ctypes.c_wchar_p, pointer, ulong, ulong],
        "BCryptGenerateSymmetricKey": [handle, ctypes.POINTER(handle), pointer, ulong, pointer, ulong, ulong],
        "BCryptDecrypt": [handle, pointer, ulong, pointer, pointer, ulong, pointer, ulong, ctypes.POINTER(ulong), ulong],
        "BCryptDestroyKey": [handle],
        "BCryptCloseAlgorithmProvider": [handle, ulong],
    }
    for name, args in declarations.items():
        function = getattr(dll, name)
        function.argtypes, function.restype = args, ctypes.c_long
    def checked(status):
        if status < 0:
            raise OSError(f"BCrypt status {status & 0xffffffff:08x}")
    algorithm, symmetric = handle(), handle()
    checked(dll.BCryptOpenAlgorithmProvider(ctypes.byref(algorithm), "AES", None, 0))
    try:
        mode = ctypes.create_unicode_buffer("ChainingModeECB")
        checked(dll.BCryptSetProperty(algorithm, "ChainingMode", mode, ctypes.sizeof(mode), 0))
        secret = ctypes.create_string_buffer(key)
        checked(dll.BCryptGenerateSymmetricKey(algorithm, ctypes.byref(symmetric), None, 0, secret, len(key), 0))
        source, dest = ctypes.create_string_buffer(data), ctypes.create_string_buffer(len(data))
        written = ulong()
        checked(dll.BCryptDecrypt(symmetric, source, len(data), None, None, 0, dest, len(data), ctypes.byref(written), 0))
        if written.value != len(data):
            raise ValueError("Incomplete AES decryption")
        return dest.raw
    finally:
        if symmetric:
            dll.BCryptDestroyKey(symmetric)
        dll.BCryptCloseAlgorithmProvider(algorithm, 0)


def decompress(data, game):
    if not data.startswith(b"DCX\0"):
        return data
    if data[24:28] != b"DCS\0" or data[36:40] != b"DCP\0":
        raise ValueError("Invalid DCX header")
    if data[40:44] not in (b"DFLT", b"KRAK"):
        raise ValueError("Unsupported DCX codec")
    size, compressed_size = struct.unpack_from(">II", data, 28)
    if not 0 < size <= 256 * 1024 * 1024 or compressed_size > len(data) - 76:
        raise ValueError("Invalid DCX sizes")
    compressed = data[76:76 + compressed_size]
    if data[40:44] == b"DFLT":
        decoded = zlib.decompress(compressed)
    else:
        # Only load the codec already shipped with the explicitly named game.
        dll = ctypes.WinDLL(str(game / "oo2core_6_win64.dll"))
        dll.OodleLZ_GetDecodeBufferSize.argtypes = [ctypes.c_longlong, ctypes.c_int]
        dll.OodleLZ_GetDecodeBufferSize.restype = ctypes.c_longlong
        buffer_size = dll.OodleLZ_GetDecodeBufferSize(size, 1)
        if not size <= buffer_size <= size + 1024 * 1024:
            raise ValueError("Invalid Oodle decode buffer size")
        source, dest = ctypes.create_string_buffer(compressed), ctypes.create_string_buffer(buffer_size)
        function = dll.OodleLZ_Decompress
        function.argtypes = [ctypes.c_void_p, ctypes.c_longlong, ctypes.c_void_p, ctypes.c_longlong,
                            ctypes.c_int, ctypes.c_int, ctypes.c_int, ctypes.c_void_p,
                            ctypes.c_longlong, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p,
                            ctypes.c_longlong, ctypes.c_int]
        function.restype = ctypes.c_longlong
        result = function(source, len(compressed), dest, size, 1, 0, 0, None, 0, None, None, None, 0, 3)
        if result != size:
            raise ValueError("Oodle decompression failed")
        decoded = dest.raw[:size]
    if len(decoded) != size:
        raise ValueError("DCX uncompressed size mismatch")
    return decoded


def binder_timelines(data, game, suffix=".tae"):
    if data[:4] != b"BND4" or data[9] != 0:
        raise ValueError("Expected little-endian BND4")
    count, = struct.unpack_from("<I", data, 12)
    header_size, = struct.unpack_from("<Q", data, 32)
    if not 0 <= count <= 100000 or 64 + count * header_size > len(data):
        raise ValueError("Invalid BND4 header")
    bit_big = not data[10]
    raw_format = data[49]
    fmt = raw_format if bit_big or (raw_format & 1 and not raw_format & 128) else int(f"{raw_format:08b}"[::-1], 2)
    if not fmt & 12:
        raise ValueError("Binder has no file names")
    for index in range(count):
        offset = 64 + index * header_size
        flags = data[offset] if bit_big else int(f"{data[offset]:08b}"[::-1], 2)
        size, = struct.unpack_from("<q", data, offset + 8)
        cursor = offset + 16 + (8 if fmt & 32 else 0)
        file_offset, = struct.unpack_from("<Q" if fmt & 16 else "<I", data, cursor)
        cursor += 8 if fmt & 16 else 4
        cursor += 4 if fmt & 2 else 0
        name_offset, = struct.unpack_from("<I", data, cursor)
        if not 0 <= name_offset < len(data) or not 0 <= file_offset <= file_offset + size <= len(data):
            raise ValueError("Binder entry exceeds bounds")
        if data[48]:
            end = name_offset
            while end + 2 <= len(data) and data[end:end + 2] != b"\0\0":
                end += 2
            name = data[name_offset:end].decode("utf-16-le")
        else:
            name = data[name_offset:data.index(b"\0", name_offset)].decode("shift_jis")
        if name.lower().endswith(suffix):
            contents = data[file_offset:file_offset + size]
            if flags & 1:
                contents = decompress(contents, game) if contents.startswith(b"DCX\0") else zlib.decompress(contents)
            yield name, contents


def tae_timelines(data):
    if data[:8] != b"TAE \0\0\0\xff" or struct.unpack_from("<I", data, 8)[0] != 0x1000d:
        raise ValueError("Expected Sekiro 64-bit TAE")
    tae_id, count, offset = struct.unpack_from("<iiq", data, 80)
    if not 0 <= count <= 100000 or not 0 <= offset <= offset + count * 16 <= len(data):
        raise ValueError("Invalid TAE animation table")
    animations = []
    for index in range(count):
        anim_id, body = struct.unpack_from("<qq", data, offset + index * 16)
        event_offset, _, _, mini, event_count = struct.unpack_from("<qqqqi", data, body)
        if not 0 <= event_count <= 100000 or not 0 <= event_offset <= event_offset + event_count * 24 <= len(data):
            raise ValueError("Invalid TAE event table")
        animation = dict(id=anim_id, events=[])
        mini_type, = struct.unpack_from("<I", data, mini)
        inner, = struct.unpack_from("<q", data, mini + 8)
        if mini_type == 1 and inner:
            animation["imports_animation"] = struct.unpack_from("<i", data, inner + 8)[0]
        for event_index in range(event_count):
            start_ptr, end_ptr, event_ptr = struct.unpack_from("<qqq", data, event_offset + event_index * 24)
            start, = struct.unpack_from("<f", data, start_ptr)
            end, = struct.unpack_from("<f", data, end_ptr)
            event_type, = struct.unpack_from("<i", data, event_ptr)
            parameter_ptr, = struct.unpack_from("<q", data, event_ptr + 8)
            # Preserve unusual source times (including pre-roll) without making
            # them eligible for a future cue calculation.
            timing_valid = math.isfinite(start) and math.isfinite(end) and 0 <= start <= end
            event = dict(type=event_type, start_seconds=start if math.isfinite(start) else None,
                         end_seconds=end if math.isfinite(end) else None, timing_valid=timing_valid)
            # Fields from DSAnimStudio's Sekiro schema. Preserve behavior IDs
            # for later PARAM/geometry joins; their existence is not contact proof.
            if event_type == 0 and parameter_ptr:
                event['action_flag'] = struct.unpack_from('<i', data, parameter_ptr)[0]
            if event_type == 1 and parameter_ptr:
                event['attack_type'] = struct.unpack_from('<i', data, parameter_ptr)[0]
                event['behavior_judge_id'] = struct.unpack_from('<i', data, parameter_ptr + 8)[0]
            if event_type == 304 and parameter_ptr:
                event['behavior_judge_id'] = struct.unpack_from('<i', data, parameter_ptr + 4)[0]
            if event_type == 700 and parameter_ptr:
                event['look_target_type'] = data[parameter_ptr + 20]
            if event_type == 67 and parameter_ptr:
                event["effect_id"] = struct.unpack_from("<i", data, parameter_ptr)[0]
            animation["events"].append(event)
        animations.append(animation)
    return dict(tae_id=tae_id, animation_count=count, animations=animations)


def extract_container(entry, game):
    with (game / (entry["archive"] + ".bdt")).open("rb") as file:
        file.seek(entry["offset"])
        data = bytearray(file.read(entry["padded_size"]))
    if len(data) != entry["padded_size"]:
        raise ValueError("Incomplete archive entry read")
    if "aes_key" in entry:
        for start, end in entry["aes_ranges"]:
            if start != -1 and end != -1 and start != end:
                data[start:end] = aes_ecb_decrypt(bytes(data[start:end]), bytes.fromhex(entry["aes_key"]))
    unpacked = decompress(bytes(data), game)
    return unpacked, hashlib.sha256(data).hexdigest()


def extract_timeline(entry, game):
    unpacked, container_hash = extract_container(entry, game)
    results = []
    for name, contents in binder_timelines(unpacked, game):
        parsed = tae_timelines(contents)
        results.append(dict(binder_name=name, sha256=hashlib.sha256(contents).hexdigest(), **parsed))
    return dict(schema_version=2, archive_path=entry["paths"][0], decrypted_container_sha256=container_hash, timelines=results)


def main():
    global OUTPUT
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("game_directory", type=Path)
    parser.add_argument("--timeline", action="append", default=[], help="Exact archive path from the inventory; repeatable")
    parser.add_argument("--all-enemies", action="store_true", help="Extract all base c1000-c7999 animation archives")
    parser.add_argument("--output-directory", type=Path, default=OUTPUT)
    args = parser.parse_args()
    OUTPUT = args.output_directory
    OUTPUT.mkdir(parents=True, exist_ok=True)
    executable = inspect_executable(args.game_directory)
    source = (REFERENCES / "ArchiveKeys.cs").read_text(encoding="utf-8-sig")
    sekiro_source = source.split("SekiroKeys =", 1)[1].split("SekiroBonusKeys", 1)[0]
    keys = dict(re.findall(r'\["(Data\d)"\]\s*=\s*@"(-----BEGIN RSA PUBLIC KEY-----.*?-----END RSA PUBLIC KEY-----)',
                           sekiro_source, re.S))
    names = collections.defaultdict(list)
    for path in (REFERENCES / "SekiroDictionary.txt").read_text(encoding="utf-8-sig").splitlines():
        if path.startswith("/"):
            names[path_hash(path)].append(path)
    summaries, entries = [], []
    for archive, key in sorted(keys.items()):
        summary, files = read_archive(args.game_directory / (archive + ".bhd"), key, names)
        summaries.append(summary)
        entries.extend(files)
        print(f"{archive}: {summary['file_count']} entries", flush=True)
    animations = [entry for entry in entries if any(path.endswith(".anibnd.dcx") for path in entry["paths"])]
    report = dict(game_directory=str(args.game_directory.resolve()), executable=executable, archives=summaries,
                  named_count=sum(bool(e["paths"]) for e in entries), animation_archives=animations,
                  reference_sha256={p.name: hashlib.sha256(p.read_bytes()).hexdigest()
                                    for p in REFERENCES.iterdir() if p.is_file()}, entries=entries)
    OUTPUT.mkdir(parents=True, exist_ok=True)
    (OUTPUT / "archive-inventory.json").write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(f"{len(animations)} character animation archives indexed; no game files modified.")
    requested_paths = list(args.timeline)
    if args.all_enemies:
        requested_paths.extend(entry["paths"][0] for entry in animations
                               if len(entry["paths"]) == 1 and re.fullmatch(r"/chr/c[1-7][0-9]{3}\.anibnd\.dcx", entry["paths"][0]))
    for requested in sorted(set(requested_paths)):
        matches = [entry for entry in animations if entry["paths"] == [requested]]
        if len(matches) != 1:
            raise ValueError(f"Missing or ambiguous archive: {requested}")
        parsed = extract_timeline(matches[0], args.game_directory)
        destination = OUTPUT / (Path(requested).name + ".timelines.json")
        destination.write_text(json.dumps(parsed, indent=2), encoding="utf-8")
        total = sum(t["animation_count"] for t in parsed["timelines"])
        attack_events = sum(e["type"] == 1 and e["timing_valid"] for t in parsed["timelines"] for a in t["animations"] for e in a["events"])
        print(f"{requested}: {total} animations, {attack_events} attack hitbox events read", flush=True)


if __name__ == "__main__":
    main()
