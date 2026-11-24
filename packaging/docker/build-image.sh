#!/bin/bash
# Build the named release's image from its own DEB and packaging files.
set -euo pipefail
src=$(realpath "$1")
deb=$(realpath "$2")
context=$3
mkdir -p "$context"
number=$(node -p "require(process.argv[1]).version" "$src/ui/package.json")
[[ $(dpkg-deb -f "$deb" Version) == "$number" ]]
dpkg-deb --fsys-tarfile "$deb" | tar -tf - > "$context/deb-files.txt"
grep -Eq '^(\./)?usr/bin/open-frame-studio$' "$context/deb-files.txt"
cp -a "$src/packaging/docker/." "$context/"
cp "$deb" "$context/app.deb"
if [[ -f $src/packaging/docker/mcp-http.mjs ]]; then
  runtime="$context/mcp-runtime"
  mkdir -p "$runtime/packaging/mcp" "$runtime/ui/src/lib"
  cp "$src"/packaging/mcp/{protocol,project-file}.mjs "$runtime/packaging/mcp/"
  cp "$src/ui/package.json" "$runtime/ui/"
  cp "$src"/ui/src/lib/{frameTools.js,frameArea.js,mcp-tools.json} "$runtime/ui/src/lib/"
fi
docker build "$context" -t "open-frame-studio:$number" \
  --build-arg APP_DEB=app.deb --build-arg APP_COMMAND=/usr/bin/open-frame-studio \
  --build-arg APP_NAME=open-frame-studio --build-arg APP_TITLE='Open Frame Studio' \
  --build-arg APP_VERSION="$number" --build-arg APP_SOURCE=https://github.com/OpenAEC-Foundation/open-frame-studio \
  --build-arg APP_DESCRIPTION='Open Frame Studio desktop application streamed to the browser'
