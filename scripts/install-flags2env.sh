#!/usr/bin/env bash
set -euo pipefail

readonly default_ref='7f72052994c71e68bcb28c322f0c2e3bac54e544'
readonly ref="${FLAGS2ENV_REF:-$default_ref}"
readonly prefix="${FLAGS2ENV_PREFIX:-$HOME/.local}"
workdir="$(mktemp -d)"
trap 'rm -rf "$workdir"' EXIT

git init -q "$workdir/source"
git -C "$workdir/source" fetch -q --depth=1 https://github.com/ORESoftware/flags-2-env.git "$ref"
git -C "$workdir/source" checkout -q --detach FETCH_HEAD
make -C "$workdir/source" all
install -d "$prefix/bin" "$prefix/lib"
install -m 0755 "$workdir/source/build/flags2env" "$prefix/bin/flags2env"

case "$(uname -s)" in
  Darwin) install -m 0755 "$workdir/source/build/libflags2env.dylib" "$prefix/lib/libflags2env.dylib" ;;
  Linux) install -m 0755 "$workdir/source/build/libflags2env.so" "$prefix/lib/libflags2env.so" ;;
esac

if [[ -n "${GITHUB_PATH:-}" ]]; then
  printf '%s\n' "$prefix/bin" >> "$GITHUB_PATH"
else
  printf 'flags2env installed at %s/bin/flags2env\n' "$prefix"
fi
