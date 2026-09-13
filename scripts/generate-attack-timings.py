"""Generate broad, conservative attack preview data from all local enemy TAEs."""
import argparse
import collections
import hashlib
import json
import math
from pathlib import Path
import re
from attack_responses import Responses

ROOT = Path(__file__).resolve().parent.parent

def resolve(animation_id, animations, seen=()):
    if animation_id in seen or len(seen) >= 32:
        raise ValueError("Cyclic animation import")
    animation = animations.get(animation_id)
    if animation is None:
        raise ValueError("Missing animation import")
    if "imports_animation" not in animation:
        return animation["events"], True, []
    # Full imports use the referenced source. Keep locally attached events as
    # exclusions only: their merge semantics are not proven, so these stay gray.
    events, certain, exclusions = resolve(animation["imports_animation"], animations, (*seen, animation_id))
    return events, certain and not animation['events'], exclusions + animation['events']

def special_kind(events):
    if any(e['type'] == 304 or (e['type'] == 0 and e.get('action_flag') in (67, 68, 69, 70)) for e in events):
        return 1  # throw-related; response unverified
    if any(e['type'] == 700 and e.get('look_target_type') in (7, 8) for e in events):
        return 2  # crouch/sweep/grab aim; not proof of a specific response
    if any(e['type'] in (2, 4) for e in events):
        return 3  # projectile or mixed attack
    return 0

