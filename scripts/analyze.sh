#!/bin/env bash
cd ../fuzzer/
echo -e "Analyzing..."
cargo build -q -r
time cargo run -q -r -- --analyze > ../analyzed/parsing_mismatches.csv

line_count=$(wc -l ../analyzed/parsing_mismatches.csv)
echo -e "Found $line_count parsing mismatches. Saved to ./analyzed/parsing_mismatches.csv"
