# Justfile for dredge
# Install just: https://just.systems

# Default: list available recipes
default:
    @just --list

# Run the full test suite
test:
    cargo test

# Check formatting and lints (mirrors CI)
check:
    cargo fmt --check
    cargo clippy --all-targets -- -W clippy::pedantic

# Auto-format the source
fmt:
    cargo fmt

# Validate conventional commit history
commits:
    cog check

# ---------------------------------------------------------------------------
# Release
# ---------------------------------------------------------------------------
#
# Two-step process to work with branch protection on master:
#
#   Step 1 — just release
#     Runs pre-flight checks, calls `cog bump --auto` on a local release
#     branch, opens a PR.  Review and merge the PR normally.
#
#   Step 2 — just push-tag
#     After the PR is merged, pulls master, tags the merge commit with the
#     version from Cargo.toml, and pushes the tag.
#     The tag push triggers release.yml which publishes to crates.io.
#
# Prerequisites:
#   - `cocogitto` installed: cargo install cocogitto
#   - `gh` CLI installed and authenticated: https://cli.github.com
# ---------------------------------------------------------------------------

# Step 1: open a release PR.
#
# Runs pre-flight checks, bumps the version on a release branch,
# generates CHANGELOG.md, and opens a pull request against master.
# After the PR is merged, run `just push-tag` to trigger the publish.
release:
    #!/usr/bin/env bash
    set -euo pipefail

    # Guard: must be on master.
    branch=$(git rev-parse --abbrev-ref HEAD)
    if [[ "$branch" != "master" ]]; then
        echo "error: releases must be cut from master (currently on '$branch')" >&2
        exit 1
    fi

    # Guard: working tree must be clean.
    if ! git diff --quiet || ! git diff --cached --quiet; then
        echo "error: working tree has uncommitted changes; commit or stash them first" >&2
        exit 1
    fi

    # Guard: local master must not be behind origin.
    git fetch --quiet origin master
    if [[ $(git rev-list --count HEAD..origin/master) -gt 0 ]]; then
        echo "error: local master is behind origin/master; run 'git pull' first" >&2
        exit 1
    fi

    # Run the full test suite before touching anything.
    echo "==> Running tests..."
    cargo test

    # Check formatting and lints.
    echo "==> Checking formatting and lints..."
    cargo fmt --check
    cargo clippy --all-targets -- -W clippy::pedantic

    # Determine the next version without making any changes yet.
    # cog bump --auto --dry-run prints e.g. "v0.1.0" to stdout.
    echo "==> Determining next version..."
    next_version=$(cog bump --auto --dry-run)
    echo "    Next version: ${next_version}"

    # Create and switch to a release branch.
    release_branch="release/${next_version}"
    git checkout -b "${release_branch}"

    # Bump version, generate CHANGELOG.md, commit, and create the local tag.
    # cog bump --auto:
    #   - Updates the version field in Cargo.toml
    #   - Writes CHANGELOG.md
    #   - Creates a commit "chore(version): bump to vX.Y.Z"
    #   - Creates an annotated tag vX.Y.Z  (local only until push-tag)
    echo "==> Bumping version with cog..."
    cog bump --auto

    # Push the release branch (not the tag — that comes after PR merge).
    echo "==> Pushing release branch..."
    git push -u origin "${release_branch}"

    # Open the pull request.
    echo "==> Opening pull request..."
    gh pr create \
        --title "chore(release): ${next_version}" \
        --body "$(cat CHANGELOG.md)" \
        --base master \
        --head "${release_branch}"

    echo ""
    echo "==> Release PR opened."
    echo "    Review and merge the PR, then run:"
    echo ""
    echo "        just push-tag"
    echo ""
    echo "    to push the tag and trigger the crates.io publish."

# Step 2: tag master HEAD and push the tag.
#
# Run this after the release PR has been merged into master.
# Pulls the latest master, reads the version from Cargo.toml, creates an
# annotated tag on the current HEAD, and pushes it to trigger release.yml.
push-tag:
    #!/usr/bin/env bash
    set -euo pipefail

    # Must be on master.
    branch=$(git rev-parse --abbrev-ref HEAD)
    if [[ "$branch" != "master" ]]; then
        echo "error: push-tag must be run from master (currently on '$branch')" >&2
        exit 1
    fi

    # Pull so we are at the merge commit.
    echo "==> Pulling latest master..."
    git pull --ff-only origin master

    # Derive the version from Cargo.toml (set by `cog bump --auto`).
    version=$(grep '^version' Cargo.toml | head -1 \
                | sed 's/version = "\(.*\)"/\1/')
    if [[ -z "$version" ]]; then
        echo "error: could not read version from Cargo.toml" >&2
        exit 1
    fi
    tag="v${version}"
    echo "==> Tagging HEAD as ${tag}..."

    # Guard: tag must not already exist on origin.
    if git ls-remote --tags origin "${tag}" | grep -q "refs/tags/${tag}$"; then
        echo "error: tag ${tag} already exists on origin" >&2
        exit 1
    fi

    # Delete stale local tag if present (leftover from the release branch).
    git tag -d "${tag}" 2>/dev/null || true

    # Create a fresh annotated tag on the current (merged) HEAD.
    git tag -a "${tag}" -m "chore(release): ${tag}"

    echo "==> Pushing tag ${tag}..."
    git push origin "${tag}"

    echo "==> Done. Monitor the release workflow at:"
    echo "    https://github.com/anthonyoteri/dredge/actions"
