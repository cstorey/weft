#!/bin/bash

set -euxo pipefail

sudo apt-get update
sudo apt-get install file shellcheck
shellcheck --version
