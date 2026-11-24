#!/usr/bin/env bash
set -euo pipefail
if [[ $# != 1 || ! -f $1 ]]; then echo "Expected exactly one package file" >&2; exit 1; fi
deb=$(realpath "$1")
version=$(dpkg-deb -f "$deb" Version)
arch=$(dpkg-deb -f "$deb" Architecture)
[[ $arch == amd64 && $version == "${OFS_EXPECTED_VERSION:-$version}" ]]
echo "DEB version $version, architecture $arch" >> "$GITHUB_STEP_SUMMARY"
if ar t "$deb" | grep -q '^_gpg'; then
  echo "The DEB contains an embedded signature member. Cryptographic verification is pending." >> "$GITHUB_STEP_SUMMARY"
else
  echo "The DEB contains no embedded signature member." >> "$GITHUB_STEP_SUMMARY"
fi
sudo apt-get install -y xvfb xauth xdotool "$deb"
xvfb-run -a env GDK_BACKEND=x11 bash -eu <<'SMOKE'
/usr/bin/open-frame-studio >"$RUNNER_TEMP/ofs-linux-launch.log" 2>&1 &
pid=$!
trap 'kill "$pid" 2>/dev/null || true; wait "$pid" 2>/dev/null || true' EXIT
timeout 60s xdotool search --sync --all --onlyvisible --pid "$pid" --name "Open Frame Studio" || {
  cat "$RUNNER_TEMP/ofs-linux-launch.log"; exit 1;
}
echo "Observed a visible Open Frame Studio window owned by the installed DEB process under Xvfb." >> "$GITHUB_STEP_SUMMARY"
SMOKE
