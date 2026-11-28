#!/usr/bin/env bash
# Fork-only development loop: build a Linux artifact in CI, install it, launch it.
# Usage: scripts/dev-fast-loop.sh [branch] [--with-tests]
# Environment: REPO=owner/repo (default: zwangzw-zw/tiny-player)
set -euo pipefail

repo=${REPO:-zwangzw-zw/tiny-player}
branch=feat/subtitle-style-dev
with_tests=0
for arg in "$@"; do
    case $arg in
        --with-tests) with_tests=1 ;;
        --*) echo "Unknown option: $arg" >&2; exit 2 ;;
        *) branch=$arg ;;
    esac
done

if [[ $branch == master ]]; then
    echo 'Refusing to run on master.' >&2
    exit 1
fi
if [[ $(git branch --show-current) != "$branch" ]]; then
    echo "Checkout $branch first." >&2
    exit 1
fi
if [[ -n $(git status --porcelain) ]]; then
    echo 'Working tree is dirty; commit your changes first.' >&2
    exit 1
fi
if ! git fetch fork "$branch" --quiet; then
    echo "Push $branch to the fork first." >&2
    exit 1
fi
if [[ $(git rev-parse HEAD) != $(git rev-parse "fork/$branch") ]]; then
    echo "Local $branch is not pushed; push it first." >&2
    exit 1
fi

skip_tests=true
[[ $with_tests == 1 ]] && skip_tests=false

echo "Dispatching Linux dev build for $branch (tests/Clippy: $([[ $skip_tests == true ]] && echo skipped || echo included))"
gh workflow run release.yml --ref "$branch" -R "$repo" \
    -f package=true -f windows=false -f skip_tests="$skip_tests"

before=$(gh run list -R "$repo" --workflow release.yml --branch "$branch" \
    --limit 1 --json databaseId --jq '.[0].databaseId' 2>/dev/null || true)
run_id=
for _ in {1..30}; do
    run_id=$(gh run list -R "$repo" --workflow release.yml --branch "$branch" \
        --limit 1 --json databaseId --jq '.[0].databaseId' 2>/dev/null || true)
    if [[ -n $run_id && $run_id != "$before" ]]; then
        break
    fi
    run_id=
    sleep 2
done
if [[ -z $run_id ]]; then
    echo 'Could not find the dispatched run.' >&2
    exit 1
fi
echo "Run: https://github.com/$repo/actions/runs/$run_id"

gh run watch "$run_id" -R "$repo" --exit-status || {
    conclusion=$(gh run view "$run_id" -R "$repo" --json conclusion --jq .conclusion)
    if [[ $conclusion != success ]]; then
        echo "Run finished: $conclusion" >&2
        exit 1
    fi
}

dest="$HOME/Downloads/tiny-player-dev-$(date +%Y%m%d-%H%M%S)"
mkdir -p "$dest"
echo "Downloading Linux artifact to $dest"
if ! gh run download "$run_id" -R "$repo" -n release-linux -D "$dest"; then
    echo 'gh run download failed; retrying with resumable curl.'
    rm -rf "$dest"
    mkdir -p "$dest"
    artifact_id=$(gh api "repos/$repo/actions/runs/$run_id/artifacts" \
        --jq '.artifacts[]|select(.name=="release-linux")|.id')
    token=$(gh auth token)
    api="https://api.github.com/repos/$repo/actions/artifacts/$artifact_id/zip"
    loc=$(curl -sS -D - -o /dev/null \
        -H "Authorization: Bearer $token" \
        -H 'Accept: application/vnd.github+json' "$api" \
        | awk 'BEGIN{IGNORECASE=1} /^location:/{print $2}' | tr -d '\r')
    if [[ -z $loc ]]; then
        echo 'Artifact download URL was not returned.' >&2
        exit 1
    fi
    curl -L --retry 10 --retry-delay 2 --retry-all-errors \
        --speed-time 30 --speed-limit 1024 -C - \
        -o "$dest/release-linux.zip" "$loc"
    unzip -q "$dest/release-linux.zip" -d "$dest"
fi

cd "$dest"
sha256sum --check --strict ./*.tar.gz.sha256 ./*.AppImage.sha256
tar -xzf ./*.tar.gz

if pid=$(pgrep -x tiny-player | head -1); then
    kill -TERM "$pid" 2>/dev/null || true
    timeout 5 tail --pid="$pid" -f /dev/null 2>/dev/null || true
fi

(cd "$dest/tiny-player" && ./install.sh)
setsid -f "$HOME/.local/bin/tiny-player" >"$dest/launch.log" 2>&1
version=$(python3 -c 'import json,os;print(json.load(open(os.path.expanduser("~/.local/tiny-player.app/manifest.json")))["version"])')
echo "Installed $version and launched. Log: $dest/launch.log"
