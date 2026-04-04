#!/usr/bin/env bash
# bump-version.sh — Update the project version across all version-stamped files.
#
# Usage:
#   ./scripts/bump-version.sh <new-version>
#
# Example:
#   ./scripts/bump-version.sh 0.3.0
#   ./scripts/bump-version.sh 0.3.0-alpha.1

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# ── Argument validation ────────────────────────────────────────────────────────

if [[ $# -ne 1 ]]; then
    echo "Usage: $0 <new-version>"
    echo ""
    echo "Examples:"
    echo "  $0 0.3.0"
    echo "  $0 0.3.0-alpha.1"
    exit 1
fi

NEW_VERSION="$1"

# Basic semver validation (X.Y.Z or X.Y.Z-pre.N)
if ! echo "$NEW_VERSION" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$'; then
    echo "Error: '$NEW_VERSION' is not a valid semantic version (e.g. 1.2.3 or 1.2.3-alpha.1)."
    exit 1
fi

# ── Detect current version from Cargo.toml ────────────────────────────────────

CURRENT_VERSION=$(grep -m1 '^version = ' "$REPO_ROOT/Cargo.toml" | sed 's/version = "\(.*\)"/\1/')

if [[ -z "$CURRENT_VERSION" ]]; then
    echo "Error: Could not detect current version from Cargo.toml."
    exit 1
fi

echo "Bumping version: $CURRENT_VERSION → $NEW_VERSION"
echo ""

# ── 1. Cargo.toml ─────────────────────────────────────────────────────────────

echo "  Updating Cargo.toml..."
sed -i "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" "$REPO_ROOT/Cargo.toml"

# ── 2. install.sh ─────────────────────────────────────────────────────────────

echo "  Updating install.sh..."
sed -i "s/^# Version: .*/# Version: $NEW_VERSION/" "$REPO_ROOT/install.sh"
# Also update the static version string shown in the banner
sed -i "s/print_info \"  Version: [^\"]*\"/print_info \"  Version: $NEW_VERSION\"/" "$REPO_ROOT/install.sh"

# ── 4. CHANGELOG.md ───────────────────────────────────────────────────────────

echo "  Updating CHANGELOG.md..."

TODAY=$(date +%Y-%m-%d)
CHANGELOG="$REPO_ROOT/CHANGELOG.md"

# Insert a new version section after the [Unreleased] heading
NEW_SECTION="## [$NEW_VERSION] - $TODAY\n\n### Added\n\n### Changed\n\n### Fixed\n\n### Removed\n"

# Use awk to insert after the first [Unreleased] section heading
awk -v section="$NEW_SECTION" '
    /^## \[Unreleased\]/ && !inserted {
        print
        print ""
        printf "%s", section
        inserted=1
        next
    }
    { print }
' "$CHANGELOG" > "$CHANGELOG.tmp" && mv "$CHANGELOG.tmp" "$CHANGELOG"

# Update the [Unreleased] comparison link at the bottom
sed -i "s|^\[Unreleased\]: \(.*\)/compare/v\(.*\)\.\.\.HEAD|\[Unreleased\]: \1/compare/v$NEW_VERSION...HEAD|" "$CHANGELOG"

# Add the new version comparison link after the [Unreleased] link
# Only add if the link does not already exist
if ! grep -q "^\[$NEW_VERSION\]:" "$CHANGELOG"; then
    sed -i "/^\[Unreleased\]:/a [$NEW_VERSION]: https://github.com/HazaVVIP/hazler/compare/v$CURRENT_VERSION...v$NEW_VERSION" "$CHANGELOG"
fi

# ── Done ──────────────────────────────────────────────────────────────────────

echo ""
echo "Done. Files updated:"
echo "  - Cargo.toml"
echo "  - install.sh"
echo "  - CHANGELOG.md"
echo ""
echo "Next steps:"
echo "  1. Fill in the new section in CHANGELOG.md"
echo "  2. Commit: git commit -am \"chore: bump version to $NEW_VERSION\""
echo "  3. Tag:    git tag v$NEW_VERSION"
echo "  4. Push:   git push origin main --tags"
