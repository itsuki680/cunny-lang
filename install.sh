#!/bin/sh
set -eu

repository="itsuki680/cunny-lang"
install_directory="${CUNNY_INSTALL_DIR:-${HOME}/.local/bin}"
system="$(uname -s)"
machine="$(uname -m)"

case "${system}-${machine}" in
    Linux-x86_64) archive="cunny-linux-x86_64.tar.gz" ;;
    Linux-aarch64|Linux-arm64) archive="cunny-linux-aarch64.tar.gz" ;;
    Darwin-x86_64) archive="cunny-macos-x86_64.tar.gz" ;;
    Darwin-arm64|Darwin-aarch64) archive="cunny-macos-aarch64.tar.gz" ;;
    *)
        echo "Unsupported platform: ${system}-${machine}" >&2
        echo "Install with Cargo instead: cargo install --git https://github.com/${repository} --locked" >&2
        exit 1
        ;;
esac

temporary_directory="$(mktemp -d)"
trap 'rm -rf "${temporary_directory}"' EXIT HUP INT TERM
release_url="https://github.com/${repository}/releases/latest/download"

curl --fail --location --silent --show-error \
    "${release_url}/${archive}" \
    --output "${temporary_directory}/${archive}"
curl --fail --location --silent --show-error \
    "${release_url}/SHA256SUMS" \
    --output "${temporary_directory}/SHA256SUMS"

expected_checksum="$(grep " ${archive}$" "${temporary_directory}/SHA256SUMS" | awk '{print $1}')"
if command -v sha256sum >/dev/null 2>&1; then
    actual_checksum="$(sha256sum "${temporary_directory}/${archive}" | awk '{print $1}')"
else
    actual_checksum="$(shasum -a 256 "${temporary_directory}/${archive}" | awk '{print $1}')"
fi

if [ -z "${expected_checksum}" ] || [ "${expected_checksum}" != "${actual_checksum}" ]; then
    echo "Checksum verification failed for ${archive}" >&2
    exit 1
fi

tar -xzf "${temporary_directory}/${archive}" -C "${temporary_directory}"
mkdir -p "${install_directory}"
install -m 755 "${temporary_directory}/cunny" "${install_directory}/cunny"

echo "Installed cunny to ${install_directory}/cunny"
case ":${PATH}:" in
    *":${install_directory}:"*) ;;
    *) echo "Add ${install_directory} to your PATH to run cunny from anywhere." ;;
esac
