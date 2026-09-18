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
        self.assertEqual(report['phases'], 2112)
        self.assertEqual(report['counts'], dict(parry=1632, unverified=293, jump=59, mikiri=66, dodge=41, avoid=21))
        keys = [(row['model'], row['animation'], row['start'], row['end']) for row in report['records']]
        self.assertEqual(keys, sorted(set(keys)))
        variant_keys = [(r['model'], r['variation'], r['animation'], r['start'], r['end'])
                        for r in report['variant_records']]
        self.assertEqual(variant_keys, sorted(set(variant_keys)))
        mapped_variations = {variation for _, variation in report['npc_variations']}
        for row in report['variant_records']:
            self.assertIn(row['variation'], mapped_variations)
            self.assertEqual(row['variation'] // 10, row['model'])
        for row in report['records'] + report['variant_records']:
            self.assertLess(row['start'], row['end'])
            if row['response'] != 'unverified':
                self.assertTrue(row['attack_params'])

    def test_spear_variant_recovers_mikiri_without_guessing_sword_variant(self):
        spear = self.responses.for_variation(10102)
        sword = self.responses.for_variation(10101)
        move = events(1010, 200003004)
        self.assertEqual(spear.phase(1010, move, 1.24, 1.36, True)[0], 'mikiri')
        self.assertNotEqual(sword.phase(1010, move, 1.24, 1.36, True)[0], 'mikiri')
        self.assertEqual(self.responses.phase(1010, move, 1.24, 1.36, True)[0], 'unverified')

    def test_counter_marker_does_not_bridge_three_spear_hits(self):
        spear = self.responses.for_variation(10102)
        move = events(1010, 200003008)
        phases = spear.intervals(1010, move)
        self.assertEqual(len(phases), 3)
        for start, end in phases:
            self.assertEqual(spear.phase(1010, move, start, end, True)[0], 'mikiri')
        self.assertAlmostEqual(phases[0][1], 1.333333, places=5)
        self.assertAlmostEqual(phases[1][0], 1.366667, places=5)

    def test_snake_eyes_grab_is_not_borrowed_by_normal_gunner(self):
        move = events(1190, 100003013)
        self.assertEqual(self.responses.for_variation(11901).phase(1190, move, 1.6, 1.73, True)[0], 'dodge')
        self.assertEqual(self.responses.for_variation(11900).phase(1190, move, 1.6, 1.73, True)[0], 'parry')
        self.assertEqual(self.responses.phase(1190, move, 1.6, 1.73, True)[0], 'unverified')
        self.assertEqual(self.responses.phase(1190, events(1190, 100004100), .2, .36, True)[0], 'dodge')

    def test_ninja_mikiri_warning_uses_verified_dummy_zero_route(self):
        move = events(1400, 3005)
        self.assertEqual(self.responses.phase(1400, move, 1.37, 1.53, True)[0], 'mikiri')
        responses = copy.deepcopy(self.responses)
        responses.attacks['0']['atkPhysCorrection'] = 10
        self.assertEqual(responses.phase(1400, move, 1.37, 1.53, True)[0], 'unverified')

    def test_genichiro_visuals_do_not_mask_sword_but_child_projectiles_do(self):
        move = events(7100, 3026)
        self.assertEqual(self.responses.phase(7100, move, .97, 1.06, True)[0], 'parry')
        for field in ('HitBulletID', 'intervalCreateBulletId', 'generateObjId', 'spEffectId0', 'autoSearchNPCThinkID'):
            responses = copy.deepcopy(self.responses)
            event = next(e for e in move if e['type'] == 2)
            for behavior in responses.bullet_behaviors[(7100, event['behavior_judge_id'])]:
                responses.bullets[str(behavior['refId'])][field] = 99
            self.assertEqual(responses.phase(7100, move, .97, 1.06, True)[0], 'unverified', field)

    def test_object_only_horse_contact_does_not_claim_attack_on_wolf(self):
        move = events(5080, 5010)
        self.assertEqual(self.responses.intervals(5080, move), [])
        responses = copy.deepcopy(self.responses)
        responses.attacks['50800991']['opposeTarget'] = 1
        self.assertEqual(len(responses.intervals(5080, move)), 1)

    def test_demon_charge_and_following_body_contact_keep_separate_actions(self):
        move = events(7020, 3026)
        phases = self.responses.intervals(7020, move)
        # Explicit non-opponent body hitbox is not a second defensive action.
        self.assertEqual([self.responses.phase(7020, move, s, e, True)[0] for s, e in phases], ['avoid'])


if __name__ == '__main__':
    unittest.main()
