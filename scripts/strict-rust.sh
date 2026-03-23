#!/bin/sh
set -eu

RUSTFLAGS="${RUSTFLAGS:+$RUSTFLAGS }-D warnings"
export RUSTFLAGS

exec "$@"
