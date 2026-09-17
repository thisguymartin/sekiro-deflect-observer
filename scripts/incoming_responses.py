"""Classify incoming melee responses independently of contact prediction.

Source joins remain unanimous across model behavior variants. Only the exact
inspected harmless danger-warning route and explicit Mikiri dummy rows can be
removed from a mixed phase; arbitrary zero-damage hits are not ignored.
"""
import collections
import json
from pathlib import Path
from attack_responses import Responses


def names(path):
    return dict(line.split(' ', 1) for line in path.read_text(encoding='utf-8-sig').splitlines()
                if ' ' in line)


class IncomingResponses(Responses):
    def __init__(self, directory, references):
        super().__init__(directory, references / 'SDT.AtkParam.names.txt')
        self.bullets = json.loads((directory / 'Bullet.json').read_text())
        self.bullet_behaviors = collections.defaultdict(list)
        for row in json.loads((directory / 'BehaviorParam.json').read_text()).values():
            if row['ezStateBehaviorType_old'] == 2:
                self.bullet_behaviors[(row['variationId'] // 10, row['behaviorJudgeId'])].append(row)
        throw_names = names(references / 'SDT.ThrowParam.names.txt')
        self.mikiri_models = {
            row['DefChrId'] for key, row in json.loads((directory / 'ThrowParam.json').read_text()).items()
            if row['AtkChrId'] == 0 and row['throwKind'] in (30000, 30100, 30110)
            and '\u898b\u5207' in throw_names.get(key, '')
        }

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
            # Local AtkParam 8 is the non-damaging warning carrier; effect 211000
            # /211001 draws the danger warning. No child projectile is allowed.
            attack = self.attacks.get(str(bullet['atkId_Bullet']))
            if (bullet['atkId_Bullet'] != 8 or attack is None
                    or not self.harmless_attack(attack)
                    or bullet['spEffectId0'] not in (211000, 211001)
                    or any(bullet['spEffectId' + str(i)] not in (0, -1) for i in range(1, 5))
                    or bullet['spEffectIDForShooter'] not in (0, -1)
                    or bullet['HitBulletID'] != -1 or bullet['intervalCreateBulletId'] != -1):
                return False
        return True

    def row_kind(self, attack_id, row):
        name = self.names.get(str(attack_id), '')
        if ('\u898b\u5207\u3089\u308c\u30c0\u30df\u30fc' in name
                and self.harmless_attack(row)
                and row.get('atkAttribute') == 3
                and row.get('staminaPhysicsAttribute') == 3
                and row.get('isDisableParry') == 0
                and row.get('staminaDamageAttackHitParry', 0) > 0):
            return 'mikiri_marker'
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
        response, rows = super().resolve(model, event)
        if response == 'unverified' and rows:
            kinds = {self.row_kind(key, self.attacks[str(key)]) for key in rows}
            if kinds <= {'parry_candidate', 'thrust'}:
                # Both variants can be deflected; a variant-specific Mikiri
                # recommendation would still be unsupported.
                response = 'parry_candidate'
        return response, rows

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
        if any(event['type'] in (5, 304) or
               (event['type'] in (2, 4) and not self.warning_bullet(model, event))
               for event in events):
            return 'unverified', rows
        kinds = {kind for kind, _ in resolved}
        marker = 'mikiri_marker' in kinds
        kinds.discard('mikiri_marker')
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
        return 'unverified', rows
