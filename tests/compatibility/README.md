# Compatibility records

There are no completed game test runs in this checkout. The [session template](session-template.md) is a blank form, not evidence of compatibility.

## Save a report

1. Copy the template to a Markdown file in this directory. Use a name such as `YYYY-MM-DD-windows11-REVISION.md`, replacing every placeholder.
2. Record the exact artifact, executable, environment, and settings before testing.
3. Complete each case you attempt using the [gameplay checklist](../manual/gameplay-checklist.md).
4. Link each result to a trial, clip time range, frame indices, or relevant log excerpt.
5. List the report below only after it contains actual observations.
6. Update [public compatibility status](../../docs/compatibility.md) only when the report supports the claim.

## Result meanings

| Result | Meaning |
|---|---|
| Pass | The stated requirement was exercised and evidence supports it. |
| Fail | The test ran and contradicted the requirement. |
| Blocked | A prerequisite or measurement capability prevented a conclusion. |
| Not run | The case was not attempted. |
| Not applicable | The case does not apply to this artifact. State why. |

Completing a research trial does not make the effect-semantics hypothesis pass. Record the observation and interpretation separately. A prototype's missing native features cannot become release passes through `Not applicable` labels.

## Completed runs

None recorded.

## Evidence handling

Keep large original videos in a local evidence directory outside the repository. Reports may use relative evidence filenames for private work. Before relying on a public compatibility claim, provide stable, accessible evidence links or a reviewable evidence archive.

For each clip, keep its original filename, SHA-256 hash, capture rate, and relevant time range in the report. Do not commit game files, saves, full memory dumps, account identifiers, or desktop footage unrelated to the test. Upload only evidence the author intends to share.
