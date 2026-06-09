#!/usr/bin/env sh
set -eu

podman build -t skrifheim:local -f Containerfile .
podman run --rm skrifheim:local
