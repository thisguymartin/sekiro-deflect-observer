# Compatibility status

The current development target is Windows x64 with Sekiro PC 1.06 and me3.

| Platform | Status | Evidence |
| --- | --- | --- |
| Windows x64 | Live HUD demonstrated. Full acceptance remains open. | [Gameplay walkthrough](../gameplay/walkthrough.md) |
| Windows 10 and Windows 11 separately | Exact coverage is not recorded. | No structured report identifies the OS build. |
| Standard Proton | Untested. | No recorded trial. |
| Proton Experimental | Untested. | No recorded trial. |
| Steam Deck Desktop Mode | Untested. | No recorded trial. |
| Steam Deck Game Mode | Unsupported until tested. | No recorded trial. |

The observer accepts one exact game executable SHA-256:

`637aca527538c0ec6e1f136c8ed66046e95dfbdbb1f51926e134d9916398b856`.
The animation hook also checks the expected function bytes. A displayed version
number alone does not establish compatibility.

The September 18 recording shows the HUD during combat, but it does not identify
the complete OS, driver, display mode, loader, or loaded artifact hash. See the
[current validation record](../archive/validation/validation-0.12.4.md).

To record a new result, use the
[compatibility report template](../../tests/compatibility/session-template.md)
and the [gameplay checklist](../../tests/manual/gameplay-checklist.md). Update
this page only when the report supports the claim.
