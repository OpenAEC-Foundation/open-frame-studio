#!/bin/bash
# Tests an OpenAEC tool image before it is published (OpenAEC-Blueprint#61):
# the container starts, becomes healthy, runs the app as a regular user, the
# stream can be photographed, and its memory at rest is measured.
#
#   templates/docker/test-container.sh <image> <app program, as started> <output folder>
#   e.g. templates/docker/test-container.sh openaec-blueprint:test /usr/bin/openaec-blueprint docker-test
#
# Writes stream.png, memory.txt and container.log to the output folder. Needs
# Docker, Node (npx) and ImageMagick, as on the GitHub ubuntu runners.
set -euo pipefail

image=$1
process=$2
out=$3
port=${PORT:-3000}
mkdir -p "$out"
files=$(mktemp -d)
chmod 777 "$files"
name="openaec-test-$$"

finish() {
  docker logs "$name" >"$out/container.log" 2>&1 || true
  docker rm -f "$name" >/dev/null 2>&1 || true
}
trap finish EXIT

docker run -d --name "$name" -p "127.0.0.1:$port:3000" -v "$files:/openaec" "$image" >/dev/null

echo "Waiting until the container is healthy"
status=starting
for _ in $(seq 60); do
  status=$(docker inspect -f '{{.State.Health.Status}}' "$name")
  [ "$status" = healthy ] && break
  [ "$(docker inspect -f '{{.State.Running}}' "$name")" = true ] || { echo "::error::the container stopped"; exit 1; }
  sleep 2
done
[ "$status" = healthy ] || { echo "::error::the container is $status, not healthy"; exit 1; }

echo "The app runs as a regular user"
pid=$(docker exec "$name" pgrep -f -n -- "^$process( |\$)" || true)
[ -n "$pid" ] || { echo "::error::$process does not run in the container"; exit 1; }
user=$(docker exec "$name" ps -o user= -p "$pid" | tr -d ' ')
[ -n "$user" ] && [ "$user" != root ] || { echo "::error::$process runs as '$user'"; exit 1; }

echo "The health endpoint answers OK"
[ "$(curl -fsS "http://127.0.0.1:$port/gezond")" = OK ] || { echo "::error::/gezond does not answer OK"; exit 1; }

echo "Photographing the stream in a browser"
command -v convert >/dev/null || { sudo apt-get update -qq && sudo apt-get install -y -qq imagemagick >/dev/null; }
npx -y playwright@1.49.1 install --with-deps chromium >/dev/null
npx -y playwright@1.49.1 screenshot --browser=chromium --viewport-size=1600,1000 --wait-for-timeout=10000 \
  "http://127.0.0.1:$port/" "$out/stream.png"
colours=$(convert "$out/stream.png" -format %k info:)
echo "The stream shows $colours colours"
[ "$colours" -ge 64 ] || { echo "::error::the stream looks empty ($colours colours)"; exit 1; }

echo "Measuring memory at rest"
sleep 15
docker stats --no-stream --format '{{.MemUsage}}' "$name" | cut -d/ -f1 | tr -d ' ' >"$out/memory.txt"
echo "Memory at rest: $(cat "$out/memory.txt")"

echo "The container stops cleanly"
docker stop -t 15 "$name" >/dev/null
code=$(docker inspect -f '{{.State.ExitCode}}' "$name")
[ "$code" = 0 ] || [ "$code" = 143 ] || { echo "::error::the container ended with code $code"; exit 1; }
echo "All container tests passed"
