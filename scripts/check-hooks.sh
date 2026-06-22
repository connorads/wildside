#!/bin/sh
# check-hooks.sh: fail-closed guard that the hk pre-commit hook is wired in this
# clone/worktree. core.hooksPath is local git state (not tracked), so a fresh
# clone or `wt-add` worktree runs no hooks until `mise run setup` points git at
# the tracked .hk-hooks shim. Runs as a verify-slice-fast dep so the canonical
# gate refuses to pass on an unwired clone.
set -eu

# CI runs checks directly; there are no local git hooks to wire there.
[ -n "${CI:-}" ] && exit 0

root=$(git rev-parse --show-toplevel 2>/dev/null || true)
hooks_path=$(git config --get core.hooksPath 2>/dev/null || true)
if [ "$hooks_path" != ".hk-hooks" ] || [ ! -x "${root:-.}/.hk-hooks/pre-commit" ]; then
  echo "pre-commit hook not wired (core.hooksPath='$hooks_path'); run: mise run setup" >&2
  exit 1
fi
