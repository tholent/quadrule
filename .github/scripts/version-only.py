#!/usr/bin/env python3
# Decides whether a push to main needs the Fuzz and Coverage runs, or only moved the version number.
# Used by fuzz.yml and coverage.yml (chris, 2026-11-17: "yes, let's go with option 2"): a rung's sign-off
# commit bumps `version` in Cargo.toml and Cargo.lock and changes nothing the reader or the tests read, so
# those two runs would repeat the last one. CI still runs on it, as the only test of the tagged commit.
#
# usage: version-only.py <event name> <base sha> <head sha>
#
# Prints one line, `CHANGES run=true|false reason=...`, and writes `run=true|false` to $GITHUB_OUTPUT when
# set. It answers run=false only when the push's whole range changed nothing but:
#   - `[workspace.package] version` in Cargo.toml,
#   - the `version` of quadrule-* packages in Cargo.lock,
#   - files outside the code paths below (docs, STATUS.md and so on).
# Anything it cannot tell (not a push, a new branch, a base it cannot read) answers run=true.

import os
import subprocess
import sys
import tomllib

# Paths the two workflows watch, besides Cargo.toml and Cargo.lock. Wider than either list on purpose.
CODE = ("crates/", ".cargo/", ".github/", "rust-toolchain.toml", "docs/device-measurements/")


def git(*args):
    return subprocess.run(["git", *args], capture_output=True, text=True, check=True).stdout


def toml_at(rev, path):
    out = subprocess.run(["git", "show", f"{rev}:{path}"], capture_output=True, text=True)
    return tomllib.loads(out.stdout) if out.returncode == 0 else None


def without_workspace_version(doc):
    doc.get("workspace", {}).get("package", {}).pop("version", None)
    return doc


def without_own_versions(lock):
    for package in lock.get("package", []):
        if package.get("name", "").startswith("quadrule"):
            package.pop("version", None)
    return lock


def decide(event, base, head):
    if event != "push":
        return True, f"event-{event}"
    if not base or set(base) == {"0"}:
        return True, "no-base"
    try:
        git("cat-file", "-e", f"{base}^{{commit}}")
        files = git("diff", "--name-only", base, head).split()
    except subprocess.CalledProcessError:
        return True, "base-unreadable"
    for path in files:
        if path.startswith(CODE):
            return True, f"code:{path}"
    strip = {"Cargo.toml": without_workspace_version, "Cargo.lock": without_own_versions}
    for path, fn in strip.items():
        if path not in files:
            continue
        before, after = toml_at(base, path), toml_at(head, path)
        if before is None or after is None or fn(before) != fn(after):
            return True, f"code:{path}"
    return False, "version-only"


def main():
    if len(sys.argv) != 4:
        sys.exit("usage: version-only.py <event name> <base sha> <head sha>")
    run, reason = decide(*sys.argv[1:])
    word = "true" if run else "false"
    print(f"CHANGES run={word} reason={reason} base={sys.argv[2][:7]} head={sys.argv[3][:7]}")
    if os.environ.get("GITHUB_OUTPUT"):
        with open(os.environ["GITHUB_OUTPUT"], "a") as out:
            out.write(f"run={word}\n")


if __name__ == "__main__":
    main()
