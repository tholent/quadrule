# Talk: the Label gray, and a gray scale built on the pen grays

**Status: open: nothing here is ruled unless marked agreed.** Agreed in talk is not ratified; the decision record
(ADR) drafted from this talk is where these get ruled.

2026-11-26 · text · chris and Claude (Opus 5.5) · Produced by talk-through.

## Context

Rung 17's bbq asked about the Label gray, and chris answered "let's chat about this more fully" (Q9,
`docs/bbq/2026-11-24-rung-17-polish.md`). The question had been parked since 2026-11-09: labels are gray 153, four
levels from the dark grey pen's 157, and lint rule L04 ("Gray or weight close to pen ink (will compete with
handwriting)", Warn, `napkins/quadrule-spec.md` §8.3) could not be built until it was settled (`docs/STATUS-debt.md`,
"Open until v1.1"; `docs/STATUS-log.md`, 2026-11-09).

## Key points

- Both tablet profiles offer 16 grays, even steps of 17 from 0 to 255 (`gray_levels` in
  `crates/profiles/data/supernote.a6x2.ron` and `supernote.a5x2.ron`). Lint rule L05 notes any gray off that list.
- The pens, measured from chris's Nomad screenshot, write 1, 157, 201 and 254 (`ink_values`; the Manta's are assumed
  from the Nomad). Each sits close to one of the 16 steps (shown by a one-line `awk` over the steps, in the talk):

  | pen | pen gray | nearest step | gap | that step's role today |
  |---|---|---|---|---|
  | black | 1 | 0 | 1 | Emphasis lines |
  | dark grey | 157 | 153 | 4 | Label text |
  | light grey | 201 | 204 | 3 | Shade fill, Margin line |
  | white | 254 | 255 | 1 | the page |

- The grays sheet (2026-11-12, `docs/device-measurements/supernote.a6x2/calibration.md`) showed every patch clean,
  the in-between ones included, and chris read each in-between patch as matching a neighbour ("Each matches a
  neighbour"). So a gray within 4 of a step very likely shows on the tablet exactly as that step does. This is
  inferred from the sheet; no sheet has carried 157 or 201.
- Supernote's own stock grid draws its lines at gray 1 (the black pen's gray) on a page of 255, not the white pen's
  254 (`docs/device-measurements/README.md`).
- The scale chris chose keeps the pen grays as fixed points and spaces the other steps evenly between them, with 255
  as the top:

  | old step | 0 | 17 | 34 | 51 | 68 | 85 | 102 | 119 | 136 | 153 | 170 | 187 | 204 | 221 | 238 | 255 |
  |---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
  | **new step** | **1** | 18 | 36 | 53 | 70 | 88 | 105 | 122 | 140 | **157** | 172 | 186 | **201** | 219 | 237 | **255** |

  Bold: the black pen, the dark grey pen, the light grey pen, the page. The spacing is about 17.3 from 1 to 157, 14.7
  from 157 to 201 and 18 from 201 to 255. Every new step is within 4 of the old step it replaces.
- With the roles moved to the nearest new step, the Label gray becomes the dark grey pen's 157, and Margin and Shade
  become the light grey pen's 201. That answers Q9.
- The size of the job: it changes export bytes (every role's gray moves, so all 80 golden lines in
  `crates/render/tests/golden/SHA256SUMS` change), both tablet profiles and the file format (a migration step). Under
  ADR-007 (amended 2026-11-10) that means a flag until chris's look. Rung 17 keeps the golden lines frozen, so this is
  its own job, after `v0.17.0`.

## Decisions

| decision | status | where it came up |
|---|---|---|
| **D1** A template gray may match a pen's gray. The 153/157 question parked since 2026-11-09 is settled. | agreed | chris: "I don't mind if they look similar / the same on screen compared to writing" |
| **D2** Offer the pen grays as extra named choices beside the 16 steps. Replaced by D4 later in the talk; kept here as history. | agreed | chris: "named pen grays as extra choices" |
| **D3** No test sheet first to check that 157 and 201 show cleanly on the tablet. | agreed | chris: "I don't think we need to check them; we know they show up fine on the screen" |
| **D4** Keep 16 steps, with the pen grays in place of the steps nearest them, and space the rest to match. | agreed | chris: "I think we should just keep the 16 steps, but use the pen values instead of the color they are closest two. redo the remaining 11/12 colors with spacing that matches what the pens already have" |
| **D5** The page white stays 255. | agreed | chris: "keep 255 as page white" |
| **D6** The scale is 16 steps ending at 255, with the white pen's 254 left out: 1 18 36 53 70 88 105 122 140 157 172 186 201 219 237 255. | agreed | chris: "stick with the recommendation", answering 17 or 16 entries; the last recommendation was 16 ending at 255 (open question 1) |
| **D7** Built as its own job after `v0.17.0`: a decision record, a flag until chris's look, new grays sheets for both tablets. | agreed | chris: "all the other bits sound good"; the sheets first raised by chris: "this will probably require new calibration/check sheets" |
| **D8** Each role moves to the new step nearest its old gray: Emphasis 0→1; Guide, Structure, Dot grid and text 51→53; Whisper 119→122; Block 102→105; Label 153→157; Margin and Shade 204→201. | proposed | Claude, after D6; not ruled (open question 2) |
| **D9** Retire lint rule L04. | agreed | chris: "retire L04" |
| **D10** A gray picked by hand in an older `.quad` moves to the nearest new step when the file is opened (a file format migration), so old files land on the clean scale with no L05 notes. | agreed | chris: "migrate them" |

