#!/bin/bash

set -euxo pipefail

sudo apt-get update
sudo apt-get install shellcheck
shellcheck --version
