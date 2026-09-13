"""Regression checks for import provenance and conservative phase classification."""
import importlib.util
from pathlib import Path
import unittest
from attack_responses import Responses

spec = importlib.util.spec_from_file_location('timings', Path(__file__).with_name('generate-attack-timings.py'))
timings = importlib.util.module_from_spec(spec)
spec.loader.exec_module(timings)


def hit(start, end, **extra):
    return dict(type=1, start_seconds=start, end_seconds=end, timing_valid=True,
                attack_type=0, behavior_judge_id=3000, **extra)


class TimingTests(unittest.TestCase):
    def test_cycle_and_missing_source_are_rejected(self):
        for data in [{1: dict(imports_animation=1, events=[])},
                     {1: dict(imports_animation=2, events=[])}]:
            with self.assertRaises(ValueError):
                timings.resolve(1, data)

    def test_nested_imports_propagate_uncertainty_without_merging_local_hits(self):
        source = [hit(1, 1.2)]
        data = {1: dict(events=source), 2: dict(imports_animation=1, events=[hit(2, 2.2)]),
                3: dict(imports_animation=2, events=[])}
        events, certain, exclusions = timings.resolve(3, data)
        self.assertEqual(events, source)
        self.assertFalse(certain)
        self.assertEqual(exclusions, [hit(2, 2.2)])
        self.assertEqual(timings.phases(events, certain), [(1, 1.2, False)])

    def test_plain_import_preserves_source_events(self):
        data = {1: dict(events=[hit(1, 1.2)]), 2: dict(imports_animation=1, events=[])}
        events, certain, _ = timings.resolve(2, data)
        self.assertEqual(timings.phases(events, certain), [(1, 1.2, True)])

    def test_overlapping_hitboxes_merge_but_combo_gaps_survive(self):
        events = [hit(1, 1.2), hit(1.1, 1.3), hit(1, 1.2), hit(1.4, 1.5)]
        self.assertEqual(timings.phases(events, True), [(1, 1.3, True), (1.4, 1.5, True)])

    def test_special_attack_effect_and_nonstandard_behavior_suppress_green(self):
        for extra in [dict(type=304), dict(type=4), dict(type=67),
                      dict(type=700, look_target_type=7), dict(type=0, action_flag=68)]:
            self.assertFalse(timings.phases([hit(1, 1.2), extra], True)[0][2])
        event = hit(1, 1.2)
        event['attack_type'] = 64
        self.assertFalse(timings.phases([event], True)[0][2])
        del event['attack_type']
        self.assertFalse(timings.phases([event], True)[0][2])

    def test_bad_attack_timing_invalidates_other_phases(self):
        self.assertFalse(timings.phases([hit(float('nan'), 2), hit(3, 3.2)], True)[0][2])

    def test_response_flags_do_not_turn_unknown_unblockables_into_jumps(self):
        responses=Responses.__new__(Responses)
        responses.names={'100':'c1010_Sword sweep','101':'c1010_Explosion'}
        row=dict(throwFlag=0,disableJustGuard_vsGuardAttribute0=1,disableJustGuard_vsGuardAttribute1=1)
        self.assertEqual(responses.row_kind(100,row),'jump')
        self.assertEqual(responses.row_kind(101,row),'unverified')
        row['throwFlag']=1
        self.assertEqual(responses.row_kind(100,row),'dodge')
        row['throwFlag']=2
        self.assertEqual(responses.row_kind(100,row),'unverified')

    def test_parameter_layout_handles_bitfields_and_rejects_mismatched_files(self):
        spec=importlib.util.spec_from_file_location('params',Path(__file__).with_name('inspect-attack-params.py'))
        params=importlib.util.module_from_spec(spec);spec.loader.exec_module(params)
        import tempfile
        with tempfile.TemporaryDirectory() as folder:
            path=Path(folder)/'definition.xml'
            path.write_text('<PARAMDEF><ParamType>TEST</ParamType><DataVersion>1</DataVersion><Fields>'
                            '<Field Def="u8 first:3"/><Field Def="u8 second:5"/><Field Def="s32 number"/>'
                            '</Fields></PARAMDEF>')
            _,_,size,fields=params.layout(path)
            self.assertEqual(size,5)
            self.assertEqual([(f[1],f[3],f[4]) for f in fields],[(0,0,3),(0,3,5),(1,0,0)])
            with self.assertRaises((ValueError,IndexError)):
                params.parse_param(bytes(64),path)


if __name__ == '__main__':
    unittest.main()
