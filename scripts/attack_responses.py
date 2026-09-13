"""Join TAE behavior IDs to local NPC attacks; never infer responses from duration.

Model/variation joins are conservative: every matching NPC behavior variation
must resolve to a row whose reference name identifies the same character model,
and all variants must agree. Runtime variation selection remains experimental.
"""
import collections
import json
from pathlib import Path
import re


class Responses:
    def __init__(self, directory, names_path):
        self.attacks=json.loads((directory/'AtkParam_Npc.json').read_text())
        behaviors=json.loads((directory/'BehaviorParam.json').read_text())
        self.names={}
        for line in names_path.read_text(encoding='utf-8-sig').splitlines():
            if ' ' in line:
                key,value=line.split(' ',1);self.names[key]=value
        self.behaviors=collections.defaultdict(list)
        for row in behaviors.values():
            # TAE type 1 is melee AttackBehavior; bullet/effect references are
            # different event families and cannot classify a melee phase.
            if row['ezStateBehaviorType_old']==2 and row['refType']==0:
                self.behaviors[(row['variationId']//10,row['behaviorJudgeId'])].append(row)

    def row_kind(self, attack_id, row):
        if row['throwFlag']==1:
            return 'dodge'
        if row['throwFlag']!=0:
            return 'unverified'  # damage within a grab is not the incoming grab
        blocked=row['disableJustGuard_vsGuardAttribute0'] or row['disableJustGuard_vsGuardAttribute1']
        if blocked:
            name=self.names.get(str(attack_id),'').lower()
            # Explicit low-sweep descriptions plus disabled deflection. The two
            # Ape rows use "ground shaving"/"horizontal slide" in the translation;
            # these are documented low sword sweeps, not inferred from aim type 7.
            sweep=bool(re.search(r'\bsweep\b|下段|足払い',name)) or attack_id in (51000541,51000560)
            if sweep and row['disableJustGuard_vsGuardAttribute0']==row['disableJustGuard_vsGuardAttribute1']==1:
                return 'jump'
            return 'unverified'
        return 'parry_candidate'

    def resolve(self, model, event):
        # Nonstandard TAE attack-type routing is not assumed to use this join.
        if event.get('attack_type')!=0:
            return 'unverified',[]
        candidates=self.behaviors.get((model,event.get('behavior_judge_id')),[])
        if not candidates: return 'unverified',[]
        kinds=set(); rows=[]
        for behavior in candidates:
            attack_id=behavior['refId'];key=str(attack_id)
            row=self.attacks.get(key)
            name=self.names.get(key,'')
            if behavior['refType']!=0 or row is None or re.search(rf'\bc{model:04}\b',name,re.I) is None:
                # Names generally use underscore separators (word characters).
                if behavior['refType']!=0 or row is None or not re.match(rf'c{model:04}(?:_|:|\s)',name,re.I):
                    return 'unverified',[]
            kinds.add(self.row_kind(attack_id,row));rows.append(attack_id)
        return (next(iter(kinds)) if len(kinds)==1 else 'unverified'),sorted(set(rows))

    def phase(self, model, events, start, end, certain):
        if not certain: return 'unverified',[]
        relevant=[e for e in events if e['type']==1 and e.get('timing_valid')
                  and e['start_seconds']<end and e['end_seconds']>start]
        resolved=[self.resolve(model,e) for e in relevant]
        kinds={kind for kind,_ in resolved}
        rows=sorted({row for _,ids in resolved for row in ids})
        # A grab alongside ordinary contact still requires avoiding the grab.
        if kinds and kinds<= {'dodge','parry_candidate'} and 'dodge' in kinds: return 'dodge',rows
        if len(kinds)==1: return next(iter(kinds)),rows
        return 'unverified',rows

    def action_interval(self, model, events, start, end, response):
        # Body/object contact can precede a grab. Use the actual grab/sweep
        # onset rather than moving its cue to an earlier unrelated hitbox.
        relevant=[e for e in events if e['type']==1 and e.get('timing_valid')
                  and e['start_seconds']<end and e['end_seconds']>start
                  and self.resolve(model,e)[0]==response]
        return min(e['start_seconds'] for e in relevant),max(e['end_seconds'] for e in relevant)
