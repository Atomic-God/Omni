#!/bin/bash
echo "Verifying Cloud Node Environment..."
if ! command -v cargo &> /dev/null; then echo "Rust is NOT installed."; exit 1; else echo "Rust is installed: $(rustc --version)"; fi
if [ -d "target" ]; then echo "Target directory exists."; else echo "Target directory missing (Clean environment)."; fi
if [ -d "seeds" ]; then echo "Seeds directory present."; else echo "Seeds directory MISSING."; fi
echo "Environment Verified."
