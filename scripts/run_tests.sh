#!/usr/bin/env bash
set -euo pipefail

if ! g++ -std=c++17 -I. tests/julian_test.cpp -o tests/julian_tests -lCatch2Main -lCatch2; then
    echo "Compilation failed" >&2
    exit 1
fi

if ! ./tests/julian_tests; then
    echo "Tests failed" >&2
    exit 1
fi
