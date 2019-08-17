#!/bin/bash

set -euxo pipefail

target=${1}

url=https://shellcheck.storage.googleapis.com/shellcheck-v0.7.0.linux.x86_64.tar.xz
sha512sum=84e06bee3c8b8c25f46906350fb32708f4b661636c04e55bd19cdd1071265112d84906055372149678d37f09a1667019488c62a0561b81fe6a6b45ad4fae4ac0
archive="shellcheck.tar.xz"

if [[ "$(uname -ms)" != "Linux x86_64" ]]; then
    echo 1>&2 "Currently only works on Linux x86_64";
    exit 1
fi

dir=$(mktemp -d /tmp/XXXXXXXX)
archive_path="${dir}/${archive}"

curl -Lo "${archive_path}" "$url"
echo  "$sha512sum ${archive_path}" | sha512sum -c

tar -Jxvf "${archive_path}" -C "$dir" shellcheck-v0.7.0/shellcheck
mkdir -vp "${target}"
mv -v "${dir}/shellcheck-v0.7.0/shellcheck" "${target}/shellcheck"
"${target}/shellcheck" --version