def phases(events, certain):
    eligible = certain and not special_kind(events) and not any(
        e['type'] in (5, 66, 67, 302, 940) for e in events)
    intervals = []
    for event in events:
        if event['type'] != 1:
            continue
        start, end = event['start_seconds'], event['end_seconds']
        if (not event['timing_valid'] or start is None or end is None
                or not math.isfinite(start) or not math.isfinite(end)
                or not 0 <= start < end <= 120):
            eligible = False
            continue
        intervals.append((start, end, event.get('attack_type') == 0 and 'behavior_judge_id' in event))
    # Overlapping/touching body and weapon hitboxes form one continuous phase,
    # not multiple parry requests. Separated combo activations remain distinct.
    merged = []
    for start, end, standard in sorted(set(intervals)):
        if merged and start <= merged[-1][1] + 0.00001:
            old_start, old_end, old_standard = merged[-1]
            merged[-1] = (old_start, max(old_end, end), old_standard and standard)
        else:
            merged.append((start, end, standard))
    return [(s, e, eligible and standard and s >= 0.1 and e-s <= 0.4)
            for s, e, standard in merged if e-s <= 5]

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input-directory", type=Path, default=ROOT / "dist/game-analysis-v2")
    args = parser.parse_args()
    entries = []
    coverage = []
    specials = []
    responses = Responses(args.input_directory, ROOT / 'dist/game-analysis/references/SDT.AtkParam.names.txt')
    actions = []
    action_evidence = []
    for path in sorted(args.input_directory.glob("c*.anibnd.dcx.timelines.json")):
        match = re.fullmatch(r"c([1-7][0-9]{3})\.anibnd\.dcx\.timelines\.json", path.name)
        if not match:
            continue
        model = int(match[1])
        raw = path.read_bytes()
        data = json.loads(raw)
        if data["archive_path"] != f"/chr/c{model:04}.anibnd.dcx":
            raise ValueError("Unexpected archive identity")
        if data.get('schema_version') != 2:
            raise ValueError(f'Re-extract {path.name} with schema version 2')
        stats = collections.Counter()
        by_id = collections.defaultdict(list)
        for tae in data["timelines"]:
            for animation in tae["animations"]:
                by_id[animation["id"]].append(animation)
        # Coalesce identical shared definitions, but exclude conflicting banks.
        animations = {key: value[0] for key, value in by_id.items() if all(a == value[0] for a in value)}
        stats["ambiguous_ids"] = len(by_id) - len(animations)
        for animation_id, animation in sorted(animations.items()):
            try:
                events, certain, exclusions = resolve(animation_id, animations)
            except ValueError:
                stats["unresolved_imports"] += 1
                continue
            kind = special_kind(events + exclusions)
            if kind:
                specials.append((model, animation_id, kind))
                stats['special_animations'] += 1
            for start, end, green in phases(events, certain and not special_kind(exclusions)):
                response, attack_ids = responses.phase(model, events, start, end, certain)
                # Unknown parameter joins cannot supply confident parry guidance.
                green = green and response == 'parry_candidate'
                if response in ('jump', 'dodge'):
                    start, end = responses.action_interval(model,events,start,end,response)
                    code = 1 if response == 'dodge' else 2
                    actions.append((model, animation_id, start, end, code))
                    stats[response + '_phases'] += 1
                    action_evidence.append(dict(model=model, animation=animation_id, start=start, end=end,
                        response=response, attack_params=attack_ids))
                entries.append((model, animation_id, start, end, green))
                stats["phases"] += 1
                stats["green_phases" if green else "uncertain_phases"] += 1
                if "imports_animation" in animation:
                    stats["resolved_import_phases"] += 1
                    if not certain:
                        stats['uncertain_import_phases'] += 1
        coverage.append(dict(model=model, source_sha256=hashlib.sha256(raw).hexdigest(), **stats))
    if not coverage or not entries:
        raise ValueError('No enemy timelines; refusing to erase generated data')
    lines = ["//! Generated activation estimates. Contact and deflectability remain unverified.",
             "//! See docs/enemy-coverage.json and docs/cue-preview.md.",
             "#[allow(clippy::excessive_precision)]",
             "pub const ATTACKS: &[(i32, i32, f32, f32, bool)] = &["]
    for model, animation_id, start, end, green in sorted(set(entries)):
        lines.append(f"    ({model}, {animation_id}, {start:.9f}_f32, {end:.9f}_f32, {str(green).lower()}),")
    lines.append("];")
    lines.append('pub const SPECIALS: &[(i32, i32, u8)] = &[')
    for model, animation_id, kind in sorted(set(specials)):
        lines.append(f'    ({model}, {animation_id}, {kind}),')
    lines.append('];')
    lines.append('#[allow(clippy::excessive_precision)]')
    lines.append('pub const RESPONSES: &[(i32, i32, f32, f32, u8)] = &[')
    for model, animation_id, start, end, kind in sorted(set(actions)):
        lines.append(f'    ({model}, {animation_id}, {start:.9f}_f32, {end:.9f}_f32, {kind}),')
    lines.append('];')
    (ROOT / "src/attack_timings.rs").write_text("\n".join(lines) + "\n", encoding="utf-8")
    report = dict(schema_version=2, dodge_phases=sum(a[4]==1 for a in actions), jump_phases=sum(a[4]==2 for a in actions), special_animations=len(set(specials)), archives=len(coverage), models_with_phases=sum(c.get("phases",0)>0 for c in coverage),
                  models_with_green=sum(c.get("green_phases",0)>0 for c in coverage),
                  phases=len(set(entries)), green_phases=sum(e[4] for e in set(entries)), coverage=coverage)
    (ROOT / "docs/enemy-coverage.json").write_text(json.dumps(report, indent=2)+"\n", encoding="utf-8")
    evidence=dict(parameter_sources=json.loads((args.input_directory/'attack-param-sources.json').read_text()),
        attack_names_sha256=hashlib.sha256((ROOT/'dist/game-analysis/references/SDT.AtkParam.names.txt').read_bytes()).hexdigest(),
        interpretation='Response classification from parameters and attack names; timing and runtime variation selection need gameplay validation.',
        phases=action_evidence)
    (ROOT / 'docs/response-coverage.json').write_text(json.dumps(evidence,indent=2)+'\n')
    print(json.dumps({k:v for k,v in report.items() if k!="coverage"}))

if __name__ == "__main__":
    main()
