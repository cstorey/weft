#!/bin/bash

set -euxo pipefail

git ls-files | \
    xargs file --mime-type | \
    grep -F text/x-shellscript | \
    cut -d : -f 1 | \
    xargs shellcheck
