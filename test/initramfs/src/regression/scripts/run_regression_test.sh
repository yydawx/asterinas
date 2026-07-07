#!/bin/sh

# SPDX-License-Identifier: MPL-2.0

SCRIPT_DIR=/test
FAILED_DIRS=""

for dir in $(find -L "${SCRIPT_DIR}" -mindepth 1 -maxdepth 1 -type d); do
    if [ -x "${dir}/run_test.sh" ]; then
        echo "Running test in $dir"
        if (cd "$dir" && ./run_test.sh); then
            echo "All test in $dir passed."
        else
            echo "ERROR: Some tests in $dir FAILED."
            FAILED_DIRS="$FAILED_DIRS $dir"
        fi
    else
        echo "Skipping $dir (no executable TEST_SCRIPT)"
    fi
done

if [ -n "$FAILED_DIRS" ]; then
    echo "=== Failed test directories:$FAILED_DIRS ==="
    exit 1
fi
echo "All regression tests passed."
