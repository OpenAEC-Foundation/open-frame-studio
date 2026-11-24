#!/usr/bin/env bash
set -euo pipefail
shopt -s nullglob
if [[ $# != 1 || ! -f $1 ]]; then echo "Expected exactly one package file" >&2; exit 1; fi
mountpoint=$(mktemp -d "$RUNNER_TEMP/ofs-dmg.XXXXXX")
hdiutil attach -readonly -nobrowse -mountpoint "$mountpoint" "$1"
trap 'hdiutil detach "$mountpoint" >/dev/null || true' EXIT
apps=("$mountpoint"/*.app)
[[ ${#apps[@]} == 1 ]]
installed="$RUNNER_TEMP/Installed Open Frame Studio.app"
ditto "${apps[0]}" "$installed"
hdiutil detach "$mountpoint"
trap - EXIT
program=$(/usr/libexec/PlistBuddy -c 'Print CFBundleExecutable' "$installed/Contents/Info.plist")
binary="$installed/Contents/MacOS/$program"
version=$(/usr/libexec/PlistBuddy -c 'Print CFBundleShortVersionString' "$installed/Contents/Info.plist")
[[ $version == "${OFS_EXPECTED_VERSION:-$version}" ]]
lipo -verify_arch arm64 x86_64 "$binary"
echo "Installed DMG app version $version, architectures $(lipo -archs "$binary")." >> "$GITHUB_STEP_SUMMARY"
# Record the identity and notarization result of this actual copy.
# The current distribution contract permits an unstapled DMG.
details=$(codesign -dv --verbose=2 "$installed" 2>&1 || true)
printf '%s\n' "$details"
printf '%s\n' "$details" | grep -E '^(Authority|Signature|TeamIdentifier)=' >> "$GITHUB_STEP_SUMMARY" || true
if codesign --verify --deep --strict "$installed"; then
  echo "macOS: codesign verified the installed app's bundle signature." >> "$GITHUB_STEP_SUMMARY"
else
  echo "macOS: codesign did not validate the installed app's bundle signature." >> "$GITHUB_STEP_SUMMARY"
fi
if xcrun stapler validate "$installed"; then
  echo "macOS: a notarization ticket was validated on the installed app." >> "$GITHUB_STEP_SUMMARY"
else
  echo "macOS: no validated stapled notarization ticket on the installed app." >> "$GITHUB_STEP_SUMMARY"
fi
if xcrun stapler validate "$1"; then
  echo "A notarization ticket was validated on the DMG." >> "$GITHUB_STEP_SUMMARY"
else
  echo "No validated stapled notarization ticket on the DMG." >> "$GITHUB_STEP_SUMMARY"
fi
"$installed/Contents/MacOS/$program" >"$RUNNER_TEMP/ofs-macos-launch.log" 2>&1 &
pid=$!
trap 'kill "$pid" 2>/dev/null || true; wait "$pid" 2>/dev/null || true' EXIT
swift - "$pid" <<'SWIFT'
import AppKit
import Darwin
let pid = pid_t(CommandLine.arguments[1])!
let deadline = Date().addingTimeInterval(45)
while Date() < deadline {
    if let app = NSRunningApplication(processIdentifier: pid), app.isFinishedLaunching && !app.isTerminated {
        print("macOS registered the installed application as fully launched")
        exit(0)
    }
    if kill(pid, 0) != 0 { break }
    RunLoop.current.run(until: Date().addingTimeInterval(0.25))
}
exit(1)
SWIFT
echo "AppKit reported the installed DMG application fully launched." >> "$GITHUB_STEP_SUMMARY"
