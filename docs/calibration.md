# Calibration — how to run it

The procedure is napkin §8.4. This page is the short version for the sheets that exist now.

## 1. Make the sheets

```
cargo xtask calibration-sheet                                # every built-in profile, into calib/
cargo xtask calibration-sheet --profile supernote.a6x2 --out calib/
```

Each profile gets a folder (`calib/supernote-a5x2/`, `calib/supernote-a6x2/`) with its pages and a `codes.txt` key
listing every sample code and what it is. `calib/` is not committed; run the command again any time.

The sheets are drawn by the same renderer as real exports (`crates/xtask/src/calib.rs`), at the device's native
resolution, so what the device shows is what a template would show. They keep clear of every docked toolbar position
and the footer at once (the profile's measured safe zones, `docs/device-measurements/README.md`), so they read the
same wherever the toolbar is docked; only the floating toolbar can cover them.

| sheet | what it holds | codes |
|---|---|---|
| lines | line widths 1–6 px × 15 grays; dashed and dotted lines × 5 grays | `L…`, `D…` |
| dots | square dots 1–4 px and round dots 2–6 px × 8 grays | `P…` |
| text | Atkinson Hyperlegible Next 6–18 pt, regular and bold × 3 grays | `T…` |
| writing | wide ruled-line and dot-row strips to write across (set 2: grays 51–187) | `W1`–`W18` |

## 2. Put them on the device

Copy one profile's PNGs to that device's **MyStyle** folder; files in a MyStyle **subfolder** appear under the
Customization tab in notes (napkin §11.3). Open a note with each sheet as its template.

## 3. Judge and write

- **lines, dots:** under normal light, note for each line role (Whisper, Guide, Structure, Emphasis) the code that
  looks best, the faintest code you can still see, and the lightest gray that is clean (no speckle or banding).
- **dots:** say whether square or round reads better at the sizes you like.
- **text:** the smallest size you read comfortably, and the gray you prefer for labels.
- **writing:** write across every strip with each pen you use. Note the strips whose lines stay visible but do not
  compete with your writing.

## 4. Report

Codes are enough, e.g. "Guide: LB10. Whisper dots: PB6, square. Smallest text: TC1. Writing: W6 and W14 best;
W1 competes with the fine pen." Photos or device screenshots go in `docs/device-measurements/<profile_id>/`
(napkin §8.4 step 4). The values then go into the profile's RON file (`crates/profiles/data/`) with
`calibration_meta.verified: true`, and the slice's golden images are regenerated.

## Not on the sheets yet

Ink-competition thresholds per pen color are not measured by these sheets; safe zones come from screenshots instead
(`docs/device-measurements/README.md`). The device's real clean gray levels are unknown, so the sheets use evenly spaced steps of 17.
