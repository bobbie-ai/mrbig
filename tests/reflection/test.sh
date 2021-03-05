#!/bin/bash

set -eux -o pipefail

BIN_SERVER=test_reflection_server

# Build the testing binaries
cargo build --quiet \
      --bin ${BIN_SERVER}

# Project path
PROJECT_DIR=$(realpath $(pwd)/../..)
# Target path with build objects
TARGET_DIR=${PROJECT_DIR}/target/debug
# Path to coverage folder
COV_DIR=${PROJECT_DIR}/target/coverage
# Coverage command
COV="kcov --exclude-pattern=${TARGET_DIR} --include-pattern=${PROJECT_DIR} ${COV_DIR}"

########
echo "Testing reflection server"
########
$COV ${TARGET_DIR}/${BIN_SERVER}
