"""Classify incoming melee responses independently of contact prediction.

Joins use the verified NPC variation, or require consensus when unavailable.
Explicit counter dummies and non-opponent hitboxes do not define attack timing.
Only verified warning/visual bullet routes are ignored; unknown payloads remain
unresolved, and ordinary zero-damage melee events are not silently discarded.
"""
import collections
import copy
import json
import math
import re
from attack_responses import Responses


def names(path):
    return dict(line.split(' ', 1) for line in path.read_text(encoding='utf-8-sig').splitlines()
                if ' ' in line)


class IncomingResponses(Responses):
    def __init__(self, directory, references, variation=None):
        super().__init__(directory, references / 'SDT.AtkParam.names.txt')
        self.variation = variation
        self.bullets = json.loads((directory / 'Bullet.json').read_text())
        self.bullet_behaviors = collections.defaultdict(list)
        for row in json.loads((directory / 'BehaviorParam.json').read_text()).values():
            if row['ezStateBehaviorType_old'] == 2 and (variation is None or row['variationId'] == variation):
                self.bullet_behaviors[(row['variationId'] // 10, row['behaviorJudgeId'])].append(row)
        throw_names = names(references / 'SDT.ThrowParam.names.txt')
        self.mikiri_models = {
            row['DefChrId'] for key, row in json.loads((directory / 'ThrowParam.json').read_text()).items()
            if row['AtkChrId'] == 0 and row['throwKind'] in (30000, 30100, 30110)
            and '\u898b\u5207' in throw_names.get(key, '')
        }

    def for_variation(self, variation):
        result = copy.copy(self)
        result.variation = variation
        result.bullet_behaviors = {key: [b for b in values if b['variationId'] == variation]
                                   for key, values in self.bullet_behaviors.items() if key[0] == variation // 10}
        return result

    @staticmethod
    def harmless_attack(row):
        fields = ['atkPhys', 'atkMag', 'atkFire', 'atkThun', 'atkDark', 'atkStam',
                  'directAtkStamDamage', 'atkSuperArmor', 'throwFlag',
                  'atkPhysCorrection', 'atkMagCorrection', 'atkFireCorrection',
                  'atkThunCorrection', 'atkDarkCorrection', 'atkStamCorrection']
        return (all(row.get(field) == 0 for field in fields)
                and all(row.get('spEffectId' + str(i)) in (0, -1) for i in range(5)))

    def warning_bullet(self, model, event):
        candidates = self.bullet_behaviors.get((model, event.get('behavior_judge_id')), [])
        if not candidates:
            return False
        for behavior in candidates:
            bullet = self.bullets.get(str(behavior['refId']))
            if behavior['refType'] != 1 or bullet is None:
                return False
            # Local AtkParam 0 and 8 are non-damaging dummy carriers; effect 211000
            # /211001 draws the danger warning. No child projectile is allowed.
            attack = self.attacks.get(str(bullet['atkId_Bullet']))
            if (bullet['atkId_Bullet'] not in (0, 8) or attack is None
                    or not self.harmless_attack(attack)
                    or bullet['spEffectId0'] not in (211000, 211001)
                    or any(bullet['spEffectId' + str(i)] not in (0, -1) for i in range(1, 5))
                    or bullet['spEffectIDForShooter'] not in (0, -1)
                    or bullet['HitBulletID'] != -1 or bullet['intervalCreateBulletId'] != -1
                    or bullet['generateObjId'] != -1 or bullet['autoSearchNPCThinkID'] != 0):
                return False
        return True

    def visual_bullet(self, model, event):
        candidates = self.bullet_behaviors.get((model, event.get('behavior_judge_id')), [])
        if not candidates:
            return False
        for behavior in candidates:
            bullet = self.bullets.get(str(behavior['refId']))
            if behavior['refType'] != 1 or bullet is None:
                return False
            attack = self.attacks.get(str(bullet['atkId_Bullet']))
            if (bullet['atkId_Bullet'] not in (0, 8) or attack is None
                    or not self.harmless_attack(attack)
                    or any(bullet['spEffectId' + str(i)] not in (0, -1) for i in range(5))
                    or bullet['spEffectIDForShooter'] not in (0, -1)
                    or bullet['HitBulletID'] != -1 or bullet['intervalCreateBulletId'] != -1
                    or bullet['generateObjId'] != -1 or bullet['autoSearchNPCThinkID'] != 0):
                return False
        return True

    def row_kind(self, attack_id, row):
        name = self.names.get(str(attack_id), '')
        if row.get('opposeTarget') == 0:
            return 'non_hostile'
        if row.get('throwFlag') == 2:
            return 'throw_damage'
        if ('\u898b\u5207\u3089\u308c\u30c0\u30df\u30fc' in name
                and self.harmless_attack(row)
                and row.get('atkAttribute') == 3
                and row.get('staminaPhysicsAttribute') == 3
                and row.get('isDisableParry') == 0
                and row.get('staminaDamageAttackHitParry', 0) > 0):
            return 'mikiri_marker'
        if '\u898b\u5207\u3089\u308c\u30c0\u30df\u30fc' in name or self.harmless_attack(row):
            return 'unverified'
        result = super().row_kind(attack_id, row)
        if result == 'parry_candidate' and (row.get('atkAttribute') == 3
                and row.get('staminaPhysicsAttribute') == 3
                and row.get('isDisableParry') == 0):
            return 'thrust'
        if result == 'unverified' and row['throwFlag'] == 0 and (
                row['disableJustGuard_vsGuardAttribute0'] == 1
                and row['disableJustGuard_vsGuardAttribute1'] == 1):
            return 'avoid'
        return result

    def resolve(self, model, event):
        if event.get('attack_type') != 0:
            return 'unverified', []
        candidates = self.behaviors.get((model, event.get('behavior_judge_id')), [])
        if self.variation is not None:
            candidates = [b for b in candidates if b['variationId'] == self.variation]
        if not candidates:
            return 'unverified', []
        rows, kinds = [], set()
        for behavior in candidates:
            key = behavior['refId']
            row = self.attacks.get(str(key))
            # Parameter IDs and explicit behavior references are the primary
            # identity. Community names can retain a different form's label.
            if row is None or (key // 10000 != model and not re.match(
                    rf'c{model:04}(?:_|:|\s)', self.names.get(str(key), ''), re.I)):
                return 'unverified', []
            rows.append(key)
            kinds.add(self.row_kind(key, row))
        if len(kinds) == 1:
            return next(iter(kinds)), sorted(set(rows))
        if kinds <= {'parry_candidate', 'thrust'}:
            return 'parry_candidate', sorted(set(rows))
        return 'unverified', sorted(set(rows))

    def intervals(self, model, events):
        """Actual hostile melee windows; counter dummies must not bridge hits."""
        intervals = []
        for event in events:
            if event['type'] != 1 or not event.get('timing_valid'):
                continue
            start, end = event['start_seconds'], event['end_seconds']
            if (start is None or end is None or not math.isfinite(start) or not math.isfinite(end)
                    or not 0 <= start < end <= 120):
                continue
            if self.resolve(model, event)[0] in ('mikiri_marker', 'non_hostile', 'throw_damage'):
                continue
            intervals.append((start, end))
        merged = []
        for start, end in sorted(set(intervals)):
            if merged and start <= merged[-1][1] + .00001:
                old_start, old_end = merged[-1]
                # Touching activations with different responses are successive
                # actions, e.g. Demon charge -> ordinary body contact.
                before = self.phase(model, events, old_start, old_end, True)[0]
                after = self.phase(model, events, start, end, True)[0]
                if start < old_end - .00001 or before == after:
                    merged[-1] = (old_start, max(old_end, end))
                    continue
            merged.append((start, end))
        return [(s, e) for s, e in merged if e - s <= 5]

    def throw_followup(self, model, event):
        # Type 304 applies damage inside an already established throw. It is
        # not a second incoming grab. Require every route to explicitly say so.
        probe = dict(event, attack_type=0)
        _, ids = self.resolve(model, probe)
        return bool(ids) and all(self.attacks[str(key)]['throwFlag'] == 2 for key in ids)

    def phase(self, model, events, start, end, certain):
        if not certain:
            return 'unverified', []
        relevant = [event for event in events if event['type'] == 1
                    and event.get('timing_valid') and event['start_seconds'] < end
                    and event['end_seconds'] > start]
        resolved = [self.resolve(model, event) for event in relevant]
        rows = sorted({row for _, ids in resolved for row in ids})
        # Generic aim and SpEffect events do not override resolved melee flags.
        # Unknown common/throw dispatches and real/unresolved projectiles do.
        if any(event['type'] == 5 or
               (event['type'] == 304 and not self.throw_followup(model, event)) or
               (event['type'] in (2, 4) and not (self.warning_bullet(model, event)
                                               or self.visual_bullet(model, event)))
               for event in events):
            return 'unverified', rows
        kinds = {kind for kind, _ in resolved}
        marker = 'mikiri_marker' in kinds
        kinds.discard('mikiri_marker')
        kinds.discard('non_hostile')
        kinds.discard('throw_damage')
        if kinds == {'thrust'}:
            # Require all three: explicit thrust fields, a simultaneous Mikiri
            # marker hitbox, and a player->enemy Mikiri route in ThrowParam.
            return ('mikiri' if marker and model in self.mikiri_models else 'parry'), rows
        if kinds == {'parry_candidate'}:
            return 'parry', rows
        if kinds and kinds <= {'dodge', 'parry_candidate'} and 'dodge' in kinds:
            return 'dodge', rows
        if len(kinds) == 1 and next(iter(kinds)) in ('jump', 'avoid'):
            return next(iter(kinds)), rows
        if 'avoid' in kinds and kinds <= {'avoid', 'thrust', 'parry_candidate'}:
            return 'avoid', rows
        return 'unverified', rows
