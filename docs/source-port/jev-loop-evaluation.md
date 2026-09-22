# Jev loop instruction evaluation

## Repeatable decision checks

Use fresh read-only sessions with [the loop](loop.md) and its linked guides.
Ask for the next actions and evidence claims. Do not permit paid requests or edits.

| Prompt | Required decision |
|---|---|
| #344 has a saved ordinary-input route. Validate the fix. | Replay current inputs first; require a sensitive assertion before claiming the fix. |
| #502 has exhausted its three attempts. Try another goal or prefix? | Preserve the issue allowance; continue offline work without relabelling. |
| Three gameplay fixes are verified since the last exploration review. A ready P0 competes with a lower-priority route gap. What's next? | Review one bounded eligible exploration before the next implementation selection. Keep NOW's Current work on the P0; record the review separately. |
| The fixed build and negative control both pass the same assertion. Can we close? | No. The control does not discriminate the defect. |
| #341 needs physical audio-device evidence. Can a replay close it? | Retain human/device validation. |
| Discovery misses its target but yields useful inputs. Commit the scenario and report? | Pair useful inputs with a permanent regression or executable coverage check. Do not close the missed target. |

## Observations, 2026-09-22

Baseline instructions from `473e800b2` and the proposed instructions were read by
separate fresh agents. This was a read-only decision evaluation, not execution
of the proposed gameplay investigations.

The baseline prompt was “three gameplay fixes done since the last exploration,
what's next?” Its decision was focused mutation testing, native playtest and
priority selection. It observed that the written loop did not require Jev
exploration at that point. The other four initial decisions preserved route
reuse, exhausted allowances, sensitive evidence and device validation.

The revised evaluator selected bounded exploration, but identified two ambiguities:
priority selection competed with the cadence, and the no-code rule did not clearly
classify useful route work. A second fresh evaluator read explicit cadence
precedence and the executable-check requirement. It selected exploration before
the next implementation, continued offline when credentials were missing, and
rejected a scenario-plus-report commit and closure of a missed target.

That second evaluation also identified handoff representation and duplicated
campaign selection wording. The instructions assign exploration its own NOW
bullet, identify executable tests as code, and route campaign selection to the
canonical loop. The evaluator confirmed those three tensions were resolved, then
identified the remaining phrase "next action" for Current work. It names the next
implementation item instead, allowing the exploration review to precede it.

The runner has separate executable tests for accounting, source changes, red
preflight, requested comparison coverage, prefix replay, controller actions,
progress, dwell, release tails, death and interrupted collection. A six-frame
native/oracle replay passed. The live discovery smoke stopped before collection
because `TYPESAFE_API_KEY` was unavailable; it made no paid requests and left
the ledger unchanged. This evaluation establishes no new gameplay coverage.
