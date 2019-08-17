#!/bin/bash

find .circleci -name '*.sh' -exec ./shellcheck {} \+