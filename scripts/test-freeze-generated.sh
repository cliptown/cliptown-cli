#!/usr/bin/env bash
set -euo pipefail

repo_root=$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
scratch=$(mktemp -d "${TMPDIR:-/tmp}/cliptown-freeze-generated.XXXXXX")
trap 'rm -rf "$scratch"' EXIT HUP INT TERM

mkdir -p "$scratch/frozen/generated" "$scratch/writable/generated" "$scratch/unmarked/generated"
printf '%s\n' '<!-- generated-policy: frozen -->' >"$scratch/frozen/generated/README.md"
printf '%s\n' '<!-- generated-policy: writable -->' >"$scratch/writable/generated/README.md"
printf '%s\n' '# generated files' >"$scratch/unmarked/generated/README.md"
printf '%s\n' frozen >"$scratch/frozen/generated/artifact.rs"
printf '%s\n' writable >"$scratch/writable/generated/artifact.rs"
printf '%s\n' unmarked >"$scratch/unmarked/generated/artifact.rs"

output=$("$repo_root/scripts/freeze-generated.sh" "$scratch")
grep -Fqx "froze $scratch/frozen/generated" <<<"$output"

if [[ -w "$scratch/frozen/generated/artifact.rs" ]]; then
  echo "frozen artifact remained writable" >&2
  exit 1
fi
[[ -w "$scratch/frozen/generated/README.md" ]]
[[ -w "$scratch/writable/generated/artifact.rs" ]]
[[ -w "$scratch/unmarked/generated/artifact.rs" ]]

mkdir -p "$scratch/empty/generated"
printf '%s\n' '<!-- generated-policy: frozen -->' >"$scratch/empty/generated/README.md"
"$repo_root/scripts/freeze-generated.sh" "$scratch/empty" >/dev/null

if "$repo_root/scripts/freeze-generated.sh" "$scratch/missing" >/dev/null 2>&1; then
  echo "missing root was accepted" >&2
  exit 1
fi

echo "freeze-generated tests passed"