## Open questions

1. **Did "stick with the recommendation" mean 16 steps ending at 255?** Earlier in the talk Claude had suggested 254
   sit beside 255 as the white pen's step; the last recommendation, the one answered, was 16 ending at 255 with 254
   left out. Recommendation: 16 ending at 255, as D6 reads.
2. **Do the roles follow the nearest new step (D8)?** Recommendation: yes. Nothing on the tablet changes, and Label,
   Margin and Shade land on pen grays exactly.
3. **Should the gray picker name the pen steps**, for example "157 · dark grey pen", taken from the profile's
   `ink_values`? D2 wanted names; D4 replaced the extra choices but said nothing about names. Recommendation: yes,
   the name beside the step, and nothing for a profile with no pens (the paper sizes).
4. **The Manta's pens.** Its `ink_values` are assumed from the Nomad (`docs/STATUS-debt.md`). Recommendation: a pen
   screenshot from the Manta before the scale is built, since the Manta's scale would be anchored on its pens.
5. **The paper profiles.** They have no gray steps or pens (`gray_levels: []`, `crates/profiles/data/paper.*.ron`) but
   carry the same role grays (`label_gray: 153`). Recommendation: move their role grays as D8 does, so a design looks
   the same whichever profile it is made for.

## Action items

1. **Draft the decision record** for D1 and D4 to D10 (through `makeadr`, for chris to ratify). Done when the ADR is
   in `docs/decisions/` and listed in its `INDEX.md` as proposed.
2. **Retire L04 in the spec**: a dated amendment under the §8.3 table in `napkins/quadrule-spec.md`, leaving the
   table's text as it is. Done when the amendment is there and `docs/STATUS-debt.md`'s "Open until v1.1" item points
   to it.
3. **Build the scale behind a flag, after `v0.17.0`**: the new `gray_levels` and role grays in both tablet profiles
   (and the paper ones, if question 5 says so), a file format step that moves hand-picked grays (D10), with a fixture
   from the old format. Done when the flag's tests pass, the old fixture opens on the new scale, and the golden lines
   change only when the flag is removed at chris's look.
4. **Make new grays sheets** for both tablets with the new steps and the points halfway between them
   (`cargo xtask calibration-sheet`, `crates/xtask/src/calib.rs`). Done when chris has read both and the readings are
   in each tablet's `calibration.md`.

## Corrections

- The clash with pen ink is wider than labels: Margin and Shade (204) sit 3 from the light grey pen (201) too. Raised in
  the talk; no earlier record named it.

## Possible next steps

- `makeadr` for action item 1, which can carry questions 1 to 5 as its questions.
- `bbq` to plan the build (action items 3 and 4) once `v0.17.0` is out.
- `looseends` on this record later, to check the open questions got their answers.

## Amendment, 2026-11-26

Open question 1 answered by chris, word for word: "yes, end on 255". D6 stands as written: 16 steps ending at 255, with
the white pen's 254 left out.

| decision | status | where it came up |
|---|---|---|
| **D6** confirmed: the scale ends on 255 (1 18 36 53 70 88 105 122 140 157 172 186 201 219 237 255). | agreed | chris: "yes, end on 255", answering open question 1 |

## Amendment, 2026-11-26 (second)

Open questions 2 to 5 answered by chris, word for word: "2. yes; 3. yes, labels; 4. manta is identical, no need to
test; 5. yes". Every open question is now answered; the record stays open until its ADR is ratified.

| decision | status | where it came up |
|---|---|---|
| **D8** confirmed: each role moves to the new step nearest its old gray (Label 157, Margin and Shade 201, and the rest as D8 lists). | agreed | chris: "2. yes" |
| **D11** The gray picker labels the pen steps with the pen's name, for example "157 · dark grey pen", taken from the profile's `ink_values`; a profile with no pens shows none. | agreed | chris: "3. yes, labels" |
| **D12** The Manta's pen grays are the Nomad's (1, 157, 201, 254); no Manta screenshot before the build. | agreed | chris: "4. manta is identical, no need to test" |
| **D13** The paper profiles' role grays move as D8 does. | agreed | chris: "5. yes" |

## Amendment, 2026-11-26 (third)

Folded into ADR-027, ratified by chris the same day. Answering the ADR's asks, chris, word for word: "1. ratify 2.
move only on-step grays 3. keep". So D10 is narrowed: a hand-picked gray moves only when it sat on one of the old 16
steps; one off them stays as it is. The flag is `--pen-grays`.

| decision | status | where it came up |
|---|---|---|
| **D10** narrowed: only hand-picked grays on the old steps move to the matching new step; others stay. | agreed | chris: "2. move only on-step grays", on ADR-027's ask 2 |
| **D14** The flag is named `--pen-grays`. | agreed | chris: "3. keep", on ADR-027's ask 3 |
