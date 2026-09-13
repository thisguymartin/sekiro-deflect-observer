"""Read-only research snapshots from a hash-gated running Sekiro process.

No injection, game-function calls, memory writes, or input generation.
"""
import argparse
import ctypes
import hashlib
import json
import math
from pathlib import Path
import re
import struct
import time

ROOT = Path(__file__).resolve().parent.parent
OUTPUT = ROOT / "dist/game-analysis"
HASH = "637aca527538c0ec6e1f136c8ed66046e95dfbdbb1f51926e134d9916398b856"


class Process:
    def __init__(self, pid):
        self.kernel = ctypes.WinDLL("kernel32", use_last_error=True)
        self.psapi = ctypes.WinDLL("psapi", use_last_error=True)
        handle, uint, pointer = ctypes.c_void_p, ctypes.c_ulong, ctypes.c_void_p
        self.kernel.OpenProcess.argtypes = [uint, ctypes.c_int, uint]
        self.kernel.OpenProcess.restype = handle
        self.kernel.CloseHandle.argtypes = [handle]
        self.kernel.ReadProcessMemory.argtypes = [handle, pointer, pointer, ctypes.c_size_t, ctypes.POINTER(ctypes.c_size_t)]
        self.kernel.ReadProcessMemory.restype = ctypes.c_int
        self.psapi.EnumProcessModulesEx.argtypes = [handle, pointer, uint, ctypes.POINTER(uint), uint]
        self.psapi.GetModuleFileNameExW.argtypes = [handle, handle, ctypes.c_wchar_p, uint]
        self.handle = self.kernel.OpenProcess(0x410, False, pid)
        if not self.handle:
            raise ctypes.WinError(ctypes.get_last_error())
        try:
            modules = (handle * 1024)()
            needed = uint()
            if not self.psapi.EnumProcessModulesEx(self.handle, modules, ctypes.sizeof(modules), ctypes.byref(needed), 3):
                raise ctypes.WinError(ctypes.get_last_error())
            self.base = modules[0]
            path = ctypes.create_unicode_buffer(32768)
            if not self.psapi.GetModuleFileNameExW(self.handle, self.base, path, len(path)):
                raise ctypes.WinError(ctypes.get_last_error())
            self.path = Path(path.value)
            if self.path.name.lower() != "sekiro.exe" or hashlib.sha256(self.path.read_bytes()).hexdigest() != HASH:
                raise ValueError("Not the supported research executable")
        except BaseException:
            self.close()
            raise

    def close(self):
        if self.handle:
            self.kernel.CloseHandle(self.handle)
            self.handle = None

    def read(self, address, size):
        if not 0x10000 <= address <= address + size <= 0x00007fffffffffff or not 0 < size <= 2 * 1024 * 1024:
            raise ValueError("Out-of-bounds research read")
        output = ctypes.create_string_buffer(size)
        copied = ctypes.c_size_t()
        if not self.kernel.ReadProcessMemory(self.handle, address, output, size, ctypes.byref(copied)) or copied.value != size:
            raise ctypes.WinError(ctypes.get_last_error())
        return output.raw

    def pointer(self, address):
        value, = struct.unpack("<Q", self.read(address, 8))
        if value and (value % 8 or not 0x10000 <= value < 0x0000800000000000):
            raise ValueError("Invalid research pointer")
        return value

    def sections(self):
        header = self.read(self.base, 4096)
        pe, = struct.unpack_from("<I", header, 60)
        if header[:2] != b"MZ" or header[pe:pe + 4] != b"PE\0\0":
            raise ValueError("Invalid loaded image")
        count, = struct.unpack_from("<H", header, pe + 6)
        optional_size, = struct.unpack_from("<H", header, pe + 20)
        result = {}
        for index in range(count):
            offset = pe + 24 + optional_size + index * 40
            name = header[offset:offset + 8].rstrip(b"\0").decode("ascii")
            size, rva = struct.unpack_from("<II", header, offset + 8)
            result[name] = dict(size=size, rva=rva)
        return result

    def signatures(self):
        section = self.sections()[".text"]
        size, rva = section["size"], section["rva"]
        if size > 64 * 1024 * 1024:
            raise ValueError("Code section exceeds research limit")
        chunks = [self.read(self.base + rva + offset, min(2 * 1024 * 1024, size - offset))
                  for offset in range(0, size, 2 * 1024 * 1024)]
        code = b"".join(chunks)
        source = (OUTPUT / "Ela_Sekiro_Table.CT").read_text(encoding="utf-8-sig")
        pattern = r'\{name = "([^"]+)", staticOffset = "([^"]+)", aob = "([^"]+)", aobOffset = (\d+)\}'
        results = []
        for name, expected, signature, instruction_offset in re.findall(pattern, source):
            if name not in ("WorldChrMan", "FieldArea", "LockTgtMan", "RendMan"):
                continue
            regex = b"".join(b"." if token == "??" else re.escape(bytes([int(token, 16)])) for token in signature.split())
            matches = list(re.finditer(regex, code, re.S))
            result = dict(name=name, match_count=len(matches), expected_rva="0x" + expected)
            candidates = []
            for match in matches:
                instruction = match.start() + int(instruction_offset)
                if code[instruction:instruction + 2] not in (b"\x48\x8b", b"\x48\x89") or code[instruction + 2] & 0xc7 != 5:
                    raise ValueError("Expected a RIP-relative pointer instruction")
                displacement, = struct.unpack_from("<i", code, instruction + 3)
                target = rva + instruction + 7 + displacement
                candidates.append(dict(instruction_rva=hex(rva + instruction), resolved_rva=hex(target)))
            result["candidates"] = candidates
            expected_matches = [c for c in candidates if int(c["resolved_rva"], 16) == int(expected, 16)]
            if len(expected_matches) == 1:
                result.update(**expected_matches[0], agrees_with_reference=True)
            results.append(result)
        return results

    def snapshot(self, signatures):
        result = {"monotonic_seconds": time.perf_counter()}
        for signature in signatures:
            if not signature.get("agrees_with_reference"):
                continue
            name = signature["name"]
            slot = self.base + int(signature["resolved_rva"], 16)
            try:
                owner = self.pointer(slot)
                entry = {"pointer": hex(owner)}
                if owner and name == "WorldChrMan":
                    player = self.pointer(owner + 0x88)
                    entry["player"] = hex(player)
                    if player:
                        modules = self.pointer(player + 0x1ff8)
                        animation = self.pointer(modules + 0x10)
                        physics = self.pointer(modules + 0x68)
                        entry.update(modules=hex(modules), animation_module=hex(animation), physics_module=hex(physics))
                        entry.update(self.animation_frame(animation))
                        entry["position"] = struct.unpack("<fff", self.read(physics + 0x80, 12))
                        if self.pointer(owner + 0x88) != player or self.pointer(player + 0x1ff8) != modules:
                            raise ValueError("Player changed during snapshot")
                if self.pointer(slot) != owner:
                    raise ValueError("Owner changed during snapshot")
                result[name] = entry
            except (OSError, ValueError) as error:
                result[name] = {"error": str(error)}
        return result

    def animation_frame(self, animation):
        """Latest completed ring entry written by loaded 0xb5c580.

        Ten entries, each 0x14 bytes. +0xe8 is the next write index.
        The first entry alone is a stale history frame, not a live clock.
        """
        head, = struct.unpack("<i", self.read(animation + 0xe8, 4))
        if not 0 <= head < 10:
            raise ValueError("Invalid animation history index")
        entry = animation + 0x20 + ((head + 9) % 10) * 0x14
        animation_id, previous, elapsed, duration, sequence = struct.unpack("<ifffi", self.read(entry, 20))
        if (not all(math.isfinite(v) for v in (previous, elapsed, duration))
                or not 0 <= previous <= 3600 or not 0 <= elapsed <= 3600
                or not 0 < duration <= 3600):
            raise ValueError("Invalid animation history frame")
        if struct.unpack("<i", self.read(animation + 0xe8, 4))[0] != head:
            raise ValueError("Animation history changed during observation")
        return dict(animation_id=animation_id, animation_seconds=elapsed,
                    animation_previous_seconds=previous, animation_duration=duration,
                    animation_sequence=sequence, animation_ring_head=head)

    def actor(self, address):
        modules = self.pointer(address + 0x1ff8)
        animation = self.pointer(modules + 0x10)
        physics = self.pointer(modules + 0x68)
        data = self.pointer(modules + 0x18)
        resource = self.pointer(address + 0x30)
        handle, = struct.unpack("<I", self.read(address + 8, 4))
        character_id, = struct.unpack("<i", self.read(address + 0x68, 4))
        frame = self.animation_frame(animation)
        position = struct.unpack("<fff", self.read(physics + 0x80, 12))
        hp, max_hp = struct.unpack("<ii", self.read(data + 0x130, 8))
        name = self.read(resource + 8, 32).decode("utf-16-le", errors="replace").split("\0", 1)[0]
        if not all(math.isfinite(v) for v in position) or max_hp < 0:
            raise ValueError("Invalid actor observation")
        if self.pointer(address + 0x1ff8) != modules or struct.unpack("<I", self.read(address + 8, 4))[0] != handle:
            raise ValueError("Actor changed during observation")
        return dict(address=hex(address), handle=hex(handle), character_id=character_id,
                    name=name, **frame,
                    position=position, hp=hp, max_hp=max_hp)

    def nearby_actors(self, world, radius=40):
        """Research traversal derived from live GetChrInsWithHandle code."""
        player = self.pointer(world + 0x88)
        player_sample = self.actor(player)
        info = self.pointer(world + 0x10)
        count, = struct.unpack("<i", self.read(info + 0x18, 4))
        groups = count + 3
        if not 0 < groups <= 64:
            raise ValueError("Character group count exceeds research limit")
        actors, failures = [], 0
        for group in range(groups):
            bucket = self.pointer(world + 0x518 + group * 8)
            if not bucket:
                continue
            count, = struct.unpack("<i", self.read(bucket, 4))
            array = self.pointer(bucket + 8)
            if not 0 <= count <= 256:
                raise ValueError("Character bucket exceeds research limit")
            entries = self.read(array, count * 0x38) if count else b""
            for index in range(count):
                address, = struct.unpack_from("<Q", entries, index * 0x38)
                if not address or address == player:
                    continue
                try:
                    sample = self.actor(address)
                    actor_handle = int(sample["handle"], 16)
                    if (actor_handle >> 28 != 1 or (actor_handle >> 14) & 63 != group
                            or actor_handle & 16383 != index):
                        raise ValueError("Character bucket handle does not match its entry")
                    if sample["hp"] <= 0 or sample["max_hp"] <= 0:
                        continue
                    distance = math.dist(sample["position"], player_sample["position"])
                    if distance <= radius:
                        sample["distance"] = distance
                        sample["group"], sample["index"] = group, index
                        actors.append(sample)
                except (OSError, ValueError):
                    failures += 1
            if self.pointer(bucket + 8) != array or self.pointer(world + 0x518 + group * 8) != bucket:
                raise ValueError("Character bucket changed during traversal")
        if self.pointer(world + 0x88) != player:
            raise ValueError("Player changed during traversal")
        actors.sort(key=lambda sample: sample["distance"])
        return dict(player=player_sample, nearby=actors[:16], actor_read_failures=failures)

    def locked_target(self, world):
        """Equivalent bounded reads of 0x9c5ef0, 0x9c31f0 and 0xa4a050.

        These functions were inspected in the hash-gated loaded executable;
        no native game function is called.
        """
        slot = self.base + 0x3d78058
        manager = self.pointer(slot)
        if not manager or not self.read(manager + 0x2830, 1)[0]:
            return None
        head = self.pointer(manager + 0x10)
        point, seen = head, set()
        while point:
            if point in seen or len(seen) >= 128:
                raise ValueError("Lock target list exceeds research limit")
            seen.add(point)
            if self.read(point + 0x90, 1)[0] & 0x20:
                break
            point = self.pointer(point + 0x80)
        if not point:
            return None
        accessor = self.pointer(point + 0x88)
        handle, = struct.unpack("<I", self.read(accessor + 0x78, 4))
        if handle >> 28 != 1:
            raise ValueError("Locked point is not a character")
        # Character handle layout from loaded 0xc33200/0xc331d0 table.
        group, index = (handle >> 14) & 63, handle & 16383
        info = self.pointer(world + 0x10)
        groups = struct.unpack("<i", self.read(info + 0x18, 4))[0] + 3
        if not 0 < groups <= 64 or group >= groups:
            raise ValueError("Locked character group is out of bounds")
        bucket_slot = world + 0x518 + group * 8
        bucket = self.pointer(bucket_slot)
        count, = struct.unpack("<i", self.read(bucket, 4))
        if not 0 <= index < count <= 256:
            raise ValueError("Locked character index is out of bounds")
        array = self.pointer(bucket + 8)
        address = self.pointer(array + index * 0x38)
        sample = self.actor(address)
        if int(sample["handle"], 16) != handle:
            raise ValueError("Locked character handle changed")
        if (self.pointer(slot) != manager or self.pointer(manager + 0x10) != head
                or not self.read(manager + 0x2830, 1)[0]
                or not self.read(point + 0x90, 1)[0] & 0x20
                or self.pointer(point + 0x88) != accessor
                or struct.unpack("<I", self.read(accessor + 0x78, 4))[0] != handle
                or self.pointer(bucket_slot) != bucket or self.pointer(bucket + 8) != array
                or self.pointer(array + index * 0x38) != address):
            raise ValueError("Locked target changed during observation")
        anchor = struct.unpack("<fff", self.read(accessor, 12))
        bounds_max = struct.unpack("<fff", self.read(accessor + 0x50, 12))
        if not all(math.isfinite(v) for v in (*anchor, *bounds_max)):
            raise ValueError("Invalid target anchor")
        # The lock anchor is around the torso. Bounding box top is a research
        # overhead anchor, not a validated head bone.
        overhead = (anchor[0], bounds_max[1] + 0.15, anchor[2])
        if (self.pointer(point + 0x88) != accessor
                or struct.unpack("<I", self.read(accessor + 0x78, 4))[0] != handle):
            raise ValueError("Target anchor owner changed during observation")
        return dict(point=hex(point), accessor=hex(accessor), actor=sample,
                    lock_anchor=anchor, bounds_max=bounds_max, overhead_anchor=overhead)

    def camera(self):
        """ChrCam base pose consumed by loaded virtual 0xf01260.

        Normal camera research only; reject the known debug-freecam mode.
        Visual alignment and other camera modes still require validation.
        """
        slot = self.base + 0x3d5c0a0
        field = self.pointer(slot)
        render = self.pointer(field + 0x20)
        mode, = struct.unpack("<i", self.read(render + 0xe0, 4))
        if mode:
            raise ValueError("Debug camera mode is outside this research reader")
        camera = self.pointer(field + 0x30)
        raw = self.read(camera + 0x10, 80)
        vectors = [struct.unpack_from("<ffff", raw, i) for i in range(0, 80, 16)]
        right, up, forward, position, lens = vectors
        if not all(math.isfinite(v) for vector in vectors for v in vector):
            raise ValueError("Invalid camera pose")
        axes = [right[:3], up[:3], forward[:3]]
        if (any(abs(sum(v*v for v in axis) - 1) > 0.02 for axis in axes)
                or any(abs(sum(a*b for a,b in zip(axes[i],axes[j]))) > 0.02
                       for i,j in ((0,1),(0,2),(1,2)))
                or not 0.1 < lens[0] < 3 or not 0.2 < lens[1] < 8
                or not 0 < lens[2] < lens[3]):
            raise ValueError("Camera basis or lens is outside research bounds")
        if (self.pointer(slot) != field or self.pointer(field + 0x30) != camera
                or self.pointer(field + 0x20) != render):
            raise ValueError("Camera owner changed during observation")
        return dict(address=hex(camera), right=right[:3], up=up[:3], forward=forward[:3],
                    position=position[:3], vertical_fov=lens[0], aspect=lens[1],
                    near=lens[2], far=lens[3], projection_status="research; visually unvalidated")


