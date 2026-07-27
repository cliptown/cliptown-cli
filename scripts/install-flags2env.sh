#!/usr/bin/env bash
set -euo pipefail

# Pin the installer for reproducible CI. Override only in a reviewed dependency PR.
readonly default_ref='7f72052994c71e68bcb28c322f0c2e3bac54e544'
readonly ref="${FLAGS2ENV_REF:-$default_ref}"
readonly installer="https://raw.githubusercontent.com/ORESoftware/flags-2-env/${ref}/scripts/install.sh"

curl --fail --silent --show-error --location "$installer" | bash
