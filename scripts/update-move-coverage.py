"""Regenerate the exact move-phase ledger and its conservative coverage summary."""

import argparse
import collections
import csv
import hashlib
import io
import json
from pathlib import Path
import re
import sys


ROOT = Path(__file__).resolve().parent.parent
OUTPUTS = ('docs/boss-move-phases.csv', 'docs/boss-move-coverage.md')


def section(source, name):
    match = re.search(
        rf'pub const {name}:.*?= &\[\n(?P<body>.*?)\n\];', source, re.S)
    if match is None:
        raise ValueError(f'missing {name} table')
    return match.group('body')


def parse_tables(source):
    attack_rows = []
    attack_pattern = re.compile(
        r'^\s*\((-?\d+), (-?\d+), ([0-9.]+)_f32, '
        r'([0-9.]+)_f32, (true|false)\),$', re.M)
    for match in attack_pattern.finditer(section(source, 'ATTACKS')):
        attack_rows.append((
            int(match[1]), int(match[2]), match[3], match[4],
            match[5] == 'true'))

    response_pattern = re.compile(
        r'^\s*\((-?\d+), (-?\d+), ([0-9.]+)_f32, '
        r'([0-9.]+)_f32, ([12])\),$', re.M)
    response_rows = {
        (int(match[1]), int(match[2]), match[3], match[4]):
            ('dodge' if match[5] == '1' else 'jump')
        for match in response_pattern.finditer(section(source, 'RESPONSES'))
    }
    special_pattern = re.compile(r'^\s*\((-?\d+), (-?\d+), ([123])\),$', re.M)
    special_rows = {
        (int(match[1]), int(match[2]), int(match[3]))
        for match in special_pattern.finditer(section(source, 'SPECIALS'))
    }
    if not attack_rows:
        raise ValueError('ATTACKS table is empty')
    if len(attack_rows) != len(set(attack_rows)):
        raise ValueError('ATTACKS table contains duplicate exact rows')
    return attack_rows, response_rows, special_rows


def evidence_key(phase):
    return (
        phase['model'], phase['animation'], f"{phase['start']:.9f}",
        f"{phase['end']:.9f}", phase['response'])


