# STATUS-debt — quadrule

Known debt and owed work. One line each. When an item is paid, remove it and note it in `STATUS-log.md`.

- The project-setup decision is not on record yet (its decision format is now chosen: ADR+).
- A lint/preflight rule for system-font use, at save and at export, is owed to the napkin's lint list (ADR-002).
- Text spike T14/T15: font discovery, layout and outlines on macOS and Windows (`spikes/text/FINDINGS.md`).
- Render spike R15: run the eframe canvas window on a display (`spikes/render/FINDINGS.md`).
- Render spike: compare R06/R09 export hashes on x86 Linux, macOS and Windows (`spikes/render/FINDINGS.md`).
- Format spike: set up cargo-fuzz (needs nightly Rust) in the real `format` crate (`spikes/format/FINDINGS.md`).
- Parked for chris: rendering — Scale vs pixel-locked rhythm on A5 X; canvas fallback to vello_cpu on weak GPUs;
  the build-folder instruction the render worker reports (`spikes/render/FINDINGS.md`).
