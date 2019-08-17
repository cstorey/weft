#!/bin/bash

set -euxo pipefail

target=${1}

url=https://github.com/rust-lang-nursery/mdBook/releases/download/v0.3.1/mdbook-v0.3.1-x86_64-unknown-linux-gnu.tar.gz
sha256sum=4511fb1d4d95331099a4c1777d6af8022ac5783af70b83f018c78c896a4027ab
archive="mdbook.tar.gz"

if [[ "$(uname -ms)" != "Linux x86_64" ]]; then
    echo 1>&2 "Currently only works on Linux x86_64";
    exit 1
fi

dir=$(mktemp -d /tmp/XXXXXXXX)
archive_path="${dir}/${archive}"

curl -Lo "${archive_path}" "$url"
echo  "$sha256sum ${archive_path}" | sha256sum -c

tar -zxvf "${archive_path}" -C "$dir" mdbook
mkdir -vp "${target}"
mv -v "${dir}/mdbook" "${target}/mdbook"
"${target}/mdbook" -V