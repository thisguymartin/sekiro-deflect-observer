# Investigate effect 105010

The handover identifies special effect `105010` as a candidate signal. A diagnostic reader now exists; [its source provenance and limits](reader-research.md) describe the implementation. Its relationship to the player's deflect window remains an unverified hypothesis. The handover's claim of a prior prototype is unconfirmed.

Use the [gameplay checklist](../tests/manual/gameplay-checklist.md) to gather evidence before changing the detector or public wording.

## Keep the questions separate

| Question | Evidence needed |
|---|---|
| Does the reader inspect the current local player on the identified build? | Executable identity, pointer provenance, ownership checks, read validity, and lifecycle tests. |
| Does the reader observe effect 105010 correctly? | Valid raw observations compared with the displayed state, including absence and failed reads. |
| What does the effect represent? | Input, guard, combat, and no-input comparisons, plus inspection of relevant runtime logic where available. |
| How accurately does the display represent observations? | Sample timestamps, render timing, capture timing, and dropped-sample evidence. |

A successful synthetic test answers a reader or model question. A video of a green indicator alone cannot establish an engine timing rule.

## Record the reader's provenance

1. Record the exact executable hash and the source of each version-sensitive offset, signature, and structure interpretation.
2. Record how the reader finds the current local player and distinguishes it from an enemy, old player instance, or missing player.
3. Record how the effect collection terminates and how the reader detects incomplete reads, cycles, excessive node counts, and player changes during traversal.
4. Record whether samples include a monotonic timestamp, session identity, read status, candidate-effect presence, and error reason. Mark fields the prototype cannot expose as missing.
5. Record the polling interval, observed scheduling gaps, freshness limit, and time spent traversing memory. Do not substitute the configured interval for measured intervals.

Do not read unknown offsets to discover whether an unsupported build "seems to work." Establish a build profile through separate reverse-engineering work first.

## Compare competing explanations

1. Start with G01 through G04. Test whether the candidate follows input edges, the entire guard hold, or another state.
2. Compare G05, G06, G07, and G08 using the same enemy attack where feasible. Record distance, posture, buffs or debuffs, and other relevant differences.
3. Classify contact as deflect, block, damage, no contact, or ambiguous using gameplay audio, animation, and visible consequences before consulting the indicator.
4. Check whether the candidate appears without a successful deflect, without enemy contact, or without player input.
5. Repeat contradictory trials in a fresh process. Keep the original trial and its context.
6. Inspect relevant game logic or independently observed contact acceptance before claiming that effect boundaries equal the complete engine deflect window.

Use this table to guide interpretation. These are hypotheses to test, not established facts about `105010`.

| Observation | Interpretation to investigate |
|---|---|
| Effect appears after a tap in an empty area | May represent player preparation rather than a successful combat outcome. |
| Effect lasts for the entire guard hold | May represent guard state rather than the brief input-created window. |
| Effect appears only after contact | May represent a result or reaction rather than readiness before contact. |
| Effect appears after an unguarded hit | May represent damage response or a broader state. |
| Effect survives process exit in the display | Indicates stale display data, not a persistent game effect. |
| Successful deflects occur outside the displayed interval | Investigate sampling delay, missed intervals, player ownership, other mechanics, and the hypothesis itself. |

Do not tune offsets, history smoothing, or colors to hide contradictions.

## Capture and align evidence

1. Give each trial a case ID and trial number before recording.
2. Capture gameplay, observer output, and independent input evidence together where possible. Preserve game audio for outcome classification.
3. If input and gameplay use separate recordings, capture a shared visible or audible event before and after the trial. Measure alignment and drift. Do not align recordings by assuming the green flash equals the input.
4. Preserve the original recording. Use an annotated copy for presentation.
5. Mark input press, input release, candidate onset, candidate end, enemy contact, and outcome using frame indices or presentation timestamps.
6. Record dropped or duplicate frames, uncertain boundaries, and hidden inputs. If a boundary cannot be resolved, label it uncertain.
7. If sample logs exist, align their monotonic clock to the recording with a recorded marker. Document the alignment error. Never compare unrelated clock origins directly.

At 60 recorded frames per second, one frame spans about 16.67 ms. At 120, it spans about 8.33 ms. These are frame intervals, not guarantees of measurement accuracy. Capture, input, polling, and rendering can add separate delays.

For constant-frame-rate video, calculate the displayed duration as:

```text
displayed duration in ms = (first non-active frame - first active frame) * 1000 / capture FPS
```

Use actual presentation timestamps for variable-frame-rate footage. Report a video duration as a displayed interval, not the exact engine window or input-to-engine latency.

## Bound durations from raw samples

Use sample timestamps only after checking the timestamp's meaning. A read-start timestamp is different from a completed-snapshot timestamp.

For one uninterrupted candidate interval, record these four valid observations:

```text
t0  inactive, immediately before the first active observation
t1  first active observation
t2  last active observation
t3  inactive, immediately after the last active observation
```

Under a single-continuous-interval assumption, with instantaneous valid observations and no unknown gaps:

```text
onset lies in (t0, t1]
end lies in (t2, t3]
duration has conservative bounds [t2 - t1, t3 - t0]
```

These bounds do not prove that the effect remained active between samples. Faster off/on changes can be missed. Expand uncertainty for read duration and synchronization error. A single active sample has a zero lower bound under this method.

If any boundary is missing, stale, or unknown, mark the interval incomplete. Do not replace missing time with zero or include that interval in complete-window averages. Do not claim sub-polling-interval precision from interpolated display bars.

## Write the finding

1. Link the exact run reports, trials, recordings, and source observations supporting the claim.
2. State the narrow behavior observed, including the tested build and conditions.
3. Count complete, incomplete, ambiguous, and contradictory trials separately.
4. Report durations with their measurement method and uncertainty. Distinguish visible input counts from observed active intervals.
5. List explanations that the evidence does not yet separate.
6. Assign qualitative confidence and explain the basis. Use a numerical confidence estimate only when a defined statistical method supports it.
7. Keep the public label as a candidate effect or observed state until the proposed deflect-window meaning has independent support.

Repeated agreement supports a scoped claim. It does not prove identical behavior for every enemy, executable build, or player state. Do not label activations as "perfect deflects" or use them as an input counter without separate evidence.
