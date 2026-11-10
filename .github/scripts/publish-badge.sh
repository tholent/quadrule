#!/usr/bin/env bash
# Turns a cargo-llvm-cov JSON summary into a shields.io endpoint file and publishes it on the `badges`
# branch of the repository, so the README's coverage badge can read it. Used by coverage.yml.
#
# usage: publish-badge.sh <llvm-cov-summary.json> <git remote url> [branch]
#
# The percentage is the line coverage over the whole workspace. The branch is created (empty, apart from
# the file) the first time. Nothing is committed when the badge is unchanged.
set -euo pipefail

summary=${1:?usage: publish-badge.sh <summary.json> <remote url> [branch]}
remote=${2:?usage: publish-badge.sh <summary.json> <remote url> [branch]}
branch=${3:-badges}

percent=$(python3 -c 'import json, sys; print(json.load(open(sys.argv[1]))["data"][0]["totals"]["lines"]["percent"])' "$summary") \
  || { echo "no line coverage in $summary" >&2; exit 1; }
rounded=$(awk -v p="$percent" 'BEGIN { printf "%.1f", p }')
color=$(awk -v p="$percent" 'BEGIN { print (p >= 80 ? "brightgreen" : p >= 60 ? "green" : p >= 40 ? "yellow" : p >= 20 ? "orange" : "red") }')

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
if git ls-remote --exit-code --heads "$remote" "$branch" >/dev/null 2>&1; then
  git clone --quiet --depth 1 --branch "$branch" "$remote" "$work/repo"
else
  git init --quiet -b "$branch" "$work/repo"
  git -C "$work/repo" remote add origin "$remote"
fi

printf '{"schemaVersion":1,"label":"coverage","message":"%s%%","color":"%s"}\n' "$rounded" "$color" > "$work/repo/coverage.json"
cd "$work/repo"
git add coverage.json
if git diff --cached --quiet; then
  echo "badge unchanged: ${rounded}%"
  exit 0
fi
git -c user.name="github-actions[bot]" -c user.email="41898282+github-actions[bot]@users.noreply.github.com" \
  commit --quiet -m "Coverage badge: ${rounded}%"
git push --quiet origin "HEAD:refs/heads/$branch"
echo "badge published: ${rounded}% ($color)"
