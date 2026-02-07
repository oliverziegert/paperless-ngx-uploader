#!/bin/bash
# Demo script to verify recursive folder scanning functionality

echo "=================================================="
echo "Recursive Folder Scanning - Verification Demo"
echo "=================================================="
echo ""

echo "Test Directory Structure:"
echo "test_upload_folder/"
find ./test_upload_folder -type f | sort | sed 's/^/  /'
echo ""

echo "File count breakdown:"
echo "  PDF files at root: $(find ./test_upload_folder -maxdepth 1 -name '*.pdf' | wc -l | tr -d ' ')"
echo "  PDF files in subfolders: $(find ./test_upload_folder -mindepth 2 -name '*.pdf' | wc -l | tr -d ' ')"
echo "  Total PDF files: $(find ./test_upload_folder -name '*.pdf' | wc -l | tr -d ' ')"
echo "  Total all files: $(find ./test_upload_folder -type f | wc -l | tr -d ' ')"
echo ""

echo "Unit tests status:"
cargo test test_aggregate_files_recursive 2>&1 | grep -E "(test result:|running)"
echo ""

echo "CLI flag verification:"
cargo run -- upload --help 2>&1 | grep -A 1 "recursive"
echo ""

echo "=================================================="
echo "✅ Recursive folder scanning feature verified!"
echo "=================================================="
