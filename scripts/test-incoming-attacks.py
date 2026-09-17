"""Regression cases against freshly extracted game parameters and TAE events."""
import copy
import importlib.util
import json
from pathlib import Path
import unittest
from incoming_responses import IncomingResponses

ROOT = Path(__file__).resolve().parent.parent
DATA = ROOT / 'dist/response-research-2026-09-17/extracted'
REFS = ROOT / 'dist/game-analysis/references'
spec = importlib.util.spec_from_file_location('incoming_generator', ROOT / 'scripts/generate-incoming-attacks.py')
generator = importlib.util.module_from_spec(spec)
spec.loader.exec_module(generator)


def events(model, animation):
    data = json.loads((DATA / f'c{model}.anibnd.dcx.timelines.json').read_text())
    return next(row['events'] for tae in data['timelines'] for row in tae['animations']
                if row['id'] == animation)


class IncomingTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.responses = IncomingResponses(DATA, REFS)

    def test_warning_bullet_and_explicit_dummy_establish_bandit_mikiri(self):
        response, rows = self.responses.phase(1550, events(1550, 3003), .8, .9, True)
        self.assertEqual(response, 'mikiri')
        self.assertEqual(rows, [15500050, 15500940])

    def test_generic_effects_and_aim_do_not_hide_general_sword_swing(self):
        self.assertEqual(self.responses.phase(1020, events(1020, 3003), .94, 1., True)[0], 'parry')
        self.assertEqual(self.responses.phase(1020, events(1020, 3007), 1.54, 1.66, True)[0], 'parry')

    def test_conflicting_deflectability_variants_stay_unknown(self):
        self.assertEqual(self.responses.phase(1020, events(1020, 3004), 1.1, 1.3, True)[0], 'unverified')

    def test_zero_damage_without_explicit_mikiri_marker_is_not_enough(self):
        responses = copy.deepcopy(self.responses)
        responses.names['15500940'] = 'c1550_Unrelated zero damage attack'
        self.assertEqual(responses.phase(1550, events(1550, 3003), .8, .9, True)[0], 'unverified')

    def test_marker_with_damage_or_damage_scaling_is_not_ignored(self):
        for field in ('atkPhys', 'atkPhysCorrection', 'spEffectId0'):
            responses = copy.deepcopy(self.responses)
            responses.attacks['15500940'][field] = 20
            self.assertEqual(responses.phase(1550, events(1550, 3003), .8, .9, True)[0], 'unverified')

    def test_real_unresolved_or_chained_projectile_cancels_response(self):
        for field, value in [('atkId_Bullet', 15500050), ('HitBulletID', 10), ('spEffectId1', 999)]:
            responses = copy.deepcopy(self.responses)
            responses.bullets['15500980'][field] = value
            self.assertEqual(responses.phase(1550, events(1550, 3003), .8, .9, True)[0], 'unverified')
        mixed = events(1550, 3003)
        mixed.append(dict(type=4, behavior_judge_id=-1234))
        self.assertEqual(self.responses.phase(1550, mixed, .8, .9, True)[0], 'unverified')

    def test_mikiri_requires_both_marker_and_supported_player_counter_route(self):
        responses = copy.deepcopy(self.responses)
        responses.mikiri_models.discard(1550)
        self.assertEqual(responses.phase(1550, events(1550, 3003), .8, .9, True)[0], 'parry')
        without_marker = [row for row in events(1550, 3003) if row.get('behavior_judge_id') != 940]
        self.assertEqual(self.responses.phase(1550, without_marker, .8, .9, True)[0], 'parry')

    def test_nonstandard_behavior_and_uncertain_imports_are_not_promoted(self):
        changed = events(1550, 3003)
        for row in changed:
            if row['type'] == 1:
                row['attack_type'] = 64
        self.assertEqual(self.responses.phase(1550, changed, .8, .9, True)[0], 'unverified')
        self.assertEqual(self.responses.phase(1550, events(1550, 3003), .8, .9, False)[0], 'unverified')

    def test_generated_table_has_exact_sorted_provenance(self):
        source, evidence, report = generator.generate(DATA)
        self.assertEqual(source, (ROOT / 'src/incoming_attacks.rs').read_text())
        self.assertEqual(evidence, (ROOT / 'docs/incoming-coverage.json').read_text())
        self.assertEqual(report['phases'], 2161)
        self.assertEqual(report['counts'], dict(parry=1673, unverified=313, jump=56, mikiri=63, dodge=40, avoid=16))
        keys = [(row['model'], row['animation'], row['start'], row['end']) for row in report['records']]
        self.assertEqual(keys, sorted(set(keys)))
        for row in report['records']:
            if row['response'] != 'unverified':
                self.assertTrue(row['attack_params'])


if __name__ == '__main__':
    unittest.main()
