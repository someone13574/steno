#!/bin/bash
set -eo pipefail

pre-commit run --all-files
cargo clippy --all -- -D warnings
