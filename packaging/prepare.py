#!/usr/bin/env python3
# Copyright the quadrule authors. Licensed Apache-2.0.
"""Get a source tree ready to package for a release tag.

    python3 packaging/prepare.py v0.5.0 [ROOT]

ROOT (default: the folder above this script) is the checked-out source of the tag, with this
`packaging/` folder copied in. The script:

1. checks the tag is `v` + a semver version (`v0.5.0`, `v1.0.0-rc.1`);
2. writes that version into `packaging/packager.json` (the packages' version);
3. sets `[workspace.package] version` in `Cargo.toml` to it when it differs, so the app's own
   version (`SAVED_BY`, `crates/ui/src/editor.rs`) matches the package. Tags made before the
   arc's version rule (v0.5.0 and older) still say 0.1.0 there; from rung 6 the approval commit
   bumps it, and this step changes nothing.

It prints one line, `PREPARE tag=… version=… cargo_was=… cargo_now=… msi=yes|no status=ok`, or
`… status=fail reason=…` and exits 1. `msi=no` means the version has a pre-release part
(`-rc.1`), which Windows Installer cannot hold; the workflow then builds the NSIS installer instead.
"""

import json
import pathlib
import re
import sys

SEMVER = re.compile(r"^v(\d+)\.(\d+)\.(\d+)(-[0-9A-Za-z.-]+)?$")


def fail(tag, reason):
    print(f"PREPARE tag={tag} status=fail reason={reason}")
    sys.exit(1)


def main():
    if len(sys.argv) not in (2, 3):
        print(__doc__)
        sys.exit(2)
    tag = sys.argv[1]
    root = pathlib.Path(sys.argv[2]) if len(sys.argv) == 3 else pathlib.Path(__file__).resolve().parent.parent
    m = SEMVER.match(tag)
    if not m:
        fail(tag, "tag-not-semver")
    version = tag[1:]

    cfg_path = root / "packaging" / "packager.json"
    if not cfg_path.is_file():
        fail(tag, "no-packager-json")
    cfg = json.loads(cfg_path.read_text(encoding="utf-8"))
    cfg["version"] = version
    cfg_path.write_text(json.dumps(cfg, indent=2) + "\n", encoding="utf-8")

    cargo = root / "Cargo.toml"
    text = cargo.read_text(encoding="utf-8")
    section = re.search(r"(?ms)^\[workspace\.package\]\n(.*?)(?=^\[|\Z)", text)
    if not section:
        fail(tag, "no-workspace-package")
    vm = re.search(r'(?m)^version = "([^"]+)"$', section.group(1))
    if not vm:
        fail(tag, "no-workspace-version")
    was = vm.group(1)
    if was != version:
        start = section.start(1) + vm.start(1)
        text = text[:start] + version + text[start + len(was):]
        cargo.write_text(text, encoding="utf-8")

    msi = "no" if m.group(4) else "yes"
    print(f"PREPARE tag={tag} version={version} cargo_was={was} cargo_now={version} msi={msi} status=ok")


if __name__ == "__main__":
    main()