def build_outputs(root=ROOT):
    source_path = root / 'src/attack_timings.rs'
    source_bytes = source_path.read_bytes()
    source = source_bytes.decode('utf-8').replace('\r\n', '\n')
    attacks, responses, specials = parse_tables(source)
    enemy = json.loads((root / 'docs/enemy-coverage.json').read_text(encoding='utf-8'))
    response_report = json.loads(
        (root / 'docs/response-coverage.json').read_text(encoding='utf-8'))

    archives = enemy['coverage']
    archive_by_model = {row['model']: row for row in archives}
    if len(archive_by_model) != len(archives):
        raise ValueError('enemy coverage contains duplicate model records')

    evidence = {evidence_key(phase): phase for phase in response_report['phases']}
    if len(evidence) != len(response_report['phases']):
        raise ValueError('response coverage contains duplicate exact phase records')

    fieldnames = [
        'model', 'animation_id', 'phase_ordinal', 'activation_animation_s',
        'deactivation_animation_s', 'response', 'status', 'calibrated',
        'gameplay_validated', 'timeline_source_sha256', 'attack_param_ids',
        'source_references',
    ]
    csv_buffer = io.StringIO(newline='')
    writer = csv.DictWriter(csv_buffer, fieldnames=fieldnames, lineterminator='\n')
    writer.writeheader()
    ordinals = collections.defaultdict(int)
    counts = collections.Counter()
    models = collections.defaultdict(collections.Counter)
    used_evidence = set()

    for model, animation, start, end, green in attacks:
        key = (model, animation)
        ordinal = ordinals[key]
        ordinals[key] += 1
        response = responses.get((model, animation, start, end))
        if response is None:
            response = 'parry' if green else 'unverified'
        elif green:
            raise ValueError(f'action response also marked parry: c{model:04}/{animation}')

        attack_ids = ''
        references = [
            'src/attack_timings.rs', 'docs/enemy-coverage.json',
            'scripts/generate-attack-timings.py', 'scripts/attack_responses.py',
        ]
        if response in ('dodge', 'jump'):
            key_with_response = (model, animation, start, end, response)
            phase = evidence.get(key_with_response)
            if phase is None:
                raise ValueError(f'missing action evidence: {key_with_response}')
            used_evidence.add(key_with_response)
            attack_ids = ';'.join(str(value) for value in phase['attack_params'])
            references = [
                'src/attack_timings.rs', 'docs/enemy-coverage.json',
                'docs/response-coverage.json', 'scripts/generate-attack-timings.py',
                'scripts/attack_responses.py',
            ]

        archive = archive_by_model.get(model)
        if archive is None:
            raise ValueError(f'missing source provenance for c{model:04}')
        status = 'classified_estimate' if response != 'unverified' else 'extracted_unverified'
        writer.writerow({
            'model': f'c{model:04}', 'animation_id': animation,
            'phase_ordinal': ordinal, 'activation_animation_s': start,
            'deactivation_animation_s': end, 'response': response,
            'status': status, 'calibrated': 'false',
            'gameplay_validated': 'false',
            'timeline_source_sha256': archive['source_sha256'],
            'attack_param_ids': attack_ids,
            'source_references': ';'.join(references),
        })
        counts[response] += 1
        models[model]['phases'] += 1
        models[model][response] += 1

    if used_evidence != set(evidence):
        extra = sorted(set(evidence) - used_evidence)
        raise ValueError(f'response evidence has rows absent from runtime table: {extra[:3]}')

    expected = {
        'phases': len(attacks), 'green_phases': counts['parry'],
        'dodge_phases': counts['dodge'], 'jump_phases': counts['jump'],
        'special_animations': len(specials), 'archives': len(archives),
        'models_with_phases': sum(row.get('phases', 0) > 0 for row in archives),
        'models_with_green': sum(row.get('green_phases', 0) > 0 for row in archives),
    }
    for field, value in expected.items():
        if enemy.get(field) != value:
            raise ValueError(
                f'enemy coverage {field}={enemy.get(field)!r}, runtime table has {value}')
    for archive in archives:
        actual = models[archive['model']]
        for field, key in (
                ('phases', 'phases'), ('green_phases', 'parry'),
                ('dodge_phases', 'dodge'), ('jump_phases', 'jump')):
            if archive.get(field, 0) != actual[key]:
                raise ValueError(
                    f"c{archive['model']:04} {field} disagrees with runtime table")

    data_hash = hashlib.sha256(source.encode('utf-8')).hexdigest()
    classified = counts['parry'] + counts['dodge'] + counts['jump']
    unverified = counts['unverified']

    inventory = []
    for archive in archives:
        model = archive['model']
        row = models[model]
        inventory.append(
            f"| c{model:04} | {row['phases']} | {row['parry']} | "
            f"{row['dodge']} | {row['jump']} | {row['unverified']} | none recorded |")

    ogre = models[5020]
    ape = models[5100]
    document = f"""# Boss and move coverage - audited 2026-09-16

This is the checked-in runtime table inventory, not a claim of complete boss
support. Run `python scripts/update-move-coverage.py --check` to verify that this
summary and the phase ledger still match the exact generated tables.

The full [phase ledger](boss-move-phases.csv) preserves all {len(attacks):,} exact phase
records: model IDs, animation IDs, zero-based phase ordinals, activation/deactivation boundary
strings from Rust, current response classifications, source JSON hashes and
available attack-parameter references. Boundaries are animation seconds; the
Rust literals are f32 approximations. Original extraction precision is retained
in [response evidence](response-coverage.json) for mapped dodge/jump phases.
Animation variants and combo phases are not unique human moves. No human move
names, calibration results or success percentages were invented.

## Status definitions and counts

| Status | Meaning | Audited result |
| --- | --- | --- |
| Extracted | Phase present in the generated table; activation is not contact | {len(attacks):,} phases across {expected['models_with_phases']} models |
| Classified estimate | Conservative response classification plus activation-based timing | {counts['parry']} parry, {counts['dodge']} dodge, {counts['jump']} jump = {classified} phases |
| Extracted, unverified | No actionable response classification | {unverified:,} phases |
| Calibrated | Compatible per-move press/contact evidence and profile | 0 records |
| Gameplay validated | Complete trial evidence identifies this exact phase and successful/failed responses | 0 complete per-move records |
| Unsupported | No implemented response or required capability source | Mikiri; special counter guidance including lightning/terror/fire/projectile handling |
| Unknown | Encounter/form or runtime behavior routing not established | Do not infer support from an archive/model name |

There are {len(archives)} extracted archive records, {expected['models_with_green']} models with parry
estimates and {len(specials):,} special-animation indicators. Special indicators
overlap other inventory and are not extra supported moves. An extracted phase
with activation at zero can have no usable advance interval; a row is not a
guarantee of an actionable cue.

Sources: [generated runtime tables](../src/attack_timings.rs),
[model/source hashes](enemy-coverage.json), [response/source evidence](response-coverage.json),
[generator](../scripts/generate-attack-timings.py),
[classifier](../scripts/attack_responses.py) and
[coverage generator](../scripts/update-move-coverage.py).
Generated data SHA-256 at audit:
`{data_hash}`.
The offline parameter join does not prove runtime behavior-variation selection.
The ledger does not manufacture missing per-parry parameter provenance: those
rows retain the exact timeline source hash and generator/classifier references;
expanded per-parry parameter provenance remains work.

An explicit type-4 `BulletBehavior_Midair` event makes every response for that
mixed animation unverified. Type-2 `BulletBehavior` and generic effect/aim markers
occur throughout existing parameter-backed Ogre, Ape and other action rows, so
their presence alone does not erase an exact dodge/jump parameter join. They
remain excluded from green/parry eligibility by the broader phase classifier.

## Encounter/form evidence

| Encounter or requested form | Exact model evidence | Current status / missing evidence |
| --- | --- | --- |
| Ordinary soldier baseline | c1010 in historical capture and current log | Extracted/classified estimate; no complete press/contact/outcome trial; validate first |
| General Naomori Kawarada | c1020 in an earlier local recording | Shared model/behavior variations unresolved; clip does not validate all c1020 actors |
| Chained Ogre | c5020; {ogre['phases']} phases, {ogre['parry']} parry, {ogre['dodge']} dodge | Historical auxiliary-40000 selection failure; corrected current-batch selector has synthetic tests, new live timing trial required |
| Guardian Ape encounter | c5100; {ape['phases']} phases, {ape['parry']} parry, {ape['dodge']} dodge, {ape['jump']} jump | Selected source-based mappings; no complete contact/press calibration |
| Guardian Ape sword/headless form | c5100 includes 100003xxx mappings; exact runtime form applicability unvalidated | Requested form remains unresolved; animation bank alone is not form validation |
| Headless Ape encounter / companion | Runtime model/form mapping not established in reviewed evidence | Requested related forms unresolved; do not transfer Guardian Ape trial claims |
| Other boss forms and variant archives | Exact inventory below, including zero-phase variants | A related model ID or shared archive is not inheritance of support |

The five Ogre dodge phases are 100003005 `[0.566666663,0.633333325)`,
100003007 `[2.166666746,2.266666651)`, 100003008 `[1.000000000,1.100000024)`,
100003013 `[1.133333325,1.366666675)` and
100003014 `[1.333333373,1.466666698)`. These are extracted animation boundaries,
not observed player-contact intervals. Ape response phases and every other
boundary are preserved in the ledger; no similar ID receives their profiles.

Runtime calibration profiles are active only when `form = "model-animation-phase"`
states that the evidence covers the entire exact model/animation/phase key.
Named encounter or boss forms remain inactive because the runtime has no validated
form observation source. A calibration profile never changes response eligibility.

## Exact model inventory

Every extracted archive is listed, including archives without resolved phases.
Unverified is the phase total minus the three classified-response counts; it is
not the generator's `uncertain_phases` field, which also includes dodge/jump.

| Model | Phases | Parry estimate | Dodge estimate | Jump estimate | Unverified | Calibration / gameplay validation |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
{chr(10).join(inventory)}

## Evidence requirements before promoting a move

Retain exact executable/DLL/data hashes and observer version; encounter and form;
model, animation and phase ID; contact and actual press observations; trial count
including successes, failures and ambiguous outcomes; distance/angle; frame rate,
display mode, game modifiers and links to original logs/video. State uncertainty
and measurement resolution. A draw call, TAE crossing or effect 105010 alone is
not a successful deflect. There are no complete records to promote in this audit.

The latest reviewed 0.7.0 log has 83 parry **submissions**, not 83 deflects.
Historical [0.6 evidence](validation-0.6.md) retains its original scope. No new
gameplay acceptance criterion is complete.

Next validation order: soldier baseline, Ogre track regression, selected Ape
responses, then representative supported boss combos. Record each run with the
[trial template](../tests/compatibility/cue-trial-template.md).
"""
    return {
        'docs/boss-move-phases.csv': csv_buffer.getvalue(),
        'docs/boss-move-coverage.md': document,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        '--check', action='store_true',
        help='fail if checked-in outputs differ; do not write files')
    args = parser.parse_args()
    outputs = build_outputs(ROOT)
    stale = []
    for relative, content in outputs.items():
        path = ROOT / relative
        if args.check:
            if not path.exists() or path.read_text(encoding='utf-8') != content:
                stale.append(relative)
        else:
            path.write_text(content, encoding='utf-8', newline='')
    if stale:
        print('stale generated coverage: ' + ', '.join(stale), file=sys.stderr)
        return 1
    counts = collections.Counter(
        row['response'] for row in csv.DictReader(
            io.StringIO(outputs['docs/boss-move-phases.csv'])))
    print(json.dumps({
        'phases': sum(counts.values()), 'parry': counts['parry'],
        'dodge': counts['dodge'], 'jump': counts['jump'],
        'unverified': counts['unverified'], 'checked': args.check,
    }))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
