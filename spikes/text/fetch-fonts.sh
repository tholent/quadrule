#!/bin/sh
# Downloads the OFL fonts this spike uses into fonts/ (gitignored). Prints sha256 of each.
set -e
cd "$(dirname "$0")"
mkdir -p fonts
B=https://raw.githubusercontent.com/google/fonts/main/ofl
curl -sfL -o "fonts/AtkinsonHyperlegibleNext[wght].ttf" "$B/atkinsonhyperlegiblenext/AtkinsonHyperlegibleNext%5Bwght%5D.ttf"
curl -sfL -o "fonts/OFL-Atkinson.txt" "$B/atkinsonhyperlegiblenext/OFL.txt"
curl -sfL -o "fonts/NotoEmoji[wght].ttf" "$B/notoemoji/NotoEmoji%5Bwght%5D.ttf"
curl -sfL -o "fonts/OFL-NotoEmoji.txt" "$B/notoemoji/OFL.txt"
sha256sum fonts/*
