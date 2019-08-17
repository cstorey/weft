#!/bin/bash

set -euxo pipefail

dir=$(mktemp -d /tmp/gh-pages-XXXXXXXXXXXXXXXXXXXXXXX)
target_branch=gh-pages
source="$1"
version=${CIRCLE_SHA1:-$(git rev-parse HEAD)}
cci_dir=${0%/*}

upstream=${CIRCLE_REPOSITORY_URL:-$(git remote  get-url origin)}

if git ls-remote "$upstream" "$target_branch" | grep -q .; then
    git clone --depth 1 --branch "$target_branch" "$upstream" "$dir"
else
    git init "$dir"
    git -C "$dir" checkout -b "$target_branch"
    git -C "$dir" commit --allow-empty -m 'Init.'
fi
if git -C "$dir" ls-files | grep -q .; then
    git -C "$dir" ls-files | xargs git -C "$dir" rm -f
fi

tar -C "$source" -c . | tar -C "$dir" -xv

mkdir -p "${dir}/.circleci"
cp "${cci_dir}/ignore-gh-pages.yml" "${dir}/.circleci/config.yml"

git -C "$dir" add .

changes=$(git -C "$dir" diff --cached --shortstat | tee /dev/stderr)
if [[ -n "$changes" ]]; then
    git -C "$dir" commit -m "Commit docs from $upstream at $version"
    git -C "$dir" push "$upstream"
fi