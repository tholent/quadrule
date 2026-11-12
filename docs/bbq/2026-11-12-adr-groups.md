# ADR record — ADR-015, groups see-through to constraints

2026-11-12 · makeadr, new ADR · goforit (chris's pick: every part cited, no rounds) · written by the rung 7 lead
(Opus 5.5, high effort).

## Premises

| premise | checked by | result |
|---|---|---|
| ADR-015 is free | `ls docs/decisions/`; `git log --all -S'ADR-015' --oneline` | highest is ADR-014; no commit names ADR-015 |
| The spec calls a group a constraint container | `grep -n "constraint container" napkins/quadrule-spec.md` | line 210, §5.2 |
| chris's ruling is recorded word for word | `docs/bbq/2026-11-11-rung-7-groups-and-align.md`, Q3 amendment | yes |
| The rule is built and tested | `crates/render/tests/groups.rs::grouping_and_ungrouping_with_every_constraint_changes_no_pixel`, `crates/layout/tests/groups.rs::a_child_resolves_against_the_page_and_a_group_is_its_children_box`, `crates/layout/tests/groups.rs::percent_is_of_the_page_inside_a_group`, `crates/ui/tests/groups.rs::resizing_a_group_stretches_everything_in_it_and_the_box_refits` | present; `make check-all` → `status=ok` (2026-11-12, after 7b) |

## Answers, each cited

| part of the ADR | source |
|---|---|
| The decision | chris at the 7a look, 2026-11-12: "I approve the build, but would like to change question #1 to reflect what figma does" |
| Group turning, selection, opacity | rung 7 record Q2, Q4, Q7 |
| The overruled alternative | rung 7 record Q3 as first ruled |
| The cost that prompted it | `docs/STATUS-log.md`, 2026-11-12, 7a builder's choice 7 (48,319 px on the Nomad) |

## Defaulted

| item | veto: |
|---|---|
| Kind `design`, as ADR-011 to ADR-014 | veto: `architecture` |
| Landed straight on `main` (`AGENTS.md`, amendment of 2026-11-09), not on a branch with a PR as makeadr's default | veto: a PR |
| No next-agent prompt: the rung 8 prompt already exists (`docs/arcs/v1/rung-8-PROMPT-2026-11-12.md`) and now names this ADR | veto: a separate prompt |
