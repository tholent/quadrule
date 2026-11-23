#!/usr/bin/env bash
# Reads the repository's GitHub rules and settings back and checks them against ADR-022
# (docs/decisions/ADR-022-github-branch-and-tag-rules.md): `main` cannot be force-pushed or deleted, a `v*` tag
# (not a `v*-test*` one) cannot be deleted or moved, secret scanning and push protection are on, and merged
# branches are deleted. Prints one GITHUB line and fails when anything is missing. Read-only; needs a signed-in `gh`.
# The CI token cannot read these settings, so this is run by hand (`make check-github`).
#
# usage: check-github-rules.sh [owner/repo]
set -euo pipefail

repo=${1:-tholent/quadrule}
missing=()

# Every active ruleset as one line: target, include, exclude, rule types.
rulesets=$(gh api "repos/$repo/rulesets" --jq '.[].id' | while read -r id; do
  gh api "repos/$repo/rulesets/$id" --jq \
    'select(.enforcement == "active") | "\(.target) \(.conditions.ref_name.include|join(",")) \(.conditions.ref_name.exclude|join(",")) \([.rules[].type]|join(","))"'
done)

has_rule() { # target include exclude rule
  grep -E "^$1 $2 $3 " <<<"$rulesets" | grep -qE "(^| |,)$4(,|$)"
}
has_rule branch '~DEFAULT_BRANCH' '' deletion || missing+=("main:deletion")
has_rule branch '~DEFAULT_BRANCH' '' non_fast_forward || missing+=("main:force-push")
has_rule tag 'refs/tags/v\*' 'refs/tags/v\*-test\*' deletion || missing+=("tags:deletion")
has_rule tag 'refs/tags/v\*' 'refs/tags/v\*-test\*' update || missing+=("tags:update")

settings=$(gh api "repos/$repo" --jq \
  '"\(.delete_branch_on_merge) \(.security_and_analysis.secret_scanning.status) \(.security_and_analysis.secret_scanning_push_protection.status)"')
read -r delete_merged scanning push <<<"$settings"
[ "$delete_merged" = true ] || missing+=("delete-merged-branches")
[ "$scanning" = enabled ] || missing+=("secret-scanning")
[ "$push" = enabled ] || missing+=("push-protection")

status=ok
[ ${#missing[@]} -eq 0 ] || status=fail
echo "GITHUB repo=$repo rulesets=$(grep -c . <<<"$rulesets" || true) missing=$(IFS=,; echo "${missing[*]:-none}") status=$status"
[ "$status" = ok ]
