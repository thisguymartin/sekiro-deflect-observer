"""Generate parameter-backed incoming attack types, without contact estimates."""
import argparse
import collections
import hashlib
import importlib.util
import json
from pathlib import Path
import re
from incoming_responses import IncomingResponses

ROOT = Path(__file__).resolve().parent.parent
spec = importlib.util.spec_from_file_location('timelines', ROOT / 'scripts/generate-attack-timings.py')
timelines = importlib.util.module_from_spec(spec)
spec.loader.exec_module(timelines)
CODES = {'unverified': 0, 'parry': 1, 'dodge': 2, 'jump': 3, 'mikiri': 4, 'avoid': 5}


def generate(directory):
    refs = ROOT / 'dist/game-analysis/references'
    responses = IncomingResponses(directory, refs)
    records, variant_records, sources = [], [], {}
    npc_rows = json.loads((directory / 'NpcParam.json').read_text())
    npc_variations = sorted((int(key), row['behaviorVariationId']) for key, row in npc_rows.items()
                            if 1000 <= row['behaviorVariationId'] // 10 <= 7999)
    variations = sorted({variation for _, variation in npc_variations})
    resolvers = {variation: responses.for_variation(variation) for variation in variations}
    for path in sorted(directory.glob('c*.anibnd.dcx.timelines.json')):
        match = re.fullmatch(r'c([1-7][0-9]{3})\.anibnd\.dcx\.timelines\.json', path.name)
        if not match:
            continue
        model = int(match[1])
        raw = path.read_bytes()
        data = json.loads(raw)
        if data['archive_path'] != f'/chr/c{model:04}.anibnd.dcx' or data['schema_version'] != 2:
            raise ValueError('Unexpected timeline identity/schema')
        sources[path.name] = hashlib.sha256(raw).hexdigest()
        by_id = collections.defaultdict(list)
        for tae in data['timelines']:
            for animation in tae['animations']:
                by_id[animation['id']].append(animation)
        animations = {key: values[0] for key, values in by_id.items()
                      if all(value == values[0] for value in values)}
        for animation_id in sorted(animations):
            try:
                events, certain, exclusions = timelines.resolve(animation_id, animations)
            except ValueError:
                continue
            for variation, resolver in [(-1, responses)] + [
                    (v, resolvers[v]) for v in variations if v // 10 == model]:
                destination = records if variation == -1 else variant_records
                for phase, (start, end) in enumerate(resolver.intervals(model, events)):
                    response, attack_ids = resolver.phase(model, events, start, end, certain and not exclusions)
                    destination.append(dict(model=model, variation=variation, animation=animation_id, phase=phase,
                                            start=start, end=end, response=response, attack_params=attack_ids))
    if not records:
        raise ValueError('No incoming attacks')
    records.sort(key=lambda row: (row['model'], row['animation'], row['start'], row['end']))
    variant_records.sort(key=lambda row: (row['model'], row['variation'], row['animation'], row['start'], row['end']))
    all_records = sorted(records + variant_records, key=lambda row: (
        row['model'], row['variation'], row['animation'], row['start'], row['end']))
    lines = ['//! Generated incoming melee classifications, not contact or press windows.',
             '//! See docs/incoming-coverage.json for exact parameter/source evidence.',
             '//! Variation -1 is the conservative fallback when NPC identity is unavailable.',
             '#[allow(clippy::excessive_precision)]',
             'pub const ATTACKS: &[(i32, i32, i32, f32, f32, u8)] = &[']
    for row in all_records:
        lines.append(f"    ({row['model']}, {row['variation']}, {row['animation']}, {row['start']:.9f}_f32, "
                     f"{row['end']:.9f}_f32, {CODES[row['response']]}),")
    lines.append('];')
    lines.append('pub const NPC_VARIATIONS: &[(i32, i32)] = &[')
    lines.extend(f'    ({npc}, {variation}),' for npc, variation in npc_variations)
    lines.append('];')
    parameters = {name: hashlib.sha256((directory / name).read_bytes()).hexdigest()
                  for name in ('AtkParam_Npc.json', 'BehaviorParam.json', 'Bullet.json', 'ThrowParam.json', 'NpcParam.json')}
    references = {name: hashlib.sha256((refs / name).read_bytes()).hexdigest() for name in (
        'SDT.AtkParam.names.txt', 'SDT.ThrowParam.names.txt', 'SDT.AtkParam.xml',
        'SDT.ThrowParam.xml', 'SDT.BulletParam.xml', 'SDT.NpcParam.xml', 'TAE.Template.SDT.xml',
        'SDT.ATKPARAM_ATKATTR_TYPE.json')}
    report = dict(schema_version=2, purpose='incoming response classification; no reach or contact prediction',
                  executable=json.loads((directory / 'archive-inventory.json').read_text())['executable'],
                  parameter_extraction=json.loads((directory / 'attack-param-sources.json').read_text()),
                  counts=dict(collections.Counter(row['response'] for row in records)),
                  phases=len(records), models=len({row['model'] for row in records}),
                  timeline_sources=sources, parameter_sources=parameters,
                  reference_sources=references, records=records, variant_records=variant_records,
                  variant_counts=dict(collections.Counter(row['response'] for row in variant_records)),
                  npc_variations=npc_variations)
    report['npc_identity_source'] = dict(
        reference='dist/game-analysis/Ela_Sekiro_Table.CT',
        sha256=hashlib.sha256((ROOT / 'dist/game-analysis/Ela_Sekiro_Table.CT').read_bytes()).hexdigest(),
        chain=['ChrIns+0x30 -> ChrRes', 'ChrRes+0x628 -> signed NpcParam row ID'],
        validation='supported executable; mapped ID; same model; rechecked owner and ID; optional fallback')
    return '\n'.join(lines) + '\n', json.dumps(report, indent=2) + '\n', report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--input-directory', type=Path,
                        default=ROOT / 'dist/response-research-2026-09-17/extracted')
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    source, evidence, report = generate(args.input_directory)
    for name, content in [('src/incoming_attacks.rs', source), ('docs/incoming-coverage.json', evidence)]:
        path = ROOT / name
        if args.check:
            if path.read_text(encoding='utf-8') != content:
                raise SystemExit(f'{name} is not current')
        else:
            path.write_text(content, encoding='utf-8')
    print(json.dumps({key: report[key] for key in ('phases', 'models', 'counts')}))


if __name__ == '__main__':
    main()
