#!/usr/bin/env bash
set -euo pipefail

g++ -std=c++17 -I. tests/julian_test.cpp -o tests/julian_tests -lCatch2Main -lCatch2
./tests/julian_tests
