#!/bin/bash

set -euxo pipefail

upstream=${CIRCLE_REPOSITORY_URL:-$(git remote  get-url origin)}

git config --global user.email "nada@circleci.example"
git config --global user.name "CircleCI build for ${upstream}"
