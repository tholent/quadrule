---
status: proposed
status-note: drafted YYYY-MM-DD from the user's answers; awaiting ratification
owner: the user
kind: process
built: not started
proof: where the work is queued
supersedes: nothing
amends: nothing
---
# ADR-NNN — The decision, as a sentence

## Decision

One paragraph that says what was decided, in plain words. A reader who stops here should know the choice and
its scope.

## Context

The problem, the constraints, and what was true when the choice was made. Cite sources (specs, measurements,
the question record's Q-numbers) rather than recollection.

When two or more options were weighed, end with a **Drivers:** list (a line `Drivers:`, then `- ` items). Delete
this paragraph otherwise.

## Alternatives

Each option that was weighed, with its honest trade-off and why it lost. A recommendation the user overruled is
kept here, marked "overruled".

When two or more options were weighed, give each a `### ` heading with a `- Good:` and a `- Bad:` bullet, put
the chosen one first, and put its reason on its heading: `### Name (chosen: the deciding reason)`.

## Consequences

What this makes easier, what it makes harder, and what it commits the project to next.

## Asks left open

Only while the status is `proposed`: the questions still waiting for the user, numbered.

## History

Ratifications, amendments, corrections and earlier values of the header fields, oldest first. When a header
field changes, the old value moves here word for word.

---

How to fill in the header (`adr-check.py` checks it against the schema in `skills/makeadr/adr-plus/`). Delete
this note and the paragraphs above that do not apply.

- **status** is one lowercase word: `proposed`, `provisional`, `accepted`, `partially accepted`, `superseded`,
  `withdrawn` or `rejected`. `status-note` holds the rest of the wording (who ruled and when). An agent writes
  `proposed`; only the user makes it `accepted`, and then the note says so. While `proposed`, leave out
  `ratified-by` and `ratified-on`.
- **owner** comes from the project settings. **kind** is `architecture`, `design` or `process` and is required.
- **built** is `landed`, `partial`, `not started` or `n/a`. **proof** says what shows it: for `landed` and
  `partial`, a test (`tests/test_x.py::test_name`), a file (`src/thing.py`) or a PR (`#123` or its link), and
  every one must resolve; for `not started`, where the work is queued; for `n/a`, the rule or file the decision
  became. A commit is never proof: it may follow a real proof, for history, but alone it fails. `partial` also
  needs **owed**, what is not built yet.
- **supersedes** and **amends** are always written, as `nothing` or an ADR number with its section.
- Add `consulted`, `confirmed-by` or `revisit-when` only when their trigger is true.
- A value that YAML would turn into a date, a number or a yes/no goes in double quotes.
- A new ADR also needs its row in `INDEX.md`, which is generated: run `adr-index.py`.