def project_anchor(camera, anchor):
    """Research normalized projection for the normal camera and 16:9+ views."""
    if camera["aspect"] < 16/9 - 0.01:
        return None  # Native camera applies an additional FOV conversion here.
    delta = [a-b for a,b in zip(anchor,camera["position"])]
    x,y,z = [sum(a*b for a,b in zip(delta,camera[axis])) for axis in ("right","up","forward")]
    if not camera["near"] < z < camera["far"]:
        return None
    tangent = math.tan(camera["vertical_fov"] / 2)
    x,y = x/(z*tangent*camera["aspect"]), y/(z*tangent)
    if not -1 < x < 1 or not -1 < y < 1:
        return None
    return [(x+1)/2,(1-y)/2]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("pid", type=int)
    parser.add_argument("--duration", type=float, default=0, help="Capture nearby actor animations for up to 60 seconds")
    args = parser.parse_args()
    process = Process(args.pid)
    try:
        signatures = process.signatures()
        snapshots = []
        for index in range(3):
            snapshots.append(process.snapshot(signatures))
            if index < 2:
                time.sleep(0.25)
        report = dict(pid=args.pid, executable=str(process.path), executable_sha256=HASH,
                      image_base=hex(process.base), signatures=signatures, snapshots=snapshots,
                      access="PROCESS_VM_READ | PROCESS_QUERY_INFORMATION")
        OUTPUT.mkdir(parents=True, exist_ok=True)
        path = OUTPUT / f"live-{args.pid}.json"
        path.write_text(json.dumps(report, indent=2, allow_nan=False), encoding="utf-8")
        print(json.dumps(report, indent=2, allow_nan=False))
        if args.duration:
            if not 0 < args.duration <= 60:
                raise ValueError("Capture duration must be between zero and 60 seconds")
            records = OUTPUT / f"live-{args.pid}-{time.time_ns()}.actors.jsonl"
            finish = time.perf_counter() + args.duration
            rows, failures = 0, 0
            with records.open("w", encoding="utf-8") as output:
                while time.perf_counter() < finish:
                    start = time.perf_counter()
                    try:
                        world = process.pointer(process.base + 0x3d7a1e0)
                        sample = process.nearby_actors(world)
                        try:
                            sample["locked_target"] = process.locked_target(world)
                        except (OSError, ValueError) as error:
                            sample["locked_target_error"] = str(error)
                        try:
                            sample["camera"] = process.camera()
                            if sample.get("locked_target"):
                                sample["overhead_uv"] = project_anchor(sample["camera"], sample["locked_target"]["overhead_anchor"])
                        except (OSError, ValueError) as error:
                            sample["camera_error"] = str(error)
                        if process.pointer(process.base + 0x3d7a1e0) != world:
                            raise ValueError("World changed during capture")
                        row = dict(start=start, end=time.perf_counter(), **sample)
                    except (OSError, ValueError) as error:
                        failures += 1
                        row = dict(start=start, end=time.perf_counter(), error=str(error))
                    output.write(json.dumps(row, allow_nan=False) + "\n")
                    rows += 1
                    time.sleep(max(0, 0.03 - (time.perf_counter() - start)))
            print(f"Captured {rows} rows ({failures} failed traversals) to {records}")
    finally:
        process.close()


if __name__ == "__main__":
    main()
