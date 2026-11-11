#!/bin/sh
# Run the example's tests in both languages, then prove untranslate is exact.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
nbma=${NBMA:-"$here/../target/debug/nbma"}
for side in before after; do
    echo "== $side"
    (cd "$here/$side" && python3 -m unittest -q test_registry)
done
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
cp -R "$here/after/." "$work"
"$nbma" untranslate "$work"
diff -r --exclude=ordbok.nbma.toml --exclude=__pycache__ "$here/before" "$work"
echo "untranslate: byte for byte identical"
