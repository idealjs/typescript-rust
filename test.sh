#!/usr/bin/env sh
# Run one test case on both implementations (tsgo + tsox) and diff against
# the tsgo own-baseline. Argument: case path/name, or a 1-based row number
# in scripts/gostd/divergence_worklist.csv.
#
#   ./test.sh compiler/arrayFind.ts
#   ./test.sh arrayFind
#   ./test.sh 42
#   ./test.sh --side rust 42
exec python3 "$(dirname "$0")/scripts/gostd/run_case.py" "$@"
