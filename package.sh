#!/bin/bash -e

echo "Installing Rust toolchain"
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env

echo "Building package"
cargo build

if [ -z "${ADDON_ARCH}" ]; then
  TARFILE_SUFFIX=
else
  TARFILE_SUFFIX="-${ADDON_ARCH}"
fi

BIN=$(cat manifest.json | jq '.id' | tr -d '"')
VERSION=$(cat manifest.json | jq '.version' | tr -d '"')
TARFILE="${BIN}-${VERSION}${TARFILE_SUFFIX}.tgz"
cp "target/debug/$BIN" .

FILES=(manifest.json LICENSE README.md $BIN)

shasum --algorithm 256 "${FILES[@]}" > SHA256SUMS
tar -czvf "${TARFILE}" "${FILES[@]}" SHA256SUMS
shasum --algorithm 256 "${TARFILE}" > "${TARFILE}".sha256sum
