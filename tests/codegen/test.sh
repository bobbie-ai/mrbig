#!/bin/bash

set -eux -o pipefail

# Build the testing binaries
cargo build --quiet \
      --bin test_codegen_greeter \
      --bin test_codegen_greeter_config \
      --bin test_codegen_greeter_extra \
      --bin test_codegen_register

# Project path
PROJECT_DIR=$(realpath $(pwd)/../..)
# Target path with build objects
TARGET_DIR=${PROJECT_DIR}/target/debug
# Path to coverage folder
COV_DIR=${PROJECT_DIR}/target/coverage
# Coverage command
COV="kcov --exclude-pattern=${TARGET_DIR} --include-pattern=${PROJECT_DIR} ${COV_DIR}"

########
echo "Testing codegen"
########
$COV ${TARGET_DIR}/test_codegen_greeter
$COV ${TARGET_DIR}/test_codegen_greeter_config
$COV ${TARGET_DIR}/test_codegen_greeter_extra
$COV ${TARGET_DIR}/test_codegen_register
